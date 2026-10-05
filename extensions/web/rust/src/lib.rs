//! web: HTTP serving for X_eTaL. axum runs on a background thread
//! (`server`); every request is queued, and the program drives: it
//! takes the next request, reads its parts, and replies (plan A15).
//! One server per process, on loopback only; the request being
//! answered is the extension's state (no handles).

pub mod server;

use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use server::{Pending, Reply, Server};
use xetal_ext_sdk::{OwnedError, Value, number, text};

/// This repository's port; programs usually pass it to `serve`.
pub const PORT: u16 = 8470;

struct Current {
    pending: Pending,
    content_type: Option<String>,
}

static SERVER: Mutex<Option<Server>> = Mutex::new(None);
static CURRENT: Mutex<Option<Current>> = Mutex::new(None);

fn failure(m: impl Into<String>) -> OwnedError {
    OwnedError::failure(m)
}

fn lock<T>(m: &Mutex<T>) -> Result<MutexGuard<'_, T>, OwnedError> {
    m.lock()
        .map_err(|_| failure("a lock is poisoned (an earlier call panicked)"))
}

/// How long a request waits for the program: `XETAL_WEB_TIMEOUT`
/// seconds, else 10.
fn reply_timeout() -> Duration {
    std::env::var("XETAL_WEB_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0)
        .map_or(Duration::from_secs(10), Duration::from_secs_f64)
}

/// serve port: start serving on 127.0.0.1 (0: any free port); the port.
/// Serving again on the same port, or on 0, returns the running one.
fn serve(args: &[Value]) -> Result<Value, OwnedError> {
    let p = number(&args[0])?;
    if !(0.0..=65535.0).contains(&p) || p.fract() != 0.0 {
        return Err(OwnedError::invalid_argument(format!(
            "{p} is not a port (0 to 65535)"
        )));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let port = p as u16;
    let mut s = lock(&SERVER)?;
    if let Some(running) = s.as_ref() {
        if port == 0 || port == running.port() {
            return Ok(Value::Int(i64::from(running.port())));
        }
        return Err(failure(format!(
            "already serving on port {}; stop it first",
            running.port()
        )));
    }
    let server = Server::start(port, reply_timeout()).map_err(failure)?;
    let port = server.port();
    *s = Some(server);
    Ok(Value::Int(i64::from(port)))
}

/// next seconds: wait up to that long for a request; "METHOD /path", or
/// "none". A request taken before and not replied to is answered 500.
fn next(args: &[Value]) -> Result<Value, OwnedError> {
    let wait = number(&args[0])?;
    if !wait.is_finite() || wait < 0.0 {
        return Err(OwnedError::invalid_argument(format!(
            "{wait} is not a number of seconds to wait"
        )));
    }
    let pending = {
        let s = lock(&SERVER)?;
        let server = s
            .as_ref()
            .ok_or_else(|| failure("not serving: call wb:s_erve! first"))?;
        *lock(&CURRENT)? = None;
        server.next(Duration::from_secs_f64(wait.min(86_400.0)))
    };
    let Some(pending) = pending else {
        return Ok(Value::Text("none".into()));
    };
    let line = format!("{} {}", pending.request.method, pending.request.path);
    *lock(&CURRENT)? = Some(Current {
        pending,
        content_type: None,
    });
    Ok(Value::Text(line))
}

/// A part of the request being answered.
fn part(f: impl FnOnce(&Current) -> String) -> Result<Value, OwnedError> {
    let cur = lock(&CURRENT)?;
    let c = cur
        .as_ref()
        .ok_or_else(|| failure("no request: call wb:n_ext! first"))?;
    Ok(Value::Text(f(c)))
}

fn method(_: &[Value]) -> Result<Value, OwnedError> {
    part(|c| c.pending.request.method.clone())
}

fn path(_: &[Value]) -> Result<Value, OwnedError> {
    part(|c| c.pending.request.path.clone())
}

fn query(_: &[Value]) -> Result<Value, OwnedError> {
    part(|c| c.pending.request.query.clone())
}

fn body(_: &[Value]) -> Result<Value, OwnedError> {
    part(|c| c.pending.request.body.clone())
}

/// param name: a query or form field, decoded; "" when absent.
fn param(args: &[Value]) -> Result<Value, OwnedError> {
    let name = text(&args[0])?;
    part(|c| c.pending.request.param(&name).unwrap_or_default())
}

/// content type: the reply's content type (else it is guessed from the
/// body: SVG, HTML, JSON, plain text).
fn content(args: &[Value]) -> Result<Value, OwnedError> {
    let t = text(&args[0])?;
    if t.is_empty() || t.chars().any(char::is_control) {
        return Err(OwnedError::invalid_argument(format!(
            "{t:?} is not a content type"
        )));
    }
    let mut cur = lock(&CURRENT)?;
    let c = cur
        .as_mut()
        .ok_or_else(|| failure("no request: call wb:n_ext! first"))?;
    c.content_type = Some(t);
    Ok(Value::Int(1))
}

/// status reply body: answer the request; the body's length in bytes.
fn reply(args: &[Value]) -> Result<Value, OwnedError> {
    let status = number(&args[0])?;
    if !(100.0..=599.0).contains(&status) || status.fract() != 0.0 {
        return Err(OwnedError::invalid_argument(format!(
            "{status} is not an HTTP status (100 to 599)"
        )));
    }
    let body = text(&args[1])?;
    let c = lock(&CURRENT)?
        .take()
        .ok_or_else(|| failure("no request to reply to: call wb:n_ext! first"))?;
    let n = body.len();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let reply = Reply {
        status: status as u16,
        content_type: c
            .content_type
            .unwrap_or_else(|| server::sniff(&body).to_string()),
        body,
    };
    // the client may have gone (a timeout): the reply is dropped
    let _ = c.pending.reply.send(reply);
    Ok(Value::Int(i64::try_from(n).unwrap_or(i64::MAX)))
}

/// files dir: serve the files under dir (relative to the working
/// directory, no `..`) directly; "" stops. How many files it holds.
fn files(args: &[Value]) -> Result<Value, OwnedError> {
    let d = text(&args[0])?;
    let s = lock(&SERVER)?;
    let server = s
        .as_ref()
        .ok_or_else(|| failure("not serving: call wb:s_erve! first"))?;
    if d.is_empty() {
        server.files(None);
        return Ok(Value::Int(0));
    }
    let dir = confine(&d)?;
    let n = std::fs::read_dir(&dir)
        .map_err(|e| failure(format!("{d}: {e}")))?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .count();
    server.files(Some(dir));
    Ok(Value::Int(i64::try_from(n).unwrap_or(i64::MAX)))
}

fn confine(path: &str) -> Result<PathBuf, OwnedError> {
    let p = Path::new(path);
    if p.is_absolute()
        || !p
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(OwnedError::invalid_argument(format!(
            "{path:?}: give a directory under the working directory (no .., not absolute)"
        )));
    }
    let cwd = std::env::current_dir().map_err(|e| failure(e.to_string()))?;
    Ok(cwd.join(p))
}

/// stop: stop serving (a request being answered gets 500); the port, or
/// 0 if none was serving.
fn stop(_: &[Value]) -> Result<Value, OwnedError> {
    *lock(&CURRENT)? = None;
    let server = lock(&SERVER)?.take();
    Ok(Value::Int(server.map_or(0, |s| i64::from(s.port()))))
}

xetal_ext_sdk::xetal_extension! {
    name: "web",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        serve: 1, "Num a => a -> Int", "Serve HTTP on 127.0.0.1 at the port (0: any free one); the port.";
        next: 1, "Num a => a -> Char", "Wait up to that many seconds for a request: \"METHOD /path\", or \"none\".";
        method: 0, "Unit -> Char", "The request's method.";
        path: 0, "Unit -> Char", "The request's path.";
        query: 0, "Unit -> Char", "The request's query, as sent.";
        body: 0, "Unit -> Char", "The request's body.";
        param: 1, "Char -> Char", "A query or form field, decoded (\"\" when absent).";
        content: 1, "Char -> Int", "Set the reply's content type.";
        reply: 2, "Num a => a -> Char -> Int", "status reply body: answer the request; the body's bytes.";
        files: 1, "Char -> Int", "Serve the files under a directory directly (\"\": stop); how many.";
        stop: 0, "Unit -> Int", "Stop serving; the port (0 if none).";
    }
}
