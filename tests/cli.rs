//! 실제 바이너리 → 루프백 HTTP → JSON/HTML/SSE 회귀 검사. 에이전트는 실행하지 않는다.
use std::{fs, io::{BufRead, BufReader, Read, Write}, net::TcpStream,
    path::PathBuf, process::{Child, Command, Stdio}, sync::mpsc, time::{Duration, SystemTime, UNIX_EPOCH}};

struct Fixture { dir: PathBuf, child: Option<Child> }
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child { let _ = child.kill(); let _ = child.wait(); }
        // 이 테스트가 temp_dir 안에 직접 만든 고유 디렉터리만 정리한다.
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn request(address: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(stream, "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n").unwrap();
    let mut text = String::new();
    stream.read_to_string(&mut text).unwrap();
    assert!(text.starts_with("HTTP/1.1 200 OK"));
    text
}

#[test]
fn native_binary_serves_transcript_without_process_and_streams_updates() {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut fixture = Fixture { dir: std::env::temp_dir().join(format!("waid-cli-{}-{suffix}", std::process::id())), child: None };
    let adapters = fixture.dir.join("adapters");
    let transcripts = fixture.dir.join("transcripts");
    fs::create_dir_all(&adapters).unwrap();
    fs::create_dir_all(&transcripts).unwrap();
    fs::write(adapters.join("fixture.toml"), format!(
        "[adapter]\nname = \"fixture\"\ndisplay = \"Fixture Agent\"\nexec = [\"never-run-waid-fixture\"]\n[transcript]\ndir = \"{}\"\n",
        transcripts.to_string_lossy().replace('\\', "/"),
    )).unwrap();
    let log = transcripts.join("session.jsonl");
    fs::write(&log, "{\"type\":\"user\",\"cwd\":\"project\",\"message\":{\"content\":\"MVP fixture\"}}\n{\"type\":\"assistant\",\"message\":{\"model\":\"test-model\",\"stop_reason\":\"end_turn\"}}\n").unwrap();
    fixture.child = Some(Command::new(env!("CARGO_BIN_EXE_waid"))
        .args(["--agent", "fixture", "--html", "--watch", "--port", "0", "--interval", "1"])
        .env("WAID_ADAPTERS", &adapters).stderr(Stdio::piped()).stdout(Stdio::null()).spawn().unwrap());
    let stderr = fixture.child.as_mut().unwrap().stderr.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        BufReader::new(stderr).read_line(&mut line).unwrap();
        let _ = tx.send(line);
    });
    let ready = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let url = ready.split_whitespace().find(|s| s.starts_with("http://127.0.0.1:")).unwrap();
    assert!(!url.ends_with(":0"));
    let address = url.strip_prefix("http://").unwrap();
    let json = request(address, "/snapshot.json");
    for expected in ["\"schema\": 1", "\"state\": \"waiting\"", "\"pid\": null", "\"confidence\": \"inferred\"", "MVP fixture"] {
        assert!(json.contains(expected), "missing {expected}");
    }
    let html = request(address, "/");
    for expected in ["project", "Fixture Agent", "MVP fixture", "test-model", "내 차례"] {
        assert!(html.contains(expected), "missing {expected}");
    }
    assert!(!html.contains("{{"));
    let mut events = TcpStream::connect(address).unwrap();
    events.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(events, "GET /events HTTP/1.1\r\nHost: {address}\r\n\r\n").unwrap();
    let mut events = BufReader::new(events);
    let mut line = String::new();
    loop {
        line.clear(); assert!(events.read_line(&mut line).unwrap() > 0);
        if line.starts_with("data: tick") { break; }
    }
    let mut log = fs::OpenOptions::new().append(true).open(log).unwrap();
    writeln!(log, "{}", r#"{"type":"user","message":{"content":"next turn"}}"#).unwrap();
    drop(log);
    loop {
        line.clear(); assert!(events.read_line(&mut line).unwrap() > 0);
        if line.starts_with("data: tick") { break; }
    }
    assert!(request(address, "/snapshot.json").contains("\"state\": \"working\""));
}
