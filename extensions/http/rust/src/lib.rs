//! http: bounded HTTP GET for X_eTaL, through ureq (rustls). Every
//! fetch is limited in size (`XETAL_HTTP_MAX` bytes, else 16 MiB), time
//! (`XETAL_HTTP_TIMEOUT` seconds, else 30) and redirects (5); only
//! `http:` and `https:` URLs are fetched. The status and headers of the
//! last response are the extension's state (no handles). Downloads go
//! to confined paths, as sqlite's databases do.

use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use ureq::Agent;
use xetal_ext_sdk::{OwnedError, Value, text};

/// The default size limit: 16 MiB.
pub const MAX_BYTES: u64 = 16 * 1024 * 1024;
/// Redirects followed before giving up.
pub const MAX_REDIRECTS: u32 = 5;

/// The last response: its status and headers (names lower case).
struct Last {
    status: u16,
    headers: Vec<(String, String)>,
}

static LAST: Mutex<Option<Last>> = Mutex::new(None);

fn failure(m: impl Into<String>) -> OwnedError {
    OwnedError::failure(m)
}

fn lock<T>(m: &Mutex<T>) -> Result<MutexGuard<'_, T>, OwnedError> {
    m.lock()
        .map_err(|_| failure("a lock is poisoned (an earlier call panicked)"))
}

fn env_number(name: &str) -> Option<f64> {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0)
}

/// The size limit: `XETAL_HTTP_MAX` bytes, else [`MAX_BYTES`].
#[must_use]
pub fn max_bytes() -> u64 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    env_number("XETAL_HTTP_MAX").map_or(MAX_BYTES, |v| v as u64)
}

/// The time limit for a whole fetch: `XETAL_HTTP_TIMEOUT` seconds, else 30.
#[must_use]
pub fn timeout() -> Duration {
    env_number("XETAL_HTTP_TIMEOUT").map_or(Duration::from_secs(30), Duration::from_secs_f64)
}

fn agent() -> Agent {
    Agent::config_builder()
        .timeout_global(Some(timeout()))
        .max_redirects(MAX_REDIRECTS)
        .http_status_as_error(false)
        .user_agent(concat!("xetal-ext-http/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

/// Fetches `url` and returns its body's bytes, keeping the status and
/// headers; a status other than 2xx is an error naming it.
pub fn fetch(url: &str) -> Result<Vec<u8>, OwnedError> {
    let scheme = url.split_once("://").map(|(s, _)| s.to_ascii_lowercase());
    if !matches!(scheme.as_deref(), Some("http" | "https")) {
        return Err(OwnedError::invalid_argument(format!(
            "{url:?}: only http:// and https:// URLs are fetched"
        )));
    }
    *lock(&LAST)? = None;
    let mut resp = agent()
        .get(url)
        .call()
        .map_err(|e| failure(describe(url, &e)))?;
    let status = resp.status();
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_ascii_lowercase(),
                String::from_utf8_lossy(v.as_bytes()).into_owned(),
            )
        })
        .collect();
    *lock(&LAST)? = Some(Last {
        status: status.as_u16(),
        headers,
    });
    if !status.is_success() {
        return Err(failure(
            format!(
                "GET {url}: {} {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("")
            )
            .trim_end()
            .to_string(),
        ));
    }
    resp.body_mut()
        .with_config()
        .limit(max_bytes())
        .read_to_vec()
        .map_err(|e| failure(describe(url, &e)))
}

fn describe(url: &str, e: &ureq::Error) -> String {
    match e {
        ureq::Error::BodyExceedsLimit(n) => {
            format!("GET {url}: the body is over {n} bytes (XETAL_HTTP_MAX)")
        }
        ureq::Error::Timeout(_) => format!(
            "GET {url}: no answer within {} s (XETAL_HTTP_TIMEOUT)",
            timeout().as_secs_f64()
        ),
        ureq::Error::TooManyRedirects => {
            format!("GET {url}: more than {MAX_REDIRECTS} redirects")
        }
        e => format!("GET {url}: {e}"),
    }
}

/// get url: the body, as text (bytes that are not UTF-8 are replaced).
fn get(args: &[Value]) -> Result<Value, OwnedError> {
    let body = fetch(&text(&args[0])?)?;
    Ok(Value::Text(String::from_utf8_lossy(&body).into_owned()))
}

/// status: the last response's status, 0 if none came.
fn status(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Int(
        lock(&LAST)?.as_ref().map_or(0, |l| i64::from(l.status)),
    ))
}

/// header name: a header of the last response, "" when absent.
fn header(args: &[Value]) -> Result<Value, OwnedError> {
    let name = text(&args[0])?.to_ascii_lowercase();
    let last = lock(&LAST)?;
    let v = last
        .as_ref()
        .and_then(|l| l.headers.iter().find(|(k, _)| *k == name))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    Ok(Value::Text(v))
}

/// Where downloads go: `XETAL_HTTP_ROOT`, else the working directory.
fn root() -> Result<PathBuf, OwnedError> {
    match std::env::var_os("XETAL_HTTP_ROOT") {
        Some(r) => Ok(PathBuf::from(r)),
        None => std::env::current_dir().map_err(|e| failure(e.to_string())),
    }
}

/// A path a program names, under the root: relative, with no `..`.
pub fn confine(path: &str) -> Result<PathBuf, OwnedError> {
    let p = Path::new(path);
    if path.is_empty()
        || p.is_absolute()
        || !p
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(OwnedError::invalid_argument(format!(
            "{path:?}: give a path under the working directory (or XETAL_HTTP_ROOT), no .."
        )));
    }
    Ok(root()?.join(p))
}

/// url save path: the body into a file (written whole, or not at all);
/// how many bytes.
fn save(args: &[Value]) -> Result<Value, OwnedError> {
    let url = text(&args[0])?;
    let name = text(&args[1])?;
    let path = confine(&name)?;
    let body = fetch(&url)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| failure(format!("{name}: {e}")))?;
    }
    let part = path.with_extension("part");
    std::fs::write(&part, &body)
        .and_then(|()| std::fs::rename(&part, &path))
        .map_err(|e| failure(format!("{name}: {e}")))?;
    Ok(Value::Int(i64::try_from(body.len()).unwrap_or(i64::MAX)))
}

xetal_ext_sdk::xetal_extension! {
    name: "http",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        get: 1, "Char -> Char", "GET a URL: its body as text (limited in size, time and redirects).";
        status: 0, "Unit -> Int", "The last response's status (0 if none).";
        header: 1, "Char -> Char", "A header of the last response (\"\" when absent).";
        save: 2, "Char -> Char -> Int", "url save path: the body into a file; its bytes.";
    }
}
