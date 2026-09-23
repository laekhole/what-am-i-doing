//! `--html --watch` 를 위한 최소 HTTP 서버.
//!
//! 매니페스트 §6: "정적 파일 서빙 외에 서버 기능은 없다." 라우트는 셋뿐이고
//! 인증도, 업로드도, 프록시도, 상태 저장도 없다. 127.0.0.1 에만 바인딩한다 —
//! 이건 대시보드지 서비스가 아니다.
//!
//! 갱신 방식: SSE 로 "바뀌었다"는 신호만 보낸다. 실제 HTML 은 클라이언트가
//! `/` 를 다시 받아 간다. 템플릿 렌더링이 서버 한 곳에만 존재하므로,
//! 사용자가 템플릿을 어떻게 갈아엎든 갱신이 따라온다.

use std::io::{BufRead, BufReader, Write};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use crate::session::{self, Session};

type Snap = Arc<dyn Fn() -> (Vec<Session>, i64) + Send + Sync>;

/// 세션의 "의미 있는" 상태만 지문으로 만든다.
///
/// 전체 JSON 을 해싱하면 안 된다 — `captured_at` 이 매초 바뀌므로 아무것도
/// 변하지 않았는데도 매 틱마다 갱신을 쏘게 된다. 사용자가 보는 화면이
/// 달라질 때만 신호를 보내는 것이 목적이다.
fn fingerprint(sessions: &[Session]) -> u64 {
    let mut hash = DefaultHasher::new();
    for x in sessions {
        // Keep every field exposed by html::session_ctx, including custom templates.
        (
            &x.id,
            &x.title,
            (x.agent.name, x.agent.display),
            (&x.llm_display, &x.llm_id),
            (&x.task.text, x.task.source, x.task.confidence.id()),
            x.state.id(),
            x.since,
            (&x.cwd, &x.branch, x.pid),
        ).hash(&mut hash);
    }
    hash.finish()
}

pub fn serve(port: u16, tpl: String, interval: u64, snap: Snap) -> Result<(), String> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)
        .map_err(|e| crate::trf!("{addr} 바인딩 실패: {e} — 다른 포트를 쓰려면 --port", "Cannot bind {addr}: {e} — use --port to choose another port"))?;

    let addr = listener.local_addr().map_err(|e| e.to_string())?;
    eprintln!("{}", crate::trf!("waid → http://{addr}  (Ctrl-C 로 종료)", "waid → http://{addr}  (Ctrl-C to stop)"));

    let tpl = Arc::new(tpl);
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let tpl = Arc::clone(&tpl);
        let snap = Arc::clone(&snap);
        std::thread::spawn(move || {
            let _ = handle(stream, &tpl, interval, snap);
        });
    }
    Ok(())
}

fn handle(mut stream: TcpStream, tpl: &str, interval: u64, snap: Snap) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let port = stream.local_addr()?.port();
    let path = match request_path(BufReader::new(stream.try_clone()?), port) {
        Ok(path) => path,
        Err(status) => return write_response(&mut stream, status, "text/plain", status),
    };

    match path.split('?').next().unwrap_or("/") {
        "/" => {
            let (sessions, now) = snap();
            let language = path.split_once('?')
                .and_then(|(_, query)| query.split('&').find_map(|part| part.strip_prefix("lang=")))
                .and_then(crate::i18n::Language::from_code)
                .unwrap_or_else(crate::i18n::language);
            let body = crate::html::render_language(&sessions, now, tpl, language);
            write_response(&mut stream, "200 OK", "text/html; charset=utf-8", &body)
        }
        "/snapshot.json" => {
            let (sessions, now) = snap();
            let body = session::to_json(&sessions, now, true);
            write_response(&mut stream, "200 OK", "application/json; charset=utf-8", &body)
        }
        "/events" => events(stream, interval, snap),
        _ => write_response(&mut stream, "404 Not Found", "text/plain; charset=utf-8", "not found\n"),
    }
}

