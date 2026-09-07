//! 코어. 프로세스와 트랜스크립트를 하나의 스냅샷으로 정규화한다.
//!
//! 이 모듈에는 색상 코드도, 컬럼 너비도, ANSI 이스케이프도 없다(매니페스트 §1.3).
//! 렌더러가 무엇을 하든 코어는 모른다.

use crate::matchers::{self, Agent};
use crate::proc::{self, Process};
use crate::time;
use crate::transcript::{self, Transcript};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// 마지막 이벤트 이후 이 시간 안이면 "지금 일하는 중"으로 본다.
pub const WORKING_WITHIN: i64 = 20;
/// 이미 끝난 세션을 목록에 남겨두는 시간.
pub const KEEP_DONE_FOR: i64 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Unknown,
    Waiting,
    Working,
    Error,
    Idle,
    Done,
}

impl State {
    pub fn id(&self) -> &'static str {
        match self {
            State::Unknown => "unknown",
            State::Waiting => "waiting",
            State::Working => "working",
            State::Error => "error",
            State::Idle => "idle",
            State::Done => "done",
        }
    }

    /// 정렬 우선순위. `waiting`이 항상 맨 위다 — 이 도구를 켜는 이유가
    /// "어느 놈이 나를 기다리나"이기 때문이다(매니페스트 §3.1).
    pub fn rank(&self) -> u8 {
        match self {
            State::Waiting => 0,
            State::Working => 1,
            State::Error => 2,
            State::Idle => 3,
            State::Done => 4,
            State::Unknown => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Explicit,
    Inferred,
    None,
}

impl Confidence {
    pub fn id(&self) -> &'static str {
        match self {
            Confidence::Explicit => "explicit",
            Confidence::Inferred => "inferred",
            Confidence::None => "none",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub text: Option<String>,
    pub source: &'static str,
    pub confidence: Confidence,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub last_answer: Option<String>,
    pub session_id: Option<String>,
    pub summary: Option<String>,
    pub request_marker: Option<String>,
    pub request_at: Option<i64>,
    pub auxiliary: bool,
    pub evidence: &'static str,
    pub id: String,
    pub title: String,
    pub agent: Agent,
    pub llm_id: Option<String>,
    pub llm_display: Option<String>,
    pub task: Task,
    pub state: State,
    pub since: i64,
    pub cwd: Option<PathBuf>,
    pub branch: Option<String>,
    pub pid: Option<i32>,
}

// ------------------------------------------------------------ 수집

pub fn collect(now: i64) -> Vec<Session> {
    collect_with_history(now, false)
}

pub fn collect_with_history(now: i64, history: bool) -> Vec<Session> {
    let processes: Vec<(Process, Agent)> = proc::list()
        .into_iter()
        .filter_map(|p| matchers::identify(&p).map(|a| (p, a)))
        .collect();

    let mut sessions = Vec::new();
    let mut claimed: HashSet<PathBuf> = HashSet::new();

    for agent in matchers::all() {
        let procs: Vec<&(Process, Agent)> = processes
            .iter()
            .filter(|(_, a)| a.name == agent.name)
            .collect();

        if !agent.has_reader {
            for (p, _) in &procs {
                sessions.push(from_process_only(p, agent, now));
            }
            continue;
        }

        let transcripts = transcript::discover_with_history(agent.name, now, history);

        // 프로세스 ↔ 트랜스크립트 짝짓기. cwd가 유일한 신뢰 가능한 열쇠다.
        for (p, _) in &procs {
            let matched = p.cwd.as_ref().and_then(|cwd| {
                transcripts
                    .iter()
                    .find(|t| !claimed.contains(&t.path) && t.cwd.as_deref() == Some(cwd.as_path()))
            });
            if let Some(t) = matched {
                claimed.insert(t.path.clone());
                sessions.push(from_pair(Some(p), agent, t, now));
            } else if p.cwd.is_some() || transcripts.is_empty() {
                sessions.push(from_process_only(p, agent, now));
            }
        }

        // 프로세스를 못 봤다는 것은 종료 증거가 아니다 (§5.1).
        for t in &transcripts {
            if claimed.contains(&t.path) {
                continue;
            }
            let session = from_pair(None, agent, t, now);
            if session.state != State::Done || now - t.last_event_at <= KEEP_DONE_FOR {
                sessions.push(session);
            }
        }
    }

    let mut seen = HashSet::new();
    sessions.retain(|s| seen.insert(s.id.clone()));
    dedupe_titles(&mut sessions);
    sessions.sort_by(|a, b| {
        a.state
            .rank()
            .cmp(&b.state.rank())
            .then(a.title.cmp(&b.title))
    });
    sessions
}

pub(crate) fn from_pair(p: Option<&Process>, agent: Agent, t: &Transcript, now: i64) -> Session {
    let cwd = p.and_then(|p| p.cwd.clone()).or_else(|| t.cwd.clone());
    let age = now - t.last_event_at;

    let state = match t.event_state {
        Some(State::Working) if age > WORKING_WITHIN => State::Unknown,
        Some(state) => state,
        None => State::Unknown,
    };

    let branch = cwd.as_deref().and_then(git_branch);
    let mut task = resolve_task(
        p,
        cwd.as_deref(),
        t.current_prompt.as_deref().or(t.first_prompt.as_deref()),
    );
    if task.source == "transcript_latest_prompt" && t.current_prompt.is_none() {
        task.source = "transcript_first_prompt";
    }

    let identity = t.session_id.clone().unwrap_or_else(|| {
        t.path
            .components()
            .collect::<PathBuf>()
            .to_string_lossy()
            .into_owned()
    });
    Session {
        last_answer: t.last_answer.clone(),
        session_id: t.session_id.clone(),
        summary: t.first_prompt.clone(),
        request_marker: t.request_marker.clone(),
        request_at: t.request_at,
        auxiliary: t.auxiliary,
        evidence: if t.event_state == Some(State::Working) && age > WORKING_WITHIN {
            "stale"
        } else if t.event_state.is_some() && t.inferred_time {
            "inferred_time"
        } else if t.event_state.is_some() {
            "transcript"
        } else {
            "unknown"
        },
        id: short_id(&[agent.name, &identity]),
        title: make_title(cwd.as_deref(), branch.as_deref()),
        agent,
        llm_display: t.model.as_deref().map(transcript::normalize_model),
        llm_id: t.model.clone(),
        task,
        state,
        since: t.last_event_at,
        cwd,
        branch,
        pid: p.map(|p| p.pid),
    }
}

/// 트랜스크립트 리더가 없는 에이전트, 또는 짝을 못 찾은 프로세스.
///
/// 알 수 없는 것은 알 수 없다고 표시한다. 추측해서 채우지 않는다(§3.1).
fn from_process_only(p: &Process, agent: Agent, _now: i64) -> Session {
    let branch = p.cwd.as_deref().and_then(git_branch);
    Session {
        last_answer: None,
        session_id: None,
        summary: None,
        request_marker: None,
        request_at: None,
        auxiliary: false,
        evidence: "process_only",
        id: short_id(&[agent.name, &p.pid.to_string()]),
        title: make_title(p.cwd.as_deref(), branch.as_deref()),
        agent,
        llm_id: None,
        llm_display: cmdline_model(p),
        task: resolve_task(Some(p), p.cwd.as_deref(), None),
        // 트랜스크립트 없이는 working/waiting을 구분할 수 없다.
        // 거짓말하느니 가장 약한 주장을 한다.
        state: State::Unknown,
        since: 0,
        cwd: p.cwd.clone(),
        branch,
        pid: Some(p.pid),
    }
}

// ------------------------------------------------------------ task

fn resolve_task(p: Option<&Process>, cwd: Option<&Path>, prompt: Option<&str>) -> Task {
    // 1) 프로세스 환경변수. 사용자가 직접 라벨을 붙인 경우.
    if let Some(p) = p {
        if let Some(v) = env_of(p.pid, "WAID_TASK") {
            return Task {
                text: Some(v),
                source: "env",
                confidence: Confidence::Explicit,
            };
        }
    }
    // 2) 리포지토리의 .waid 파일.
    if let Some(cwd) = cwd {
        if let Some(v) = waid_file(cwd, "task") {
            return Task {
                text: Some(v),
                source: "waid_file",
                confidence: Confidence::Explicit,
            };
        }
    }
    // 3) 커맨드라인 프롬프트 인자. `claude -p "..."` 같은 원샷 실행.
    if let Some(p) = p {
        if let Some(v) = cmdline_prompt(p) {
            return Task {
                text: Some(transcript::normalize_prompt(&v)),
                source: "cmdline",
                confidence: Confidence::Explicit,
            };
        }
    }
    // 4) 최근 사용자 요청. 첫 요청 폴백은 호출자가 출처를 별도로 표시한다.
    if let Some(t) = prompt {
        return Task {
            text: Some(t.to_string()),
            source: "transcript_latest_prompt",
            confidence: Confidence::Inferred,
        };
    }
    Task {
        text: None,
        source: "none",
        confidence: Confidence::None,
    }
}

/// `/proc/<pid>/environ`은 같은 사용자의 프로세스라면 읽을 수 있다.
/// 덕분에 `WAID_TASK=... claude` 라벨링이 실제로 동작한다.
fn env_of(pid: i32, key: &str) -> Option<String> {
    let raw = std::fs::read(format!("/proc/{pid}/environ")).ok()?;
    let want = format!("{key}=");
    raw.split(|b| *b == 0)
        .filter_map(|s| std::str::from_utf8(s).ok())
        .find_map(|kv| kv.strip_prefix(&want))
        .map(str::to_string)
        .filter(|v| !v.is_empty())
}

fn cmdline_prompt(p: &Process) -> Option<String> {
    let mut it = p.argv.iter().skip(1);
    while let Some(arg) = it.next() {
        if matches!(arg.as_str(), "-p" | "--print" | "--prompt" | "--message") {
            if let Some(v) = it.next() {
                if !v.starts_with('-') {
                    return Some(v.clone());
                }
            }
        }
        if let Some(v) = arg.strip_prefix("--prompt=") {
            return Some(v.to_string());
        }
    }
    None
}

fn cmdline_model(p: &Process) -> Option<String> {
    let mut it = p.argv.iter().skip(1);
    while let Some(arg) = it.next() {
        if matches!(arg.as_str(), "-m" | "--model") {
            if let Some(v) = it.next() {
                return Some(transcript::normalize_model(v));
            }
        }
        if let Some(v) = arg.strip_prefix("--model=") {
            return Some(transcript::normalize_model(v));
        }
    }
    None
}

/// `.waid` — 의도적으로 TOML이 아니다. `key = value` 한 줄씩.
/// 파서를 하나 더 들이는 것보다 이게 싸다.
fn waid_file(start: &Path, key: &str) -> Option<String> {
    for dir in start.ancestors().take(8) {
        let f = dir.join(".waid");
        if let Ok(text) = std::fs::read_to_string(&f) {
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    if k.trim() == key {
                        let v = v.trim().trim_matches('"').trim_matches('\'');
                        if !v.is_empty() {
                            return Some(v.to_string());
                        }
                    }
                }
            }
        }
        if dir.join(".git").exists() {
            break;
        }
    }
    None
}

