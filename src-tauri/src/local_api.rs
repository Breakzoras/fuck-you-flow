//! A fixed door on this computer through which the user's own programs, their
//! bots for example, have audio written out by the speech engine. Off unless
//! the user turns it on under Privacy.
//!
//! The engine's own port changes on every start, so a program that wanted it
//! had to hunt for it in the process list (the Glucose Mentor did exactly that
//! on 26 September 2026). The door stays on one port and hands each request to
//! whichever engine is running at that moment. It speaks the engine's own API:
//! POST /inference with a WAV file, the same fields and the same JSON back.
//!
//! It listens on 127.0.0.1 only. Anything a web page sends is refused, so a
//! site open in the browser cannot use it, and a cloud provider is never used
//! through here: what comes in stays on the computer.

use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

pub const DEFAULT_PORT: u16 = 47600;
/// Request line and headers. Audio goes in the body, which is not limited here.
const MAX_HEAD: usize = 16 * 1024;
const HEAD_TIMEOUT: Duration = Duration::from_secs(10);
/// A long recording on the CPU can take minutes; anything past this is stuck.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);

/// Where the engine answers right now, or why it cannot: "loading", "cloud",
/// "missing".
pub type Upstream = Arc<dyn Fn() -> Result<u16, &'static str> + Send + Sync>;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LocalApiStatus {
    pub enabled: bool,
    pub listening: bool,
    pub port: u16,
    pub error: Option<String>,
}

static STATUS: Mutex<LocalApiStatus> = parking_lot::const_mutex(LocalApiStatus { enabled: false, listening: false, port: 0, error: None });
/// The running door: its port and the switch that stops it.
static RUNNING: Mutex<Option<(u16, oneshot::Sender<()>)>> = parking_lot::const_mutex(None);

pub fn status() -> LocalApiStatus {
    STATUS.lock().clone()
}

/// Open, move or close the door to match the settings. Safe to call on every
/// save: an open door on the same port is left alone.
pub fn apply(engine: Arc<crate::engine::EngineManager>, enabled: bool, port: u16) {
    let mut running = RUNNING.lock();
    if enabled {
        if let Some((p, stop)) = running.as_ref() {
            if *p == port && !stop.is_closed() {
                return;
            }
        }
    }
    // Dropping the sender is what stops the old one.
    if running.take().is_some() {
        tracing::info!("local door: closed");
    }
    *STATUS.lock() = LocalApiStatus { enabled, listening: false, port, error: None };
    if !enabled {
        return;
    }
    let (tx, rx) = oneshot::channel();
    *running = Some((port, tx));
    let upstream: Upstream = Arc::new(move || engine.local_upstream());
    tauri::async_runtime::spawn(async move {
        match bind(port).await {
            Ok(listener) => serve(listener, port, upstream, rx).await,
            Err(err) => {
                tracing::warn!("local door: could not open 127.0.0.1:{port}: {err}");
                let mut s = STATUS.lock();
                if s.port == port {
                    s.error = Some(err.to_string());
                }
            }
        }
    });
}

