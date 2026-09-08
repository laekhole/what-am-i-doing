//! 스냅샷 진단 회귀 (src/diag.rs).
//!
//! 진단은 스레드마다 따로 쌓인다. `#[test]` 는 각자 자기 스레드에서 도므로
//! 여기 테스트들은 서로의 목록을 보지 않는다 — 줄 세울 잠금이 필요 없다.

use std::fs;
use std::path::PathBuf;

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("waid-diag-{name}-{}", std::process::id()))
}

fn mentions(needle: &str) -> bool {
    crate::diag::warnings().iter().any(|w| w.contains(needle))
}

#[test]
fn warnings_dedupe_and_stay_within_the_wire_bounds() {
    crate::diag::reset();
    crate::diag::warn("같은 문장".into());
    crate::diag::warn("같은 문장".into());
    assert_eq!(crate::diag::warnings(), vec!["같은 문장".to_string()]);

    // 긴 한 줄이 상한을 우회하지 못한다.
    crate::diag::reset();
    crate::diag::warn("가".repeat(crate::diag::MAX_CHARS * 3));
    let long = crate::diag::warnings();
    assert_eq!(long[0].chars().count(), crate::diag::MAX_CHARS);

    // 망가진 폴더 하나가 스냅샷을 수천 줄로 부풀리지 않는다.
    crate::diag::reset();
    for i in 0..crate::diag::MAX_WARNINGS + 5 {
        crate::diag::warn(format!("경고 {i}"));
    }
    let warnings = crate::diag::warnings();
    // 요약 줄까지 세어 전체가 상한을 넘지 않는다.
    assert_eq!(warnings.len(), crate::diag::MAX_WARNINGS);
    assert!(warnings.last().unwrap().contains("6건이 더 있습니다"));
}

#[test]
fn absent_root_is_normal_but_unreadable_root_is_reported() {
    let missing = temp("missing-root");
    let _ = fs::remove_dir_all(&missing);
    let mut found = Vec::new();

    crate::diag::reset();
    crate::transcript::walk(&missing, 1_000, &mut found, 0, true, "jsonl", None);
    // 기본 소스 루트가 없다는 건 그 도구를 안 쓴다는 뜻이다. 오류가 아니다.
    assert!(crate::diag::warnings().is_empty(), "{:?}", crate::diag::warnings());

    // 폴더 자리에 파일이 있으면 "없음"이 아니라 진짜로 훑지 못한 것이다.
    let not_a_dir = temp("not-a-dir");
    fs::write(&not_a_dir, "{}\n").unwrap();
    crate::diag::reset();
    crate::transcript::walk(&not_a_dir, 1_000, &mut found, 0, true, "jsonl", None);
    assert!(mentions("not-a-dir"), "{:?}", crate::diag::warnings());
    let _ = fs::remove_file(&not_a_dir);
}

#[test]
fn unparsed_lines_are_counted_without_hiding_the_session() {
    let path = temp("unparsed").with_extension("jsonl");
    fs::write(
        &path,
        "{\"type\":\"user\",\"message\":{\"content\":\"진단 픽스처\"}}\n\
         오염된 줄\n\
         {\"type\":\"assistant\",\"message\":{\"stop_reason\":\"end_turn\"}}\n",
    )
    .unwrap();

    crate::diag::reset();
    let t = crate::transcript::read(&path, 100).unwrap();
    // 건강한 행은 사라지지 않는다. 빠진 줄만 따로 말한다.
    assert_eq!(t.current_prompt.as_deref(), Some("진단 픽스처"));
    assert!(mentions("1줄") && mentions("unparsed"), "{:?}", crate::diag::warnings());

    // 캐시로 답하는 회차도 같은 진단을 낸다. 두 번째 스냅샷부터 조용해지면
    // 사용자는 문제가 사라졌다고 읽는다.
    crate::diag::reset();
    assert!(crate::transcript::read(&path, 100).is_some());
    assert!(mentions("1줄"), "{:?}", crate::diag::warnings());

    // 고쳐 쓰면 다음 스냅샷에서 조용해진다.
    fs::write(
        &path,
        "{\"type\":\"user\",\"message\":{\"content\":\"진단 픽스처\"}}\n\
         {\"type\":\"assistant\",\"message\":{\"stop_reason\":\"end_turn\"}}\n",
    )
    .unwrap();
    crate::diag::reset();
    assert!(crate::transcript::read(&path, 101).is_some());
    assert!(!mentions("unparsed"), "{:?}", crate::diag::warnings());
    let _ = fs::remove_file(&path);
}

#[test]
fn line_still_being_written_is_not_a_format_error() {
    let path = temp("partial").with_extension("jsonl");
    // 마지막 줄은 개행 없이 잘려 있다 — 에이전트가 지금 쓰는 중이다.
    fs::write(
        &path,
        "{\"type\":\"user\",\"message\":{\"content\":\"쓰는 중\"}}\n{\"type\":\"assi",
    )
    .unwrap();

    crate::diag::reset();
    assert!(crate::transcript::read(&path, 100).is_some());
    assert!(crate::diag::warnings().is_empty(), "{:?}", crate::diag::warnings());
    let _ = fs::remove_file(&path);
}

