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
    pub last_answer: Option<String>,
    pub session_id: Option<String>,
    pub current_prompt: Option<String>,
    pub request_marker: Option<String>,
    pub request_at: Option<i64>,
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
    /// 머리 구간에서 건너뛴 줄 수, 그리고 이 회차 전체의 합. 캐시로 답할 때도
    /// 같은 진단을 다시 내보내야 한다 — 두 번째 스냅샷부터 조용해지면 사용자는
    /// 문제가 사라졌다고 읽는다.
    head_unparsed: usize,
    head_counted_until: u64,
    unparsed: usize,
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
pub(crate) fn walk(
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
        // 루트가 없는 것은 정상이다 — 그 도구를 안 쓰는 것뿐이다. 권한 거부처럼
        // 읽을 수 있었어야 하는 폴더만 진단에 남긴다.
        Err(e) => return crate::diag::dir_error(dir, &e),
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                crate::diag::dir_error(dir, &e);
                continue;
            }
        };
        let path = entry.path();
        let ft = match entry.file_type() {
            Ok(t) => t,
            Err(e) => {
                crate::diag::file_error(&path, &e);
                continue;
            }
        };
        if ft.is_dir() {
            walk(&path, now, out, depth + 1, history, ext, name);
        } else if path.extension().map(|e| e == ext).unwrap_or(false)
            && name.is_none_or(|want| path.file_name().is_some_and(|got| got == want))
        {
            // mtime 을 못 읽으면 그 파일은 목록에서 통째로 빠진다. 조용히 빠지면
            // 사용자는 세션이 없는 것과 구분할 수 없다.
            match time::mtime(&path) {
                Some(m) => {
                    if history || now - m <= MAX_AGE_SECS {
                        out.push((path, m));
                    }
                }
                // 실패한 경우에만 사유를 다시 묻는다. 성공 경로에 stat 을 더하지 않는다.
                None => match std::fs::metadata(&path).and_then(|m| m.modified()) {
                    Ok(_) => {}
                    Err(e) => crate::diag::file_error(&path, &e),
                },
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

/// 파일 한 구간의 줄들.
///
/// `partial_end` 는 구간이 개행으로 끝나지 않았다는 뜻이다 — 상한에서 잘렸거나
/// 에이전트가 지금 그 줄을 쓰고 있다. 그 줄은 파싱은 해 보되 형식 오류로는
/// 세지 않는다. 살아 있는 로그가 매 스냅샷마다 경고를 내면 진단이 무의미해진다.
#[derive(Default)]
struct Span {
    lines: Vec<String>,
    partial_end: bool,
}

fn read_span(f: &mut File, from: SeekFrom, len: usize, path: &Path) -> Span {
    if let Err(e) = f.seek(from) {
        crate::diag::file_error(path, &e);
        return Span::default();
    }
    let mut buf = vec![0u8; len];
    let n = match f.read(&mut buf) {
        Ok(n) => n,
        Err(e) => {
            crate::diag::file_error(path, &e);
            return Span::default();
        }
    };
    buf.truncate(n);
    Span {
        partial_end: buf.last().is_some_and(|b| *b != b'\n'),
        lines: String::from_utf8_lossy(&buf)
            .split_terminator('\n')
            .map(str::to_string)
            .collect(),
    }
}

/// 줄을 JSON 으로 모으면서 읽지 못한 줄을 센다.
///
/// 세지 **않는** 줄이 셋이다. `skip` 은 구간 상한에서 잘린 경계 줄이고(파싱은
/// 그대로 시도한다), 빈 줄은 애초에 기록이 아니며, 줄 끝이 `count_from` 이하면
/// 앞 구간에서 이미 셌다. 경계를 걸친 줄은 뒤 구간에서 센다. 겹치는 구간을 두 번 세면
/// 파일이 스냅샷마다 늘어나는 숫자를 달고 나온다.
fn parse_lines(
    lines: &[String],
    skip: Option<usize>,
    mut at: u64,
    count_from: u64,
    unparsed: &mut usize,
) -> Vec<Json> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            at += line.len() as u64 + 1; // CR은 보존하고 LF 한 바이트만 더한다.
            match parse(line) {
                Ok(v) => Some(v),
                Err(_) => {
                    if skip != Some(i) && at > count_from && !line.trim().is_empty() {
                        *unparsed += 1;
                    }
                    None
                }
            }
        })
        .collect()
}