// ------------------------------------------------------------ title

fn make_title(cwd: Option<&Path>, branch: Option<&str>) -> String {
    let base = cwd
        .and_then(|c| c.file_name())
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "?".into());
    match branch {
        // 기본 브랜치는 정보량이 없다. 붙이지 않는다.
        Some(b) if b != "main" && b != "master" => format!("{base}/{b}"),
        _ => base,
    }
}

fn git_branch(cwd: &Path) -> Option<String> {
    for dir in cwd.ancestors().take(8) {
        let head = dir.join(".git/HEAD");
        if let Ok(s) = std::fs::read_to_string(&head) {
            let s = s.trim();
            if let Some(r) = s.strip_prefix("ref: refs/heads/") {
                return Some(r.to_string());
            }
            // detached HEAD
            return Some(s.chars().take(7).collect());
        }
    }
    None
}

/// 같은 저장소에서 여러 세션이 돌면 제목이 겹친다. 짧은 id를 덧붙인다.
pub(crate) fn dedupe_titles(sessions: &mut [Session]) {
    let mut seen: Vec<String> = Vec::new();
    let mut dup: HashSet<String> = HashSet::new();
    for s in sessions.iter() {
        if seen.contains(&s.title) {
            dup.insert(s.title.clone());
        }
        seen.push(s.title.clone());
    }

    // 4자리로 시작하되, 그래도 겹치면 늘린다. 접미사를 붙이는 목적이
    // 구분인데 접미사가 겹치면 아무 일도 하지 않은 것과 같다.
    let mut len = 4usize;
    loop {
        let candidates: Vec<String> = sessions
            .iter()
            .map(|s| {
                if dup.contains(&s.title) {
                    format!("{}·{}", s.title, &s.id[..len.min(s.id.len())])
                } else {
                    s.title.clone()
                }
            })
            .collect();

        let unique: HashSet<&String> = candidates.iter().collect();
        if unique.len() == candidates.len() || len >= 8 {
            for (s, t) in sessions.iter_mut().zip(candidates) {
                s.title = t;
            }
            return;
        }
        len += 2;
    }
}

