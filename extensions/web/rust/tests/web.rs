//! web served on loopback and asked by a real HTTP client (std's
//! TcpStream, HTTP/1.1): through the loader as a host calls it, and the
//! server on its own (the queue, timeouts, files).
#![allow(unsafe_code)]

use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use xetal_ext_loader::{ArrayData, CallError, Registry, Value};
use xetal_ext_web::server::{Request, Server, sniff};

/// One HTTP request on loopback: (status, content type, body).
fn ask(port: u16, method: &str, target: &str, form: Option<&str>) -> (u16, String, String) {
    let out = ask_raw(port, method, target, form);
    let (head, body) = out.split_once("\r\n\r\n").unwrap();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    (status, header_of(head, "content-type"), body.to_string())
}

/// A header of a response's head, "" when absent.
fn header_of(head: &str, name: &str) -> String {
    head.lines()
        .find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.eq_ignore_ascii_case(name).then(|| v.trim().to_string())
        })
        .unwrap_or_default()
}

/// One HTTP request on loopback: the whole response.
fn ask_raw(port: u16, method: &str, target: &str, form: Option<&str>) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(20))).unwrap();
    let body = form.unwrap_or("");
    let extra = if form.is_some() {
        format!(
            "Content-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n",
            body.len()
        )
    } else {
        String::new()
    };
    write!(
        s,
        "{method} {target} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n{extra}\r\n{body}"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out
}

fn t(s: &str) -> Value {
    Value::Text(s.into())
}

fn text(v: &Value) -> String {
    match v {
        Value::Text(s) => s.clone(),
        Value::Array(a) => match a.data() {
            ArrayData::Char(c) => c.iter().collect(),
            d => panic!("{d:?}"),
        },
        v => panic!("{v:?}"),
    }
}

fn int(v: &Value) -> i64 {
    match v {
        Value::Int(i) => *i,
        v => panic!("{v:?}"),
    }
}

fn message(e: CallError) -> String {
    e.to_string()
}

/// The whole flow through the loader: one test, since the extension
/// serves once per process.
#[test]
fn a_program_takes_requests_and_replies() {
    let mut r = Registry::new();
    r.load_static(xetal_ext_web::__xetal_extension::descriptor)
        .unwrap();
    let call = |f: &str, args: &[Value]| r.call("web", f, args);

    // before serving
    assert!(message(call("next", &[Value::Float(0.0)]).unwrap_err()).contains("not serving"));
    assert!(message(call("path", &[]).unwrap_err()).contains("no request"));
    assert!(message(call("serve", &[Value::Float(70000.0)]).unwrap_err()).contains("not a port"));

    let port = int(&call("serve", &[Value::Int(0)]).unwrap());
    assert!(port > 0);
    let port = u16::try_from(port).unwrap();
    assert_eq!(
        int(&call("serve", &[Value::Int(0)]).unwrap()),
        i64::from(port),
        "serving again"
    );
    assert!(message(call("serve", &[Value::Int(1)]).unwrap_err()).contains("already serving"));
    assert_eq!(text(&call("next", &[Value::Float(0.05)]).unwrap()), "none");

    // a GET with a query: the parts, a guessed content type
    let client = thread::spawn(move || ask(port, "GET", "/life?n=3&name=a%20b", None));
    assert_eq!(
        text(&call("next", &[Value::Float(10.0)]).unwrap()),
        "GET /life"
    );
    assert_eq!(text(&call("method", &[]).unwrap()), "GET");
    assert_eq!(text(&call("path", &[]).unwrap()), "/life");
    assert_eq!(text(&call("query", &[]).unwrap()), "n=3&name=a%20b");
    assert_eq!(text(&call("param", &[t("name")]).unwrap()), "a b");
    assert_eq!(text(&call("param", &[t("missing")]).unwrap()), "");
    let svg = "<svg xmlns='http://www.w3.org/2000/svg'/>";
    assert_eq!(int(&call("reply", &[Value::Int(200), t(svg)]).unwrap()), 41);
    assert_eq!(
        client.join().unwrap(),
        (200, "image/svg+xml".into(), svg.into())
    );
    assert!(message(call("reply", &[Value::Int(200), t("")]).unwrap_err()).contains("no request"));

    // a form POST, a set content type, a status of the program's choosing
    let client = thread::spawn(move || ask(port, "POST", "/todos", Some("title=milk+%26+eggs")));
    assert_eq!(
        text(&call("next", &[Value::Float(10.0)]).unwrap()),
        "POST /todos"
    );
    assert_eq!(text(&call("body", &[]).unwrap()), "title=milk+%26+eggs");
    assert_eq!(text(&call("param", &[t("title")]).unwrap()), "milk & eggs");
    assert_eq!(int(&call("content", &[t("text/csv")]).unwrap()), 1);
    assert!(
        message(call("reply", &[Value::Int(42), t("")]).unwrap_err())
            .contains("not an HTTP status")
    );
    call("reply", &[Value::Int(201), t("a,b\n")]).unwrap();
    assert_eq!(
        client.join().unwrap(),
        (201, "text/csv".into(), "a,b\n".into())
    );

    // a redirect: headers the program sets, and those it may not
    let client = thread::spawn(move || ask_raw(port, "POST", "/add", Some("title=x")));
    assert_eq!(
        text(&call("next", &[Value::Float(10.0)]).unwrap()),
        "POST /add"
    );
    assert_eq!(int(&call("header", &[t("Location: /")]).unwrap()), 1);
    assert_eq!(
        int(&call("header", &[t("Cache-Control: no-store")]).unwrap()),
        2
    );
    for bad in ["no colon", "Content-Type: text/x", "Bad Name: x", "X: a\nb"] {
        assert!(
            message(call("header", &[t(bad)]).unwrap_err()).contains("header")
                || bad.starts_with("Content"),
            "{bad}"
        );
    }
    assert!(message(call("header", &[t("Content-Length: 3")]).unwrap_err()).contains("set by"));
    call("reply", &[Value::Int(303), t("")]).unwrap();
    let out = client.join().unwrap();
    let head = out.split_once("\r\n\r\n").unwrap().0;
    assert!(head.starts_with("HTTP/1.1 303"), "{head}");
    assert_eq!(header_of(head, "location"), "/");
    assert_eq!(header_of(head, "cache-control"), "no-store");

    // taken and never answered: the next take answers it 500
    let client = thread::spawn(move || ask(port, "GET", "/lost", None));
    assert_eq!(
        text(&call("next", &[Value::Float(10.0)]).unwrap()),
        "GET /lost"
    );
    assert_eq!(text(&call("next", &[Value::Float(0.0)]).unwrap()), "none");
    assert_eq!(client.join().unwrap().0, 500);

    // files: served without the program; outside the working directory refused
    assert!(message(call("files", &[t("../x")]).unwrap_err()).contains("no .."));
    let n = int(&call("files", &[t("src")]).unwrap());
    assert_eq!(n, 2, "lib.rs and server.rs");
    let (status, ctype, body) = ask(port, "GET", "/lib.rs", None);
    assert_eq!((status, ctype.as_str()), (200, "application/octet-stream"));
    assert!(body.contains("xetal_extension!"));
    assert_eq!(int(&call("files", &[t("")]).unwrap()), 0);

    assert_eq!(int(&call("stop", &[]).unwrap()), i64::from(port));
    assert_eq!(int(&call("stop", &[]).unwrap()), 0);
    assert!(TcpStream::connect(("127.0.0.1", port)).is_err(), "stopped");
}

