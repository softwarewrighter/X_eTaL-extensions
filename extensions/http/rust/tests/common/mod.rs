//! An independent HTTP fixture on loopback. Linking another extension's
//! rlib here would also link its unmangled ABI entry point on Linux.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

/// Serves on a free port until the test ends; the base URL.
pub fn serve() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    thread::spawn(move || {
        for connection in listener.incoming().flatten() {
            thread::spawn(move || respond(connection));
        }
    });
    base
}

fn respond(mut stream: TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let path = {
        let mut reader = BufReader::new((&mut stream).take(16 * 1024));
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
        let path = line.split_whitespace().nth(1).unwrap_or("/").to_owned();
        loop {
            line.clear();
            if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                break;
            }
        }
        path
    };
    let (status, headers, body) = match path.as_str() {
        "/text" => ("200 OK", "X-Answer: 42\r\n", "hello, X_eTaL".to_owned()),
        "/moved" => ("302 Found", "Location: /text\r\n", String::new()),
        "/loop" => ("302 Found", "Location: /loop\r\n", String::new()),
        "/big" => ("200 OK", "", "x".repeat(100_000)),
        "/slow" => {
            thread::sleep(Duration::from_secs(3));
            return;
        }
        "/csv" => ("200 OK", "", "a,b\n1,2\n".to_owned()),
        _ => ("404 Not Found", "", "no such page".to_owned()),
    };
    // Close each connection so the client can make independent requests.
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}",
        body.len()
    );
}