fn request_path(reader: impl BufRead, port: u16) -> Result<String, &'static str> {
    // Bound total header memory even when a client sends no newline.
    let mut reader = reader.take(16 * 1024);
    let read_error = |error: std::io::Error| match error.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => "408 Request Timeout",
        _ => "400 Bad Request",
    };
    let mut line = String::new();
    reader.read_line(&mut line).map_err(read_error)?;
    let mut request = line.split_whitespace();
    let method = request.next().unwrap_or("");
    let path = request.next().unwrap_or("").to_string();
    if !line.ends_with('\n') || !path.starts_with('/')
        || !matches!(request.next(), Some("HTTP/1.0" | "HTTP/1.1"))
        || request.next().is_some()
    {
        return Err("400 Bad Request");
    }
    if method != "GET" {
        return Err("405 Method Not Allowed");
    }
    let mut host = None;
    let mut h = String::new();
    loop {
        h.clear();
        let read = reader.read_line(&mut h).map_err(read_error)?;
        if reader.limit() == 0 {
            return Err("431 Request Header Fields Too Large");
        }
        if read == 0 || !h.ends_with('\n') {
            return Err("400 Bad Request");
        }
        if h.trim().is_empty() { break; }
        let Some((name, value)) = h.split_once(':') else {
            return Err("400 Bad Request");
        };
        if name.eq_ignore_ascii_case("host") {
            if host.is_some() {
                return Err("400 Bad Request");
            }
            host = Some(value.trim().to_ascii_lowercase());
        }
    }
    // Loopback binding alone does not reject a browser's DNS-rebound hostname.
    let host = host.as_deref().unwrap_or("");
    if host != format!("127.0.0.1:{port}") && host != format!("localhost:{port}")
        && !(port == 80 && matches!(host, "127.0.0.1" | "localhost"))
    {
        return Err("403 Forbidden");
    }
    Ok(path)
}

fn write_response(stream: &mut TcpStream, status: &str, ctype: &str, body: &str) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n{}\r\n{}",
        status, ctype,
        body.len(),
        if status.starts_with("405 ") { "Allow: GET\r\n" } else { "" },
        body
    )
}