#[test]
fn span_boundaries_of_a_long_valid_log_are_not_format_errors() {
    // 머리(128KiB)와 꼬리(256KiB) 양쪽에서 잘리도록 충분히 길게 쓴다.
    // 두 경계 조각과 겹치는 구간이 모두 멀쩡한 파일로 읽혀야 한다.
    let path = temp("long-valid").with_extension("jsonl");
    let mut log = String::from("{\"type\":\"user\",\"message\":{\"content\":\"긴 로그\"}}\n");
    while log.len() < 400 * 1024 {
        log.push_str("{\"type\":\"assistant\",\"message\":{\"content\":\"");
        log.push_str(&"채움 ".repeat(40));
        log.push_str("\",\"stop_reason\":\"end_turn\"}}\n");
    }
    fs::write(&path, &log).unwrap();

    crate::diag::reset();
    assert!(crate::transcript::read(&path, 100).is_some());
    assert!(crate::diag::warnings().is_empty(), "{:?}", crate::diag::warnings());
    let _ = fs::remove_file(&path);
}

#[test]
fn sampled_jsonl_warnings_cover_append_and_boundary_lines() {
    use std::io::Write;
    for (name, newline) in [("lf", "\n"), ("crlf", "\r\n")] {
        let path = temp(&format!("append-{name}")).with_extension("jsonl");
        let first = format!(r#"{{"type":"user","message":{{"content":"request"}}}}{newline}"#);
        fs::write(&path, &first).unwrap();
        crate::diag::reset();
        assert!(crate::transcript::read(&path, 100).is_some());
        assert!(crate::diag::warnings().is_empty());
        let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
        write!(file, "broken{newline}").unwrap();
        drop(file);
        crate::diag::reset();
        assert!(crate::transcript::read(&path, 101).is_some());
        assert!(mentions("1줄"), "new malformed line lost: {:?}", crate::diag::warnings());
        fs::remove_file(&path).unwrap();

        let path = temp(&format!("boundary-{name}")).with_extension("jsonl");
        let mut log = first;
        while log.len() + 4 + newline.len() < 128 * 1024 {
            log.push_str(&format!("{{}}{newline}"));
        }
        log.push_str(&" ".repeat(128 * 1024 - 2 - log.len()));
        log.push_str(&format!("broken{newline}{{}}{newline}"));
        fs::write(&path, log).unwrap();
        crate::diag::reset();
        assert!(crate::transcript::read(&path, 100).is_some());
        assert!(mentions("1줄"), "boundary malformed line lost: {:?}", crate::diag::warnings());
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn missing_file_is_silent_but_unreadable_one_is_reported() {
    let gone = temp("gone").with_extension("jsonl");
    let _ = fs::remove_file(&gone);
    crate::diag::reset();
    assert!(crate::transcript::read(&gone, 100).is_none());
    assert!(crate::diag::warnings().is_empty(), "{:?}", crate::diag::warnings());

    // 폴더를 파일처럼 열면 존재하지만 읽을 수 없는 경우가 된다.
    let dir = temp("as-file");
    fs::create_dir_all(&dir).unwrap();
    crate::diag::reset();
    assert!(crate::transcript::read(&dir, 100).is_none());
    assert!(mentions("as-file"), "{:?}", crate::diag::warnings());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn oversized_and_broken_json_documents_are_named() {
    // 상한(4 MiB)을 넘는 문서는 파싱하지 않는다. 조용히 빠지면 원인을 못 찾는다.
    let big = temp("oversized").with_extension("json");
    fs::write(&big, format!("{{\"pad\":\"{}\"}}", "a".repeat(5 * 1024 * 1024))).unwrap();
    crate::diag::reset();
    assert!(crate::transcript::read_json(&big, 100).is_none());
    assert!(mentions("oversized") && mentions("MiB"), "{:?}", crate::diag::warnings());
    let _ = fs::remove_file(&big);

    let path = temp("broken").with_extension("json");
    fs::write(&path, "{\"messages\": [ 이건 JSON 이 아니다").unwrap();
    crate::diag::reset();
    assert!(crate::transcript::read_json(&path, 100).is_none());
    assert!(mentions("broken"), "{:?}", crate::diag::warnings());

    // 사용자 요청이 없는 설정 JSON 은 정상적인 제외다. 경고하지 않는다.
    fs::write(&path, "{\"theme\":\"dark\"}").unwrap();
    crate::diag::reset();
    assert!(crate::transcript::read_json(&path, 101).is_none());
    assert!(crate::diag::warnings().is_empty(), "{:?}", crate::diag::warnings());
    let _ = fs::remove_file(&path);
}

#[test]
fn snapshot_json_carries_warnings_only_when_there_are_any() {
    crate::diag::reset();
    // 문제가 없으면 키 자체가 없다. 빈 배열은 소비자에게 상태를 하나 더 만든다.
    assert!(!crate::session::to_json(&[], 100, false).contains("\"warnings\""));

    crate::diag::warn("소스 하나가 막혔습니다".into());
    let json = crate::session::to_json(&[], 100, false);
    assert!(
        json.contains("\"warnings\":[\"소스 하나가 막혔습니다\"]"),
        "{json}"
    );
    crate::diag::reset();
}
