//! 프로세스 → 에이전트 식별.
//!
//! 정규식 크레이트를 쓰지 않는다. 그리고 그게 오히려 낫다는 게 이 모듈의
//! 주장이다.
//!
//! 커맨드라인 전체를 정규식으로 훑는 방식은 오탐이 잦다. `vim claude.md`를
//! 편집 중인 프로세스가 `claude`로 잡히고, `grep -r codex .`도 잡힌다.
//! 우리가 실제로 알고 싶은 건 "이 프로세스의 **실행 파일**이 에이전트인가"이므로,
//! argv[0]의 basename을 정규화해서 비교한다. 훨씬 좁고, 훨씬 정확하다.
//!
//! 부수 효과로, 사용자 어댑터(§4)를 쓰는 사람이 정규식을 배울 필요가 없어졌다.
//! `exec = ["mytool"]` 이면 끝이다.
//!
//! "무엇을 찾을 것인가"는 adapters.rs 가 안다. 이 모듈은 "이 프로세스가
//! 그 중 무엇인가"만 판단한다.

use crate::adapters;
use crate::proc::Process;

pub use crate::adapters::{all, by_name, Agent};

/// 경로와 Windows 셔임 확장자를 벗겨 실행 파일 이름만 남긴다.
///
/// npm으로 설치한 CLI는 Windows에서 `claude.cmd`로, 맨 실행 파일은
/// `claude.exe`로 나타난다. 이걸 벗기지 않으면 Windows에서 아무것도
/// 감지되지 않는다.
fn basename(arg: &str) -> String {
    let file = arg.rsplit(['/', '\\']).next().unwrap_or(arg).to_ascii_lowercase();
    for ext in [".exe", ".cmd", ".ps1", ".bat"] {
        if let Some(stripped) = file.strip_suffix(ext) { return stripped.into(); }
    }
    file
}

fn is_launcher(name: &str) -> bool {
    matches!(name, "node" | "bun" | "deno" | "python" | "python3" | "pythonw")
        || name.strip_prefix("python3.").is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
}

fn by_exec(candidate: &str) -> Option<Agent> {
    adapters::table().iter()
        .find(|d| d.exec.iter().any(|e| e.eq_ignore_ascii_case(candidate)))
        .map(|d| d.agent)
}

/// Query arguments only for known executables/interpreters, never arbitrary desktop apps.
#[cfg(windows)]
pub(crate) fn needs_arguments(exe: &str) -> bool {
    let name = basename(exe);
    is_launcher(&name) || by_exec(&name).is_some()
}

pub fn identify(p: &Process) -> Option<Agent> {
    let head = basename(p.argv.first()?);
    if let Some(a) = by_exec(&head) { return Some(a); }
    if !is_launcher(&head) { return None; }
    let mut args = p.argv.iter().skip(1);
    let script = loop {
        let arg = args.next()?;
        // Only known flag shapes may precede the entrypoint. Eval/prompt arguments
        // and package-manager supervisor processes must not become extra sessions.
        if head.starts_with("python") && arg == "-m" {
            return match args.next()?.as_str() {
                "aider" | "aider.main" => by_name("aider"),
                _ => None,
            };
        }
        if matches!(arg.as_str(), "--no-warnings" | "--enable-source-maps" | "-u" | "-B")
            || arg.starts_with("--max-old-space-size=") { continue; }
        if arg == "--" { break args.next()?; }
        if arg.starts_with('-') { return None; }
        break arg;
    };
    if let Some(a) = by_exec(&basename(script)) { return Some(a); }
    if head.starts_with("python") && basename(script) == "aider-script.py" {
        return by_name("aider");
    }
    let entry = script.replace('\\', "/").to_ascii_lowercase();
    adapters::table().iter().find(|d| d.markers.iter().any(|m| {
        let marker = m.replace('\\', "/").to_ascii_lowercase();
        entry.match_indices(&marker).any(|(i, _)| {
            (i == 0 || entry.as_bytes()[i - 1] == b'/')
                && (i + marker.len() == entry.len() || entry.as_bytes()[i + marker.len()] == b'/')
        })
    })).map(|d| d.agent)
}
