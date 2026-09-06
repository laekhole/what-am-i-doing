//! 트랜스크립트 읽기.
//!
//! 두 가지 원칙이 이 모듈의 모양을 결정한다.
//!
//! **하나. 전체를 읽지 않는다.** 긴 세션의 JSONL은 수십 MB까지 간다. 우리가
//! 필요한 건 첫 사용자 프롬프트(파일 앞)와 마지막 이벤트(파일 뒤)뿐이다.
//! 앞에서 몇 KB, 뒤에서 몇 KB만 읽는다. 성능 예산(§8) 대부분이 여기서 지켜진다.
//!
//! **둘. 상태 이벤트의 시각을 우선한다.** 비용·설정 기록에 의한 파일 갱신은
//! 활동 증거가 아니다. 이벤트 시각을 읽을 수 없을 때만 mtime으로 추론한다.

use crate::json::{parse, Json};
use crate::session::State;
use crate::time;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

// Codex의 주입 문맥 뒤 첫 사용자 요청이 실제 로그에서 약 79KiB에 있었다.
// ponytail: 앞 128KiB까지만 읽는다. 더 긴 머리말의 실례가 나오면 범위를 재검토한다.
const HEAD_BYTES: u64 = 128 * 1024;
const TAIL_BYTES: u64 = 256 * 1024;
/// CLI 기본 수집 기간. 네이티브 앱은 --history로 기간 제한을 해제한다.
const MAX_AGE_SECS: i64 = 24 * 3600;

#[derive(Debug, Clone)]
pub struct Transcript {
    pub session_id: Option<String>,
    pub current_prompt: Option<String>,
    pub request_marker: Option<String>,
    pub auxiliary: bool,
    pub path: PathBuf,
    pub last_event_at: i64,
    pub inferred_time: bool,
    pub cwd: Option<PathBuf>,
    pub model: Option<String>,
    pub first_prompt: Option<String>,
    pub event_state: Option<State>,
}

struct Cached {
    event_time: Option<i64>,
    event_signature: Option<u64>,
    prefix: Vec<u8>,
    size: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    transcript: Transcript,
}

/// 에이전트별 트랜스크립트 루트.
///
/// 하드코딩하지 않는다. 어댑터 테이블(§4)이 유일한 출처이므로, 사용자가
/// `[transcript] dir = "~/.mytool/sessions"` 한 줄을 적으면 이 함수가
/// 자동으로 새 에이전트를 훑기 시작한다.
fn roots(agent: &str) -> Vec<PathBuf> {
    crate::adapters::transcript_dirs(agent).to_vec()
}

/// 루트 아래에서 확장자가 맞는 파일을 재귀 수집한다. `name` 을 주면 그
/// 이름의 파일만 본다 — 한 세션 폴더에 파일이 여럿인 형식에 필요하다.
fn walk(
    dir: &Path,
    now: i64,
    out: &mut Vec<(PathBuf, i64)>,
    depth: u32,
    history: bool,
    ext: &str,
    name: Option<&str>,
) {
    if depth > 6 {
        return; // Codex는 년/월/일로 중첩된다. 6이면 충분하고도 남는다.
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let ft = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if ft.is_dir() {
            walk(&path, now, out, depth + 1, history, ext, name);
        } else if path.extension().map(|e| e == ext).unwrap_or(false)
            && name.is_none_or(|want| path.file_name().is_some_and(|got| got == want))
        {
            if let Some(m) = time::mtime(&path) {
                if history || now - m <= MAX_AGE_SECS {
                    out.push((path, m));
                }
            }
        }
    }
}

pub fn discover(agent: &str, now: i64) -> Vec<Transcript> {
    discover_with_history(agent, now, false)
}

