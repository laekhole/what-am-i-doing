//! Resolve exact session identities, then activate through Orca's public CLI.
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
    let mut child = Command::new(exe)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
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

fn target(row: &Row, hooks: &Value, live: &Value) -> Option<String> {
    if row.session_id.is_empty() || row.state == "done" || hooks["version"] != 2
        || live["ok"] != true || live["result"]["truncated"] != false {
        return None;
    }
    let mut candidates = Vec::new();
    for entry in hooks["entries"].as_object()?.values() {
        if entry["source"].as_str() != Some(&row.agent_id)
            || entry["providerSession"]["key"] != "session_id"
            || entry["providerSession"]["id"].as_str() != Some(&row.session_id)
            || !entry["connectionId"].is_null()
            || entry["hookEventName"] == "SessionEnd" {
            continue;
        }
        let Some(pane) = nonempty(&entry["paneKey"]) else { continue };
        let Some((tab, leaf)) = pane.split_once(':') else { continue };
        let authority = &hooks["authorityCommitments"][pane];
        let Some(token) = nonempty(&entry["launchTokenHash"]) else { continue };
        if nonempty(&authority["launchTokenHash"]) != Some(token)
            || authority["paneKey"] != pane || entry["tabId"] != tab
            || authority["tabId"] != tab || !authority["connectionId"].is_null()
            || nonempty(&entry["worktreeId"]).is_none()
            || entry["worktreeId"] != authority["worktreeId"] {
            continue;
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    #[ignore = "requires the current Codex session inside a running local Orca"]
    fn current_orca_session_switches_to_its_existing_terminal() {
        let row = Row {
            session_id: std::env::var("CODEX_THREAD_ID").expect("current Codex session"),
            agent_id: "codex".into(),
            ..Row::default()
        };
        let expected = std::env::var("ORCA_TERMINAL_HANDLE").expect("current terminal");
        let live = cli(&["terminal", "list", "--json"]).expect("live terminals");
        assert_eq!(target(&row, &hooks().expect("hook snapshot"), &live), Some(expected));
        open(&row).expect("switch to the already-current terminal");
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
