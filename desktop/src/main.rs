#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::BTreeMap,
    io::{self, BufRead, BufReader, Read},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

#[cfg(windows)]
mod native;
mod ui;
#[allow(dead_code)]
#[path = "../../src/time.rs"]
mod time;
#[cfg(windows)]
mod activate;
#[cfg(windows)]
mod visual;

type Updates = Arc<Mutex<Option<Result<Vec<Row>, String>>>>;
const MAX_SNAPSHOT: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Row {
    last_answer: String,
    session_id: String,
    agent_id: String,
    request_marker: String,
    request_at: Option<i64>,
    summary: String,
    cwd: String,
    since: String,
    evidence: String,
    task_source: String,
    auxiliary: bool,
    id: String,
    title: String,
    agent: String,
    model: String,
    task: String,
    state: String,
}

impl Row {
    fn date_label(&self, expanded: bool, now: i64) -> String {
        if !expanded {
            if let Some(age) = time::from_iso8601(&self.since).and_then(|stamp| now.checked_sub(stamp)) {
                match age {
                    0..=59 => return "방금 전".into(),
                    60..=3599 => return format!("{}분 전", age / 60),
                    3600..=86399 => return format!("{}시간 전", age / 3600),
                    _ => {}
                }
            }
        }
        self.short_date().into()
    }
    fn short_date(&self) -> &str {
        let date = self.since.split('T').next().unwrap_or_default();
        match date.as_bytes() {
            [a, b, c, d, b'-', e, f, b'-', g, h]
                if [a, b, c, d, e, f, g, h].iter().all(|c| c.is_ascii_digit()) => &date[2..],
            _ => date,
        }
    }
    fn project(&self) -> &str {
        self.title
            .rsplit_once('·')
            .filter(|(_, suffix)| suffix.len() >= 4 && self.id.starts_with(suffix))
            .map_or(&self.title, |(project, _)| project)
    }
    fn status(&self) -> &str {
        match self.state.as_str() {
            "waiting" => "내 차례",
            "working" => "작업 중",
            "idle" => "유휴",
            "done" => "종결",
            "error" => "오류",
            _ => "미확인",
        }
    }

    fn accessible_text(&self) -> String {
        format!(
            "프로젝트  {}\n태스크  {}\n상태  {}\n{} · {}",
            self.project(),
            self.task,
            self.status(),
            self.agent,
            self.model
        )
    }
}

// Observe every core snapshot before the UI's latest-value slot can replace it.
struct Activity {
    started_at: i64,
    requests: BTreeMap<String, (String, bool)>,
}
impl Activity {
    fn apply(&mut self, rows: &mut [Row]) {
        for row in rows {
            // These sources have no reliable turn boundaries.
            if !["transcript", "inferred_time", "stale"].contains(&row.evidence.as_str())
            {
                continue;
            }
            let (marker, active) = self.requests.entry(row.id.clone()).or_insert_with(|| {
                (row.request_marker.clone(), row.request_at.is_some_and(|at| at >= self.started_at))
            });
            if !row.request_marker.is_empty() && *marker != row.request_marker {
                *marker = row.request_marker.clone();
                *active = true;
            }
            if !*active {
                row.state = "idle".into();
                row.evidence = "before_launch".into();
            } else if row.state == "unknown" && row.evidence == "stale" {
                // No timeout: only a logged completion, interruption or error ends the turn.
                row.state = "working".into();
                row.evidence = "awaiting_completion".into();
            }
        }
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
    let mut activity = Activity {
        started_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs() as i64,
        requests: BTreeMap::new(),
    };
    let mut command = Command::new(exe);
    command
        .args(["--json", "--watch", "--history"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let child = command.spawn()?;
    let mut core = Core {
        child,
        reader: None,
    };
    let stdout = core
        .child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("missing core stdout"))?;
    let updates = Updates::default();
    let output = Arc::clone(&updates);
    core.reader = Some(std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let (message, ended) = match read_snapshot(&mut reader) {
                Ok(Some(mut rows)) => {
                    activity.apply(&mut rows);
                    (Ok(rows), false)
                }
                Ok(None) => (
                    Err("코어가 종료됐습니다. 앱을 다시 실행하세요.".into()),
                    true,
                ),
                Err(e) => (Err(format!("상태를 읽지 못했습니다: {e}")), true),
            };
            *output.lock().unwrap_or_else(|e| e.into_inner()) = Some(message);
            if ended {
                break;
            }
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
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "snapshot exceeds 16 MiB",
        ));
    }
    snapshot_rows(&line).map(Some)
}

