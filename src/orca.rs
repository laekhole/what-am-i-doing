//! Read Orca's locally mirrored SSH hooks; never connect to or command an agent.
use crate::{
    json::{self, Json},
    session::{Confidence, Session, State, Task},
};
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

pub fn path() -> Option<PathBuf> {
    std::env::var_os("ORCA_USER_DATA_PATH")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            if cfg!(target_os = "macos") {
                crate::adapters::home().map(|p| p.join("Library/Application Support/orca"))
            } else {
                std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("orca"))
            }
        })
        .map(|p| p.join("agent-hooks/last-status.json"))
}

fn text<'a>(v: &'a Json, key: &str) -> Option<&'a str> {
    v.get(key)?.as_str().filter(|s| !s.is_empty())
}

fn timestamp(v: &Json) -> Option<i64> {
    match v.get("evidenceObservedAt")? {
        Json::Num(n) if n.is_finite() && *n > 0.0 && *n < i64::MAX as f64 => Some(*n as i64),
        _ => None,
    }
}

fn entry(v: &Json, hooks: &Json, previous: Option<&Session>) -> Option<Session> {
    let connection = text(v, "connectionId")?;
    let pane = text(v, "paneKey")?;
    let (tab, _) = pane.split_once(':')?;
    let authority = hooks.get("authorityCommitments")?.get(pane)?;
    let token = text(v, "launchTokenHash")?;
    let worktree = text(v, "worktreeId")?;
    if text(authority, "launchTokenHash") != Some(token)
        || text(authority, "connectionId") != Some(connection)
        || text(authority, "paneKey") != Some(pane)
        || text(authority, "worktreeId") != Some(worktree)
        || text(authority, "tabId") != Some(tab)
        || text(v, "tabId") != Some(tab)
    {
        return None;
    }
    let agent = crate::adapters::by_name(text(v, "source")?)?;
    if !matches!(agent.name, "codex" | "claude") {
        return None;
    }
    let provider = v.get("providerSession")?;
    if text(provider, "key") != Some("session_id") {
        return None;
    }
    let session_id = text(provider, "id")?;
    // Host scope prevents a copied provider ID on another server from merging rows.
    let id = format!(
        "orca:{}:{connection}:{}:{}{session_id}",
        connection.len(),
        agent.name.len(),
        agent.name
    );
    let previous = previous.filter(|p| p.id == id);
    let stamp = timestamp(v)?;
    let (_, cwd) = worktree.split_once("::")?;
    let title = cwd
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())?;
    let payload = v.get("payload")?;
    let prompt = text(payload, "prompt").map(|s| {
        s.chars()
            .filter(|c| *c != '\0')
            .take(4000)
            .collect::<String>()
    });
    let event = text(v, "hookEventName")?;
    let state = match event {
        "UserPromptSubmit" | "PreToolUse" | "PostToolUse" => State::Working,
        "Stop" | "PermissionRequest" => State::Waiting,
        "StopFailure" => State::Error,
        "SessionEnd" => State::Idle,
        // SubagentStop describes a child, not completion of this parent session.
        _ => State::Unknown,
    };
    let same_request = previous.filter(|p| p.prompt == prompt);
    let request_marker = prompt.as_ref().map(|prompt| {
        if event != "UserPromptSubmit" {
            if let Some(marker) = same_request.and_then(|p| p.request_marker.clone()) {
                return marker;
            }
        }
        format!("orca-request:{stamp}:{prompt}")
    });
    let request_at = if event == "UserPromptSubmit" {
        Some(stamp / 1000)
    } else {
        same_request.and_then(|p| p.request_at)
    };
    Some(Session {
        legacy_id: crate::session::short_id(&[&id]),
        id,
        session_id: Some(session_id.into()),
        title: title.into(),
        agent,
        llm_id: text(payload, "model").map(str::to_string),
        llm_display: text(payload, "model").map(crate::transcript::normalize_model),
        task: Task {
            text: prompt.clone(),
            source: "orca_hook",
            confidence: Confidence::Inferred,
        },
        summary: None,
        prompt,
        request_marker,
        request_at,
        last_answer: text(payload, "lastAssistantMessage").map(|s| s.chars().take(4000).collect()),
        auxiliary: text(payload, "prompt").is_some_and(|s| {
            s.starts_with(
                "You are working inside Orca, a multi-agent IDE. You are a dispatched worker.",
            ) && s
                .lines()
                .any(|line| line.starts_with("Your task ID is: task_"))
        }) || previous.is_some_and(|p| p.auxiliary),
        evidence: "orca_hook",
        state,
        since: stamp / 1000,
        cwd: Some(cwd.into()),
        branch: None,
        pid: None,
    })
}