/// FNV-1a. 암호학적 용도가 아니라 화면에 4~8자를 띄우기 위한 것.
/// 세션 식별자.
///
/// FNV-1a 만으로는 부족하다. 마지막 몇 바이트만 다른 입력(`claude`+`794` vs
/// `claude`+`798`)은 해시의 상위 비트가 거의 같아서, 앞자리를 잘라 쓰면
/// 충돌한다. 실제로 pid 세 개가 같은 `·7a2f` 접미사를 받는 걸 목격했다 —
/// 중복을 구분하려고 붙인 접미사가 중복되면 존재 이유가 없다.
///
/// 그래서 FNV 뒤에 최종 믹싱(murmur3 의 fmix64)을 한 번 돌린다. 앞자리든
/// 뒷자리든 어디를 잘라 써도 안전해진다.
pub(crate) fn short_id(parts: &[&str]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        for b in p.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    format!("{:08x}", h as u32)
}

// ------------------------------------------------------------ 직렬화

pub fn to_json(sessions: &[Session], now: i64, pretty: bool) -> String {
    let mut w = crate::json::Writer::new(pretty);
    w.begin_obj();
    w.field_num("schema", 1);
    w.field_str("captured_at", &time::to_iso8601(now));
    w.field_arr("sessions");
    for s in sessions {
        w.begin_obj();
        w.field_str("id", &s.id);
        w.field_opt_str("session_id", s.session_id.as_deref());
        w.field_str("title", &s.title);

        w.field_obj("agent");
        w.field_str("name", s.agent.name);
        w.field_str("display", s.agent.display);
        w.end_obj();

        w.field_obj("llm");
        w.field_opt_str("id", s.llm_id.as_deref());
        w.field_opt_str("display", s.llm_display.as_deref());
        w.end_obj();

        w.field_obj("task");
        w.field_opt_str("text", s.task.text.as_deref());
        w.field_str("source", s.task.source);
        w.field_str("confidence", s.task.confidence.id());
        w.end_obj();

        w.field_opt_str("summary", s.summary.as_deref());
        w.field_opt_str("last_answer", s.last_answer.as_deref());
        w.field_opt_str("request_marker", s.request_marker.as_deref());
        match s.request_at {
            Some(at) => w.field_num("request_at", at),
            None => w.field_opt_str("request_at", None),
        }
        w.field_bool("auxiliary", s.auxiliary);
        w.field_obj("status");
        w.field_str("evidence", s.evidence);
        w.field_str("state", s.state.id());
        if s.since > 0 {
            w.field_str("since", &time::to_iso8601(s.since));
        } else {
            w.field_opt_str("since", None);
        }
        w.end_obj();

        w.field_opt_str(
            "cwd",
            s.cwd.as_ref().map(|c| c.to_string_lossy()).as_deref(),
        );
        w.field_opt_str("branch", s.branch.as_deref());
        match s.pid {
            Some(p) => w.field_num("pid", p as i64),
            None => w.field_opt_str("pid", None),
        }
        if s.pid.is_some() {
            w.field_bool("alive", true);
        } else {
            w.field_opt_str("alive", None);
        }
        w.end_obj();
    }
    w.end_arr();
    w.end_obj();
    w.buf.push('\n');
    w.buf
}