pub fn discover_with_history(agent: &str, now: i64, history: bool) -> Vec<Transcript> {
    let mut files = Vec::new();
    for root in roots(agent) {
        walk(&root, now, &mut files, 0, history, "jsonl", None);
    }
    // 최근 것부터. 아래에서 프로세스와 짝지을 때 최신이 우선권을 갖는다.
    files.sort_by(|a, b| b.1.cmp(&a.1));
    let mut found: Vec<_> = files
        .iter()
        .filter(|(p, _)| agent != "copilot" || p.file_name().is_some_and(|n| n == "events.jsonl"))
        .filter_map(|(p, m)| read(p, *m))
        .filter(|t| agent != "copilot" || t.session_id.is_some() || t.current_prompt.is_some())
        .filter(|t| history || now - t.last_event_at <= MAX_AGE_SECS)
        .collect();
    // JSONL 이 아닌 소스도 같은 목록에 들어간다 (§4.1).
    found.extend(json_sessions(agent, now, history));
    found.extend(sqlite_sessions(agent, now, history));
    found.retain(|t| history || now - t.last_event_at <= MAX_AGE_SECS);
    found.sort_by(|a, b| b.last_event_at.cmp(&a.last_event_at));
    // 같은 세션을 여러 질의가 물어 올 수 있다. 최근 것 하나만 남긴다.
    let mut seen = std::collections::HashSet::new();
    found.retain(|t| match &t.session_id {
        Some(id) => seen.insert(id.clone()),
        None => true,
    });
    found
}

fn read_span(f: &mut File, from: SeekFrom, len: usize) -> Vec<String> {
    if f.seek(from).is_err() {
        return Vec::new();
    }
    let mut buf = vec![0u8; len];
    let n = match f.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return Vec::new(),
    };
    buf.truncate(n);
    String::from_utf8_lossy(&buf)
        .lines()
        .map(str::to_string)
        .collect()
}

