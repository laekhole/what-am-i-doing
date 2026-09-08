//! Exact Orca / user-associated ChatGPT links; explicitly manual window returns.
mod windows;
pub use windows::{open_window, probe_requested, windows};
use crate::Row;
use serde_json::Value;
use std::{
    fs::File,
    io::Read,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const LIMIT: u64 = 16 * 1024 * 1024;

fn read_json(reader: impl Read) -> Option<Value> {
    let mut bytes = Vec::new();
    reader.take(LIMIT + 1).read_to_end(&mut bytes).ok()?;
    (bytes.len() as u64 <= LIMIT)
        .then(|| serde_json::from_slice(&bytes).ok())
        .flatten()
}

fn hooks() -> Option<Value> {
    let root = std::env::var_os("ORCA_USER_DATA_PATH")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("orca")))?;
    // ponytail: Orca 1.4 hook snapshot is read-only and version-gated; replace with
    // public session metadata when terminal list exposes provider session IDs.
    let value = read_json(File::open(root.join("agent-hooks/last-status.json")).ok()?)?;
    (value["version"] == 2).then_some(value)
}

fn cli(args: &[&str]) -> Option<Value> {
    let exe = std::env::var_os("ORCA_CLI_COMMAND").unwrap_or_else(|| "orca".into());
    let mut command = Command::new(exe);
    command.args(args).creation_flags(0x08000000); // CREATE_NO_WINDOW
    command_json(command)
}

fn command_json(mut command: Command) -> Option<Value> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn().ok()?;
    let stdout = child.stdout.take()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || { let _ = tx.send(read_json(stdout)); });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if start.elapsed() < Duration::from_secs(3) => {
                std::thread::sleep(Duration::from_millis(20));
            }
            _ => {
                let _ = child.kill(); // Only the CLI subprocess we created.
                let _ = child.wait();
                break None;
            }
        }
    }?;
    if !status.success() { return None; }
    let value = rx.recv_timeout(Duration::from_millis(100)).ok()??;
    (value["ok"] == true).then_some(value)
}

fn nonempty(value: &Value) -> Option<&str> {
    value.as_str().filter(|s| !s.is_empty())
}

fn hook_matches(row: &Row, entry: &Value, hooks: &Value) -> bool {
    if row.session_id.is_empty() || hooks["version"] != 2
        || entry["source"].as_str() != Some(&row.agent_id)
        || entry["providerSession"]["key"] != "session_id"
        || entry["providerSession"]["id"].as_str() != Some(&row.session_id)
        || !entry["connectionId"].is_null()
        || entry["hookEventName"] == "SessionEnd" {
        return false;
    }
    let Some(pane) = nonempty(&entry["paneKey"]) else { return false };
    let Some((tab, _)) = pane.split_once(':') else { return false };
    let authority = &hooks["authorityCommitments"][pane];
    let Some(token) = nonempty(&entry["launchTokenHash"]) else { return false };
    nonempty(&authority["launchTokenHash"]) == Some(token)
        && authority["paneKey"] == pane && entry["tabId"] == tab
        && authority["tabId"] == tab && authority["connectionId"].is_null()
        && nonempty(&entry["worktreeId"]).is_some()
        && entry["worktreeId"] == authority["worktreeId"]
}

pub fn identify_hosts(rows: &mut [Row]) {
    let Some(hooks) = hooks() else { return };
    let Some(entries) = hooks["entries"].as_object() else { return };
    for row in rows {
        if row.evidence == "orca_hook" || entries.values().any(|entry| hook_matches(row, entry, &hooks)) {
            row.host = "orca".into();
        }
    }
}

fn target(row: &Row, hooks: &Value, live: &Value) -> Option<String> {
    if row.session_id.is_empty() || row.state == "done" || hooks["version"] != 2
        || live["ok"] != true || live["result"]["truncated"] != false {
        return None;
    }
    let mut candidates = Vec::new();
    for entry in hooks["entries"].as_object()?.values() {
        if !hook_matches(row, entry, hooks) {
            continue;
        }
        let Some(pane) = nonempty(&entry["paneKey"]) else { continue };
        let Some((tab, leaf)) = pane.split_once(':') else { continue };
        for terminal in live["result"]["terminals"].as_array()? {
            if terminal["tabId"] == tab && terminal["leafId"] == leaf
                && terminal["worktreeId"] == entry["worktreeId"]
                && terminal["agentIdentity"].as_str() == Some(&row.agent_id)
                && terminal["executionHostId"] == "local"
                && terminal["connected"] == true && terminal["writable"] == true
                && terminal["orphaned"] == false && terminal["exitCause"].is_null()
            {
                if let Some(handle) = nonempty(&terminal["handle"]).filter(|h|
                    h.starts_with("term_") && h.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')) {
                    candidates.push(handle.to_string());
                }
            }
        }
    }
    candidates.sort();
    candidates.dedup();
    (candidates.len() == 1).then(|| candidates.remove(0))
}