fn events(mut stream: TcpStream, interval: u64, snap: Snap) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/event-stream\r\n\
         Cache-Control: no-store\r\n\
         Connection: keep-alive\r\n\r\n"
    )?;
    stream.flush()?;

    let mut last = 0u64;
    let mut quiet = 0u64;

    loop {
        let (sessions, _now) = snap();
        let h = fingerprint(&sessions);

        if h != last {
            last = h;
            quiet = 0;
            // 페이로드는 무의미하다. "다시 가져가라"는 신호일 뿐.
            stream.write_all(b"data: tick\n\n")?;
            stream.flush()?;
        } else {
            quiet += interval;
            // 프록시·브라우저가 유휴 연결을 끊지 않도록 주석 프레임.
            if quiet >= 15 {
                quiet = 0;
                stream.write_all(b": keepalive\n\n")?;
                stream.flush()?;
            }
        }

        std::thread::sleep(Duration::from_secs(interval));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Confidence, State, Task};

    #[test]
    fn http_only_serves_bounded_get_requests_for_its_local_host() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let cases = [
            ("GET /snapshot.json HTTP/1.1\r\nHost: {address}\r\n\r\n".to_string(), "200", 1),
            ("GET / HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n".to_string(), "200", 1),
            ("GET /snapshot.json HTTP/1.1\r\nHost: attacker.example\r\n\r\n".to_string(), "403", 0),
            ("GET /events HTTP/1.1\r\nHost: 127.0.0.1:1\r\n\r\n".to_string(), "403", 0),
            ("GET / HTTP/1.1\r\n\r\n".to_string(), "403", 0),
            ("POST / HTTP/1.1\r\nHost: {address}\r\n\r\n".to_string(), "405", 0),
            ("GET / HTTP/1.1\r\nHost: {address}\r\nHost: {address}\r\n\r\n".to_string(), "400", 0),
            ("GET / HTTP/1.1\r\nHost: {address}".to_string(), "400", 0),
            ("GET / HTTP/1.1\r\nHost: {address}\r\nX: ".to_string() + &"x".repeat(17 * 1024), "431", 0),
        ];
        for (request, status, expected_calls) in cases {
            let raw = request.replace("{address}", "127.0.0.1:7423").replace("{port}", "7423");
            let parsed = request_path(raw.as_bytes(), 7423);
            match parsed {
                Ok(_) => assert_eq!(status, "200"),
                Err(error) => assert!(error.starts_with(status), "{error}"),
            }
            // Test malformed bytes above directly: local HTTP proxies may normalize
            // duplicate headers or buffer incomplete requests before forwarding.
            if request.matches("Host:").count() > 1 || !request.ends_with("\r\n\r\n") {
                continue;
            }
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let calls = Arc::new(AtomicUsize::new(0));
            let snap_calls = Arc::clone(&calls);
            let worker = std::thread::spawn(move || {
                let (stream, _) = listener.accept().unwrap();
                let _ = handle(stream, "dashboard", 1, Arc::new(move || {
                    snap_calls.fetch_add(1, Ordering::Relaxed);
                    (Vec::new(), 0)
                }));
            });
            let mut stream = TcpStream::connect(address).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
            stream.write_all(request.replace("{address}", &address.to_string())
                .replace("{port}", &address.port().to_string()).as_bytes()).unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            let mut response = String::new();
            BufReader::new(stream).read_line(&mut response).unwrap();
            worker.join().unwrap();
            assert!(response.starts_with(&format!("HTTP/1.1 {status}")), "expected {status}, got {response:?}; request prefix: {:?}", &request[..request.len().min(100)]);
            assert_eq!(calls.load(Ordering::Relaxed), expected_calls);
        }
    }

    #[test]
    fn fingerprint_covers_template_fields_and_keys_match_renderer() {
        let session = Session {
            context: crate::ContextUsage::default(), prompt: None, last_answer: None,
            session_id: None, summary: None, request_marker: None, request_at: None,
            auxiliary: false, evidence: "unknown", id: "session".into(),
            legacy_id: "old".into(), title: "project".into(),
            agent: crate::adapters::Agent { name: "codex", display: "Codex", has_reader: true },
            llm_id: None, llm_display: None,
            task: Task { text: Some("task".into()), source: "none", confidence: Confidence::None },
            state: State::Waiting, since: 100, cwd: None, branch: None, pid: None,
        };
        let original = fingerprint(std::slice::from_ref(&session));
        let changes: &[fn(&mut Session)] = &[
            |s| s.agent.display = "Agent",
            |s| s.llm_id = Some("model".into()),
            |s| s.task.source = "env",
            |s| s.task.confidence = Confidence::Explicit,
            |s| s.cwd = Some("other".into()),
            |s| s.branch = Some("main".into()),
            |s| s.pid = Some(1),
        ];
        for change in changes {
            let mut changed = session.clone();
            change(&mut changed);
            assert_ne!(original, fingerprint(&[changed]));
        }
        assert_eq!(original, fingerprint(std::slice::from_ref(&session)));
        let actual = crate::html::context(&[session], 120);
        let keys = crate::html::key_context();
        assert_eq!(actual.keys().collect::<Vec<_>>(), keys.keys().collect::<Vec<_>>());
        let (Some(crate::tmpl::Val::List(actual)), Some(crate::tmpl::Val::List(keys))) =
            (actual.get("sessions"), keys.get("sessions")) else { panic!("session context missing") };
        assert_eq!(actual[0].keys().collect::<Vec<_>>(), keys[0].keys().collect::<Vec<_>>());
        assert!(keys[0].contains_key("status.unknown"));
    }
}