pub fn read(path: &Path, mtime: i64) -> Option<Transcript> {
    let mut f = File::open(path).ok()?;
    let metadata = f.metadata().ok()?;
    let size = metadata.len();
    let modified = metadata.modified().ok();
    let created = metadata.created().ok();
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Cached>>> = OnceLock::new();
    // ponytail: 짧은 파일 읽기 동안 하나의 잠금. 동시 독자가 많아지면 파일별로 분리한다.
    let mut cache = CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let cached = cache.get(path);
    if let Some(c) =
        cached.filter(|c| modified.is_some() && c.modified == modified && c.size == size)
    {
        return Some(c.transcript.clone());
    }
    // 세션 JSONL은 append-only. 잘리거나 교체되면 머리말도 다시 읽는다.
    let mut prefix = vec![0; size.min(512) as usize];
    f.read_exact(&mut prefix).ok()?;
    let previous_cached =
        cached.filter(|c| c.size < size && c.created == created && prefix.starts_with(&c.prefix));
    let previous = previous_cached.map(|c| &c.transcript);
    let reuse_head = previous.is_some_and(|t| t.first_prompt.is_some());

    let head = if reuse_head {
        Vec::new()
    } else {
        read_span(&mut f, SeekFrom::Start(0), HEAD_BYTES.min(size) as usize)
    };

    let mut tail = if reuse_head || size > HEAD_BYTES {
        let len = TAIL_BYTES.min(size);
        let mut t = read_span(&mut f, SeekFrom::End(-(len as i64)), len as usize);
        // 중간에서 잘렸으므로 첫 줄은 깨진 JSON일 수 있다.
        if size > len && !t.is_empty() {
            t.remove(0);
        }
        t
    } else {
        head.clone()
    };
    tail.reverse();

    let head_json: Vec<Json> = head.iter().filter_map(|l| parse(l).ok()).collect();
    let tail_json: Vec<Json> = tail.iter().filter_map(|l| parse(l).ok()).collect();

    let cwd = tail_json
        .iter()
        .chain(head_json.iter())
        .find_map(|v| {
            envelope_field(
                v,
                &["cwd", "workingDirectory", "working_dir", "project_path"],
            )
        })
        .map(PathBuf::from)
        .or_else(|| previous.and_then(|t| t.cwd.clone()));

    // 모델은 뒤에서부터 찾는다. 세션 중간에 모델을 바꿀 수 있고,
    // 우리가 보여줘야 하는 건 지금 쓰는 모델이다.
    let model = tail_json
        .iter()
        .chain(head_json.iter())
        .find_map(|v| envelope_field(v, &["model", "model_id", "modelName"]))
        .or_else(|| previous.and_then(|t| t.model.clone()));

    let first_prompt = head_json
        .iter()
        .find_map(first_user_text)
        .map(|t| normalize_prompt(&t))
        .or_else(|| previous.and_then(|t| t.first_prompt.clone()));

    let request = tail_json
        .iter()
        .find_map(|v| first_user_text(v).map(|text| (v, text)));
    let current_prompt = request
        .as_ref()
        .map(|(_, text)| text.chars().take(4000).collect())
        .or_else(|| previous.and_then(|t| t.current_prompt.clone()));
    // Only user request identity counts: metadata, model changes and tool output do not.
    let request_marker = request
        .as_ref()
        .map(|(v, text)| {
            request_marker(&[
                text.as_str(),
                v.get("timestamp").and_then(Json::as_str).unwrap_or(""),
                v.get("uuid").and_then(Json::as_str).unwrap_or(""),
                v.get("id").and_then(Json::as_str).unwrap_or(""),
            ])
        })
        .or_else(|| previous.and_then(|t| t.request_marker.clone()));
    let session_id = head_json
        .iter()
        .find_map(|v| {
            if v.get("type").and_then(Json::as_str) == Some("session_meta") {
                v.get("payload")
                    .and_then(|p| p.get("id").or_else(|| p.get("session_id")))
            } else if v.get("type").and_then(Json::as_str) == Some("session.start") {
                v.get("data").and_then(|d| d.get("sessionId"))
            } else {
                v.get("sessionId")
            }
            .and_then(Json::as_str)
            .map(str::to_string)
        })
        .or_else(|| previous.and_then(|t| t.session_id.clone()));
    let auxiliary = head_json.iter().any(|v| {
        v.get("isSidechain") == Some(&Json::Bool(true))
            || (v.get("type").and_then(Json::as_str) == Some("session_meta")
                && v.get("payload")
                    .and_then(|p| p.get("source"))
                    .and_then(|s| s.get("subagent"))
                    .is_some())
    }) || previous.is_some_and(|t| t.auxiliary);

    // Claude sidechains can share the parent's sessionId. Their file identifies the child.
    let session_id = if auxiliary
        && !head_json
            .iter()
            .any(|v| v.get("type").and_then(Json::as_str) == Some("session_meta"))
        && previous.is_none_or(|t| t.session_id.is_none())
    {
        None
    } else {
        session_id
    };
    let latest_event = tail_json.iter().find_map(|v| {
        event_state(v).map(|state| {
            use std::hash::{Hash, Hasher};
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            format!("{v:?}").hash(&mut hash);
            (
                state,
                v.get("timestamp")
                    .and_then(Json::as_str)
                    .and_then(time::from_iso8601),
                hash.finish(),
            )
        })
    });
    let event_state = latest_event
        .map(|(state, _, _)| state)
        .or_else(|| previous.and_then(|t| t.event_state));
    let event_time = match latest_event {
        Some((_, timestamp, _)) => timestamp,
        None => previous_cached.and_then(|c| c.event_time),
    };

    let event_signature = latest_event
        .map(|(_, _, signature)| signature)
        .or_else(|| previous_cached.and_then(|c| c.event_signature));
    let inferred_at = previous_cached
        .filter(|c| c.event_signature == event_signature)
        .map(|c| c.transcript.last_event_at)
        .unwrap_or(mtime);
    let transcript = Transcript {
        session_id,
        current_prompt,
        request_marker,
        auxiliary,
        path: path.to_path_buf(),
        last_event_at: event_time.unwrap_or(inferred_at),
        inferred_time: event_time.is_none(),
        cwd,
        model,
        first_prompt,
        event_state,
    };
    // 원문/JSON 트리는 저장하지 않는다. 오래 켜둬도 항목 수는 유한하다.
    // 기존 캐시 전체를 비우지 않는다. 상한 이후 새 파일만 캐시 없이 읽는다.
    if cache.len() < 4096 || cache.contains_key(path) {
        cache.insert(
            path.to_path_buf(),
            Cached {
                event_time,
                event_signature,
                prefix,
                size,
                modified,
                created,
                transcript: transcript.clone(),
            },
        );
    }
    Some(transcript)
}