/// The previous door may still be letting go of the same port for a moment.
async fn bind(port: u16) -> std::io::Result<TcpListener> {
    let mut last = None;
    for _ in 0..10 {
        match TcpListener::bind(("127.0.0.1", port)).await {
            Ok(l) => return Ok(l),
            Err(e) => last = Some(e),
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Err(last.expect("tried at least once"))
}

pub async fn serve(listener: TcpListener, port: u16, upstream: Upstream, mut stop: oneshot::Receiver<()>) {
    tracing::info!("local door: listening on 127.0.0.1:{port}");
    {
        let mut s = STATUS.lock();
        if s.port == port {
            s.listening = true;
        }
    }
    loop {
        tokio::select! {
            _ = &mut stop => break,
            got = listener.accept() => match got {
                Ok((sock, _)) => {
                    let up = upstream.clone();
                    tokio::spawn(async move {
                        if let Err(err) = tokio::time::timeout(REQUEST_TIMEOUT, handle(sock, port, up)).await {
                            tracing::warn!("local door: a request ran past {} s: {err}", REQUEST_TIMEOUT.as_secs());
                        }
                    });
                }
                Err(err) => {
                    tracing::warn!("local door: accept failed: {err}");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
        }
    }
}

#[derive(Debug, PartialEq)]
enum Route {
    Inference,
    Health,
}

/// Decide from the request line and headers alone. Pure, so the rules can be
/// tested without a socket.
fn check(head: &str, port: u16) -> Result<Route, (u16, &'static str)> {
    let mut lines = head.split("\r\n");
    let first = lines.next().unwrap_or_default();
    let mut parts = first.split_whitespace();
    let (method, target) = (parts.next().unwrap_or_default(), parts.next().unwrap_or_default());
    let path = target.split('?').next().unwrap_or_default();
    let mut host = None;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else { continue };
        let name = name.trim().to_ascii_lowercase();
        if name == "origin" {
            // Browsers put this on every request a page makes to another
            // address. Programs like curl and Python's requests never do.
            return Err((403, "requests from web pages are refused"));
        }
        if name == "host" {
            host = Some(value.trim().to_ascii_lowercase());
        }
    }
    // A page can also point a name it controls at 127.0.0.1; the Host header
    // still carries that name.
    let ok_hosts = [format!("127.0.0.1:{port}"), format!("localhost:{port}")];
    if !host.as_deref().is_some_and(|h| ok_hosts.iter().any(|ok| ok == h)) {
        return Err((403, "wrong host: use 127.0.0.1"));
    }
    match (method, path) {
        ("POST", "/inference") => Ok(Route::Inference),
        ("GET", "/health") => Ok(Route::Health),
        (_, "/inference") | (_, "/health") => Err((405, "method not allowed")),
        _ => Err((404, "only POST /inference and GET /health are here")),
    }
}

/// The same head with the engine's address, and a connection that closes after
/// one answer, so the reply can be passed back byte for byte.
fn rewrite(head: &str, upstream_port: u16) -> String {
    let mut lines = head.split("\r\n");
    let mut out = String::with_capacity(head.len() + 64);
    out.push_str(lines.next().unwrap_or_default());
    out.push_str("\r\n");
    for line in lines {
        let name = line.split_once(':').map(|(n, _)| n.trim().to_ascii_lowercase()).unwrap_or_default();
        if matches!(name.as_str(), "host" | "connection" | "keep-alive" | "proxy-connection") {
            continue;
        }
        out.push_str(line);
        out.push_str("\r\n");
    }
    out.push_str(&format!("Host: 127.0.0.1:{upstream_port}\r\nConnection: close\r\n\r\n"));
    out
}

fn reason_text(why: &str) -> &'static str {
    match why {
        "loading" => "the speech engine is still loading; try again in a few seconds",
        "cloud" => "Fuck You Flow is set to a cloud provider; this door only uses the engine on this computer",
        _ => "the speech engine is not running",
    }
}

async fn reply(sock: &mut TcpStream, code: u16, body: serde_json::Value, retry: bool) -> std::io::Result<()> {
    let body = body.to_string();
    let reason = match code {
        200 => "OK",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        431 => "Request Header Fields Too Large",
        502 => "Bad Gateway",
        _ => "Service Unavailable",
    };
    let retry = if retry { "Retry-After: 5\r\n" } else { "" };
    let head = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{retry}Connection: close\r\n\r\n",
        body.len()
    );
    sock.write_all(head.as_bytes()).await?;
    sock.write_all(body.as_bytes()).await?;
    sock.shutdown().await
}

async fn handle(mut client: TcpStream, port: u16, upstream: Upstream) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(4096);
    let head_end = loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        if buf.len() > MAX_HEAD {
            return reply(&mut client, 431, serde_json::json!({ "error": "headers too large" }), false).await;
        }
        let mut chunk = [0u8; 4096];
        let n = match tokio::time::timeout(HEAD_TIMEOUT, client.read(&mut chunk)).await {
            Ok(r) => r?,
            Err(_) => return Ok(()),
        };
        if n == 0 {
            return Ok(());
        }
        buf.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let route = match check(&head, port) {
        Ok(r) => r,
        Err((code, msg)) => return reply(&mut client, code, serde_json::json!({ "error": msg }), false).await,
    };
    match (route, upstream()) {
        (Route::Health, Ok(_)) => reply(&mut client, 200, serde_json::json!({ "status": "ready" }), false).await,
        (Route::Health, Err(why)) => reply(&mut client, 503, serde_json::json!({ "status": why, "error": reason_text(why) }), why == "loading").await,
        (Route::Inference, Err(why)) => reply(&mut client, 503, serde_json::json!({ "error": reason_text(why) }), why == "loading").await,
        (Route::Inference, Ok(up)) => {
            let mut server = match TcpStream::connect(("127.0.0.1", up)).await {
                Ok(s) => s,
                Err(err) => {
                    tracing::warn!("local door: the engine on port {up} did not answer: {err}");
                    return reply(&mut client, 502, serde_json::json!({ "error": "the speech engine did not answer" }), false).await;
                }
            };
            server.write_all(rewrite(&head, up).as_bytes()).await?;
            // Whatever of the body came in with the headers.
            server.write_all(&buf[head_end + 4..]).await?;
            // The engine closes after its answer (Connection: close), which
            // ends this. Errors here are a client that hung up.
            let _ = tokio::io::copy_bidirectional(&mut client, &mut server).await;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD_OK: &str = "POST /inference HTTP/1.1\r\nHost: 127.0.0.1:47600\r\nContent-Length: 4";

    #[test]
    fn the_gate_lets_programs_in_and_keeps_pages_out() {
        assert_eq!(check(HEAD_OK, 47600), Ok(Route::Inference));
        assert_eq!(check("GET /health HTTP/1.1\r\nhost: LOCALHOST:47600", 47600), Ok(Route::Health));
        // a page in the browser
        assert_eq!(check(&format!("{HEAD_OK}\r\nOrigin: https://evil.example"), 47600).unwrap_err().0, 403);
        assert_eq!(check("POST /inference HTTP/1.1\r\nORIGIN: null\r\nHost: 127.0.0.1:47600", 47600).unwrap_err().0, 403);
        // a name pointed at 127.0.0.1, a wrong port, no host at all
        assert_eq!(check("POST /inference HTTP/1.1\r\nHost: evil.example:47600", 47600).unwrap_err().0, 403);
        assert_eq!(check("POST /inference HTTP/1.1\r\nHost: 127.0.0.1:1234", 47600).unwrap_err().0, 403);
        assert_eq!(check("POST /inference HTTP/1.1", 47600).unwrap_err().0, 403);
        // only the two routes
        assert_eq!(check("GET /inference HTTP/1.1\r\nHost: 127.0.0.1:47600", 47600).unwrap_err().0, 405);
        assert_eq!(check("POST /load HTTP/1.1\r\nHost: 127.0.0.1:47600", 47600).unwrap_err().0, 404);
        assert_eq!(check("POST /inference?x=1 HTTP/1.1\r\nHost: 127.0.0.1:47600", 47600), Ok(Route::Inference));
    }

    #[test]
    fn rewrite_points_at_the_engine_and_closes() {
        let out = rewrite("POST /inference HTTP/1.1\r\nHost: 127.0.0.1:47600\r\nconnection: keep-alive\r\nContent-Length: 4", 51234);
        assert_eq!(out, "POST /inference HTTP/1.1\r\nContent-Length: 4\r\nHost: 127.0.0.1:51234\r\nConnection: close\r\n\r\n");
    }

    async fn ask(port: u16, request: &[u8]) -> String {
        let mut s = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        s.write_all(request).await.unwrap();
        let mut out = Vec::new();
        s.read_to_end(&mut out).await.unwrap();
        String::from_utf8_lossy(&out).into_owned()
    }

    #[tokio::test]
    async fn a_request_reaches_the_engine_and_the_answer_comes_back() {
        // a stand-in for whisper-server: checks what arrived, answers, closes
        let engine = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let engine_port = engine.local_addr().unwrap().port();
        let seen = tokio::spawn(async move {
            let (mut s, _) = engine.accept().await.unwrap();
            let mut got = Vec::new();
            let mut chunk = [0u8; 1024];
            while !String::from_utf8_lossy(&got).ends_with("RIFFdata") {
                let n = s.read(&mut chunk).await.unwrap();
                assert!(n > 0, "the body never arrived");
                got.extend_from_slice(&chunk[..n]);
            }
            let body = r#"{"text":"hello"}"#;
            s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            s.shutdown().await.unwrap();
            String::from_utf8_lossy(&got).into_owned()
        });

        let door = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = door.local_addr().unwrap().port();
        let (_stop, rx) = oneshot::channel();
        let up: Upstream = Arc::new(move || Ok(engine_port));
        tokio::spawn(serve(door, port, up, rx));

        let req = format!("POST /inference HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: keep-alive\r\nContent-Length: 8\r\n\r\nRIFFdata");
        let answer = ask(port, req.as_bytes()).await;
        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        assert!(answer.ends_with(r#"{"text":"hello"}"#), "{answer}");
        let arrived = seen.await.unwrap();
        assert!(arrived.contains(&format!("Host: 127.0.0.1:{engine_port}\r\n")), "{arrived}");
        assert!(arrived.contains("Connection: close\r\n"), "{arrived}");
        assert!(!arrived.contains("keep-alive"), "{arrived}");
    }

    #[tokio::test]
    async fn a_loading_engine_says_so_and_a_page_is_turned_away() {
        let door = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = door.local_addr().unwrap().port();
        let (_stop, rx) = oneshot::channel();
        let up: Upstream = Arc::new(|| Err("loading"));
        tokio::spawn(serve(door, port, up, rx));

        let loading = ask(port, format!("POST /inference HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\n\r\n").as_bytes()).await;
        assert!(loading.starts_with("HTTP/1.1 503"), "{loading}");
        assert!(loading.contains("Retry-After: 5"), "{loading}");
        let health = ask(port, format!("GET /health HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n").as_bytes()).await;
        assert!(health.contains(r#""status":"loading""#), "{health}");
        let page = ask(port, format!("POST /inference HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: https://x.example\r\n\r\n").as_bytes()).await;
        assert!(page.starts_with("HTTP/1.1 403"), "{page}");
    }
}
