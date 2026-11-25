//! The cell's page beside the MCP server: the live page's files
//! (extensions/cell/web/dist, built by `just ext cell live`;
//! XETAL_CELL_PAGE for another) and the session, on port 8470
//! (XETAL_CELL_PORT; 0 serves nothing) at 127.0.0.1 (XETAL_CELL_BIND for
//! the network). With `?mcp` the page asks:
//!   GET  /mcp/version   {"version": n}: bumped each time the cell changes
//!   GET  /mcp/last      the last commands' outcomes and times
//!   GET  /mcp/scene     the cell's timeline as a scene for the player
//!   GET  /mcp/state     the cell now (as cell_state)
//!   POST /mcp/command   commands (the body) run on the cell
//!   POST /mcp/reset     a new cell (?circuit=demo|none)
//! A plain HTTP/1.1 server on a thread: one request a connection.

use crate::Session;
use serde_json::json;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use xetal_ext_cell::command::parse_program;

/// Starts the page's server on its own thread, if a port is given;
/// a port already in use is said on stderr, and MCP goes on without it.
pub fn serve(session: Arc<Mutex<Session>>) {
    let port: u16 = std::env::var("XETAL_CELL_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8470);
    if port == 0 {
        return;
    }
    let bind = std::env::var("XETAL_CELL_BIND").unwrap_or_else(|_| "127.0.0.1".into());
    let listener = match TcpListener::bind((bind.as_str(), port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("xetal-cell-mcp: no page: {bind}:{port}: {e}");
            return;
        }
    };
    eprintln!("xetal-cell-mcp: the cell's page at http://{bind}:{port}/?mcp");
    let root = std::env::var_os("XETAL_CELL_PAGE")
        .map(Into::into)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../web/dist"));
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let (session, root) = (session.clone(), root.clone());
            std::thread::spawn(move || {
                let _ = answer(stream, &session, &root);
            });
        }
    });
}

fn kind(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript",
        "wasm" => "application/wasm",
        "glb" => "model/gltf-binary",
        "css" => "text/css",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

fn answer(
    stream: TcpStream,
    session: &Arc<Mutex<Session>>,
    root: &std::path::Path,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));
    let mut length = 0;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h == "\r\n" || h == "\n" {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                length = v.trim().parse().unwrap_or(0);
            }
        }
    }
    let mut body = vec![0; length.min(1 << 20)];
    reader.read_exact(&mut body)?;
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let (status, content_type, bytes): (&str, &str, Vec<u8>) = match (method, path) {
        ("GET", p) if p.starts_with("/mcp/") || p.starts_with("/mcp") && p.len() == 4 => {
            let s = session
                .lock()
                .map_err(|_| std::io::Error::other("poisoned"))?;
            match p {
                "/mcp/version" => (
                    "200 OK",
                    "application/json",
                    json!({ "version": s.version }).to_string().into_bytes(),
                ),
                "/mcp/last" => (
                    "200 OK",
                    "application/json",
                    s.last.to_string().into_bytes(),
                ),
                "/mcp/scene" => (
                    "200 OK",
                    "text/plain; charset=utf-8",
                    xetal_ext_cell::scene::calls("mcp", &s.world)
                        .unwrap_or_default()
                        .into_bytes(),
                ),
                "/mcp/state" => (
                    "200 OK",
                    "application/json",
                    s.world.summary().to_string().into_bytes(),
                ),
                _ => ("404 Not Found", "text/plain", b"no such thing".to_vec()),
            }
        }
        ("POST", "/mcp/command") => {
            let mut s = session
                .lock()
                .map_err(|_| std::io::Error::other("poisoned"))?;
            let r = match parse_program(&String::from_utf8_lossy(&body)) {
                Ok(c) => s.run(&c),
                Err(e) => json!({ "error": e }),
            };
            ("200 OK", "application/json", r.to_string().into_bytes())
        }
        ("POST", "/mcp/reset") => {
            let circuit = query
                .split('&')
                .find_map(|kv| kv.strip_prefix("circuit="))
                .unwrap_or("demo");
            let mut s = session
                .lock()
                .map_err(|_| std::io::Error::other("poisoned"))?;
            (
                "200 OK",
                "application/json",
                s.reset(circuit).to_string().into_bytes(),
            )
        }
        ("GET", p) => {
            let rel = if p == "/" {
                "index.html"
            } else {
                p.trim_start_matches('/')
            };
            if rel.split('/').any(|c| c == "..") {
                ("403 Forbidden", "text/plain", b"no".to_vec())
            } else {
                match std::fs::read(root.join(rel)) {
                    Ok(b) => ("200 OK", kind(rel), b),
                    Err(_) => (
                        "404 Not Found",
                        "text/plain",
                        format!("no {rel} (just ext cell live builds the page)").into_bytes(),
                    ),
                }
            }
        }
        _ => (
            "405 Method Not Allowed",
            "text/plain",
            b"GET or POST".to_vec(),
        ),
    };
    let mut out = stream;
    write!(
        out,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        bytes.len()
    )?;
    out.write_all(&bytes)?;
    out.flush()
}
