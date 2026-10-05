//! http called as a host calls it, fetching from a server on loopback.
#![allow(unsafe_code)]

mod common;

use xetal_ext_loader::{CallError, Registry, Value};

fn ht() -> Registry {
    let mut r = Registry::new();
    r.load_static(xetal_ext_http::__xetal_extension::descriptor)
        .unwrap();
    r
}

fn t(s: &str) -> Value {
    Value::Text(s.into())
}

fn text(v: &Value) -> String {
    match v {
        Value::Text(s) => s.clone(),
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

#[test]
fn fetches_text_and_reports_the_response() {
    let base = common::serve();
    let r = ht();
    let call = |f: &str, args: &[Value]| r.call("http", f, args);
    assert_eq!(int(&call("status", &[]).unwrap()), 0, "nothing fetched yet");

    let body = call("get", &[t(&format!("{base}/text"))]).unwrap();
    assert_eq!(text(&body), "hello, X_eTaL");
    assert_eq!(int(&call("status", &[]).unwrap()), 200);
    assert_eq!(text(&call("header", &[t("X-Answer")]).unwrap()), "42");
    assert_eq!(
        text(&call("header", &[t("content-type")]).unwrap()),
        "text/plain; charset=utf-8"
    );
    assert_eq!(text(&call("header", &[t("x-absent")]).unwrap()), "");

    // a redirect is followed; a loop is not
    let body = call("get", &[t(&format!("{base}/moved"))]).unwrap();
    assert_eq!(text(&body), "hello, X_eTaL");
    let e = message(call("get", &[t(&format!("{base}/loop"))]).unwrap_err());
    assert!(e.contains("more than 5 redirects"), "{e}");

    // a status other than 2xx is an error, and the status is kept
    let e = message(call("get", &[t(&format!("{base}/nowhere"))]).unwrap_err());
    assert!(e.contains("404 Not Found"), "{e}");
    assert_eq!(int(&call("status", &[]).unwrap()), 404);

    // only http and https
    for url in ["file:///etc/passwd", "ftp://example.com/x", "example.com"] {
        let e = message(call("get", &[t(url)]).unwrap_err());
        assert!(e.contains("only http:// and https://"), "{url}: {e}");
    }

    // nobody listening
    let e = message(call("get", &[t("http://127.0.0.1:9/x")]).unwrap_err());
    assert!(e.contains("GET http://127.0.0.1:9/x"), "{e}");
}

#[test]
fn saves_to_a_confined_file() {
    let base = common::serve();
    let dir = tempfile::tempdir().unwrap();
    let r = ht();
    let call = |f: &str, args: &[Value]| r.call("http", f, args);
    // SAFETY: only this test in this binary reads the variable.
    unsafe { std::env::set_var("XETAL_HTTP_ROOT", dir.path()) };
    let n = call("save", &[t(&format!("{base}/csv")), t("data/a.csv")]).unwrap();
    assert_eq!(int(&n), 8);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("data/a.csv")).unwrap(),
        "a,b\n1,2\n"
    );
    for bad in ["../x.csv", "/tmp/x.csv", ""] {
        let e = message(call("save", &[t(&format!("{base}/csv")), t(bad)]).unwrap_err());
        assert!(e.contains("no .."), "{bad}: {e}");
    }
    // a failed fetch writes nothing
    assert!(call("save", &[t(&format!("{base}/nowhere")), t("b.csv")]).is_err());
    assert!(!dir.path().join("b.csv").exists());
}
