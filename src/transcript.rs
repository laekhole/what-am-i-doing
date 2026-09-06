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

/// 루트 아래 `.jsonl` 파일을 재귀 수집한다.
fn walk(dir: &Path, now: i64, out: &mut Vec<(PathBuf, i64)>, depth: u32, history: bool) {
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
            walk(&path, now, out, depth + 1, history);
        } else if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
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
        walk(&root, now, &mut files, 0, history);
    }
    // 최근 것부터. 아래에서 프로세스와 짝지을 때 최신이 우선권을 갖는다.
    files.sort_by(|a, b| b.1.cmp(&a.1));
    let mut found: Vec<_> = files
        .iter()
        .filter_map(|(p, m)| read(p, *m))
        .filter(|t| history || now - t.last_event_at <= MAX_AGE_SECS)
        .collect();
    found.sort_by(|a, b| b.last_event_at.cmp(&a.last_event_at));
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
            // Persisted bookmarks need a defined hash, not Rust's unspecified DefaultHasher.
            let mut hash = 0xcbf29ce484222325u64;
            for part in [
                text.as_str(),
                v.get("timestamp").and_then(Json::as_str).unwrap_or(""),
                v.get("uuid").and_then(Json::as_str).unwrap_or(""),
                v.get("id").and_then(Json::as_str).unwrap_or(""),
            ] {
                for byte in (part.len() as u64)
                    .to_le_bytes()
                    .iter()
                    .chain(part.as_bytes())
                {
                    hash = (hash ^ *byte as u64).wrapping_mul(0x100000001b3);
                }
            }
            format!("r1:{hash:016x}")
        })
        .or_else(|| previous.and_then(|t| t.request_marker.clone()));
    let session_id = head_json
        .iter()
        .find_map(|v| {
            if v.get("type").and_then(Json::as_str) == Some("session_meta") {
                v.get("payload")
                    .and_then(|p| p.get("id").or_else(|| p.get("session_id")))
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
pub(crate) fn event_state(v: &Json) -> Option<State> {
    let kind = v.get("type").and_then(Json::as_str)?;
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
