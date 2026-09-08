//! 실제 바이너리 → 루프백 HTTP → JSON/HTML/SSE 회귀 검사. 에이전트는 실행하지 않는다.
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[allow(dead_code)]
#[path = "../src/json.rs"]
mod json;

struct Fixture {
    dir: PathBuf,
    child: Option<Child>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        // 이 테스트가 temp_dir 안에 직접 만든 고유 디렉터리만 정리한다.
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn request(address: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut text = String::new();
    stream.read_to_string(&mut text).unwrap();
    assert!(text.starts_with("HTTP/1.1 200 OK"));
    text
}

#[test]
fn native_binary_serves_transcript_without_process_and_streams_updates() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut fixture = Fixture {
        dir: std::env::temp_dir().join(format!("waid-cli-{}-{suffix}", std::process::id())),
        child: None,
    };
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
    fixture.child = Some(
        Command::new(env!("CARGO_BIN_EXE_waid"))
            .current_dir(&fixture.dir)
            .args([
                "--agent",
                "fixture",
                "--html",
                "--watch",
                "--port",
                "0",
                "--interval",
                "1",
            ])
            .env("WAID_ADAPTERS", &adapters)
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let stderr = fixture.child.as_mut().unwrap().stderr.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        BufReader::new(stderr).read_line(&mut line).unwrap();
        let _ = tx.send(line);
    });
    let ready = rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let url = ready
        .split_whitespace()
        .find(|s| s.starts_with("http://127.0.0.1:"))
        .unwrap();
    assert!(!url.ends_with(":0"));
    let address = url.strip_prefix("http://").unwrap();
    let json = request(address, "/snapshot.json");
    for expected in [
        "\"schema\": 2",
        "\"state\": \"waiting\"",
        "\"pid\": null",
        "\"confidence\": \"inferred\"",
        "MVP fixture",
    ] {
        assert!(json.contains(expected), "missing {expected}");
    }
    let html = request(address, "/");
    for expected in [
        "project",
        "Fixture Agent",
        "MVP fixture",
        "test-model",
        "내 차례",
    ] {
        assert!(html.contains(expected), "missing {expected}");
    }
    assert!(!html.contains("{{"));
    let mut events = TcpStream::connect(address).unwrap();
    events
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(events, "GET /events HTTP/1.1\r\nHost: {address}\r\n\r\n").unwrap();
    let mut events = BufReader::new(events);
    let mut line = String::new();
    loop {
        line.clear();
        assert!(events.read_line(&mut line).unwrap() > 0);
        if line.starts_with("data: tick") {
            break;
        }
    }
    let mut log = fs::OpenOptions::new().append(true).open(log).unwrap();
    writeln!(
        log,
        "{}",
        r#"{"type":"user","message":{"content":"next turn"}}"#
    )
    .unwrap();
    drop(log);
    loop {
        line.clear();
        assert!(events.read_line(&mut line).unwrap() > 0);
        if line.starts_with("data: tick") {
            break;
        }
    }
    assert!(request(address, "/snapshot.json").contains("\"state\": \"working\""));
}

