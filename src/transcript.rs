//! 트랜스크립트 읽기.
//!
//! 두 가지 원칙이 이 모듈의 모양을 결정한다.
//!
//! **하나. 전체를 읽지 않는다.** 긴 세션의 JSONL은 수십 MB까지 간다. 우리가
//! 필요한 건 첫 사용자 프롬프트(파일 앞)와 마지막 이벤트(파일 뒤)뿐이다.
//! 앞에서 몇 KB, 뒤에서 몇 KB만 읽는다. 성능 예산(§8) 대부분이 여기서 지켜진다.
//!
//! **둘. 마지막 이벤트 시각은 파싱하지 않는다.** 파일의 mtime이 곧 마지막
//! 이벤트 시각이다. 공짜이고, 정확하고, 스키마가 바뀌어도 안 깨진다.

use crate::json::{parse, Json};
use crate::time;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const HEAD_BYTES: u64 = 32 * 1024;
const TAIL_BYTES: u64 = 64 * 1024;
/// 이보다 오래된 트랜스크립트는 후보에서 뺀다. 수백 개씩 쌓이는 과거
/// 세션 파일을 매 폴링마다 stat 하지 않기 위한 방어선.
const MAX_AGE_SECS: i64 = 24 * 3600;

#[derive(Debug, Clone)]
pub struct Transcript {
    pub path: PathBuf,
    pub last_event_at: i64,
    pub cwd: Option<PathBuf>,
    pub model: Option<String>,
    pub first_prompt: Option<String>,
    pub errored: bool,
}

/// 에이전트별 트랜스크립트 루트.
///
/// 하드코딩하지 않는다. 어댑터 테이블(§4)이 유일한 출처이므로, 사용자가
/// `[transcript] dir = "~/.mytool/sessions"` 한 줄을 적으면 이 함수가
/// 자동으로 새 에이전트를 훑기 시작한다.
fn roots(agent: &str) -> Vec<PathBuf> {
    crate::adapters::transcript_dirs(agent).to_vec()
}

/// 루트 아래 `.jsonl` 파일을 재귀 수집한다. 최근 것만.
fn walk(dir: &Path, now: i64, out: &mut Vec<(PathBuf, i64)>, depth: u32) {
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
            walk(&path, now, out, depth + 1);
        } else if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
            if let Some(m) = time::mtime(&path) {
                if now - m <= MAX_AGE_SECS {
                    out.push((path, m));
                }
            }
        }
    }
}

pub fn discover(agent: &str, now: i64) -> Vec<Transcript> {
    let mut files = Vec::new();
    for root in roots(agent) {
        walk(&root, now, &mut files, 0);
    }
    // 최근 것부터. 아래에서 프로세스와 짝지을 때 최신이 우선권을 갖는다.
    files.sort_by(|a, b| b.1.cmp(&a.1));
    files.iter().filter_map(|(p, m)| read(p, *m)).collect()
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
    let size = f.metadata().ok()?.len();

    let head = read_span(&mut f, SeekFrom::Start(0), HEAD_BYTES.min(size) as usize);

    let mut tail = if size > TAIL_BYTES {
        let mut t = read_span(&mut f, SeekFrom::End(-(TAIL_BYTES as i64)), TAIL_BYTES as usize);
        // 중간에서 잘렸으므로 첫 줄은 깨진 JSON일 수 있다.
        if !t.is_empty() {
            t.remove(0);
        }
        t
    } else {
        head.clone()
    };
    tail.reverse();

    let head_json: Vec<Json> = head.iter().filter_map(|l| parse(l).ok()).collect();
    let tail_json: Vec<Json> = tail.iter().filter_map(|l| parse(l).ok()).collect();

    let cwd = head_json
        .iter()
        .find_map(|v| v.find_str(&["cwd", "workingDirectory", "working_dir", "project_path"]))
        .map(PathBuf::from);

    // 모델은 뒤에서부터 찾는다. 세션 중간에 모델을 바꿀 수 있고,
    // 우리가 보여줘야 하는 건 지금 쓰는 모델이다.
    let model = tail_json
        .iter()
        .chain(head_json.iter())
        .find_map(|v| v.find_str(&["model", "model_id", "modelName"]));

    let first_prompt = head_json.iter().find_map(first_user_text);

    let errored = tail_json.iter().take(5).any(|v| {
        v.has_kv("type", "error") || v.has_kv("subtype", "error") || v.has_kv("status", "failed")
    });

    Some(Transcript {
        path: path.to_path_buf(),
        last_event_at: mtime,
        cwd,
        model: model.map(|m| normalize_model(&m)),
        first_prompt: first_prompt.map(|t| normalize_prompt(&t)),
        errored,
    })
}

/// 이 줄이 사용자 발화인가?
fn first_user_text(v: &Json) -> Option<String> {
    let is_user = v.has_kv("role", "user") || v.has_kv("type", "user");
    if !is_user {
        return None;
    }
    // 도구 실행 결과도 role=user로 기록된다. 그건 사용자가 시킨 일이 아니다.
    if v.has_kv("type", "tool_result") || v.get("toolUseResult").is_some() {
        return None;
    }
    let content = v
        .get("message")
        .and_then(|m| m.get("content"))
        .or_else(|| v.get("content"))?;
    let text = content.text_content()?;
    if is_noise(&text) {
        None
    } else {
        Some(text)
    }
}

/// 사용자가 친 게 아니라 하네스가 주입한 텍스트를 걸러낸다.
///
/// 이걸 안 하면 거의 모든 세션의 task 컬럼이 "Caveat: The messages below..."
/// 로 똑같이 채워진다. 실제로 처음 만들면 반드시 밟는 함정이다.
fn is_noise(t: &str) -> bool {
    let t = t.trim_start();
    t.is_empty()
        || t.starts_with('<')
        || t.starts_with("Caveat:")
        || t.starts_with("[Request interrupted")
        || t.starts_with("This session is being continued")
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
    out
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