fn snapshot_rows(bytes: &[u8]) -> io::Result<Vec<Row>> {
    let snapshot: serde_json::Value = serde_json::from_slice(bytes)?;
    if snapshot["schema"] != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported schema",
        ));
    }
    let sessions = snapshot["sessions"]
        .as_array()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing sessions"))?;
    Ok(sessions
        .iter()
        .map(|s| {
            let inferred = if s["task"]["confidence"] == "inferred" {
                "≈ "
            } else {
                ""
            };
            Row {
                last_answer: field(&s["last_answer"], 4000),
                session_id: s["session_id"].as_str().unwrap_or("").to_string(),
                request_marker: s["request_marker"].as_str().unwrap_or("").to_string(),
                request_at: s["request_at"].as_i64(),
                summary: field(&s["summary"], 4000),
                cwd: field(&s["cwd"], 1000),
                since: field(&s["status"]["since"], 48),
                evidence: field(&s["status"]["evidence"], 48),
                task_source: field(&s["task"]["source"], 48),
                auxiliary: s["auxiliary"].as_bool().unwrap_or(false),
                id: field(&s["id"], 128),
                title: field(&s["title"], 160),
                agent_id: field(&s["agent"]["name"], 48),
                agent: field(&s["agent"]["display"], 48),
                model: field(&s["llm"]["display"], 80),
                task: format!("{inferred}{}", field(&s["task"]["text"], 4000)),
                state: field(&s["status"]["state"], 16),
            }
        })
        .collect())
}

fn field(value: &serde_json::Value, limit: usize) -> String {
    let text = value
        .as_str()
        .unwrap_or("")
        .replace('\0', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if text.is_empty() {
        return "—".into();
    }
    let mut chars = text.chars();
    let mut shortened: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        shortened.push('…');
    }
    shortened
}