#[test]
fn configured_codex_home_streams_turn_transitions_and_partial_writes() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut fixture = Fixture {
        dir: std::env::temp_dir().join(format!("waid-codex-{}-{suffix}", std::process::id())),
        child: None,
    };
    let home = fixture.dir.join("home");
    let codex = fixture.dir.join("한글 Codex home");
    let sessions = codex.join("sessions/2026/09/06");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&sessions).unwrap();
    let log = sessions.join("session.jsonl");
    fs::write(&log, concat!(
        "{\"type\":\"session_meta\",\"payload\":{\"cwd\":\"project\"}}\n",
        "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"실제 작업 검증\"}]}}\n",
        "{\"type\":\"turn_context\",\"payload\":{\"model\":\"model-one\"}}\n",
        "{\"type\":\"event_msg\",\"payload\":{\"type\":\"task_complete\"}}\n",
    )).unwrap();
    fixture.child = Some(
        Command::new(env!("CARGO_BIN_EXE_waid"))
            .current_dir(&fixture.dir)
            .args(["--agent", "codex", "--json", "--watch", "--interval", "1"])
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env("CODEX_HOME", &codex)
            .env("WAID_ADAPTERS", fixture.dir.join("no-adapters"))
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let stdout = fixture.child.as_mut().unwrap().stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if tx.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    let expect = |state: &str, model: &str| {
        let deadline = std::time::Instant::now() + Duration::from_secs(6);
        loop {
            let line = rx
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .expect("core must emit the expected state within six seconds");
            if line.contains(&format!("\"state\":\"{state}\"")) && line.contains(model) {
                assert!(line.contains("실제 작업 검증"));
                assert!(line.contains("\"title\":\"project\""));
                assert!(line.contains("\"confidence\":\"inferred\""));
                assert!(line.contains("\"pid\":null"));
                break;
            }
        }
    };
    expect("waiting", "model-one");
    let mut output = fs::OpenOptions::new().append(true).open(&log).unwrap();
    writeln!(
        output,
        "{}",
        r#"{"type":"event_msg","payload":{"type":"task_started"}}"#
    )
    .unwrap();
    writeln!(
        output,
        "{}",
        r#"{"type":"turn_context","payload":{"model":"model-two"}}"#
    )
    .unwrap();
    output.flush().unwrap();
    expect("working", "model-two");
    write!(
        output,
        "{{\"type\":\"event_msg\",\"payload\":{{\"type\":\"task_"
    )
    .unwrap();
    output.flush().unwrap();
    expect("working", "model-two"); // Incomplete records must not invent a state.
    writeln!(output, "complete\"}}}}").unwrap();
    output.flush().unwrap();
    expect("waiting", "model-two");
    writeln!(
        output,
        "{}",
        r#"{"type":"event_msg","payload":{"type":"turn_aborted"}}"#
    )
    .unwrap();
    output.flush().unwrap();
    expect("idle", "model-two");
    writeln!(
        output,
        "{}",
        r#"{"type":"error","message":"fixture failure"}"#
    )
    .unwrap();
    output.flush().unwrap();
    expect("error", "model-two");
    writeln!(
        output,
        "{}",
        r#"{"type":"event_msg","payload":{"type":"task_started"}}"#
    )
    .unwrap();
    output.flush().unwrap();
    expect("working", "model-two");
}

#[test]
fn history_includes_old_idle_and_metadata_only_logs_without_completing_them() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture = Fixture {
        dir: std::env::temp_dir().join(format!("waid-history-{}-{suffix}", std::process::id())),
        child: None,
    };
    let adapters = fixture.dir.join("adapters");
    let logs = fixture.dir.join("logs");
    fs::create_dir_all(&adapters).unwrap();
    fs::create_dir_all(&logs).unwrap();
    fs::write(adapters.join("fixture.toml"), format!(
        "[adapter]\nname = \"fixture\"\ndisplay = \"History Fixture\"\nexec = [\"never-run-waid-fixture\"]\n[transcript]\ndir = \"{}\"\n",
        logs.to_string_lossy().replace('\\', "/")
    )).unwrap();
    for (name, event) in [
        ("old-idle", "turn_aborted"),
        ("old-waiting", "task_complete"),
    ] {
        let path = logs.join(format!("{name}.jsonl"));
        fs::write(&path, format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"{name}\",\"cwd\":\"history-project\"}}}}\n{{\"type\":\"response_item\",\"timestamp\":\"2020-01-01T00:00:00Z\",\"payload\":{{\"type\":\"message\",\"role\":\"user\",\"content\":[{{\"type\":\"input_text\",\"text\":\"{name}\"}}]}}}}\n{{\"type\":\"event_msg\",\"timestamp\":\"2020-01-01T00:00:01Z\",\"payload\":{{\"type\":\"{event}\"}}}}\n{{\"type\":\"turn_context\",\"payload\":{{\"model\":\"metadata-only\"}}}}\n"
        )).unwrap();
        if name == "old-idle" {
            fs::File::options()
                .write(true)
                .open(path)
                .unwrap()
                .set_times(
                    fs::FileTimes::new()
                        .set_modified(UNIX_EPOCH + Duration::from_secs(1_577_836_801)),
                )
                .unwrap();
        }
    }
    let run = |history| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_waid"));
        command.current_dir(&fixture.dir);
        command
            .args(["--json", "--agent", "fixture"])
            .env("WAID_ADAPTERS", &adapters);
        if history {
            command.arg("--history");
        }
        let output = command.output().unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    };
    let recent = run(false);
    assert!(!recent.contains("old-idle") && !recent.contains("old-waiting"));
    let history = run(true);
    assert!(history.contains("old-idle") && history.contains("old-waiting"));
    assert!(history.contains("\"state\": \"idle\"") && history.contains("\"state\": \"waiting\""));
    assert!(!history.contains("\"state\": \"done\""));
    assert!(history.contains("2020-01-01T00:00:01Z"));
}