#[test]
fn an_unanswered_request_times_out() {
    let s = Server::start(0, Duration::from_millis(200)).unwrap();
    let port = s.port();
    let client = thread::spawn(move || ask(port, "GET", "/slow", None));
    let p = s.next(Duration::from_secs(10)).unwrap();
    assert_eq!(p.request.path, "/slow");
    let (status, _, body) = client.join().unwrap();
    assert_eq!(status, 504);
    assert!(body.contains("did not reply in time"));
    drop(p);
}

#[test]
fn a_full_queue_is_busy() {
    let s = Server::start(0, Duration::from_secs(20)).unwrap();
    let port = s.port();
    let more = 8;
    let clients: Vec<_> = (0..xetal_ext_web::server::QUEUE + more)
        .map(|_| thread::spawn(move || ask(port, "GET", "/wait", None)))
        .collect();
    thread::sleep(Duration::from_secs(2)); // all arrived: queued or refused
    drop(s); // stopping answers the queued ones (500)
    let statuses: Vec<u16> = clients.into_iter().map(|c| c.join().unwrap().0).collect();
    let busy = statuses.iter().filter(|&&s| s == 503).count();
    let dropped = statuses.iter().filter(|&&s| s == 500).count();
    assert_eq!(
        (busy, dropped),
        (more, xetal_ext_web::server::QUEUE),
        "{statuses:?}"
    );
}

#[test]
fn params_and_content_types() {
    let q = Request {
        query: "a=1&b=x+y".into(),
        body: "b=from-form&c=%3C%3E".into(),
        form: true,
        ..Request::default()
    };
    assert_eq!(q.param("a").as_deref(), Some("1"));
    assert_eq!(q.param("b").as_deref(), Some("x y"), "the query first");
    assert_eq!(q.param("c").as_deref(), Some("<>"));
    assert_eq!(q.param("d"), None);
    let not_form = Request {
        body: "c=1".into(),
        ..Request::default()
    };
    assert_eq!(not_form.param("c"), None);
    assert_eq!(sniff("  <svg/>"), "image/svg+xml");
    assert_eq!(sniff("<!doctype html>"), "text/html; charset=utf-8");
    assert_eq!(sniff("[1,2]"), "application/json");
    assert_eq!(sniff("hello"), "text/plain; charset=utf-8");
}

#[test]
fn only_loopback() {
    let s = Server::start(0, Duration::from_secs(1)).unwrap();
    assert!(TcpStream::connect(("127.0.0.1", s.port())).is_ok());
    assert!(
        Server::start(s.port(), Duration::from_secs(1)).is_err(),
        "the port is taken"
    );
}
