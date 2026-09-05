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
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use crate::session::{self, Session};

type Snap = Arc<dyn Fn() -> (Vec<Session>, i64) + Send + Sync>;

/// FNV-1a. 스냅샷이 실제로 바뀌었을 때만 SSE 를 쏘기 위한 것이라
/// 암호학적 성질은 필요 없다.
fn hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in s.as_bytes() {
        h ^= *byte as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

/// 세션의 "의미 있는" 상태만 지문으로 만든다.
///
/// 전체 JSON 을 해싱하면 안 된다 — `captured_at` 이 매초 바뀌므로 아무것도
/// 변하지 않았는데도 매 틱마다 갱신을 쏘게 된다. 사용자가 보는 화면이
/// 달라질 때만 신호를 보내는 것이 목적이다.
fn fingerprint(sessions: &[Session]) -> u64 {
    let mut buf = String::with_capacity(sessions.len() * 96);
    for x in sessions {
        buf.push_str(&x.id);
        buf.push('\x1f');
        buf.push_str(&x.title);
        buf.push('\x1f');
        buf.push_str(x.agent.name);
        buf.push('\x1f');
        buf.push_str(x.llm_display.as_deref().unwrap_or(""));
        buf.push('\x1f');
        buf.push_str(x.task.text.as_deref().unwrap_or(""));
        buf.push('\x1f');
        buf.push_str(x.state.id());
        buf.push('\x1f');
        buf.push_str(&x.since.to_string());
        buf.push('\x1e');
    }
    hash(&buf)
}

pub fn serve(port: u16, tpl: String, interval: u64, snap: Snap) -> Result<(), String> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)
        .map_err(|e| format!("{addr} 바인딩 실패: {e} — 다른 포트를 쓰려면 --port"))?;

    let addr = listener.local_addr().map_err(|e| e.to_string())?;
    eprintln!("waid → http://{addr}  (Ctrl-C 로 종료)");

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
    let path = {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
        // 헤더를 끝까지 버린다. 본문은 읽지 않는다 — GET 만 받는다.
        let mut h = String::new();
        while reader.read_line(&mut h)? > 0 && h.trim() != "" {
            h.clear();
        }
        path
    };

    match path.split('?').next().unwrap_or("/") {
        "/" => {
            let (sessions, now) = snap();
            let body = crate::html::render(&sessions, now, tpl);
            write_page(&mut stream, "text/html; charset=utf-8", &body)
        }
        "/snapshot.json" => {
            let (sessions, now) = snap();
            let body = session::to_json(&sessions, now, true);
            write_page(&mut stream, "application/json; charset=utf-8", &body)
        }
        "/events" => events(stream, interval, snap),
        _ => {
            let body = "not found\n";
            write!(
                stream,
                "HTTP/1.1 404 Not Found\r\n\
                 Content-Type: text/plain; charset=utf-8\r\n\
                 Content-Length: {}\r\n\r\n{}",
                body.len(),
                body
            )
        }
    }
}

fn write_page(stream: &mut TcpStream, ctype: &str, body: &str) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 200 OK\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\r\n{}",
        ctype,
        body.len(),
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
