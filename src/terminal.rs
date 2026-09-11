//! Bounded terminal-screen observations when SSH hides the real processes/logs.
use crate::{
    json::{self, Json},
    proc::Process,
    session::{Confidence, Session, State, Task},
};
use std::{
    io::Read,
    os::windows::process::CommandExt,
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

fn text<'a>(value: &'a Json, key: &str) -> Option<&'a str> {
    value.get(key)?.as_str().filter(|s| !s.is_empty())
}

fn clean(line: &str) -> &str {
    line.trim().trim_matches(['│', '┃', '|']).trim()
}

fn observation(value: &Json, now: i64) -> Option<Session> {
    let key = text(value, "key")?;
    let title = text(value, "title").unwrap_or("PowerShell");
    let body = text(value, "body")?;
    let lines: Vec<&str> = body.lines().map(clean).collect();
    // ponytail: screen signatures are heuristics, not provider session identities.
    // Use authenticated transcripts for hidden tmux panes and exact turn events.
    let footer = lines
        .iter()
        .rev()
        .take(4)
        .find(|s| s.starts_with("gpt-") && s.split('·').count() >= 2)
        .copied();
    let codex = (lines.iter().any(|s| s.starts_with(">_ OpenAI Codex ("))
        && lines
            .iter()
            .any(|s| s.starts_with("model:") && s.contains("/model"))
        && lines.iter().any(|s| s.starts_with("directory:")))
        || (footer.is_some()
            && lines.iter().rev().take(6).any(|s| {
                s.starts_with("› Ask Codex")
                    || s.contains("esc to interrupt")
                    || s.starts_with("› ")
            }));
    let claude = lines.iter().any(|s| s.contains("Claude Code v"))
        && lines
            .iter()
            .any(|s| s.contains("Opus") || s.contains("Sonnet") || s.contains("Haiku"));
    let name = match (codex, claude) {
        (true, false) => "codex",
        (false, true) => "claude",
        _ => return None, // Mixed tmux panes cannot be safely split by text alone.
    };
    // A terminal can show old agent output after returning to its remote shell.
    if lines
        .iter()
        .rev()
        .filter(|s| !s.is_empty())
        .take(3)
        .any(|s| (s.ends_with('$') || s.ends_with('#')) && s.contains('@'))
    {
        return None;
    }
    let model = if codex {
        lines
            .iter()
            .find_map(|s| s.strip_prefix("model:")?.split_whitespace().next())
            .or_else(|| footer.and_then(|s| s.split_whitespace().next()))
            .filter(|s| {
                s.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-._".contains(&c))
            })
            .map(str::to_string)
    } else {
        None
    };
    let directory = if codex {
        lines
            .iter()
            .find_map(|s| s.strip_prefix("directory:").map(str::trim))
            .or_else(|| footer.and_then(|s| s.split('·').nth(1)).map(str::trim))
    } else {
        None
    };
    let prompt = lines
        .iter()
        .rev()
        .find_map(|s| {
            s.strip_prefix('›')
                .or_else(|| s.strip_prefix('❯'))
                .map(str::trim)
                .filter(|s| !s.is_empty() && !s.starts_with("Ask Codex") && !s.starts_with("Try "))
        })
        .map(|s| s.chars().take(1000).collect::<String>());
    let label = footer
        .and_then(|s| s.split('·').nth(2))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(1000).collect::<String>());
    let project = directory.filter(|s| !s.is_empty()).unwrap_or(title);
    let id = format!("terminal:{}:{key}:{name}", key.len());
    Some(Session {
        context: Default::default(),
        prompt: None,
        last_answer: None,
        session_id: None,
        summary: None,
        request_marker: None,
        request_at: None,
        auxiliary: false,
        evidence: "terminal_screen",
        legacy_id: crate::session::short_id(&[&id]),
        id,
        title: format!(
            "{} · 화면 관찰",
            project.chars().take(100).collect::<String>()
        ),
        agent: crate::matchers::by_name(name)?,
        llm_id: model.clone(),
        llm_display: model.as_deref().map(crate::transcript::normalize_model),
        task: Task {
            text: prompt
                .or(label)
                .or_else(|| Some("터미널에서 에이전트 화면 감지".into())),
            source: "terminal_screen",
            confidence: Confidence::Inferred,
        },
        state: State::Unknown,
        since: now,
        // Never resolve screen-supplied remote paths or assign the local SSH PID to an agent.
        cwd: None,
        branch: None,
        pid: None,
    })
}

