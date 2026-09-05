#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    io::{self, BufRead, BufReader, Read},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

#[cfg(windows)]
mod native;

type Updates = Arc<Mutex<Option<Result<Vec<Row>, String>>>>;
const MAX_SNAPSHOT: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    id: String,
    title: String,
    agent: String,
    model: String,
    task: String,
    state: String,
}

impl Row {
    fn status(&self) -> &str {
        match self.state.as_str() {
            "waiting" => "내 차례", "working" => "작업 중", "idle" => "유휴",
            "done" => "완료", "error" => "오류", _ => "—",
        }
    }

    fn accessible_text(&self) -> String {
        format!("{} · {}\n{}\n{} · {}", self.title, self.status(), self.task, self.agent, self.model)
    }
}

struct Core {
    child: Child,
    reader: Option<JoinHandle<()>>,
}

impl Drop for Core {
    fn drop(&mut self) {
        let _ = self.child.kill(); // 자신이 띄운 waid만. 관찰 대상 에이전트는 건드리지 않는다.
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

fn start_core(exe: &Path) -> io::Result<(Core, Updates)> {
    let mut command = Command::new(exe);
    command.args(["--json", "--watch"])
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let child = command.spawn()?;
    let mut core = Core { child, reader: None };
    let stdout = core.child.stdout.take().ok_or_else(|| io::Error::other("missing core stdout"))?;
    let updates = Updates::default();
    let output = Arc::clone(&updates);
    core.reader = Some(std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let (message, ended) = match read_snapshot(&mut reader) {
                Ok(Some(rows)) => (Ok(rows), false),
                Ok(None) => (Err("코어가 종료됐습니다. 앱을 다시 실행하세요.".into()), true),
                Err(e) => (Err(format!("상태를 읽지 못했습니다: {e}")), true),
            };
            *output.lock().unwrap_or_else(|e| e.into_inner()) = Some(message);
            if ended { break; }
        }
    }));
    Ok((core, updates))
}

fn read_snapshot(reader: &mut impl BufRead) -> io::Result<Option<Vec<Row>>> {
    let mut line = Vec::new();
    if reader.take(MAX_SNAPSHOT + 1).read_until(b'\n', &mut line)? == 0 {
        return Ok(None);
    }
    if line.len() as u64 > MAX_SNAPSHOT {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "snapshot exceeds 2 MiB"));
    }
    snapshot_rows(&line).map(Some)
}

fn snapshot_rows(bytes: &[u8]) -> io::Result<Vec<Row>> {
    let snapshot: serde_json::Value = serde_json::from_slice(bytes)?;
    if snapshot["schema"] != 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "unsupported schema"));
    }
    let sessions = snapshot["sessions"].as_array()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing sessions"))?;
    Ok(sessions.iter().map(|s| {
        let inferred = if s["task"]["confidence"] == "inferred" { "≈ " } else { "" };
        Row {
            id: field(&s["id"], 128), title: field(&s["title"], 160),
            agent: field(&s["agent"]["display"], 48), model: field(&s["llm"]["display"], 80),
            task: format!("{inferred}{}", field(&s["task"]["text"], 200)),
            state: field(&s["status"]["state"], 16),
        }
    }).collect())
}

fn field(value: &serde_json::Value, limit: usize) -> String {
    let text = value.as_str().unwrap_or("").replace('\0', "")
        .split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() { return "—".into(); }
    let mut chars = text.chars();
    let mut shortened: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() { shortened.push('…'); }
    shortened
}

#[cfg(windows)]
fn main() {
    if let Err(error) = native::run() { native::show_error(&error.to_string()); }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("이 네이티브 껍데기는 현재 Windows용입니다. 다른 OS에서는 waid 코어를 사용하세요.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_fields_preserve_unknowns_and_inference() {
        let value = serde_json::json!({"schema":1,"sessions":[{
            "title":"repo\nmain", "agent":{"display":"Codex"}, "llm":{"display":null},
            "task":{"text":"한글\t작업", "confidence":"inferred"}, "status":{"state":"waiting"}
        }, {}]});
        let rows = snapshot_rows(value.to_string().as_bytes()).unwrap();
        assert_eq!(rows[0].accessible_text(), "repo main · 내 차례\n≈ 한글 작업\nCodex · —");
        assert_eq!(rows[1].accessible_text(), "— · —\n—\n— · —");
        assert_eq!(field(&serde_json::json!("가나다"), 2), "가나…");
        assert_eq!(field(&serde_json::json!(" \n "), 2), "—");
        assert_eq!(field(&serde_json::json!("\0"), 2), "—");
        assert!(snapshot_rows(br#"{"schema":1,"sessions":[]}"#).unwrap().is_empty());
    }

    #[test]
    fn stream_framing_and_size_are_checked() {
        let mut stream = &b"{\"schema\":1,\"sessions\":[]}\n{\"schema\":2,\"sessions\":[]}\n"[..];
        assert!(read_snapshot(&mut stream).unwrap().is_some());
        assert!(read_snapshot(&mut stream).is_err());
        assert!(read_snapshot(&mut stream).unwrap().is_none());
        assert!(snapshot_rows(br#"{"schema":1}"#).is_err());
        let oversized = vec![b' '; (MAX_SNAPSHOT + 1) as usize];
        assert!(read_snapshot(&mut oversized.as_slice()).is_err());
    }

    #[test]
    fn bundled_core_streams_and_stops_with_its_owner() {
        let test_exe = std::env::current_exe().unwrap();
        let exe = test_exe.parent().unwrap().parent().unwrap()
            .join(if cfg!(windows) { "waid.exe" } else { "waid" });
        let (core, updates) = start_core(&exe).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Some(rows) = updates.lock().unwrap().take() {
                rows.expect("core must produce a valid snapshot");
                break;
            }
            assert!(std::time::Instant::now() < deadline, "core produced no snapshot");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        drop(core);
    }
}
