//! 스냅샷 진단 (§3.1 — 모르는 것은 모른다고 말한다).
//!
//! 수집 도중 **읽을 수 있었어야 하는데 못 읽은 것**만 모은다. 기본 소스 루트가
//! 없다는 건 그 도구를 안 쓴다는 뜻이므로 문제가 아니다. 경고는 이번 수집의
//! 사실이라 회복되면 다음 스냅샷에서 사라진다. 경로와 실패 사유까지만 담는다 —
//! 프롬프트·응답 본문은 어떤 경우에도 들어가지 않는다.

use std::cell::RefCell;
use std::path::Path;

/// 한 스냅샷이 싣는 경고 총 개수(요약 줄 포함)와 한 줄의 최대 길이.
/// 데스크톱과 합의한 전송 상한이다.
pub const MAX_WARNINGS: usize = 32;
pub const MAX_CHARS: usize = 512;

thread_local! {
    /// 수집과 직렬화는 한 스레드에서 끝난다(CLI·doctor·serve 의 연결 스레드).
    /// 전역 하나를 잠그면 동시 스냅샷이 서로의 목록을 지운다.
    static WARNINGS: RefCell<(Vec<String>, usize)> = const { RefCell::new((Vec::new(), 0)) };
}

/// 긴 어댑터 SQL 오류 하나가 상한을 우회하지 못하게 한다.
fn clamp(text: String) -> String {
    if text.chars().count() <= MAX_CHARS {
        return text;
    }
    text.chars().take(MAX_CHARS - 1).chain(['…']).collect()
}

/// 새 스냅샷의 시작.
pub fn reset() {
    WARNINGS.with(|w| {
        let (list, dropped) = &mut *w.borrow_mut();
        list.clear();
        *dropped = 0;
    });
}

/// 경고 한 줄. 같은 문장은 한 번만, 상한을 넘으면 개수만 센다.
pub fn warn(text: String) {
    let text = clamp(text);
    WARNINGS.with(|w| {
        let (list, dropped) = &mut *w.borrow_mut();
        if list.contains(&text) {
            return;
        }
        if list.len() < MAX_WARNINGS {
            list.push(text);
        } else {
            *dropped += 1;
        }
    });
}

/// 이번 스냅샷의 경고. 잘려나간 만큼은 마지막 한 줄에 개수로 붙고,
/// 그 줄까지 세어 전체가 상한을 넘지 않는다.
pub fn warnings() -> Vec<String> {
    WARNINGS.with(|w| {
        let (list, dropped) = &*w.borrow();
        let mut out = list.clone();
        if *dropped > 0 {
            let hidden = dropped + out.len() - (MAX_WARNINGS - 1);
            out.truncate(MAX_WARNINGS - 1);
            out.push(clamp(format!(
                "경고 {hidden}건이 더 있습니다 (상한 {MAX_WARNINGS}건)"
            )));
        }
        out
    })
}

// 문장은 여기 모아 둔다. 같은 사건이 어디서 났든 같은 문장이어야 중복 제거가
// 실제로 동작한다.

/// 루트가 없는 것과 못 읽는 것은 다르다. 이 판단이 "안 쓰는 에이전트"와
/// "권한 문제"를 가른다.
pub fn dir_error(dir: &Path, error: &std::io::Error) {
    if error.kind() != std::io::ErrorKind::NotFound {
        warn(format!("폴더를 읽지 못했습니다: {} — {error}", dir.display()));
    }
}

pub fn file_error(path: &Path, error: &std::io::Error) {
    if error.kind() != std::io::ErrorKind::NotFound {
        warn(format!("파일을 읽지 못했습니다: {} — {error}", path.display()));
    }
}

/// 파일은 남고 그 줄만 빠진다. 잘린 경계 줄은 여기 포함되지 않는다.
pub fn unparsed_lines(path: &Path, lines: usize) {
    warn(format!("형식 오류로 {lines}줄을 건너뜁니다: {}", path.display()));
}

/// 문서 전체가 JSON 이 아니었다. 세션 한 장이 통째로 빠진다.
pub fn unparsed_document(path: &Path) {
    warn(format!("JSON 형식 오류로 건너뜁니다: {}", path.display()));
}

/// 상한을 넘는 JSON 은 파싱하지 않는다(§8). 조용히 빠지면 원인을 못 찾는다.
pub fn oversized(path: &Path, size: u64, limit: u64) {
    let mib = (1024 * 1024) as f64;
    warn(format!(
        "크기 상한 {:.0} MiB 를 넘어 건너뜁니다: {} ({:.1} MiB)",
        limit as f64 / mib,
        path.display(),
        size as f64 / mib
    ));
}

/// 편집기 업데이트로 스키마가 바뀌거나 DB 가 잠긴 경우. 그 소스만 빈다.
pub fn sqlite_error(file: &Path, detail: &str) {
    warn(format!("SQLite 소스를 읽지 못했습니다: {} — {detail}", file.display()));
}