fn merge(hooks: &Json, sessions: &mut HashMap<String, Session>) -> bool {
    if hooks.get("version") != Some(&Json::Num(2.0)) {
        return false;
    }
    let Some(Json::Obj(entries)) = hooks.get("entries") else {
        return false;
    };
    for v in entries.values() {
        let Some(candidate) = entry(v, hooks, None) else {
            continue;
        };
        let previous = sessions.get(&candidate.id);
        if previous.is_some_and(|p| p.since > candidate.since) {
            continue;
        }
        if let Some(row) = entry(v, hooks, previous) {
            sessions.insert(row.id.clone(), row);
        }
    }
    true
}

pub fn collect(now: i64, history: bool) -> Vec<Session> {
    let Some(path) = path() else {
        return Vec::new();
    };
    // ponytail: retain observed sessions for this process only; durable remote history
    // needs a provider transcript source, since Orca overwrites each pane's snapshot.
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, HashMap<String, Session>>>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let sessions = cache.entry(path.clone()).or_default();
    const LIMIT: u64 = 16 * 1024 * 1024;
    let mut bytes = Vec::new();
    match File::open(&path).and_then(|f| f.take(LIMIT + 1).read_to_end(&mut bytes)) {
        Ok(_) if bytes.len() as u64 > LIMIT => {
            crate::diag::oversized(&path, bytes.len() as u64, LIMIT)
        }
        Ok(_) => match std::str::from_utf8(&bytes)
            .ok()
            .and_then(|s| json::parse(s).ok())
        {
            Some(hooks) if merge(&hooks, sessions) => {}
            _ => crate::diag::warn(format!(
                "Orca hook 형식 또는 버전을 읽지 못했습니다: {}",
                path.display()
            )),
        },
        Err(e) => crate::diag::file_error(&path, &e),
    }
    sessions
        .values()
        .filter(|s| history || now - s.since <= 24 * 3600)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_projects_survive_pane_replacement_without_false_requests_or_parent_completion() {
        fn hooks(project: &str, session: &str, event: &str, stamp: i64, token: &str) -> Json {
            json::parse(&format!(r#"{{"version":2,"entries":{{"pane":{{
                "source":"codex","connectionId":"ssh-server","paneKey":"tab:leaf","tabId":"tab",
                "worktreeId":"repo::{project}","launchTokenHash":"{token}","hookEventName":"{event}",
                "evidenceObservedAt":{stamp},"providerSession":{{"key":"session_id","id":"{session}"}},
                "payload":{{"prompt":"same request","model":"gpt-test","state":"done"}}
            }}}},"authorityCommitments":{{"tab:leaf":{{"connectionId":"ssh-server","paneKey":"tab:leaf",
                "tabId":"tab","worktreeId":"repo::{project}","launchTokenHash":"current"}}}}}}"#)).unwrap()
        }
        let mut sessions = HashMap::new();
        let submit = hooks(
            "/app/integrations",
            "one",
            "UserPromptSubmit",
            100_000,
            "current",
        );
        assert!(merge(&submit, &mut sessions));
        let first = sessions.values().next().unwrap().clone();
        assert_eq!(first.title, "integrations");
        assert_eq!(first.request_at, Some(100));
        assert_eq!(first.state, State::Working);
        merge(
            &hooks("/app/integrations", "one", "Stop", 101_000, "current"),
            &mut sessions,
        );
        assert_eq!(sessions[&first.id].request_marker, first.request_marker);
        assert_eq!(sessions[&first.id].state, State::Waiting);
        merge(
            &hooks(
                "/app/integrations",
                "one",
                "SubagentStop",
                102_000,
                "current",
            ),
            &mut sessions,
        );
        assert_eq!(sessions[&first.id].state, State::Unknown);
        merge(
            &hooks(
                "/app/integrations",
                "one",
                "UserPromptSubmit",
                103_000,
                "current",
            ),
            &mut sessions,
        );
        assert_ne!(sessions[&first.id].request_marker, first.request_marker);
        merge(
            &hooks("/app/AUDPlatform", "two", "Stop", 104_000, "current"),
            &mut sessions,
        );
        assert_eq!(sessions.len(), 2);
        merge(
            &hooks("/app/renew-bid", "three", "Stop", 105_000, "stale"),
            &mut sessions,
        );
        assert_eq!(sessions.len(), 2);
        assert!(!merge(
            &json::parse(r#"{"version":3}"#).unwrap(),
            &mut sessions
        ));
        assert_eq!(sessions.len(), 2);
    }
}