/// 상태는 이벤트 자체에서만 읽는다. 프롬프트나 도구 출력 안의 문자열은 증거가 아니다.
fn copilot_child(v: &Json) -> bool {
    // Claude sidechains also have agentId; retain their independently displayed logs.
    v.get("data").is_some() && v.get("agentId").and_then(Json::as_str).is_some()
}

pub(crate) fn event_state(v: &Json) -> Option<State> {
    let kind = v.get("type").and_then(Json::as_str)?;
    if copilot_child(v) { return None; }
    match kind {
        "session.error" => return Some(State::Error),
        "session.idle" => return Some(if v.get("data").and_then(|d| d.get("aborted")) == Some(&Json::Bool(true)) { State::Idle } else { State::Waiting }),
        // Explicit CLI shutdown does not mean the conversation cannot be resumed.
        "abort" => return Some(State::Idle),
        "session.shutdown" => return Some(if v.get("data").and_then(|d| d.get("shutdownType")).and_then(Json::as_str) == Some("error") { State::Error } else { State::Idle }),
        "user.message" => return first_user_text(v).map(|_| State::Working),
        "assistant.turn_start" | "assistant.turn_end" | "assistant.message"
        | "tool.execution_start" | "tool.execution_complete" => return Some(State::Working),
        _ => {}
    }
    let event = if matches!(kind, "event_msg" | "response_item") {
        v.get("payload")?
    } else {
        v
    };
    let kind = event.get("type").and_then(Json::as_str).unwrap_or(kind);
    match kind {
        "error" => Some(State::Error),
        "task_complete" => Some(State::Waiting), // 턴 종료는 세션 종료가 아니다.
        "task_started"
        | "user_message"
        | "function_call"
        | "function_call_output"
        | "custom_tool_call"
        | "custom_tool_call_output"
        | "reasoning" => Some(State::Working),
        "turn_aborted" => Some(State::Idle),
        "user" if event.get("toolUseResult").is_some() => Some(State::Working),
        "user" => first_user_text(event).map(|_| State::Working),
        "assistant" => {
            let message = event.get("message")?;
            match message.get("stop_reason").and_then(Json::as_str) {
                Some("end_turn" | "stop_sequence") => Some(State::Waiting),
                _ => Some(State::Working),
            }
        }
        "message" => match event.get("role").and_then(Json::as_str) {
            Some("user") => first_user_text(event).map(|_| State::Working),
            Some("assistant") if event.get("phase").and_then(Json::as_str) == Some("final") => {
                Some(State::Waiting)
            }
            Some("assistant") => Some(State::Working),
            _ => None,
        },
        _ => None,
    }
}

/// 이 줄이 사용자 발화인가?
fn first_user_text(v: &Json) -> Option<String> {
    if copilot_child(v) { return None; }
    if v.get("type").and_then(Json::as_str) == Some("user.message") {
        return clean_user_text(&v.get("data")?.get("content")?.text_content()?);
    }
    let v = if v.get("type").and_then(Json::as_str) == Some("response_item") {
        v.get("payload")?
    } else {
        v
    };
    let is_user = v.get("role").and_then(Json::as_str) == Some("user")
        || v.get("type").and_then(Json::as_str) == Some("user");
    if !is_user || v.get("toolUseResult").is_some() {
        return None;
    }
    let content = v
        .get("message")
        .and_then(|m| m.get("content"))
        .or_else(|| v.get("content"))?;
    let text = content.text_content()?;
    clean_user_text(&text)
}

/// 사용자가 친 게 아니라 하네스가 주입한 텍스트를 걸러낸다.
///
/// 이걸 안 하면 거의 모든 세션의 task 컬럼이 "Caveat: The messages below..."
/// 로 똑같이 채워진다. 실제로 처음 만들면 반드시 밟는 함정이다.
fn clean_user_text(text: &str) -> Option<String> {
    let mut text = text.trim();
    for _ in 0..16 {
        let tag = [
            "environment_context",
            "instructions",
            "system-reminder",
            "recommended_plugins",
            "permissions instructions",
            "collaboration_mode",
        ]
        .into_iter()
        .find(|tag| text.starts_with(&format!("<{tag}>")));
        let Some(tag) = tag else {
            break;
        };
        let close = format!("</{tag}>");
        let end = text.find(&close)?;
        text = text[end + close.len()..].trim();
    }
    if text.is_empty()
        || text.starts_with("Caveat:")
        || text.starts_with("[Request interrupted")
        || text.starts_with("This session is being continued")
        || text.starts_with("# AGENTS.md instructions")
    {
        None
    } else {
        Some(text.to_string())
    }
}