#[cfg(windows)]
fn main() {
    if let Err(error) = native::run() {
        native::show_error(&error.to_string());
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("이 네이티브 껍데기는 현재 Windows용입니다. 다른 OS에서는 waid 코어를 사용하세요.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_tracks_requests_since_launch_until_explicit_completion() {
        let mut activity = Activity { started_at: 100, requests: BTreeMap::new() };
        let mut observe = |id: &str, marker: &str, at, state: &str, evidence: &str| {
            let mut rows = vec![Row {
                id: id.into(), request_marker: marker.into(), request_at: at,
                state: state.into(), evidence: evidence.into(), ..Row::default()
            }];
            activity.apply(&mut rows);
            rows.remove(0)
        };
        assert_eq!(observe("old", "a", Some(90), "waiting", "transcript").state, "idle");
        assert_eq!(observe("old", "a", Some(90), "working", "transcript").state, "idle");
        assert_eq!(observe("old", "b", Some(110), "working", "transcript").state, "working");
        assert_eq!(observe("old", "b", Some(110), "unknown", "stale").state, "working");
        assert_eq!(observe("old", "b", Some(110), "waiting", "transcript").state, "waiting");
        assert_eq!(observe("old", "b", Some(110), "waiting", "transcript").state, "waiting");
        // Even an identical prompt has a distinct request marker; fast replies may finish between polls.
        assert_eq!(observe("old", "c", Some(120), "waiting", "transcript").state, "waiting");
        assert_eq!(observe("old", "d", None, "working", "inferred_time").state, "working");
        assert_eq!(observe("old", "d", None, "idle", "transcript").state, "idle");
        assert_eq!(observe("old", "e", Some(130), "error", "transcript").state, "error");
        assert_eq!(observe("new", "a", Some(140), "working", "transcript").state, "working");
        assert_eq!(observe("late-history", "a", Some(80), "waiting", "transcript").state, "idle");
        assert_eq!(observe("unsupported", "a", None, "unknown", "unknown").state, "unknown");
    }

    #[test]
    fn card_dates_use_two_digit_years_and_preserve_unknowns() {
        for (since, expected) in [
            ("2026-09-08T12:34:56Z", "26-09-08"),
            ("2024-02-29", "24-02-29"),
            ("—", "—"),
            ("날짜 없음", "날짜 없음"),
            ("2026-09", "2026-09"),
            ("", ""),
        ] {
            assert_eq!(Row { since: since.into(), ..Row::default() }.short_date(), expected);
        }
    }

    #[test]
    fn collapsed_dates_age_by_minutes_and_hours_but_expanded_dates_stay_absolute() {
        let now = time::from_iso8601("2026-09-08T12:00:00Z").unwrap();
        for (age, expected) in [
            (0, "방금 전"), (59, "방금 전"), (60, "1분 전"),
            (28 * 60, "28분 전"), (50 * 60, "50분 전"), (3599, "59분 전"),
            (3600, "1시간 전"), (7200, "2시간 전"), (86399, "23시간 전"),
            (86400, "26-09-07"), (-1, "26-09-08"),
        ] {
            let row = Row { since: time::to_iso8601(now - age), ..Row::default() };
            assert_eq!(row.date_label(false, now), expected, "age {age}");
            assert_eq!(row.date_label(true, now), row.short_date());
        }
        let row = Row { since: "2026-09-08T20:32:00+09:00".into(), ..Row::default() };
        assert_eq!(row.date_label(false, now), "28분 전");
        let row = Row { since: "—".into(), ..Row::default() };
        assert_eq!(row.date_label(false, now), "—");
    }

    #[test]
    fn project_labels_hide_only_the_session_suffix() {
        let mut row = Row {
            id: "1206c6e9".into(),
            title: "what-am-i-doing·1206".into(),
            ..Row::default()
        };
        assert_eq!(row.project(), "what-am-i-doing");
        assert!(!row.accessible_text().contains("1206"));
        row.title = "project·release".into();
        assert_eq!(row.project(), "project·release");
        row.title = "project·3383".into();
        assert_eq!(row.project(), "project·3383");
    }

    #[test]
    fn harness_identity_is_separate_from_model_and_display_name() {
        let value = serde_json::json!({"schema":1,"sessions":[
            {"session_id":"original-id","agent":{"name":"claude","display":"My coding tool"},"llm":{"display":"gpt-test"}},
            {"agent":{"name":"codex","display":"Custom alias"},"llm":{"display":"claude-test"}},
            {"agent":{"name":"custom","display":"Codex"},"llm":{"display":"gpt-test"}}
        ]});
        let rows = snapshot_rows(value.to_string().as_bytes()).unwrap();
        assert_eq!(rows[0].agent_id, "claude");
        assert_eq!(rows[0].session_id, "original-id");
        assert!(rows[1].session_id.is_empty());
        assert_eq!(rows[1].agent_id, "codex");
        assert_eq!(rows[2].agent_id, "custom");
    }
    #[test]
    fn five_fields_preserve_unknowns_and_inference() {
        let value = serde_json::json!({"schema":1,"sessions":[{
            "title":"repo\nmain", "agent":{"display":"Codex"}, "llm":{"display":null},
            "task":{"text":"한글\t작업", "confidence":"inferred"}, "status":{"state":"waiting"}
        }, {}]});
        let rows = snapshot_rows(value.to_string().as_bytes()).unwrap();
        assert_eq!(
            rows[0].accessible_text(),
            "프로젝트  repo main\n태스크  ≈ 한글 작업\n상태  내 차례\nCodex · —"
        );
        assert_eq!(rows[1].accessible_text(), "프로젝트  —\n태스크  —\n상태  미확인\n— · —");
        assert_eq!(field(&serde_json::json!("가나다"), 2), "가나…");
        assert_eq!(field(&serde_json::json!(" \n "), 2), "—");
        assert_eq!(field(&serde_json::json!("\0"), 2), "—");
        assert!(snapshot_rows(br#"{"schema":1,"sessions":[]}"#)
            .unwrap()
            .is_empty());
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
        let exe = test_exe
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(if cfg!(windows) { "waid.exe" } else { "waid" });
        let (core, updates) = start_core(&exe).unwrap();
        let (mut independent, _) = start_core(&exe).unwrap();
        #[cfg(windows)]
        let owned_handle = {
            use std::os::windows::io::AsHandle;
            core.child.as_handle().try_clone_to_owned().unwrap()
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Some(rows) = updates.lock().unwrap().take() {
                rows.expect("core must produce a valid snapshot");
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "core produced no snapshot"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        drop(core);
        assert!(
            independent.child.try_wait().unwrap().is_none(),
            "another core must survive"
        );
        #[cfg(windows)]
        unsafe {
            use std::os::windows::io::AsRawHandle;
            extern "system" {
                fn WaitForSingleObject(handle: *mut std::ffi::c_void, milliseconds: u32) -> u32;
            }
            assert_eq!(
                WaitForSingleObject(owned_handle.as_raw_handle(), 1000),
                0,
                "owned child must exit"
            );
        }
    }
}
