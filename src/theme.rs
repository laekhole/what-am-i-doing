//! 테마 — 커스터마이징 레이어 1 (매니페스트 §6).
//!
//! TOML 크레이트를 쓰지 않는다. 우리가 읽는 것은 `[section]`과 `key = value`,
//! 그리고 문자열 배열뿐이다. 완전한 TOML을 지원한다고 약속하지 않았으므로
//! 완전한 TOML 파서가 필요하지도 않다.
//!
//! 중요한 성질: **테마 파일이 없어도, 깨져 있어도 프로그램은 동작한다.**
//! 파싱 실패는 조용히 기본값으로 되돌아간다(§1.4).

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Column {
    pub key: String,
    pub width: usize,
    pub truncate: Truncate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Truncate {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub columns: Vec<Column>,
    pub status_priority: Vec<String>,
    pub symbols: HashMap<String, String>,
    pub colors: HashMap<String, String>,
    pub dim_when_inferred: bool,
}

impl Default for Theme {
    fn default() -> Self {
        let col = |key: &str, width: usize, truncate: Truncate| Column {
            key: key.to_string(),
            width,
            truncate,
        };
        Theme {
            columns: vec![
                col("status", 9, Truncate::End),
                col("title", 22, Truncate::Middle),
                col("agent", 12, Truncate::End),
                col("llm", 14, Truncate::End),
                col("task", 40, Truncate::End),
            ],
            status_priority: ["waiting", "working", "error", "idle", "done"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            symbols: [
                ("working", "●"),
                ("waiting", "◐"),
                ("idle", "○"),
                ("done", "✓"),
                ("error", "✗"),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
            colors: [
                // waiting이 가장 눈에 띄어야 한다.
                ("waiting", "#f5a623"),
                ("working", "#4ade80"),
                ("idle", "#6b7280"),
                ("done", "#6b7280"),
                ("error", "#ef4444"),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
            dim_when_inferred: true,
        }
    }
}

pub fn path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("WAID_THEME") {
        return Some(PathBuf::from(p));
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| crate::adapters::home().map(|h| h.join(".config")))?;
    Some(base.join("waid/theme.toml"))
}

pub fn load() -> Theme {
    let mut t = Theme::default();
    let p = match path() {
        Some(p) => p,
        None => return t,
    };
    let text = match std::fs::read_to_string(&p) {
        Ok(s) => s,
        Err(_) => return t,
    };
    apply(&mut t, &parse(&text));
    t
}

// ------------------------------------------------------- 최소 TOML

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Str(String),
    Int(i64),
    Bool(bool),
    List(Vec<String>),
}

/// `"section.key" -> Val` 로 평탄화해서 돌려준다. 중첩 구조를 만들 이유가 없다.
pub fn parse(text: &str) -> HashMap<String, Val> {
    let mut out = HashMap::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(s) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = s.trim().to_string();
            continue;
        }
        let (k, v) = match line.split_once('=') {
            Some(kv) => kv,
            None => continue,
        };
        let key = if section.is_empty() {
            k.trim().to_string()
        } else {
            format!("{}.{}", section, k.trim())
        };
        if let Some(v) = value(v.trim()) {
            out.insert(key, v);
        }
    }
    out
}

/// 문자열 리터럴 안의 `#`은 주석이 아니다. 색상값이 전부 `#`으로 시작하므로
/// 이걸 틀리면 테마 색이 통째로 사라진다.
fn strip_comment(line: &str) -> &str {
    let b = line.as_bytes();
    let mut in_str = false;
    for (i, c) in b.iter().enumerate() {
        match c {
            b'"' => in_str = !in_str,
            b'#' if !in_str => return &line[..i],
            _ => {}
        }
    }
    line
}

fn value(s: &str) -> Option<Val> {
    if s.is_empty() {
        return None;
    }
    if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let items: Vec<String> = inner
            .split(',')
            .map(|i| i.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|i| !i.is_empty())
            .collect();
        return Some(Val::List(items));
    }
    if s == "true" {
        return Some(Val::Bool(true));
    }
    if s == "false" {
        return Some(Val::Bool(false));
    }
    if let Ok(n) = s.parse::<i64>() {
        return Some(Val::Int(n));
    }
    Some(Val::Str(s.trim_matches('"').trim_matches('\'').to_string()))
}

// ---------------------------------------------------------- 적용

fn apply(t: &mut Theme, kv: &HashMap<String, Val>) {
    if let Some(Val::List(order)) = kv.get("columns.order") {
        let known = ["status", "title", "agent", "llm", "task"];
        let picked: Vec<Column> = order
            .iter()
            .filter(|k| known.contains(&k.as_str()))
            .map(|k| {
                // 순서만 바꾸고 너비는 기본값을 승계한다.
                t.columns
                    .iter()
                    .find(|c| &c.key == k)
                    .cloned()
                    .unwrap_or(Column { key: k.clone(), width: 16, truncate: Truncate::End })
            })
            .collect();
        if !picked.is_empty() {
            t.columns = picked;
        }
    }

    for c in t.columns.iter_mut() {
        if let Some(Val::Int(w)) = kv.get(&format!("columns.{}.width", c.key)) {
            // 0이나 음수 너비로 렌더러를 죽이지 않는다.
            c.width = (*w).clamp(3, 200) as usize;
        }
        if let Some(Val::Str(m)) = kv.get(&format!("columns.{}.truncate", c.key)) {
            c.truncate = match m.as_str() {
                "start" => Truncate::Start,
                "middle" => Truncate::Middle,
                _ => Truncate::End,
            };
        }
    }

    if let Some(Val::Bool(b)) = kv.get("columns.task.dim_when_inferred") {
        t.dim_when_inferred = *b;
    }
    if let Some(Val::List(p)) = kv.get("sort.status_priority") {
        if !p.is_empty() {
            t.status_priority = p.clone();
        }
    }
    for state in ["working", "waiting", "idle", "done", "error"] {
        if let Some(Val::Str(v)) = kv.get(&format!("symbols.{state}")) {
            t.symbols.insert(state.to_string(), v.clone());
        }
        if let Some(Val::Str(v)) = kv.get(&format!("colors.{state}")) {
            t.colors.insert(state.to_string(), v.clone());
        }
    }
}