pub fn read(path: &Path, mtime: i64) -> Option<Transcript> {
    let mut f = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            crate::diag::file_error(path, &e);
            return None;
        }
    };
    let metadata = match f.metadata() {
        Ok(m) => m,
        Err(e) => {
            crate::diag::file_error(path, &e);
            return None;
        }
    };
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
        if c.unparsed > 0 {
            crate::diag::unparsed_lines(path, c.unparsed);
        }
        return Some(c.transcript.clone());
    }
    // 세션 JSONL은 append-only. 잘리거나 교체되면 머리말도 다시 읽는다.
    let mut prefix = vec![0; size.min(512) as usize];
    if let Err(e) = f.read_exact(&mut prefix) {
        crate::diag::file_error(path, &e);
        return None;
    }
    let previous_cached =
        cached.filter(|c| c.size < size && c.created == created && prefix.starts_with(&c.prefix));
    let previous = previous_cached.map(|c| &c.transcript);
    let reuse_head = previous.is_some_and(|t| t.first_prompt.is_some());

    let head = if reuse_head {
        Span::default()
    } else {
        read_span(
            &mut f,
            SeekFrom::Start(0),
            HEAD_BYTES.min(size) as usize,
            path,
        )
    };

    let separate_tail = reuse_head || size > HEAD_BYTES;
    // 꼬리 구간이 파일의 어느 바이트에서 시작하는지. 머리와 겹치는 줄을
    // 두 번 세지 않기 위해 필요하다.
    let mut tail_at = 0u64;
    let tail = if separate_tail {
        let len = TAIL_BYTES.min(size);
        tail_at = size - len;
        let mut t = read_span(&mut f, SeekFrom::End(-(len as i64)), len as usize, path);
        // 중간에서 잘렸으므로 첫 줄은 깨진 JSON일 수 있다.
        if size > len && !t.lines.is_empty() {
            tail_at += t.lines.remove(0).len() as u64 + 1;
        }
        t
    } else {
        Span {
            lines: head.lines.clone(),
            partial_end: head.partial_end,
        }
    };

    // 형식 오류 집계. 머리는 구간 상한에서, 꼬리는 쓰는 중인 끝줄에서 잘린다.
    // 그 두 줄과 겹치는 구간은 세지 않는다.
    let mut head_unparsed = 0usize;
    let head_json = parse_lines(
        &head.lines,
        head.partial_end.then(|| head.lines.len().saturating_sub(1)),
        0,
        0,
        &mut head_unparsed,
    );
    if reuse_head {
        // 머리를 다시 읽지 않은 회차다. 앞서 센 값을 이어받는다.
        head_unparsed = previous_cached.map_or(0, |c| c.head_unparsed);
    }
    let head_counted_until = if reuse_head {
        previous_cached.map_or(0, |c| c.head_counted_until)
    } else {
        size.min(HEAD_BYTES)
    };
    let mut unparsed = head_unparsed;
    let mut tail_json = parse_lines(
        &tail.lines,
        tail.partial_end.then(|| tail.lines.len().saturating_sub(1)),
        tail_at,
        if separate_tail {
            head_counted_until
        } else {
            u64::MAX
        },
        &mut unparsed,
    );
    // 최신 이벤트부터 본다. 줄이 아니라 결과를 뒤집으면 바이트 위치를 잃지 않는다.
    tail_json.reverse();
    if unparsed > 0 {
        crate::diag::unparsed_lines(path, unparsed);
    }

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
    let last_answer = tail_json
        .iter()
        .take_while(|v| first_user_text(v).is_none())
        .find_map(assistant_text)
        .or_else(|| {
            if request.is_none() {
                previous.and_then(|t| t.last_answer.clone())
            } else {
                None
            }
        });
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
    let request_at = match request.as_ref() {
        Some((v, _)) => v
            .get("timestamp")
            .and_then(Json::as_str)
            .and_then(time::from_iso8601),
        None => previous.and_then(|t| t.request_at),
    };
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
    let auxiliary = head_json.iter().chain(&tail_json).any(auxiliary_event)
        || previous.is_some_and(|t| t.auxiliary);

    // Claude sidechains can share the parent's sessionId. Their file identifies the child.
    let session_id = if head_json.iter().chain(&tail_json)
        .any(|v| v.get("isSidechain") == Some(&Json::Bool(true)))
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
        last_answer,
        session_id,
        current_prompt,
        request_marker,
        request_at,
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
                head_unparsed,
                head_counted_until,
                unparsed,
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
    if copilot_child(v) {
        return None;
    }
    match kind {
        "session.error" => return Some(State::Error),
        "session.idle" => {
            return Some(
                if v.get("data").and_then(|d| d.get("aborted")) == Some(&Json::Bool(true)) {
                    State::Idle
                } else {
                    State::Waiting
                },
            )
        }
        // Explicit CLI shutdown does not mean the conversation cannot be resumed.
        "abort" => return Some(State::Idle),
        "session.shutdown" => {
            return Some(
                if v.get("data")
                    .and_then(|d| d.get("shutdownType"))
                    .and_then(Json::as_str)
                    == Some("error")
                {
                    State::Error
                } else {
                    State::Idle
                },
            )
        }
        "user.message" => return first_user_text(v).map(|_| State::Working),
        "assistant.turn_start"
        | "assistant.turn_end"
        | "assistant.message"
        | "tool.execution_start"
        | "tool.execution_complete" => return Some(State::Working),
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

/// Evidence that this session belongs to another agent.
fn auxiliary_event(v: &Json) -> bool {
    if v.get("isSidechain") == Some(&Json::Bool(true)) {
        return true;
    }
    if v.get("type").and_then(Json::as_str) == Some("session_meta") {
        if let Some(meta) = v.get("payload") {
            if meta.get("parent_thread_id").and_then(Json::as_str).is_some_and(|s| !s.is_empty())
                || ["source", "thread_source"].iter().any(|key| meta.get(key).is_some_and(|source|
                    source.as_str() == Some("subagent")
                        || source.get("subagent").is_some_and(|v| !matches!(v, Json::Null))))
            {
                return true;
            }
        }
    }
    // Orca launches ordinary provider sessions and injects this worker preamble.
    // Match the user envelope, never quoted tool output or coordinator instructions.
    first_user_text(v).is_some_and(|text| text.starts_with(
        "You are working inside Orca, a multi-agent IDE. You are a dispatched worker."
    ) && text.lines().any(|line| line.starts_with("Your task ID is: task_")))
}

/// 이 줄이 사용자 발화인가?
fn first_user_text(v: &Json) -> Option<String> {
    if copilot_child(v) {
        return None;
    }
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

/// Visible assistant text only; thinking and tool blocks are excluded.
fn assistant_text(v: &Json) -> Option<String> {
    if copilot_child(v) {
        return None;
    }
    let kind = v.get("type").and_then(Json::as_str);
    let content = match kind {
        Some("assistant.message") => v.get("data")?.get("content")?,
        Some("event_msg") if v.get("payload")?.get("type")?.as_str() == Some("agent_message") => {
            v.get("payload")?.get("message")?
        }
        _ => {
            let message = if kind == Some("response_item") {
                v.get("payload")?
            } else {
                v
            };
            if message.get("role").and_then(Json::as_str) != Some("assistant")
                && message.get("type").and_then(Json::as_str) != Some("assistant")
            {
                return None;
            }
            message.get("message").unwrap_or(message).get("content")?
        }
    };
    content
        .text_content()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.chars().take(4000).collect())
}

/// Filter harness-injected context from real user requests.
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
        || text.starts_with("<task-notification>")
    {
        None
    } else {
        Some(text.to_string())
    }
}

