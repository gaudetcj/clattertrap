//! A recorded chat endpoint: a plain HTTP/1.1 server on a thread that answers each request with the
//! next scripted reply and keeps every request body and Authorization header it saw.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use serde_json::Value;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

/// One request as the server saw it.
#[derive(Debug, Clone)]
pub struct Seen {
    pub body: Value,
    pub authorization: Option<String>,
}

/// A running fake.
pub struct Fake {
    /// The base URL to give `--endpoint`.
    pub url: String,
    /// Every request, in order.
    pub seen: Arc<Mutex<Vec<Seen>>>,
}

/// Starts a fake that answers with `script` in order, then 500 when the script runs out.
pub fn serve(script: Vec<(u16, Value)>) -> Fake {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::new(Mutex::new(vec![]));
    let log = Arc::clone(&seen);
    std::thread::spawn(move || {
        let mut script = script.into_iter();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let request = read_request(&mut stream);
            log.lock().unwrap().push(request);
            let (status, body) = script.next().unwrap_or((
                500,
                serde_json::json!({ "error": { "message": "script exhausted" } }),
            ));
            respond(&mut stream, status, &body);
        }
    });
    Fake { url, seen }
}

/// Reads one request: headers up to the blank line, then `Content-Length` bytes of body.
fn read_request(stream: &mut TcpStream) -> Seen {
    let mut bytes = Vec::new();
    let mut buf = [0u8; 1024];
    let split = loop {
        let n = stream.read(&mut buf).unwrap();
        assert!(n > 0, "the client closed before sending headers");
        bytes.extend_from_slice(&buf[..n]);
        if let Some(at) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            break at + 4;
        }
    };
    let head = String::from_utf8_lossy(&bytes[..split]).into_owned();
    let header = |name: &str| {
        head.lines()
            .find(|l| l.to_ascii_lowercase().starts_with(&format!("{name}:")))
            .map(|l| l[name.len() + 1..].trim().to_owned())
    };
    let length: usize = header("content-length").map_or(0, |v| v.parse().unwrap());
    let mut body = bytes[split..].to_vec();
    while body.len() < length {
        let n = stream.read(&mut buf).unwrap();
        assert!(n > 0, "the client closed mid-body");
        body.extend_from_slice(&buf[..n]);
    }
    Seen {
        body: serde_json::from_slice(&body).unwrap(),
        authorization: header("authorization"),
    }
}

fn respond(stream: &mut TcpStream, status: u16, body: &Value) {
    let text = body.to_string();
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        429 => "Too Many Requests",
        _ => "Error",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
        text.len()
    )
    .unwrap();
    stream.flush().unwrap();
}
