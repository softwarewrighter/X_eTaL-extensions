//! The server: axum on a background thread with its own tokio runtime,
//! answering every request by queueing it for the program (plan A15).
//! The program pulls requests one at a time ([`Server::next`]) and
//! replies to each; the async world never calls into X_eTaL.

use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{HeaderName, HeaderValue, Method, StatusCode, Uri, header};
use axum::response::Response;
use tokio::sync::oneshot;

/// Requests waiting for the program; one more is refused with 503.
pub const QUEUE: usize = 64;

/// A request, as the program sees it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
    pub body: String,
    /// The body is a form (`application/x-www-form-urlencoded`).
    pub form: bool,
}

impl Request {
    /// A query or form field, decoded; `None` when absent.
    #[must_use]
    pub fn param(&self, name: &str) -> Option<String> {
        let body = if self.form { self.body.as_str() } else { "" };
        [self.query.as_str(), body].into_iter().find_map(|s| {
            form_urlencoded::parse(s.as_bytes())
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.into_owned())
        })
    }
}

/// The program's answer.
#[derive(Debug)]
pub struct Reply {
    pub status: u16,
    pub content_type: String,
    /// More headers, each checked when the program set it.
    pub headers: Vec<(HeaderName, HeaderValue)>,
    pub body: String,
}

/// A header the program sets: `"Name: value"`, checked; content type and
/// length are the server's to set.
pub fn parse_header(line: &str) -> Result<(HeaderName, HeaderValue), String> {
    let (name, value) = line
        .split_once(':')
        .ok_or_else(|| format!("{line:?} is not a header (\"Name: value\")"))?;
    let name = HeaderName::from_bytes(name.trim().as_bytes())
        .map_err(|_| format!("{:?} is not a header name", name.trim()))?;
    if name == header::CONTENT_TYPE || name == header::CONTENT_LENGTH {
        return Err(format!("{name} is set by wb:c_ontent! and the server"));
    }
    let value = HeaderValue::from_str(value.trim())
        .map_err(|_| format!("{:?} is not a header value", value.trim()))?;
    Ok((name, value))
}

/// A request and where its reply goes.
pub struct Pending {
    pub request: Request,
    pub reply: oneshot::Sender<Reply>,
}

struct Shared {
    queue: SyncSender<Pending>,
    timeout: Duration,
    files: RwLock<Option<PathBuf>>,
}

/// A running server: its port, the queue's far end, its thread.
pub struct Server {
    port: u16,
    requests: Mutex<Receiver<Pending>>,
    shared: Arc<Shared>,
    stop: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Server {
    /// Serves on 127.0.0.1:`port` (0: any free port). A request waits up
    /// to `timeout` for the program's reply, then gets 504.
    pub fn start(port: u16, timeout: Duration) -> Result<Self, String> {
        let std_listener = std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port)))
            .map_err(|e| format!("cannot serve on port {port}: {e}"))?;
        std_listener
            .set_nonblocking(true)
            .map_err(|e| e.to_string())?;
        let port = std_listener.local_addr().map_err(|e| e.to_string())?.port();
        let (queue, requests) = sync_channel(QUEUE);
        let shared = Arc::new(Shared {
            queue,
            timeout,
            files: RwLock::new(None),
        });
        let (stop, stopped) = oneshot::channel::<()>();
        let state = Arc::clone(&shared);
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("xetal-web")
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        let thread = std::thread::Builder::new()
            .name("xetal-web".into())
            .spawn(move || {
                runtime.block_on(async move {
                    let Ok(listener) = tokio::net::TcpListener::from_std(std_listener) else {
                        return;
                    };
                    let app = Router::new().fallback(handle).with_state(state);
                    let _ = axum::serve(listener, app)
                        .with_graceful_shutdown(async {
                            let _ = stopped.await;
                        })
                        .await;
                });
                runtime.shutdown_timeout(Duration::from_millis(100));
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            port,
            requests: Mutex::new(requests),
            shared,
            stop: Some(stop),
            thread: Some(thread),
        })
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// The next request, waiting up to `wait`; `None` if none came.
    pub fn next(&self, wait: Duration) -> Option<Pending> {
        let requests = self.requests.lock().ok()?;
        requests.recv_timeout(wait).ok()
    }

    /// Serves the files under `dir` directly, without the program.
    pub fn files(&self, dir: Option<PathBuf>) {
        if let Ok(mut f) = self.shared.files.write() {
            *f = dir;
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // queued requests are dropped first, so their handlers answer
        // 500 at once instead of waiting out the timeout
        if let Ok(r) = self.requests.get_mut() {
            drop(std::mem::replace(r, sync_channel(0).1));
        }
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn plain(status: StatusCode, text: &str) -> Response {
    let mut r = Response::new(Body::from(format!("{text}\n")));
    *r.status_mut() = status;
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    r
}

async fn handle(
    State(shared): State<Arc<Shared>>,
    method: Method,
    uri: Uri,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Response {
    if method == Method::GET {
        let dir = shared.files.read().ok().and_then(|d| d.clone());
        if let Some(r) = dir.and_then(|d| file(&d, uri.path())) {
            return r;
        }
    }
    let form = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/x-www-form-urlencoded"));
    let request = Request {
        method: method.to_string(),
        path: uri.path().to_string(),
        query: uri.query().unwrap_or("").to_string(),
        body: String::from_utf8_lossy(&body).into_owned(),
        form,
    };
    let (reply, answer) = oneshot::channel();
    match shared.queue.try_send(Pending { request, reply }) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            return plain(
                StatusCode::SERVICE_UNAVAILABLE,
                "busy: too many requests waiting",
            );
        }
        Err(TrySendError::Disconnected(_)) => {
            return plain(StatusCode::SERVICE_UNAVAILABLE, "the server is stopping");
        }
    }
    match tokio::time::timeout(shared.timeout, answer).await {
        Ok(Ok(r)) => {
            let mut resp = Response::new(Body::from(r.body));
            *resp.status_mut() = StatusCode::from_u16(r.status).unwrap_or(StatusCode::OK);
            if let Ok(v) = HeaderValue::from_str(&r.content_type) {
                resp.headers_mut().insert(header::CONTENT_TYPE, v);
            }
            for (k, v) in r.headers {
                resp.headers_mut().append(k, v);
            }
            resp
        }
        Ok(Err(_)) => plain(
            StatusCode::INTERNAL_SERVER_ERROR,
            "the program took the request and did not reply",
        ),
        Err(_) => plain(
            StatusCode::GATEWAY_TIMEOUT,
            "the program did not reply in time",
        ),
    }
}

/// A file under `dir` named by a request path, if there is one.
fn file(dir: &Path, path: &str) -> Option<Response> {
    let rel = Path::new(path.trim_start_matches('/'));
    if rel.as_os_str().is_empty() || !rel.components().all(|c| matches!(c, Component::Normal(_))) {
        return None;
    }
    let full = dir.join(rel);
    let bytes = std::fs::read(&full).ok()?;
    let mut r = Response::new(Body::from(bytes));
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(by_extension(&full)),
    );
    Some(r)
}

fn by_extension(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// The content type a reply's body suggests, when the program set none.
#[must_use]
pub fn sniff(body: &str) -> &'static str {
    let t = body.trim_start();
    if t.starts_with("<svg") || t.starts_with("<?xml") {
        "image/svg+xml"
    } else if t.starts_with('<') {
        "text/html; charset=utf-8"
    } else if t.starts_with('{') || t.starts_with('[') {
        "application/json"
    } else {
        "text/plain; charset=utf-8"
    }
}