// Read envelope metadata only; a tool result's nested model/cwd is not session metadata.
fn envelope_field(v: &Json, keys: &[&str]) -> Option<String> {
    if copilot_child(v) {
        return None;
    }
    let kind = v.get("type").and_then(Json::as_str);
    if matches!(
        kind,
        Some(
            "session.start"
                | "session.model_change"
                | "session.context_changed"
                | "session.shutdown"
        )
    ) {
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
        return field
            .and_then(Json::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
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
const REQUEST_ID_KEYS: &[&str] = &[
    "requestId",
    "request_id",
    "messageId",
    "message_id",
    "uuid",
    "id",
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
    time_value_from(v, TIME_KEYS)
}

fn request_time_value(v: &Json) -> Option<i64> {
    time_value_from(
        v,
        &[
            "requestAt",
            "request_at",
            "timestamp",
            "ts",
            "time",
            "createdAt",
            "created_at",
        ],
    )
}

fn request_time_evidence(v: &Json) -> Option<String> {
    time_evidence_from(
        v,
        &[
            "requestAt",
            "request_at",
            "timestamp",
            "ts",
            "time",
            "createdAt",
            "created_at",
        ],
    )
}

fn request_id(v: &Json) -> Option<&str> {
    REQUEST_ID_KEYS
        .iter()
        .find_map(|key| v.get(key).and_then(Json::as_str))
}

fn time_value_from(v: &Json, keys: &[&str]) -> Option<i64> {
    for key in keys {
        match v.get(key) {
            Some(Json::Num(n)) if *n > 0.0 => return Some(epoch_seconds(*n as i64)),
            Some(Json::Str(s)) => {
                if let Some(t) = time_text_value(s) {
                    return Some(t);
                }
            }
            _ => {}
        }
    }
    None
}

fn time_text_value(text: &str) -> Option<i64> {
    time::from_iso8601(text).or_else(|| {
        text.parse::<f64>()
            .ok()
            .filter(|n| n.is_finite() && *n > 0.0)
            .map(|n| epoch_seconds(n as i64))
    })
}

fn time_evidence_from(v: &Json, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| match v.get(key) {
        Some(Json::Num(n)) if n.is_finite() && *n > 0.0 => Some(n.to_string()),
        Some(Json::Str(s)) if time_text_value(s).is_some() => Some(s.clone()),
        _ => None,
    })
}

/// `file:///d%3A/work` 같은 URI 도 경로로 받는다. 편집기 계열은 경로를
/// URI 로 저장한다.
pub(crate) fn local_path(raw: &str) -> Option<PathBuf> {
    let raw = raw.trim();
    if raw.is_empty() || raw.chars().any(char::is_control) {
        return None;
    }
    let Some(scheme) = raw.get(..7) else {
        let path = PathBuf::from(raw);
        return (path.is_absolute() || path.has_root()).then_some(path);
    };
    if !scheme.eq_ignore_ascii_case("file://") {
        let path = PathBuf::from(raw);
        return (path.is_absolute() || path.has_root()).then_some(path);
    }
    let rest = &raw[7..];
    if rest.contains(['?', '#']) {
        return None;
    }
    let decode = |text: &str| {
        let bytes = text.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' {
                let hex = bytes.get(i + 1..i + 3)?;
                out.push(u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?);
                i += 3;
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        }
        String::from_utf8(out)
            .ok()
            .filter(|s| !s.chars().any(char::is_control))
    };
    let decoded = if rest.starts_with('/') {
        let mut path = decode(rest)?;
        let bytes = path.as_bytes();
        if cfg!(windows)
            && bytes.len() >= 3
            && bytes[0] == b'/'
            && bytes[1].is_ascii_alphabetic()
            && bytes[2] == b':'
        {
            path.remove(0);
        }
        path
    } else {
        let (authority, path) = rest.split_once('/')?;
        let authority = decode(authority)?;
        if authority.eq_ignore_ascii_case("localhost") {
            let mut path = decode(path)?;
            if !(cfg!(windows)
                && path.len() >= 2
                && path.as_bytes()[0].is_ascii_alphabetic()
                && path.as_bytes()[1] == b':')
            {
                path.insert(0, '/');
            }
            path
        } else {
            if authority.is_empty()
                || authority == "."
                || authority == ".."
                || authority.contains(['\\', '/', ':', '@'])
            {
                return None;
            }
            if cfg!(windows) {
                format!(r"\\{}\{}", authority, decode(path)?.replace('/', "\\"))
            } else {
                format!("//{authority}/{}", decode(path)?)
            }
        }
    };
    // 상대 경로는 어느 기준인지 알 수 없다. 잘못된 프로젝트를 보여주느니 비운다.
    let path = PathBuf::from(decoded);
    (path.is_absolute() || path.has_root()).then_some(path)
}

/// 사용자가 친 요청 하나의 식별자. 종결한 세션이 새 요청으로 돌아올 때
/// 쓰이므로 실행마다 같은 값이어야 한다(Rust 기본 해셔는 그렇지 않다).
fn request_marker(parts: &[&str]) -> String {
    request_marker_version("r1", parts)
}

fn stable_request_marker(parts: &[&str]) -> String {
    request_marker_version("r3", parts)
}

fn request_marker_version(version: &str, parts: &[&str]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for part in parts {
        for byte in (part.len() as u64)
            .to_le_bytes()
            .iter()
            .chain(part.as_bytes())
        {
            hash = (hash ^ *byte as u64).wrapping_mul(0x100000001b3);
        }
    }
    format!("{version}:{hash:016x}")
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
    let size = match std::fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            crate::diag::file_error(path, &e);
            return None;
        }
    };
    if size == 0 {
        return None;
    }
    if size > MAX_JSON_BYTES {
        crate::diag::oversized(path, size, MAX_JSON_BYTES);
        return None;
    }
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            crate::diag::file_error(path, &e);
            return None;
        }
    };
    let mut bytes = Vec::with_capacity(size.min(MAX_JSON_BYTES + 1) as usize);
    if let Err(e) = file.take(MAX_JSON_BYTES + 1).read_to_end(&mut bytes) {
        crate::diag::file_error(path, &e);
        return None;
    }
    if bytes.len() as u64 > MAX_JSON_BYTES {
        crate::diag::oversized(path, bytes.len() as u64, MAX_JSON_BYTES);
        return None;
    }
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            crate::diag::unparsed_document(path);
            return None;
        }
    };
    let doc = match parse(&text) {
        Ok(doc) => doc,
        Err(_) => {
            crate::diag::unparsed_document(path);
            return None;
        }
    };

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
    let document_model = ["model", "model_id", "modelName"]
        .iter()
        .find_map(|key| doc.get(key).and_then(Json::as_str))
        .filter(|model| !model.is_empty())
        .map(str::to_string);
    let stamp = time_value(&doc);

    let events = json_events(doc);
    let model = events
        .iter()
        .rev()
        .find_map(|event| envelope_field(event, &["model", "model_id", "modelName"]))
        .or(document_model)
        .map(|model| normalize_model(&model));
    let requests: Vec<_> = events
        .iter()
        .filter_map(|event| first_user_text(event).map(|text| (event, text)))
        .collect();
    let first_prompt = requests.first().map(|(_, text)| normalize_prompt(text));
    // 사용자 요청이 하나도 없으면 세션이 아니다. 편집기 폴더에 굴러다니는
    // 설정 JSON 을 카드로 만들지 않는다.
    first_prompt.as_ref()?;
    let (request, request_text) = requests.last()?;
    let current_prompt = Some(
        request_text
            .chars()
            .take(4000)
            .collect::<String>(),
    );
    let request_id = request_id(request).unwrap_or("");
    let request_at = request_time_value(request);
    let request_time = request_time_evidence(request).unwrap_or_default();
    let request_count = requests.len().to_string();
    let observed = stamp.or_else(|| events.iter().rev().find_map(time_value));
    let last_event_at = observed.unwrap_or(mtime);

    Some(Transcript {
        request_at,
        last_answer: events
            .iter()
            .rev()
            .take_while(|v| first_user_text(v).is_none())
            .find_map(assistant_text),
        request_marker: Some(stable_request_marker(&[
            request_text,
            request_id,
            &request_time,
            &request_count,
        ])),
        session_id,
        current_prompt,
        first_prompt,
        auxiliary: events.iter().any(auxiliary_event),
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
    // 크기 상한·형식 오류는 read_json 이 직접 진단에 남긴다. 여기서 미리
    // stat 을 한 번 더 하면 같은 사실을 두 번 확인하는 것뿐이다.
    files.iter().filter_map(|(p, m)| read_json(p, *m)).collect()
}

// ------------------------------------------------------------ SQLite 소스

/// 질의 결과 캐시. 편집기 DB 는 크고 갱신 주기는 2초다. 파일과 WAL 의 시각이
/// 그대로면 질의 자체를 하지 않는다.
type MetadataStamp = (Option<SystemTime>, u64);
type SqliteStamp = (Option<MetadataStamp>, Option<MetadataStamp>);

static SQLITE_CACHE: OnceLock<Mutex<HashMap<(PathBuf, String), (SqliteStamp, Vec<Transcript>)>>> =
    OnceLock::new();

fn sqlite_stamp(file: &Path) -> SqliteStamp {
    let stamp = |path: &Path| {
        std::fs::metadata(path)
            .ok()
            .map(|metadata| (metadata.modified().ok(), metadata.len()))
    };
    (
        stamp(file),
        stamp(&PathBuf::from(format!("{}-wal", file.display()))),
    )
}

fn stamp_seconds(stamp: &MetadataStamp) -> Option<i64> {
    stamp
        .0
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
}

#[cfg(test)]
pub(crate) fn sqlite_cache_fingerprint(file: &Path) -> String {
    format!("{:?}", sqlite_stamp(file))
}
/// 이번 스냅샷에서 읽지 못한 것들. `doctor` 와 JSON `warnings` 가 같은 목록을
/// 본다 — 진단이 두 벌이 되면 둘 중 하나는 반드시 낡는다.
pub fn source_problems() -> Vec<String> {
    crate::diag::warnings()
}

fn truthy(raw: &str) -> bool {
    !matches!(raw, "" | "0" | "false" | "null")
}

pub(crate) fn row_transcript(
    file: &Path,
    row: &crate::sqlite::Row,
    stamp: i64,
) -> Option<Transcript> {
    let id = crate::sqlite::get(row, "id")?.to_string();
    let blob = crate::sqlite::get(row, "blob").and_then(|b| parse(b).ok());
    let field = |column: &str, keys: &[&str]| {
        crate::sqlite::get(row, column)
            .map(str::to_string)
            .or_else(|| blob.as_ref().and_then(|b| b.find_str(keys)))
            .filter(|v| !v.trim().is_empty())
    };

    let events = blob.clone().map(json_events).unwrap_or_default();
    let requests: Vec<_> = events
        .iter()
        .filter_map(|event| first_user_text(event).map(|text| (event, text)))
        .collect();
    let request = requests.last();
    // Only an explicit task or a parsed user event is request evidence. A title/summary is not.
    let stored_request = crate::sqlite::get(row, "task")
        .map(str::to_string)
        .or_else(|| {
            blob.as_ref()
                .and_then(|v| v.get("text"))
                .and_then(Json::as_str)
                .map(str::to_string)
        })
        .filter(|v| !v.trim().is_empty());
    let request_text = request
        .map(|(_, text)| text.as_str())
        .or(stored_request.as_deref());
    let current_prompt = request_text
        .map(|t| t.chars().take(4000).collect::<String>())
        .filter(|t| !t.is_empty());
    // 실제 첫 요청이 제목보다 낫다. 제목은 대화가 비었을 때만 쓴다.
    let first_prompt = requests
        .first()
        .map(|(_, text)| normalize_prompt(text))
        .or_else(|| field("summary", &["textPreview"]).map(|t| normalize_prompt(&t)))
        .or_else(|| field("summary", &["name", "title"]).map(|t| normalize_prompt(&t)))
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
    let request_id = request
        .and_then(|(event, _)| request_id(event))
        .or_else(|| crate::sqlite::get(row, "request_id"))
        .or_else(|| crate::sqlite::get(row, "message_id"))
        .or_else(|| {
            blob.as_ref().and_then(|value| {
                ["requestId", "request_id", "messageId", "message_id"]
                    .iter()
                    .find_map(|key| value.get(key).and_then(Json::as_str))
            })
        })
        .unwrap_or("");
    let request_at = request
        .and_then(|(event, _)| request_time_value(event))
        .or_else(|| crate::sqlite::get(row, "request_at").and_then(time_text_value))
        .or_else(|| {
            blob.as_ref()
                .and_then(|value| time_value_from(value, &["requestAt", "request_at"]))
        });
    let request_time = request
        .and_then(|(event, _)| request_time_evidence(event))
        .or_else(|| {
            crate::sqlite::get(row, "request_at")
                .filter(|value| time_text_value(value).is_some())
                .map(str::to_string)
        })
        .or_else(|| blob.as_ref().and_then(request_time_evidence))
        .unwrap_or_default();
    let request_count = requests.len().to_string();

    Some(Transcript {
        request_at,
        last_answer: None,
        request_marker: request_text
            .map(|text| stable_request_marker(&[text, request_id, &request_time, &request_count])),
        session_id: Some(id.clone()),
        current_prompt,
        first_prompt,
        auxiliary: crate::sqlite::get(row, "auxiliary").is_some_and(truthy)
            || events.iter().any(auxiliary_event),
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
        let signature = sqlite_stamp(&source.file);
        let file_time = match signature.0.as_ref().and_then(stamp_seconds) {
            Some(time) => time,
            // DB 가 없는 게 정상이다 — 그 편집기를 안 쓰는 것뿐이다. 있는데
            // 못 읽는 것만 남긴다.
            None => {
                if let Err(e) = std::fs::metadata(&source.file) {
                    crate::diag::file_error(&source.file, &e);
                }
                continue;
            }
        };
        // WAL 에 최신 내용이 있을 수 있다. 둘 중 나중 시각을 본다.
        let stamp = signature
            .1
            .as_ref()
            .and_then(stamp_seconds)
            .unwrap_or(file_time)
            .max(file_time);
        if !history && now - stamp > MAX_AGE_SECS {
            continue;
        }
        let key = (source.file.clone(), source.sql.clone());
        let mut cache = SQLITE_CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some((_, rows)) = cache.get(&key).filter(|(at, _)| *at == signature) {
            out.extend(rows.iter().cloned());
            continue;
        }
        match crate::sqlite::query(&source.file, &source.sql) {
            Ok(rows) => {
                let found: Vec<Transcript> = rows
                    .iter()
                    .filter_map(|r| row_transcript(&source.file, r, stamp))
                    .collect();
                cache.insert(key, (signature, found.clone()));
                out.extend(found);
            }
            // 스키마는 편집기 업데이트마다 바뀔 수 있다. 그 소스만 비우고
            // 나머지 에이전트는 그대로 보여준다.
            Err(e) => crate::diag::sqlite_error(&source.file, &e),
        }
    }
    out
}