pub fn open(row: &Row) -> Result<(), String> {
    let missing = "열린 창·탭과의 연결을 확인할 수 없어 상세 내용을 표시합니다.";
    if row.session_id.is_empty() || row.state == "done" {
        return Err(missing.into());
    }
    let before = hooks().ok_or(missing)?;
    let live = cli(&["terminal", "list", "--json"]).ok_or(missing)?;
    let handle = target(row, &before, &live).ok_or(missing)?;
    // A new session in the same pane must not inherit the old session's link.
    if target(row, &hooks().ok_or(missing)?, &live).as_ref() != Some(&handle) {
        return Err(missing.into());
    }
    cli(&["terminal", "switch", "--terminal", &handle, "--json"])
        .ok_or("탭으로 이동하지 못했습니다. 상세 내용을 표시합니다.")?;
    Ok(())
}

/// Only the documented existing-local-chat route is accepted. Never accept new
/// chats, composer parameters, remote hosts, or a different session's identifier.
pub fn validate_chatgpt_link(row: &Row, link: &str) -> Result<(), String> {
    if row.agent_id != "codex" || row.session_id.is_empty() || row.state == "done"
        || row.session_id == "new" || row.session_id.len() > 128
        || !row.session_id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        || link != format!("codex://threads/{}", row.session_id)
    {
        return Err("ChatGPT에서 복사한 현재 세션의 codex://threads/ 링크만 연결할 수 있습니다.".into());
    }
    Ok(())
}

/// The user must associate a copied link first: Codex CLI logs alone do not prove
/// that this session belongs to ChatGPT. Shell success means dispatched, not that
/// the app confirmed the conversation is present.
pub fn open_chatgpt_link(row: &Row, link: &str) -> Result<(), String> {
    validate_chatgpt_link(row, link)?;
    let wide: Vec<u16> = link.encode_utf16().chain(Some(0)).collect();
    let result = unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(
            std::ptr::null_mut(), std::ptr::null(), wide.as_ptr(),
            std::ptr::null(), std::ptr::null(),
            windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        )
    };
    if result as isize <= 32 {
        Err("ChatGPT 세션 링크를 열지 못했습니다. 앱 설치와 링크 연결을 확인하세요.".into())
    } else { Ok(()) }
}