#[test]
fn copilot_default_and_configured_roots_discover_only_session_logs() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture = Fixture {
        dir: std::env::temp_dir().join(format!(
            "waid-copilot-roots-{}-{suffix}",
            std::process::id()
        )),
        child: None,
    };
    let home = fixture.dir.join("home");
    let configured = fixture.dir.join("한글 Copilot home");
    for (root, id) in [
        (home.join(".copilot"), "default-session"),
        (configured.clone(), "configured-session"),
    ] {
        let session = root.join("session-state").join(id);
        fs::create_dir_all(&session).unwrap();
        fs::write(session.join("events.jsonl"), format!(concat!(
            "{{\"type\":\"session.start\",\"data\":{{\"sessionId\":\"{}\",\"selectedModel\":\"test-model\"}}}}\n",
            "{{\"type\":\"user.message\",\"data\":{{\"content\":\"root discovery fixture\"}}}}\n",
            "{{\"type\":\"session.idle\",\"data\":{{}}}}\n"
        ), id)).unwrap();
        fs::write(
            session.join("telemetry.jsonl"),
            "{\"type\":\"user.message\",\"data\":{\"content\":\"not-a-session\"}}\n",
        )
        .unwrap();
    }
    let output = Command::new(env!("CARGO_BIN_EXE_waid"))
        .current_dir(&fixture.dir)
        .args(["--agent", "copilot", "--json", "--history"])
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("COPILOT_HOME", &configured)
        .env("WAID_ADAPTERS", fixture.dir.join("no-adapters"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let json = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        json.matches("root discovery fixture").count(),
        6,
        "task + summary + prompt for two sessions"
    );
    assert_eq!(json.matches("\"state\": \"waiting\"").count(), 2);
    assert!(!json.contains("not-a-session"));
}

#[test]
fn partial_json_collection_warns_preserves_rows_and_recovers() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut fixture = Fixture {
        dir: std::env::temp_dir().join(format!("waid-warnings-{}-{suffix}", std::process::id())),
        child: None,
    };
    let adapters = fixture.dir.join("adapters");
    let logs = fixture.dir.join("logs");
    let home = fixture.dir.join("home");
    for path in [&adapters, &logs, &home] {
        fs::create_dir_all(path).unwrap();
    }
    fs::write(adapters.join("fixture.toml"), format!(
        "[adapter]\nname = \"fixture\"\ndisplay = \"Warning Fixture\"\nexec = [\"never-run-waid-fixture\"]\n[transcript]\njson_dir = \"{}\"\n",
        logs.to_string_lossy().replace('\\', "/")
    )).unwrap();
    fs::write(logs.join("healthy.json"), r#"[{"role":"user","content":"healthy request","cwd":"file:///C:/%ED%95%9C%EA%B8%80/repo"}]"#).unwrap();
    let broken = logs.join("broken.json");
    fs::write(&broken, "{invalid-json\n").unwrap();
    fixture.child = Some(
        Command::new(env!("CARGO_BIN_EXE_waid"))
            .current_dir(&fixture.dir)
            .args([
                "--agent",
                "fixture",
                "--json",
                "--history",
                "--watch",
                "--interval",
                "1",
            ])
            .env("WAID_ADAPTERS", &adapters)
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env("APPDATA", &home)
            .env("LOCALAPPDATA", &home)
            .env("CODEX_HOME", &home)
            .env("COPILOT_HOME", &home)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let stdout = fixture.child.as_mut().unwrap().stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if tx.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    let first = json::parse(&rx.recv_timeout(Duration::from_secs(10)).unwrap()).unwrap();
    let rows = first
        .get("sessions")
        .and_then(json::Json::as_array)
        .unwrap();
    assert_eq!(
        rows.len(),
        1,
        "a broken source must not discard the healthy session"
    );
    assert_eq!(
        rows[0].get("cwd").and_then(json::Json::as_str),
        Some("C:/한글/repo")
    );
    let warnings = first
        .get("warnings")
        .and_then(json::Json::as_array)
        .unwrap();
    assert!(warnings
        .iter()
        .any(|w| w.as_str().is_some_and(|s| s.contains("broken.json"))));
    fs::write(&broken, r#"[{"role":"user","content":"repaired request"}]"#).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let line = rx
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
            .expect("the collector must recover after repairing the source");
        let snapshot = json::parse(&line).unwrap();
        if snapshot
            .get("sessions")
            .and_then(json::Json::as_array)
            .unwrap()
            .len()
            == 2
        {
            assert!(
                snapshot
                    .get("warnings")
                    .and_then(json::Json::as_array)
                    .is_none_or(|w| w.is_empty()),
                "warnings must clear after a successful rescan"
            );
            break;
        }
    }
}
