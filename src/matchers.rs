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
fn basename(arg: &str) -> &str {
    let file = arg.rsplit(['/', '\\']).next().unwrap_or(arg);
    for ext in [".exe", ".cmd", ".ps1", ".bat"] {
        if let Some(stripped) = file.strip_suffix(ext) {
            return stripped;
        }
    }
    file
}

/// 인터프리터 런처인가? 그렇다면 진짜 이름은 다음 인자에 있다.
fn is_launcher(name: &str) -> bool {
    matches!(
        name,
        "node" | "bun" | "deno" | "python" | "python3" | "npx" | "pnpm" | "uv" | "uvx"
    )
}

fn by_exec(candidate: &str) -> Option<Agent> {
    adapters::table()
        .iter()
        .find(|d| d.exec.iter().any(|e| e == candidate))
        .map(|d| d.agent)
}

pub fn identify(p: &Process) -> Option<Agent> {
    // 1) argv[0]의 basename. 가장 신뢰도 높은 신호.
    let head = basename(p.argv.first()?);
    if let Some(a) = by_exec(head) {
        return Some(a);
    }

    // 2) 런처를 거쳐 실행된 경우 (`node /usr/lib/node_modules/.../cli.js`).
    //    argv[1]까지만 본다. 더 들어가면 오탐이 시작된다.
    if is_launcher(head) {
        if let Some(second) = p.argv.get(1) {
            if let Some(a) = by_exec(basename(second)) {
                return Some(a);
            }
        }
    }

    // 3) 패키지 경로 표지. `node .../@anthropic-ai/claude-code/cli.js`처럼
    //    파일명이 `cli.js`라서 2)로 안 잡히는 경우를 위한 것.
    //    표지 문자열이 충분히 구체적이라 오탐 위험이 낮다.
    let full = p.cmdline();
    adapters::table()
        .iter()
        .find(|d| d.markers.iter().any(|m| full.contains(m.as_str())))
        .map(|d| d.agent)
}
