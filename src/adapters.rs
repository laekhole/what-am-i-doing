//! 에이전트 어댑터 (매니페스트 §4).
//!
//! "어떤 에이전트가 존재하는가"를 아는 유일한 곳. 감지 규칙과 트랜스크립트
//! 위치가 여기 한 테이블에 모인다. 내장 정의도 사용자 정의와 **똑같은 형식**을
//! 쓴다 — 내장이 특권을 갖는 순간 사용자 어댑터는 이류 시민이 되고, 그러면
//! 아무도 쓰지 않는다.
//!
//! 사용자 어댑터는 `~/.config/waid/adapters/*.toml` 에서 읽는다. 같은 `name`
//! 이면 내장 정의를 **대체**한다. 새 에이전트 지원에 코드 수정이 필요 없다.
//!
//! ```toml
//! [adapter]
//! name    = "mytool"
//! display = "My Coding Tool"
//! exec    = ["mytool", "mytool-cli"]   # argv[0] 의 basename 후보
//! markers = ["@vendor/mytool"]         # 커맨드라인에 나타나는 패키지 경로 (선택)
//!
//! [transcript]
//! dir = "~/.mytool/sessions"           # 선택. 없으면 감지만 하고 llm/task 는 —
//! ```
//!
//! 정규식이 아니라 실행 파일 이름 목록인 이유는 matchers.rs 를 보라.

use crate::theme::{parse, Val};
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Agent {
    /// 정본 식별자. `--agent` 필터와 JSON에 쓰인다.
    pub name: &'static str,
    /// 표에 표시할 이름.
    pub display: &'static str,
    /// 트랜스크립트를 읽을 수 있는가. 아니면 감지만 하고 llm/task는 `—`.
    pub has_reader: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Builtin,
    User(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Def {
    pub agent: Agent,
    /// argv[0] (또는 런처 뒤 argv[1]) 의 basename 후보.
    pub exec: Vec<String>,
    /// 커맨드라인 어딘가에 나타나는 패키지 경로 표지.
    pub markers: Vec<String>,
    /// 트랜스크립트 루트 **목록**. 비어 있으면 프로세스 감지만.
    ///
    /// 목록인 이유는 §5.1 이다. 같은 에이전트가 WSL·PowerShell·Git Bash·
    /// 데스크톱 앱에서 각각 돌면 트랜스크립트도 각각 다른 홈에 쌓인다.
    /// 프로세스 열거로는 그 경계를 넘을 수 없지만 파일시스템으로는 넘는다.
    pub transcript_dirs: Vec<PathBuf>,
    pub origin: Origin,
}

// ------------------------------------------------------------ 내장 정의

/// (name, display, exec, markers, HOME 기준 트랜스크립트 경로)
const BUILTIN: &[(&str, &str, &[&str], &[&str], Option<&str>)] = &[
    (
        "claude",
        "Claude Code",
        &["claude", "claude-code"],
        &["@anthropic-ai/claude-code"],
        Some(".claude/projects"),
    ),
    (
        "codex",
        "Codex",
        &["codex", "codex-cli"],
        &["@openai/codex"],
        Some(".codex/sessions"),
    ),
    (
        "gemini",
        "Gemini CLI",
        &["gemini", "gemini-cli"],
        &["@google/gemini-cli"],
        None,
    ),
    ("opencode", "opencode", &["opencode"], &["opencode-ai"], None),
    ("aider", "Aider", &["aider"], &[], None),
    ("cursor", "Cursor CLI", &["cursor-agent"], &[], None),
    (
        "copilot",
        "Copilot CLI",
        &["copilot", "github-copilot-cli"],
        &["@github/copilot"],
        Some(".copilot/session-state"),
    ),
    ("goose", "Goose", &["goose"], &[], None),
];

/// 이 프로세스가 보는 "내 홈".
///
/// Windows 는 `HOME` 을 두지 않는다. `USERPROFILE` 로 떨어지지 않으면
/// 네이티브 Windows 에서 트랜스크립트를 한 건도 못 찾는다.
pub(crate) fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// WSL 안에서 도는가?
fn in_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|v| {
            let v = v.to_ascii_lowercase();
            v.contains("microsoft") || v.contains("wsl")
        })
        .unwrap_or(false)
}

