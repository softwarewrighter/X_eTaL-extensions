//! The size and time limits (their variables are process-wide, so they
#![allow(unsafe_code)]
//! have this test binary to themselves).

mod common;

use xetal_ext_loader::{Registry, Value};

#[test]
fn a_body_too_big_and_an_answer_too_slow_are_errors() {
    // SAFETY: set before any call, in the only test of this binary.
    unsafe {
        std::env::set_var("XETAL_HTTP_MAX", "1000");
        std::env::set_var("XETAL_HTTP_TIMEOUT", "1");
    }
    let base = common::serve();
    let mut r = Registry::new();
    r.load_static(xetal_ext_http::__xetal_extension::descriptor)
        .unwrap();
    let get = |p: &str| r.call("http", "get", &[Value::Text(format!("{base}{p}"))]);
    assert!(get("/text").is_ok(), "small enough");
    let e = get("/big").unwrap_err().to_string();
    assert!(e.contains("over 1000 bytes (XETAL_HTTP_MAX)"), "{e}");
    let start = std::time::Instant::now();
    let e = get("/slow").unwrap_err().to_string();
    assert!(
        e.contains("no answer within 1 s (XETAL_HTTP_TIMEOUT)"),
        "{e}"
    );
    assert!(start.elapsed().as_secs_f64() < 2.5);
}