pub fn open_associated(row: &Row, target: &Value) -> Result<String, String> {
    if row.state == "done" {
        return Err("현재 세션에 연결된 창이나 링크가 필요합니다.".into());
    }
    if target["kind"] == "chatgpt_link" {
        open_chatgpt_link(row, target["url"].as_str().ok_or("ChatGPT 링크가 없습니다.")?)?;
        Ok("ChatGPT에 연결한 세션 링크를 전달했습니다. 앱에서 대화를 확인하세요.".into())
    } else {
        open_window(target)?;
        Ok(if target["kind"] == "powershell" {
            "직접 연결한 PowerShell 창으로 이동했습니다. 창 안의 세션은 직접 확인하세요."
        } else {
            "직접 연결한 앱 창으로 이동했습니다. 앱에서 원하는 대화를 선택하세요."
        }.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    #[ignore = "requires the current Codex session inside a running local Orca"]
    fn current_orca_session_maps_to_its_existing_terminal_read_only() {
        let row = Row {
            session_id: std::env::var("CODEX_THREAD_ID").expect("current Codex session"),
            agent_id: "codex".into(),
            ..Row::default()
        };
        let expected = std::env::var("ORCA_TERMINAL_HANDLE").expect("current terminal");
        let live = cli(&["terminal", "list", "--json"]).expect("live terminals");
        assert_eq!(target(&row, &hooks().expect("hook snapshot"), &live), Some(expected));
        // Never switch real sessions from a validation test.
    }

    #[test]
    fn chatgpt_links_cannot_create_chats_or_cross_session_boundaries() {
        let row = Row { agent_id: "codex".into(), session_id: "session-a".into(), ..Row::default() };
        assert!(validate_chatgpt_link(&row, "codex://threads/session-a").is_ok());
        for link in ["codex://threads/new", "codex://threads/session-b", "codex://threads/session-a?prompt=hello", "codex://threads/session-a#x", "https://example.com", "codex://threads/session-a/", "codex://threads/%73ession-a"] {
            assert!(validate_chatgpt_link(&row, link).is_err(), "{link}");
        }
        for id in ["", "new", "../new", "a?prompt=hello"] {
            let changed = Row { session_id: id.into(), ..row.clone() };
            assert!(validate_chatgpt_link(&changed, &format!("codex://threads/{id}")).is_err());
        }
        assert!(validate_chatgpt_link(&Row { agent_id: "claude".into(), ..row.clone() }, "codex://threads/session-a").is_err());
        assert!(validate_chatgpt_link(&Row { state: "done".into(), ..row }, "codex://threads/session-a").is_err());
    }

    #[test]
    fn manual_window_return_does_not_require_a_provider_session_id() {
        let row = Row { id: "process-only-row".into(), ..Row::default() };
        let stale = json!({"kind":"chatgpt","hwnd":0});
        // It reaches window identity validation instead of rejecting the row.
        assert_eq!(open_associated(&row, &stale), open_window(&stale).map(|_| String::new()));
        assert!(open_associated(&row, &json!({"kind":"chatgpt_link","url":"codex://threads/a"})).is_err());
    }

    #[test]
    fn only_one_current_exact_session_can_be_activated() {
        let row = Row { session_id: "session-a".into(), agent_id: "codex".into(), ..Row::default() };
        let hooks = json!({"version":2,"entries":{"pane":{
            "source":"codex","providerSession":{"key":"session_id","id":"session-a"},
            "paneKey":"tab:leaf","tabId":"tab","worktreeId":"same-folder",
            "launchTokenHash":"current","hookEventName":"Stop"
        }},"authorityCommitments":{"tab:leaf":{
            "paneKey":"tab:leaf","tabId":"tab","worktreeId":"same-folder","launchTokenHash":"current"
        }}});
        let live = json!({"ok":true,"result":{"truncated":false,"terminals":[{
            "handle":"term_one","tabId":"tab","leafId":"leaf","worktreeId":"same-folder",
            "agentIdentity":"codex","executionHostId":"local","connected":true,"writable":true,"orphaned":false
        }]}});
        assert_eq!(target(&row, &hooks, &live).as_deref(), Some("term_one"));
        assert!(hook_matches(&row, &hooks["entries"]["pane"], &hooks));
        let wrong = Row { session_id: "session-b".into(), ..row.clone() };
        assert_eq!(target(&wrong, &hooks, &live), None);
        for (pointer, value) in [
            ("/entries/pane/source", json!("claude")),
            ("/entries/pane/launchTokenHash", json!("previous-launch")),
            ("/entries/pane/hookEventName", json!("SessionEnd")),
            ("/entries/pane/connectionId", json!("remote")),
            ("/authorityCommitments/tab:leaf/tabId", json!("other-tab")),
            ("/version", json!(3)),
        ] {
            let mut changed = hooks.clone();
            if pointer.ends_with("connectionId") { changed["entries"]["pane"]["connectionId"] = value; }
            else { *changed.pointer_mut(pointer).unwrap() = value; }
            assert_eq!(target(&row, &changed, &live), None, "{pointer}");
            assert!(!hook_matches(&row, &changed["entries"]["pane"], &changed), "{pointer}");
        }
        for (field, value) in [
            ("connected", json!(false)), ("orphaned", json!(true)),
            ("executionHostId", json!("remote")), ("leafId", json!("other-leaf")),
            ("exitCause", json!({"kind":"exit"})), ("handle", json!("term_a & bad")),
        ] {
            let mut changed = live.clone();
            changed["result"]["terminals"][0][field] = value;
            assert_eq!(target(&row, &hooks, &changed), None, "{field}");
        }
        let mut ambiguous = live.clone();
        let mut other = live["result"]["terminals"][0].clone();
        other["handle"] = json!("term_two");
        ambiguous["result"]["terminals"].as_array_mut().unwrap().push(other);
        assert_eq!(target(&row, &hooks, &ambiguous), None);
        assert_eq!(target(&Row { state: "done".into(), ..row }, &hooks, &live), None);
    }
}