/// 사람이 쓸 법한 홈 디렉터리 후보 전부 (§5.1).
///
/// 내 홈 하나로 끝나지 않는다. WSL 에서 돌면 Windows 쪽 사용자 홈이
/// `/mnt/c/Users/<user>` 로 보이고, Windows 에서 돌면 WSL 쪽이
/// `\\wsl$\<배포판>\home\<user>` 로 보인다. 양쪽을 다 훑어야
/// "어느 셸에서 띄웠든 한 화면에" 가 성립한다.
///
/// 탐색 실패는 전부 조용히 무시한다. 경계 너머가 없는 환경(순수 Linux,
/// WSL 미설치 Windows)이 정상이고, 거기서 경고를 띄우면 소음일 뿐이다.
fn home_candidates() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    if let Some(h) = home() {
        out.push(h);
    }

    if in_wsl() {
        // WSL → Windows 쪽 사용자 홈
        if let Ok(entries) = std::fs::read_dir("/mnt/c/Users") {
            for e in entries.flatten() {
                let name = e.file_name();
                let name = name.to_string_lossy();
                // 시스템 계정은 건너뛴다.
                if matches!(
                    name.as_ref(),
                    "Public" | "Default" | "Default User" | "All Users" | "desktop.ini"
                ) {
                    continue;
                }
                if e.path().is_dir() {
                    out.push(e.path());
                }
            }
        }
    } else if cfg!(windows) {
        // Windows → WSL 배포판 안의 사용자 홈
        if let Ok(distros) = std::fs::read_dir(r"\\wsl$") {
            for d in distros.flatten() {
                if let Ok(users) = std::fs::read_dir(d.path().join("home")) {
                    for u in users.flatten() {
                        if u.path().is_dir() {
                            out.push(u.path());
                        }
                    }
                }
            }
        }
    }

    out.sort();
    out.dedup();
    out
}

/// `~/x` 를 **모든 홈 후보**에 대해 펼친다. 절대 경로는 그대로 하나.
///
/// 어댑터 작성자는 `~/.mytool/sessions` 한 줄만 적으면 되고, 그게 WSL 쪽인지
/// Windows 쪽인지는 신경 쓰지 않아도 된다. 경계 처리는 여기서 끝난다.
fn expand(raw: &str) -> Vec<PathBuf> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    match raw.strip_prefix("~/") {
        Some(rest) => home_candidates().into_iter().map(|h| h.join(rest)).collect(),
        None if raw == "~" => home_candidates(),
        None => vec![PathBuf::from(raw)],
    }
}

fn builtins() -> Vec<Def> {
    BUILTIN
        .iter()
        .map(|(name, display, exec, markers, dir)| {
            let mut transcript_dirs: Vec<PathBuf> = dir
                .map(|d| home_candidates().into_iter().map(|h| h.join(d)).collect())
                .unwrap_or_default();
            // Include the explicitly configured Codex home alongside other local sessions.
            // User adapters still replace this built-in definition below.
            if *name == "codex" {
                if let Some(root) = std::env::var_os("CODEX_HOME").filter(|root| !root.is_empty()) {
                    transcript_dirs.push(PathBuf::from(root).join("sessions"));
                }
            }
            if *name == "copilot" {
                if let Some(root) = std::env::var_os("COPILOT_HOME").filter(|root| !root.is_empty()) {
                    transcript_dirs.push(PathBuf::from(root).join("session-state"));
                }
            }
            transcript_dirs.sort();
            transcript_dirs.dedup();
            Def {
                agent: Agent {
                    name,
                    display,
                    has_reader: !transcript_dirs.is_empty(),
                },
                exec: exec.iter().map(|s| s.to_string()).collect(),
                markers: markers.iter().map(|s| s.to_string()).collect(),
                transcript_dirs,
                origin: Origin::Builtin,
            }
        })
        .collect()
}

// ------------------------------------------------------------ 사용자 어댑터

pub fn dir() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("WAID_ADAPTERS") {
        return Some(PathBuf::from(p));
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home().map(|h| h.join(".config")))?;
    Some(base.join("waid/adapters"))
}

fn strings(kv: &std::collections::HashMap<String, Val>, key: &str) -> Vec<String> {
    match kv.get(key) {
        Some(Val::List(v)) => v.clone(),
        // 항목이 하나면 따옴표 문자열로 쓰는 사람이 반드시 있다.
        Some(Val::Str(s)) if !s.is_empty() => vec![s.clone()],
        _ => Vec::new(),
    }
}

