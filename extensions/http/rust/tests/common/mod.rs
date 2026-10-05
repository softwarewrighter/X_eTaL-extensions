//! A server on loopback to fetch from: the web extension's, answering
//! by path from a thread (as an X_eTaL program would).

use std::thread;
use std::time::Duration;

use xetal_ext_web::server::{Reply, Server, parse_header};

/// Serves on a free port until the test ends; the base URL.
pub fn serve() -> String {
    let s = Server::start(0, Duration::from_secs(20)).unwrap();
    let base = format!("http://127.0.0.1:{}", s.port());
    thread::spawn(move || {
        while let Some(p) = s.next(Duration::from_secs(60)) {
            let path = p.request.path.clone();
            let reply = |status: u16, body: &str, headers: &[&str]| Reply {
                status,
                content_type: "text/plain; charset=utf-8".into(),
                headers: headers.iter().map(|h| parse_header(h).unwrap()).collect(),
                body: body.into(),
            };
            let r = match path.as_str() {
                "/text" => reply(200, "hello, X_eTaL", &["X-Answer: 42"]),
                "/moved" => reply(302, "", &["Location: /text"]),
                "/loop" => reply(302, "", &["Location: /loop"]),
                "/big" => reply(200, &"x".repeat(100_000), &[]),
                "/slow" => {
                    thread::sleep(Duration::from_secs(3));
                    continue;
                }
                "/csv" => reply(200, "a,b\n1,2\n", &[]),
                _ => reply(404, "no such page", &[]),
            };
            let _ = p.reply.send(r);
        }
    });
    base
}