// Read envelope metadata only; a tool result's nested model/cwd is not session metadata.
fn envelope_field(v: &Json, keys: &[&str]) -> Option<String> {
    if copilot_child(v) { return None; }
    let kind = v.get("type").and_then(Json::as_str);
    if matches!(kind, Some("session.start" | "session.model_change" | "session.context_changed" | "session.shutdown")) {
        let data = v.get("data")?;
        let field = if keys.contains(&"model") {
            match kind? {
                "session.start" => data.get("selectedModel"),
                "session.model_change" => data.get("newModel"),
                "session.shutdown" => data.get("currentModel"),
                _ => None,
            }
        } else if kind == Some("session.context_changed") {
            data.get("cwd")
        } else {
            data.get("context").and_then(|v| v.get("cwd"))
        };
        return field.and_then(Json::as_str).filter(|s| !s.is_empty()).map(str::to_string);
    }
    let value = match v.get("type").and_then(Json::as_str) {
        Some("session_meta" | "turn_context") => v.get("payload")?,
        Some("assistant") => {
            for key in keys {
                if let Some(s) = v.get(key).and_then(Json::as_str).filter(|s| !s.is_empty()) {
                    return Some(s.into());
                }
            }
            v.get("message")?
        }
        _ => v,
    };
    keys.iter().find_map(|key| {
        value
            .get(key)
            .and_then(Json::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    })
}

/// 표에 넣을 수 있는 한 줄로 만든다.
///
/// 실제 첫 프롬프트는 코드 블록과 파일 첨부가 섞인 300자짜리 장문인 경우가
/// 많다. 자르기 전에 구조적 노이즈부터 걷어낸다.
pub fn normalize_prompt(raw: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(trimmed);
        // 첫 문장 하나면 충분하다. 대개 거기에 의도가 다 들어 있다.
        if out.len() > 200 {
            break;
        }
    }
    if out.is_empty() {
        out = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    out.chars().take(200).collect()
}

/// `claude-opus-4-6-20260514` → `opus-4.6`
///
/// 표시용으로만 줄인다. 정본 id는 JSON에 그대로 남긴다.
pub fn normalize_model(raw: &str) -> String {
    let s = raw.trim();
    // 날짜 접미사 제거
    let mut parts: Vec<&str> = s.split('-').collect();
    if let Some(last) = parts.last() {
        if last.len() == 8 && last.chars().all(|c| c.is_ascii_digit()) {
            parts.pop();
        }
    }
    // 벤더 접두사 제거
    if matches!(parts.first(), Some(&"claude") | Some(&"models")) && parts.len() > 1 {
        parts.remove(0);
    }
    let joined = parts.join("-");

    // `opus-4-6` → `opus-4.6` : 숫자 사이의 하이픈만 점으로.
    let seg: Vec<&str> = joined.split('-').collect();
    let mut out = String::new();
    for (i, s) in seg.iter().enumerate() {
        if i > 0 {
            let prev_num = seg[i - 1].chars().all(|c| c.is_ascii_digit());
            let cur_num = s.chars().all(|c| c.is_ascii_digit());
            out.push(if prev_num && cur_num { '.' } else { '-' });
        }
        out.push_str(s);
    }
    out
}

// ------------------------------------------------- JSONL 이 아닌 소스 (§4.1)
//
// 코딩 에이전트의 절반은 대화를 append-only JSONL 로 남기지 않는다. 편집기
// 확장은 세션 하나를 JSON 파일 하나로 쓰고, Cursor 같은 앱은 SQLite 에 넣는다.
// 형식만 다르지 우리가 찾는 것은 같다 — 첫 요청, 최근 요청, 경로, 모델,
// 마지막 활동 시각. 그래서 아래 두 리더는 값을 뽑는 방식만 다르고 결과는
// 전부 같은 `Transcript` 다. 그 뒤 단계(session.rs)는 출처를 모른다.

/// JSON 파일은 앞뒤만 잘라 읽을 수 없다 — 통째로 파싱해야 하므로 상한을 둔다.
/// ponytail: 넘으면 건너뛴다. 카드 한 장 때문에 수십 MB를 파싱하지 않는다.
const MAX_JSON_BYTES: u64 = 4 * 1024 * 1024;

/// 이 이름의 파일은 세션 식별자를 파일명 대신 상위 폴더에서 가져온다.
/// 한 폴더에 고정된 이름으로 쌓는 형식(Cline·Gemini)이 여기 해당한다.
const FIXED_NAMES: &[&str] = &["api_conversation_history", "ui_messages", "logs", "info"];

const TIME_KEYS: &[&str] = &[
    "lastUpdatedAt",
    "updatedAt",
    "updated_at",
    "timestamp",
    "ts",
    "time",
    "createdAt",
    "created_at",
];

/// 밀리초로 보이면 초로 내린다. 2001년 이전 시각을 다룰 일은 없다.
fn epoch_seconds(raw: i64) -> i64 {
    if raw > 100_000_000_000 {
        raw / 1000
    } else {
        raw
    }
}

fn time_value(v: &Json) -> Option<i64> {
    for key in TIME_KEYS {
        match v.get(key) {
            Some(Json::Num(n)) if *n > 0.0 => return Some(epoch_seconds(*n as i64)),
            Some(Json::Str(s)) => {
                if let Some(t) = time::from_iso8601(s) {
                    return Some(t);
                }
                if let Ok(n) = s.parse::<i64>() {
                    if n > 0 {
                        return Some(epoch_seconds(n));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// `file:///d%3A/work` 같은 URI 도 경로로 받는다. 편집기 계열은 경로를
/// URI 로 저장한다.
pub(crate) fn local_path(raw: &str) -> Option<PathBuf> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let decoded = match raw.strip_prefix("file://") {
        Some(rest) => {
            let rest = rest.strip_prefix('/').unwrap_or(rest);
            let bytes = rest.as_bytes();
            let mut out = String::with_capacity(rest.len());
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'%' && i + 2 < bytes.len() {
                    if let Ok(b) = u8::from_str_radix(&rest[i + 1..i + 3], 16) {
                        out.push(b as char);
                        i += 3;
                        continue;
                    }
                }
                out.push(bytes[i] as char);
                i += 1;
            }
            out
        }
        None => raw.to_string(),
    };
    // 상대 경로는 어느 기준인지 알 수 없다. 잘못된 프로젝트를 보여주느니 비운다.
    let path = PathBuf::from(decoded);
    (path.is_absolute() || path.has_root()).then_some(path)
}

/// 사용자가 친 요청 하나의 식별자. 종결한 세션이 새 요청으로 돌아올 때
/// 쓰이므로 실행마다 같은 값이어야 한다(Rust 기본 해셔는 그렇지 않다).
fn request_marker(parts: &[&str]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for part in parts {
        for byte in (part.len() as u64).to_le_bytes().iter().chain(part.as_bytes()) {
            hash = (hash ^ *byte as u64).wrapping_mul(0x100000001b3);
        }
    }
    format!("r1:{hash:016x}")
}

/// JSON 문서를 이벤트 목록으로 편다. 배열이면 그대로, 객체면 대화 배열을
/// 찾고, 없으면 객체 하나짜리 목록이다. 그다음은 JSONL 과 같은 추출기를 쓴다.
fn json_events(v: Json) -> Vec<Json> {
    if let Some(items) = v.as_array() {
        return items.to_vec();
    }
    for key in [
        "messages",
        "apiConversationHistory",
        "history",
        "conversation",
        "turns",
        "events",
        "requests",
        "entries",
    ] {
        if let Some(items) = v.get(key).and_then(Json::as_array) {
            if !items.is_empty() {
                return items.to_vec();
            }
        }
    }
    vec![v]
}

fn file_identity(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    if FIXED_NAMES.contains(&stem.as_str()) {
        if let Some(parent) = path.parent().and_then(|p| p.file_name()) {
            return Some(parent.to_string_lossy().into_owned());
        }
    }
    Some(stem)
}

/// JSON 파일 하나 = 세션 하나.
pub fn read_json(path: &Path, mtime: i64) -> Option<Transcript> {
    let size = std::fs::metadata(path).ok()?.len();
    if size == 0 || size > MAX_JSON_BYTES {
        return None;
    }
    let doc = parse(&std::fs::read_to_string(path).ok()?).ok()?;

    let session_id = doc
        .find_str(&["sessionId", "session_id", "taskId", "composerId"])
        .filter(|s| !s.is_empty())
        .or_else(|| file_identity(path));
    let cwd = doc
        .find_str(&[
            "cwd",
            "workingDirectory",
            "working_dir",
            "project_path",
            "workspaceDirectory",
            "folder",
        ])
        .and_then(|s| local_path(&s));
    let model = doc
        .find_str(&["model", "model_id", "modelName"])
        .map(|m| normalize_model(&m));
    let stamp = time_value(&doc);

    let events = json_events(doc);
    let first_prompt = events
        .iter()
        .find_map(first_user_text)
        .map(|t| normalize_prompt(&t));
    // 사용자 요청이 하나도 없으면 세션이 아니다. 편집기 폴더에 굴러다니는
    // 설정 JSON 을 카드로 만들지 않는다.
    first_prompt.as_ref()?;
    let current_prompt = events
        .iter()
        .rev()
        .find_map(first_user_text)
        .map(|t| normalize_prompt(&t).chars().take(4000).collect::<String>());
    let observed = stamp.or_else(|| events.iter().rev().find_map(time_value));
    let last_event_at = observed.unwrap_or(mtime);

    Some(Transcript {
        request_marker: Some(request_marker(&[
            current_prompt.as_deref().unwrap_or_default(),
            &last_event_at.to_string(),
        ])),
        session_id,
        current_prompt,
        first_prompt,
        auxiliary: false,
        path: path.to_path_buf(),
        last_event_at,
        // 이 형식들은 턴 종료 이벤트가 없다. 시각이 파일 것이면 그렇다고 말한다.
        inferred_time: observed.is_none(),
        cwd,
        model,
        event_state: None,
    })
}

fn json_sessions(agent: &str, now: i64, history: bool) -> Vec<Transcript> {
    let (dirs, name) = crate::adapters::json_source(agent);
    let mut files = Vec::new();
    for root in dirs {
        walk(root, now, &mut files, 0, history, "json", name);
    }
    files.sort_by(|a, b| b.1.cmp(&a.1));
    files
        .iter()
        .filter_map(|(p, m)| read_json(p, *m))
        .collect()
}

// ------------------------------------------------------------ SQLite 소스

/// 질의 결과 캐시. 편집기 DB 는 크고 갱신 주기는 2초다. 파일과 WAL 의 시각이
/// 그대로면 질의 자체를 하지 않는다.
static SQLITE_CACHE: OnceLock<Mutex<HashMap<(PathBuf, String), (i64, Vec<Transcript>)>>> =
    OnceLock::new();
/// 질의 실패 사유. `doctor` 전용이라 조용히 모아둔다.
static SQLITE_PROBLEMS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

pub fn source_problems() -> Vec<String> {
    SQLITE_PROBLEMS
        .get()
        .map(|m| m.lock().unwrap_or_else(|e| e.into_inner()).clone())
        .unwrap_or_default()
}

fn note_problem(problem: String) {
    let mut list = SQLITE_PROBLEMS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if !list.contains(&problem) {
        list.push(problem);
    }
}

fn truthy(raw: &str) -> bool {
    !matches!(raw, "" | "0" | "false" | "null")
}

pub(crate) fn row_transcript(file: &Path, row: &crate::sqlite::Row, stamp: i64) -> Option<Transcript> {
    let id = crate::sqlite::get(row, "id")?.to_string();
    let blob = crate::sqlite::get(row, "blob").and_then(|b| parse(b).ok());
    let field = |column: &str, keys: &[&str]| {
        crate::sqlite::get(row, column)
            .map(str::to_string)
            .or_else(|| blob.as_ref().and_then(|b| b.find_str(keys)))
            .filter(|v| !v.trim().is_empty())
    };

    // richText 는 편집기 내부 구조체다. 사람이 읽을 문장만 쓴다.
    let current_prompt = field("task", &["text"])
        .map(|t| normalize_prompt(&t).chars().take(4000).collect::<String>())
        .filter(|t| !t.is_empty());
    // 실제 첫 요청이 제목보다 낫다. 제목은 대화가 비었을 때만 쓴다.
    let first_prompt = field("summary", &["textPreview"])
        .or_else(|| field("summary", &["name", "title"]))
        .map(|t| normalize_prompt(&t))
        .filter(|t| !t.is_empty());
    // 요청도 제목도 없으면 빈 대화다. 카드로 만들지 않는다.
    if current_prompt.is_none() && first_prompt.is_none() {
        return None;
    }

    let last_event_at = crate::sqlite::get(row, "updated_ms")
        .and_then(|v| v.parse::<i64>().ok())
        .or_else(|| crate::sqlite::get(row, "updated_s").and_then(|v| v.parse::<i64>().ok()))
        .filter(|v| *v > 0)
        .map(epoch_seconds)
        .or_else(|| blob.as_ref().and_then(time_value));

    Some(Transcript {
        request_marker: Some(request_marker(&[
            current_prompt.as_deref().or(first_prompt.as_deref()).unwrap_or_default(),
            &last_event_at.unwrap_or(stamp).to_string(),
            &id,
        ])),
        session_id: Some(id.clone()),
        current_prompt,
        first_prompt,
        auxiliary: crate::sqlite::get(row, "auxiliary").is_some_and(truthy),
        // 실제 파일이 아니라 식별자다. 열지 않고 세션을 구분하는 데만 쓴다.
        path: file.join(&id),
        last_event_at: last_event_at.unwrap_or(stamp),
        // DB 행에는 턴 경계가 없다. 시각이 행에서 나오지 않았으면 추정이다.
        inferred_time: last_event_at.is_none(),
        cwd: field("project", &["cwd", "workspaceDirectory", "folder"])
            .and_then(|p| local_path(&p)),
        model: field("model", &["model", "model_id", "modelName"]).map(|m| normalize_model(&m)),
        event_state: None,
    })
}

fn sqlite_sessions(agent: &str, now: i64, history: bool) -> Vec<Transcript> {
    let mut out = Vec::new();
    for source in crate::adapters::queries(agent) {
        let file_time = match time::mtime(&source.file) {
            Some(t) => t,
            // DB 가 없는 게 정상이다 — 그 편집기를 안 쓰는 것뿐이다.
            None => continue,
        };
        // WAL 에 최신 내용이 있을 수 있다. 둘 중 나중 시각을 본다.
        let wal = PathBuf::from(format!("{}-wal", source.file.display()));
        let stamp = time::mtime(&wal).unwrap_or(file_time).max(file_time);
        if !history && now - stamp > MAX_AGE_SECS {
            continue;
        }
        let key = (source.file.clone(), source.sql.clone());
        let mut cache = SQLITE_CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some((cached_at, rows)) = cache.get(&key).filter(|(at, _)| *at == stamp) {
            let _ = cached_at;
            out.extend(rows.iter().cloned());
            continue;
        }
        match crate::sqlite::query(&source.file, &source.sql) {
            Ok(rows) => {
                let found: Vec<Transcript> = rows
                    .iter()
                    .filter_map(|r| row_transcript(&source.file, r, stamp))
                    .collect();
                cache.insert(key, (stamp, found.clone()));
                out.extend(found);
            }
            // 스키마는 편집기 업데이트마다 바뀔 수 있다. 그 소스만 비우고
            // 나머지 에이전트는 그대로 보여준다.
            Err(e) => note_problem(format!("{}: {e}", source.file.display())),
        }
    }
    out
}