/// 파일 하나를 Def 로. 실패 사유를 문자열로 돌려준다 — `doctor` 가 보여준다.
fn parse_def(path: &PathBuf, text: &str) -> Result<Def, String> {
    let kv = parse(text);

    let name = match kv.get("adapter.name") {
        Some(Val::Str(s)) if !s.is_empty() => s.clone(),
        _ => return Err("adapter.name 이 없습니다".into()),
    };
    if name.chars().any(|c| c.is_whitespace()) {
        return Err(format!("adapter.name '{name}' 에 공백은 쓸 수 없습니다"));
    }

    let exec = strings(&kv, "adapter.exec");
    let markers = strings(&kv, "adapter.markers");
    if exec.is_empty() && markers.is_empty() {
        return Err("adapter.exec 또는 adapter.markers 중 하나는 있어야 합니다".into());
    }

    let display = match kv.get("adapter.display") {
        Some(Val::Str(s)) if !s.is_empty() => s.clone(),
        _ => name.clone(),
    };

    // `dir` 는 한 줄, `dirs` 는 목록. 둘 다 쓰면 합친다.
    // 각 항목의 `~` 는 홈 후보 전체로 펼쳐진다(§5.1).
    let mut transcript_dirs: Vec<PathBuf> = Vec::new();
    if let Some(Val::Str(d)) = kv.get("transcript.dir") {
        transcript_dirs.extend(expand(d));
    }
    for d in strings(&kv, "transcript.dirs") {
        transcript_dirs.extend(expand(&d));
    }
    transcript_dirs.sort();
    transcript_dirs.dedup();

    // 프로세스가 살아있는 동안만 유효하면 되므로 한 번 새는 건 문제가 아니다.
    // 어댑터는 시작 시 한 번만 읽는다.
    let name: &'static str = Box::leak(name.into_boxed_str());
    let display: &'static str = Box::leak(display.into_boxed_str());

    Ok(Def {
        agent: Agent {
            name,
            display,
            has_reader: !transcript_dirs.is_empty(),
        },
        exec,
        markers,
        transcript_dirs,
        origin: Origin::User(path.clone()),
    })
}

/// 로딩 중 발견한 문제. `doctor` 전용이라 조용히 모아둔다 —
/// 어댑터 오타 하나로 매 실행마다 stderr 가 시끄러워지면 안 된다.
pub fn problems() -> &'static [String] {
    load().1.as_slice()
}

pub fn table() -> &'static [Def] {
    load().0.as_slice()
}

fn load() -> &'static (Vec<Def>, Vec<String>) {
    static CACHE: OnceLock<(Vec<Def>, Vec<String>)> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut defs = builtins();
        let mut problems = Vec::new();

        let d = match dir() {
            Some(d) => d,
            None => return (defs, problems),
        };
        let entries = match std::fs::read_dir(&d) {
            Ok(e) => e,
            // 디렉터리가 없는 게 정상이다. 문제로 치지 않는다.
            Err(_) => return (defs, problems),
        };

        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "toml").unwrap_or(false))
            .collect();
        // 읽는 순서가 결과를 바꾸면 안 된다.
        paths.sort();

        for p in paths {
            let text = match std::fs::read_to_string(&p) {
                Ok(t) => t,
                Err(e) => {
                    problems.push(format!("{}: 읽기 실패 ({e})", p.display()));
                    continue;
                }
            };
            match parse_def(&p, &text) {
                Ok(def) => match defs.iter().position(|x| x.agent.name == def.agent.name) {
                    // 같은 이름이면 대체한다. 내장 감지 규칙을 고치고 싶은
                    // 사람에게 유일한 수단이다.
                    Some(i) => defs[i] = def,
                    None => defs.push(def),
                },
                Err(e) => problems.push(format!("{}: {e}", p.display())),
            }
        }

        (defs, problems)
    })
}

pub fn by_name(name: &str) -> Option<Agent> {
    table().iter().find(|d| d.agent.name == name).map(|d| d.agent)
}

pub fn all() -> Vec<Agent> {
    table().iter().map(|d| d.agent).collect()
}

/// 트랜스크립트 루트 목록. transcript.rs 가 이것만 보고 움직인다.
pub fn transcript_dirs(name: &str) -> &'static [PathBuf] {
    table()
        .iter()
        .find(|d| d.agent.name == name)
        .map(|d| d.transcript_dirs.as_slice())
        .unwrap_or(&[])
}

#[cfg(test)]
pub fn parse_def_for_test(text: &str) -> Result<Def, String> {
    parse_def(&PathBuf::from("test.toml"), text)
}