fn scan(now: i64) -> Result<(Vec<Session>, bool), String> {
    let powershell = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .ok_or("Windows 경로를 찾지 못했습니다.")?
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut child = Command::new(powershell)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-MTA",
            "-Command",
            include_str!("terminal.ps1"),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("화면 관찰 실행 실패: {e}"))?;
    let stdout = child.stdout.take().ok_or("화면 관찰 출력 없음")?;
    const LIMIT: u64 = 2 * 1024 * 1024;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout
            .take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if started.elapsed() < Duration::from_secs(4) => {
                std::thread::sleep(Duration::from_millis(20))
            }
            result => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(match result {
                    Err(e) => format!("화면 관찰 실패: {e}"),
                    _ => "화면 관찰 시간 초과 (4초)".into(),
                });
            }
        }
    };
    let bytes = reader
        .join()
        .map_err(|_| "화면 관찰 출력 실패")?
        .map_err(|_| "화면 관찰 출력 실패")?;
    if !status?.success() {
        return Err("Windows UI Automation 화면을 읽지 못했습니다.".into());
    }
    if bytes.len() as u64 > LIMIT {
        return Err("화면 관찰 출력 상한 초과".into());
    }
    let value = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|s| json::parse(s.trim_start_matches('\u{feff}')).ok())
        .ok_or("화면 관찰 응답 형식 오류")?;
    let screens = value
        .get("screens")
        .and_then(Json::as_array)
        .ok_or("화면 관찰 목록 없음")?;
    Ok((
        screens
            .iter()
            .take(16)
            .filter_map(|v| observation(v, now))
            .collect(),
        value.get("failed") == Some(&Json::Bool(true)),
    ))
}

pub fn collect(processes: &[Process], now: i64) -> Vec<Session> {
    if std::env::var("WAID_TERMINAL_SCAN").as_deref() == Ok("0")
        || !processes.iter().any(|p| {
            p.argv.first().is_some_and(|s| {
                s.rsplit(['/', '\\'])
                    .next()
                    .is_some_and(|s| s.eq_ignore_ascii_case("ssh.exe") || s == "ssh")
            })
        })
    {
        return Vec::new();
    }
    // The collector runs outside the desktop UI. Limit UIA to one bounded scan per five seconds.
    static CACHE: OnceLock<Mutex<Option<(Instant, Result<(Vec<Session>, bool), String>)>>> =
        OnceLock::new();
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache
        .as_ref()
        .is_none_or(|(at, _)| at.elapsed() >= Duration::from_secs(5))
    {
        *cache = Some((Instant::now(), scan(now)));
    }
    match &cache.as_ref().unwrap().1 {
        Ok((rows, partial)) => {
            if *partial {
                crate::diag::warn("일부 터미널 화면을 읽지 못했습니다.".into());
            }
            rows.clone()
        }
        Err(error) => {
            crate::diag::warn(error.clone());
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn screen(body: &str) -> Json {
        Json::Obj(
            [
                ("key".into(), Json::Str("123:456:tab:pane".into())),
                ("title".into(), Json::Str("remote@host:~".into())),
                ("body".into(), Json::Str(body.into())),
            ]
            .into(),
        )
    }
    #[test]
    fn screen_observation_is_bounded_inferred_and_never_a_local_session() {
        let body = "│ >_ OpenAI Codex (v1.0) │\n│ model: gpt-test /model to change │\n│ directory: /remote/repo │\n› 이전 요청\n› 최근 요청\n";
        let row = observation(&screen(body), 123).unwrap();
        assert_eq!(row.agent.name, "codex");
        assert_eq!(row.task.text.as_deref(), Some("최근 요청"));
        assert_eq!(row.llm_id.as_deref(), Some("gpt-test"));
        assert_eq!(row.state, State::Unknown);
        assert_eq!(row.evidence, "terminal_screen");
        assert!(row.context.used_tokens.is_none() && row.cwd.is_none() && row.pid.is_none());
        assert!(
            row.session_id.is_none() && row.request_marker.is_none() && row.request_at.is_none()
        );
        assert!(observation(&screen("Discuss OpenAI Codex and Claude Code"), 0).is_none());
        assert!(observation(&screen(&format!("{body}\n[user@host ~]$")), 0).is_none());
        assert!(observation(&screen(&format!("{body}\nClaude Code v1.0\nOpus")), 0).is_none());
        assert_eq!(
            observation(&screen("Claude Code v1.0\nSonnet\n❯ test"), 0)
                .unwrap()
                .agent
                .name,
            "claude"
        );
        let footer = observation(
            &screen("› Ask Codex to do anything\ngpt-test medium · ~ · 작업 이름"),
            0,
        )
        .unwrap();
        assert_eq!(footer.agent.name, "codex");
        assert_eq!(footer.task.text.as_deref(), Some("작업 이름"));
        assert_eq!(footer.llm_id.as_deref(), Some("gpt-test"));
        assert!(observation(&screen("gpt-test medium · ~ · 작업 이름"), 0).is_none());
        assert_eq!(
            row.id,
            observation(&screen(&body.replace("최근 요청", "다음 요청")), 456)
                .unwrap()
                .id
        );
        assert!(
            observation(&screen(&format!("{body}› {}", "a".repeat(5000))), 0)
                .unwrap()
                .task
                .text
                .unwrap()
                .len()
                <= 1000
        );
    }
}
