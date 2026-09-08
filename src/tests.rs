//! 테스트.
//!
//! 여기서 검증하려는 것은 대체로 "실제로 만들면 반드시 밟는 함정" 목록이다.
//! 한글 폭 계산, 색상값 안의 `#`, 하네스가 주입한 가짜 첫 프롬프트,
//! `vim claude.md` 오탐 같은 것들.

use crate::json;
use crate::matchers;
use crate::proc::Process;
use crate::render::{self, width};
use crate::session::State;
use crate::theme::{self, Truncate, Val};
use crate::transcript;
use crate::transcript::{normalize_model, normalize_prompt};

fn p(argv: &[&str]) -> Process {
    Process {
        pid: 1,
        argv: argv.iter().map(|s| s.to_string()).collect(),
        cwd: None,
    }
}

// ---------------------------------------------------------------- JSON

#[test]
fn parses_nested_and_unicode() {
    let v = json::parse(r#"{"a":{"b":["x",1,true,null]},"k":"\ud83d\ude80 \uac00"}"#).unwrap();
    assert_eq!(
        v.get("a")
            .unwrap()
            .get("b")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(v.get("k").unwrap().as_str().unwrap(), "🚀 가");
}

#[test]
fn parses_raw_utf8_in_strings() {
    // 트랜스크립트는 한글을 이스케이프하지 않고 그대로 쓴다.
    let v = json::parse(r#"{"t":"리프레시 토큰 만료"}"#).unwrap();
    assert_eq!(v.get("t").unwrap().as_str().unwrap(), "리프레시 토큰 만료");
}

#[test]
fn rejects_malformed() {
    assert!(json::parse(r#"{"a":}"#).is_err());
    assert!(json::parse(r#"{"a":1"#).is_err());
    assert!(json::parse(r#"{"a":1} trailing"#).is_err());
}

#[test]
fn find_str_searches_recursively() {
    // 스키마가 바뀌어도 model을 찾아내야 한다.
    let v = json::parse(r#"{"payload":{"info":{"model":"claude-opus-4-6"}}}"#).unwrap();
    assert_eq!(v.find_str(&["model"]).unwrap(), "claude-opus-4-6");
}

#[test]
fn find_str_skips_empty_values() {
    let v = json::parse(r#"{"model":"","message":{"model":"gpt-5.2"}}"#).unwrap();
    assert_eq!(v.find_str(&["model"]).unwrap(), "gpt-5.2");
}

#[test]
fn text_content_handles_block_arrays() {
    let v = json::parse(r#"{"content":[{"type":"image"},{"type":"text","text":"인증 리팩터링"}]}"#)
        .unwrap();
    assert_eq!(
        v.get("content").unwrap().text_content().unwrap(),
        "인증 리팩터링"
    );
}

#[test]
fn writer_escapes_and_nests() {
    let mut w = json::Writer::new(false);
    w.begin_obj();
    w.field_str("q", "a\"b\nc");
    w.field_arr("xs");
    w.begin_obj();
    w.field_num("n", 7);
    w.end_obj();
    w.end_arr();
    w.end_obj();
    assert_eq!(w.buf, r#"{"q":"a\"b\nc","xs":[{"n":7}]}"#);
    // 쓴 것을 다시 읽을 수 있어야 한다.
    assert!(json::parse(&w.buf).is_ok());
}

#[test]
fn writer_pretty_has_no_leading_newline() {
    let mut w = json::Writer::new(true);
    w.begin_obj();
    w.field_num("a", 1);
    w.end_obj();
    assert!(w.buf.starts_with('{'), "got {:?}", w.buf);
    assert!(json::parse(&w.buf).is_ok());
}

// ------------------------------------------------------------ 감지

#[test]
fn detects_direct_invocation() {
    assert_eq!(matchers::identify(&p(&["claude"])).unwrap().name, "claude");
    assert_eq!(
        matchers::identify(&p(&["/usr/local/bin/codex", "--yolo"]))
            .unwrap()
            .name,
        "codex"
    );
}

#[test]
fn detects_windows_shims() {
    assert_eq!(
        matchers::identify(&p(&["C:\\npm\\claude.cmd"]))
            .unwrap()
            .name,
        "claude"
    );
    assert_eq!(
        matchers::identify(&p(&["claude.exe"])).unwrap().name,
        "claude"
    );
}

#[test]
fn detects_through_launcher() {
    let proc = p(&[
        "/usr/bin/node",
        "/home/mk/.nvm/versions/node/v22/bin/claude",
        "--resume",
    ]);
    assert_eq!(matchers::identify(&proc).unwrap().name, "claude");
}

#[test]
fn detects_via_package_marker() {
    // 파일명이 cli.js라서 basename으로는 안 잡히는 경우.
    let proc = p(&[
        "node",
        "/usr/lib/node_modules/@anthropic-ai/claude-code/cli.js",
    ]);
    assert_eq!(matchers::identify(&proc).unwrap().name, "claude");
}

#[test]
fn does_not_match_incidental_mentions() {
    // 정규식으로 커맨드라인 전체를 훑었다면 전부 오탐이 났을 것들.
    assert!(matchers::identify(&p(&["vim", "claude.md"])).is_none());
    assert!(matchers::identify(&p(&["grep", "-r", "codex", "."])).is_none());
    assert!(matchers::identify(&p(&["git", "commit", "-m", "add aider config"])).is_none());
    assert!(matchers::identify(&p(&["tail", "-f", "/var/log/gemini.log"])).is_none());
}

#[test]
fn does_not_recurse_past_second_arg() {
    // node script.js --wrapper claude  → 이건 에이전트가 아니다.
    assert!(matchers::identify(&p(&["node", "server.js", "--wrapper", "claude"])).is_none());
}

// ------------------------------------------------------- 표시 폭

#[test]
fn korean_counts_as_double_width() {
    assert_eq!(width("abc"), 3);
    assert_eq!(width("인증"), 4);
    assert_eq!(width("JWT 토큰"), 4 + 4); // "JWT " = 4, "토큰" = 4
}

#[test]
fn fit_never_exceeds_budget() {
    // 표가 어긋나지 않으려면 이게 절대 깨지면 안 된다.
    for s in [
        "api-server",
        "인증 리팩터링 작업",
        "a",
        "가나다라마바사아자차",
        "🚀 배포",
    ] {
        for max in 1..20 {
            let out = render::fit(s, max, Truncate::End);
            assert!(
                width(&out) <= max,
                "fit({s:?},{max}) = {out:?} 폭 {}",
                width(&out)
            );
        }
    }
}

#[test]
fn fit_modes_keep_the_right_end() {
    assert_eq!(render::fit("abcdefghij", 5, Truncate::End), "abcd…");
    assert_eq!(render::fit("abcdefghij", 5, Truncate::Start), "…ghij");
    let mid = render::fit("abcdefghij", 5, Truncate::Middle);
    assert!(mid.starts_with("ab") && mid.ends_with("ij"), "{mid}");
}

#[test]
fn fit_leaves_short_strings_alone() {
    assert_eq!(render::fit("인증", 10, Truncate::End), "인증");
}

// ------------------------------------------------------- 정규화

#[test]
fn model_display_names() {
    assert_eq!(normalize_model("claude-opus-4-6-20260514"), "opus-4.6");
    assert_eq!(normalize_model("claude-sonnet-4-6"), "sonnet-4.6");
    assert_eq!(normalize_model("gpt-5.2"), "gpt-5.2");
    assert_eq!(
        normalize_model("gemini-3-pro"),
        "gemini-3.pro".replace("3.pro", "3-pro")
    );
}

#[test]
fn prompt_strips_code_fences_and_headings() {
    let raw = "# 배경\n인증 리팩터링을 해줘\n```rust\nfn main() {}\n```\n끝";
    let out = normalize_prompt(raw);
    assert!(out.starts_with("인증 리팩터링을 해줘"));
    assert!(!out.contains("fn main"));
    assert!(!out.contains('#'));
}

#[test]
fn prompt_collapses_to_single_line() {
    let out = normalize_prompt("첫 줄\n\n둘째 줄");
    assert!(!out.contains('\n'));
    assert_eq!(out, "첫 줄 둘째 줄");
}

// ---------------------------------------------------------- 테마

#[test]
fn theme_hash_inside_string_is_not_a_comment() {
    // 이걸 틀리면 사용자가 설정한 색이 전부 사라진다.
    let kv = theme::parse("[colors]\nwaiting = \"#f5a623\"  # 주황\n");
    assert_eq!(kv.get("colors.waiting"), Some(&Val::Str("#f5a623".into())));
}

#[test]
fn theme_parses_lists_bools_ints() {
    let kv = theme::parse(
        "[columns]\norder = [\"status\", \"title\"]\n\n[columns.title]\nwidth = 30\n\n[columns.task]\ndim_when_inferred = false\n",
    );
    assert_eq!(
        kv.get("columns.order"),
        Some(&Val::List(vec!["status".into(), "title".into()]))
    );
    assert_eq!(kv.get("columns.title.width"), Some(&Val::Int(30)));
    assert_eq!(
        kv.get("columns.task.dim_when_inferred"),
        Some(&Val::Bool(false))
    );
}

#[test]
fn theme_ignores_garbage() {
    // 깨진 테마 파일이 프로그램을 죽이면 안 된다(§1.4).
    let kv = theme::parse("완전히 = \n[[[\nkey without equals\n= value without key\n");
    assert!(kv.len() <= 1);
}

#[test]
fn default_theme_has_the_five_columns() {
    let t = theme::Theme::default();
    let keys: Vec<&str> = t.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, vec!["status", "title", "agent", "llm", "task"]);
}

// -------------------------------------------------- 상태 우선순위

#[test]
fn waiting_sorts_first() {
    use crate::session::State;
    let mut states = vec![
        State::Done,
        State::Idle,
        State::Working,
        State::Waiting,
        State::Error,
    ];
    states.sort_by_key(|s| s.rank());
    assert_eq!(states[0], State::Waiting);
    assert_eq!(states[1], State::Working);
}

// ------------------------------------------------------- 템플릿 엔진 (§6 레이어 2)

use crate::tmpl::{self, b, s, Ctx, Val as TVal};

fn ctx(pairs: &[(&str, TVal)]) -> Ctx {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

#[test]
fn tmpl_substitutes_and_escapes() {
    let c = ctx(&[("title", s("a<b> & \"c\""))]);
    assert_eq!(
        tmpl::render("[{{title}}]", &c),
        "[a&lt;b&gt; &amp; &quot;c&quot;]"
    );
}

#[test]
fn tmpl_dotted_keys_are_flat_not_nested() {
    // {{task.text}} 는 중첩 조회가 아니라 "task.text" 라는 키를 찾는다.
    let c = ctx(&[("task.text", s("리팩터링"))]);
    assert_eq!(tmpl::render("{{task.text}}", &c), "리팩터링");
}

#[test]
fn tmpl_unknown_key_renders_empty() {
    // 템플릿 오타가 페이지를 죽이면 안 된다.
    assert_eq!(tmpl::render("a{{nope}}b", &ctx(&[])), "ab");
}

#[test]
fn tmpl_each_pushes_scope_and_keeps_outer_visible() {
    let mut root = ctx(&[("version", s("0.1"))]);
    root.insert(
        "sessions".into(),
        TVal::List(vec![
            ctx(&[("title", s("api"))]),
            ctx(&[("title", s("web"))]),
        ]),
    );
    assert_eq!(
        tmpl::render("{{#each sessions}}{{title}}@{{version}} {{/each}}", &root),
        "api@0.1 web@0.1 "
    );
}

#[test]
fn tmpl_each_over_missing_or_empty_list_emits_nothing() {
    let mut c = ctx(&[]);
    c.insert("xs".into(), TVal::List(vec![]));
    assert_eq!(tmpl::render("[{{#each xs}}x{{/each}}]", &c), "[]");
    assert_eq!(tmpl::render("[{{#each nope}}x{{/each}}]", &c), "[]");
}

#[test]
fn tmpl_if_else_and_unless() {
    let c = ctx(&[("on", b(true)), ("off", b(false)), ("empty", s(""))]);
    assert_eq!(tmpl::render("{{#if on}}Y{{else}}N{{/if}}", &c), "Y");
    assert_eq!(tmpl::render("{{#if off}}Y{{else}}N{{/if}}", &c), "N");
    // 빈 문자열은 거짓이다 — llm 이 없을 때 칩이 빈 채로 그려지면 안 된다.
    assert_eq!(tmpl::render("{{#if empty}}Y{{else}}N{{/if}}", &c), "N");
    assert_eq!(tmpl::render("{{#unless off}}Y{{/unless}}", &c), "Y");
    assert_eq!(tmpl::render("{{#unless on}}Y{{/unless}}", &c), "");
}

#[test]
fn tmpl_nested_blocks_match_correct_close_tag() {
    // 안쪽 {{/if}} 가 바깥 블록을 조기 종료시키면 카드가 통째로 사라진다.
    let mut root = ctx(&[]);
    root.insert(
        "xs".into(),
        TVal::List(vec![
            ctx(&[("t", s("a")), ("on", b(true))]),
            ctx(&[("t", s("b")), ("on", b(false))]),
        ]),
    );
    assert_eq!(
        tmpl::render("{{#each xs}}<{{t}}{{#if on}}!{{/if}}>{{/each}}", &root),
        "<a!><b>"
    );
}

#[test]
fn tmpl_unclosed_tag_is_literal_not_panic() {
    assert_eq!(tmpl::render("ok {{oops", &ctx(&[])), "ok {{oops");
    // 닫는 태그가 없어도 죽지 않는다.
    let mut c = ctx(&[]);
    c.insert("xs".into(), TVal::List(vec![ctx(&[("t", s("x"))])]));
    assert_eq!(tmpl::render("{{#each xs}}{{t}}", &c), "x");
}

#[test]
fn tmpl_else_belongs_to_its_own_block_kind() {
    let mut root = ctx(&[("any", b(true))]);
    root.insert("xs".into(), TVal::List(vec![ctx(&[("on", b(false))])]));
    assert_eq!(tmpl::render(
        "{{#if any}}{{#each xs}}A{{#if on}}Y{{else}}N{{/if}}B{{#unless on}}U{{else}}V{{/unless}}C{{/each}}{{else}}EMPTY{{/if}}",
        &root,
    ), "ANBUC");
}

#[test]
fn populated_default_template_keeps_all_five_fields() {
    use crate::session::{Confidence, Session, State, Task};
    let mut session = Session {
        prompt: None,
        request_at: None,
        last_answer: None,
        session_id: None,
        summary: None,
        request_marker: None,
        auxiliary: false,
        evidence: "unknown",
        id: "test0001".into(),
        legacy_id: "test0001".into(),
        title: "Windows MVP".into(),
        agent: crate::adapters::Agent {
            name: "codex",
            display: "Codex",
            has_reader: true,
        },
        llm_id: Some("model-test".into()),
        llm_display: Some("model-test".into()),
        task: Task {
            text: Some("작업 <검증>".into()),
            source: "transcript_first_prompt",
            confidence: Confidence::Inferred,
        },
        state: State::Waiting,
        since: 100,
        cwd: None,
        branch: None,
        pid: None,
    };
    let html = crate::html::render(&[session.clone()], 110, crate::html::DEFAULT_TEMPLATE);
    for field in [
        "Windows MVP",
        ">Codex</span>",
        "model-test",
        "작업 &lt;검증&gt;",
        "내 차례",
        "task inferred",
    ] {
        assert!(html.contains(field), "missing {field}");
    }
    assert!(!html.contains("{{"));
    session.task.text = None;
    session.llm_id = None;
    session.llm_display = None;
    let html = crate::html::render(&[session], 110, crate::html::DEFAULT_TEMPLATE);
    assert!(html.contains("— 지시 내용 없음"));
    assert!(html.contains("<span class=\"chip\">—</span>"));
    assert!(html.contains(">Codex</span>"));
}

#[test]
fn default_template_renders_without_leftover_tags() {
    // 기본 템플릿과 컨텍스트가 어긋나면 화면에 {{...}} 가 그대로 남는다.
    let out = crate::html::render(&[], crate::time::now(), crate::html::DEFAULT_TEMPLATE);
    assert!(!out.contains("{{"), "미치환 태그 잔존: {out}");
    assert!(out.contains("data-waid-root"), "갱신 훅이 사라졌다");
    assert!(
        out.contains("돌고 있는 코딩 에이전트가 없습니다"),
        "빈 상태가 안 나온다"
    );
}

// ------------------------------------------------------- 어댑터 (§4)

use crate::adapters;

#[test]
fn adapter_minimal_definition_loads() {
    let d = adapters::parse_def_for_test(
        r#"
        [adapter]
        name = "mytool"
        exec = ["mytool", "mytool-cli"]
        "#,
    )
    .expect("최소 정의는 통과해야 한다");
    assert_eq!(d.agent.name, "mytool");
    // display 를 생략하면 name 을 쓴다.
    assert_eq!(d.agent.display, "mytool");
    assert_eq!(d.exec, vec!["mytool", "mytool-cli"]);
    // 트랜스크립트 경로가 없으면 리더도 없다 — 없는 능력을 주장하지 않는다.
    assert!(!d.agent.has_reader);
}

#[test]
fn adapter_transcript_dir_enables_reader_and_expands_tilde() {
    // HOME 을 변경하지 않는다 — 테스트가 병렬로 돌고 어댑터 테이블은
    // OnceLock 으로 캐시되므로, 환경을 건드리면 다른 테스트가 흔들린다.
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .expect("home");
    let d = adapters::parse_def_for_test(
        r#"
        [adapter]
        name = "mytool"
        display = "My Tool"
        exec = ["mytool"]

        [transcript]
        dir = "~/.mytool/sessions"
        "#,
    )
    .unwrap();
    assert_eq!(d.agent.display, "My Tool");
    assert!(d.agent.has_reader);
    // 내 홈은 반드시 후보에 들어 있어야 한다. 경계 너머 후보가
    // 추가로 붙는 것은 환경에 따라 다르므로 개수는 고정하지 않는다.
    let expected = std::path::PathBuf::from(home).join(".mytool/sessions");
    assert!(
        d.transcript_dirs.contains(&expected),
        "내 홈 루트가 빠졌다: {:?}",
        d.transcript_dirs
    );
}

#[test]
fn adapter_accepts_bare_string_for_single_entry() {
    // 항목이 하나면 목록 대신 문자열로 쓰는 사람이 반드시 있다.
    let d = adapters::parse_def_for_test("[adapter]\nname = \"x\"\nexec = \"x\"\n").unwrap();
    assert_eq!(d.exec, vec!["x"]);
}

#[test]
fn adapter_rejects_incomplete_definitions() {
    // 이름 없음
    assert!(adapters::parse_def_for_test("[adapter]\nexec = [\"x\"]\n").is_err());
    // 감지 수단 없음 — 잡을 수 없는 어댑터는 무의미하다
    assert!(adapters::parse_def_for_test("[adapter]\nname = \"x\"\n").is_err());
    // 공백 섞인 이름은 --agent 필터를 망가뜨린다
    assert!(
        adapters::parse_def_for_test("[adapter]\nname = \"my tool\"\nexec = [\"x\"]\n").is_err()
    );
}

#[test]
fn adapter_markers_alone_are_enough() {
    // exec 없이 패키지 표지만으로도 성립한다 (`node .../cli.js` 형태).
    let d = adapters::parse_def_for_test("[adapter]\nname = \"x\"\nmarkers = [\"@vendor/x\"]\n")
        .unwrap();
    assert!(d.exec.is_empty());
    assert_eq!(d.markers, vec!["@vendor/x"]);
}

#[test]
fn builtin_table_is_intact_after_refactor() {
    let names: Vec<&str> = adapters::all().iter().map(|a| a.name).collect();
    for expected in [
        "claude", "codex", "gemini", "opencode", "aider", "cursor", "copilot", "goose", "cline",
        "roo", "vscode", "continue",
    ] {
        assert!(names.contains(&expected), "{expected} 가 사라졌다");
    }
    // 로그 리더 지원과 프로세스 전용 감지를 구분한다.
    assert!(adapters::by_name("claude").unwrap().has_reader);
    assert!(adapters::by_name("copilot").unwrap().has_reader);
    assert!(!adapters::by_name("aider").unwrap().has_reader);
    // JSONL 이 아닌 소스도 리더다 — Cursor 는 SQLite, Cline 은 JSON 파일이다.
    assert!(adapters::by_name("cursor").unwrap().has_reader);
    assert!(adapters::by_name("cline").unwrap().has_reader);
    assert!(!adapters::queries("cursor").is_empty());
    assert!(!adapters::json_source("cline").0.is_empty());
}

// ------------------------------------------------------- 세션 식별자

#[test]
fn short_id_avalanches_on_trailing_bytes() {
    // pid 만 다른 입력들이 같은 앞자리를 갖던 버그의 회귀 테스트.
    // FNV-1a 의 상위 비트를 쓰면 794/796/798 이 전부 같은 4자리를 받는다.
    let ids: Vec<String> = ["794", "796", "798", "800", "802"]
        .iter()
        .map(|pid| crate::session::short_id(&["claude", pid]))
        .collect();
    let prefixes: std::collections::HashSet<&str> = ids.iter().map(|i| &i[..4]).collect();
    assert_eq!(prefixes.len(), ids.len(), "앞 4자리가 충돌한다: {ids:?}");
}

#[test]
fn full_source_ids_survive_legacy_short_id_collisions() {
    let agent = crate::adapters::by_name("codex").unwrap();
    let make = |session_id: &str| crate::transcript::Transcript {
        last_answer: None,
        session_id: Some(session_id.into()),
        current_prompt: Some("same request".into()),
        request_marker: None,
        request_at: None,
        auxiliary: false,
        path: session_id.into(),
        last_event_at: 1,
        inferred_time: false,
        cwd: None,
        model: None,
        first_prompt: Some("same request".into()),
        event_state: None,
    };
    let mut sessions = [
        "00000000-0000-0000-0000-000000006c3c",
        "00000000-0000-0000-0000-00000000b713",
    ]
    .map(|id| crate::session::from_pair(None, agent, &make(id), 1))
    .to_vec();
    assert_eq!(sessions[0].legacy_id, "adee487a");
    assert_eq!(sessions[0].legacy_id, sessions[1].legacy_id);
    assert_ne!(sessions[0].id, sessions[1].id);
    let mut seen = std::collections::HashSet::new();
    sessions.retain(|session| seen.insert(session.id.clone()));
    assert_eq!(sessions.len(), 2);
    for session in &mut sessions { session.title = "same-project".into(); }
    crate::session::dedupe_titles(&mut sessions);
    assert_ne!(sessions[0].title, sessions[1].title);
    assert!(sessions.iter().all(|s| s.title.starts_with("same-project·adee487a-")));
    let json = crate::session::to_json(&sessions, 1, false);
    assert!(json.contains("\"schema\":2"));
    assert_eq!(json.matches("\"legacy_id\":\"adee487a\"").count(), 2);
}

#[test]
fn dedupe_extends_suffix_until_titles_are_unique() {
    use crate::adapters::Agent;
    use crate::session::{dedupe_titles, Confidence, State, Task};

    let agent = Agent {
        name: "claude",
        display: "Claude Code",
        has_reader: true,
    };
    let mk = |id: &str| crate::session::Session {
        prompt: None,
        request_at: None,
        last_answer: None,
        session_id: None,
        summary: None,
        request_marker: None,
        auxiliary: false,
        evidence: "unknown",
        id: id.to_string(),
        legacy_id: id.to_string(),
        title: "api".into(),
        agent,
        llm_id: None,
        llm_display: None,
        task: Task {
            text: None,
            source: "none",
            confidence: Confidence::None,
        },
        state: State::Idle,
        since: 0,
        cwd: None,
        branch: None,
        pid: None,
    };

    // 앞 4자리가 일부러 겹치는 id 들.
    let mut sessions = vec![mk("aaaa0001"), mk("aaaa0002"), mk("bbbb0003")];
    dedupe_titles(&mut sessions);

    let titles: std::collections::HashSet<&String> = sessions.iter().map(|s| &s.title).collect();
    assert_eq!(
        titles.len(),
        3,
        "구분되지 않았다: {:?}",
        sessions.iter().map(|s| &s.title).collect::<Vec<_>>()
    );
}

// ------------------------------------------------------- 경계 넘기 (§5.1, v0.4)

#[test]
fn adapter_accepts_multiple_transcript_roots() {
    let d = adapters::parse_def_for_test(
        r#"
        [adapter]
        name = "x"
        exec = ["x"]

        [transcript]
        dir  = "/abs/one"
        dirs = ["/abs/two", "/abs/three"]
        "#,
    )
    .unwrap();
    // dir 과 dirs 는 합쳐진다. 한 줄만 쓰던 어댑터가 계속 동작해야 한다.
    let got: Vec<String> = d
        .transcript_dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    for want in ["/abs/one", "/abs/two", "/abs/three"] {
        assert!(got.contains(&want.to_string()), "{want} 가 빠졌다: {got:?}");
    }
}

#[test]
fn duplicate_roots_are_collapsed() {
    let d = adapters::parse_def_for_test(
        "[adapter]\nname = \"x\"\nexec = [\"x\"]\n\n[transcript]\ndir = \"/same\"\ndirs = [\"/same\"]\n",
    )
    .unwrap();
    // 같은 루트를 두 번 훑으면 세션이 두 번 뜬다.
    assert_eq!(d.transcript_dirs.len(), 1, "{:?}", d.transcript_dirs);
}

#[test]
fn absolute_roots_are_not_multiplied_across_homes() {
    // `~` 만 홈 후보 전체로 펼쳐진다. 절대 경로는 하나 그대로여야 한다.
    let d = adapters::parse_def_for_test(
        "[adapter]\nname = \"x\"\nexec = [\"x\"]\n\n[transcript]\ndir = \"/opt/agent/logs\"\n",
    )
    .unwrap();
    assert_eq!(d.transcript_dirs.len(), 1);
}

#[test]
fn builtin_roots_cover_the_current_home() {
    // Windows 에서 HOME 이 없어 USERPROFILE 로 떨어지는 경로까지 포함해,
    // 내 홈의 트랜스크립트 루트는 어떤 환경에서도 후보에 있어야 한다.
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .expect("home");
    let roots = adapters::transcript_dirs("claude");
    let want = std::path::PathBuf::from(home).join(".claude/projects");
    assert!(roots.contains(&want), "내 홈 루트가 빠졌다: {roots:?}");
}

#[test]
fn agent_without_transcript_root_has_no_reader() {
    let roots = adapters::transcript_dirs("aider");
    assert!(roots.is_empty());
    assert!(!adapters::by_name("aider").unwrap().has_reader);
    // 없는 에이전트를 물어도 패닉하지 않는다.
    assert!(adapters::transcript_dirs("nope").is_empty());
}

#[test]
fn theme_path_uses_native_home_unless_overridden() {
    let expected = if let Some(p) = std::env::var_os("WAID_THEME") {
        std::path::PathBuf::from(p)
    } else {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME")
                    .or_else(|| std::env::var_os("USERPROFILE"))
                    .unwrap();
                std::path::PathBuf::from(home).join(".config")
            });
        base.join("waid/theme.toml")
    };
    assert_eq!(theme::path(), Some(expected));
}

#[test]
fn transcript_events_distinguish_turn_end_from_session_end() {
    use crate::session::State;
    for (event, want) in [
        (
            r#"{"type":"event_msg","payload":{"type":"task_complete"}}"#,
            Some(State::Waiting),
        ),
        (
            r#"{"type":"event_msg","payload":{"type":"task_started"}}"#,
            Some(State::Working),
        ),
        (
            r#"{"type":"assistant","message":{"stop_reason":"end_turn"}}"#,
            Some(State::Waiting),
        ),
        (
            r#"{"type":"assistant","message":{"stop_reason":"tool_use"}}"#,
            Some(State::Working),
        ),
        (
            r#"{"type":"response_item","payload":{"type":"message","role":"assistant","phase":"final"}}"#,
            Some(State::Waiting),
        ),
        (
            r#"{"type":"response_item","payload":{"type":"function_call_output","output":{"type":"error"}}}"#,
            Some(State::Working),
        ),
        (
            r#"{"type":"event_msg","payload":{"type":"token_count"}}"#,
            None,
        ),
        (
            r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"<environment_context>metadata</environment_context>"}]}}"#,
            None,
        ),
    ] {
        assert_eq!(
            crate::transcript::event_state(&json::parse(event).unwrap()),
            want,
            "{event}"
        );
    }
}

#[test]
fn transcript_without_process_keeps_cwd_task_and_waiting() {
    use crate::session::{self, Confidence, State};
    let mut transcript = crate::transcript::Transcript {
        request_at: None,
        last_answer: None,
        inferred_time: false,
        session_id: Some("original-session-id".into()),
        current_prompt: None,
        request_marker: None,
        auxiliary: false,
        path: "fixture.jsonl".into(),
        last_event_at: 100,
        // Keep this fixture outside the checkout, including detached tag builds.
        cwd: Some(std::env::temp_dir().join("waid-session-fixture/project")),
        model: Some("claude-opus-4-6".into()),
        first_prompt: Some("Windows MVP".into()),
        event_state: Some(State::Waiting),
    };
    let agent = crate::adapters::by_name("claude").unwrap();
    let session = session::from_pair(None, agent, &transcript, 200);
    assert_eq!(session.state, State::Waiting);
    assert_eq!(session.title, "project");
    assert_eq!(session.pid, None);
    assert_eq!(session.session_id.as_deref(), Some("original-session-id"));
    assert!(session::to_json(&[session.clone()], 200, false)
        .contains("\"session_id\":\"original-session-id\""));
    assert_eq!(session.task.confidence, Confidence::Inferred);
    assert_eq!(session.llm_id.as_deref(), Some("claude-opus-4-6"));
    assert_eq!(session.llm_display.as_deref(), Some("opus-4.6"));
    transcript.event_state = None;
    assert_eq!(
        session::from_pair(None, agent, &transcript, 101).state,
        State::Unknown
    );
    assert_eq!(
        session::from_pair(None, agent, &transcript, 200).state,
        State::Unknown
    );
}

#[test]
fn transcript_reads_tail_between_32_and_64_kib_and_codex_payload() {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!("waid-tail-{}.jsonl", std::process::id()));
    let mut file = std::fs::File::create(&path).unwrap();
    writeln!(file, "{}", r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"실제 첫 지시"}]}}"#).unwrap();
    writeln!(file, "{}", " ".repeat(40 * 1024)).unwrap();
    writeln!(
        file,
        "{}",
        r#"{"type":"turn_context","payload":{"model":"model-latest"}}"#
    )
    .unwrap();
    writeln!(
        file,
        "{}",
        r#"{"type":"event_msg","payload":{"type":"task_complete"}}"#
    )
    .unwrap();
    drop(file);
    let t = crate::transcript::read(&path, 100).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(t.model.as_deref(), Some("model-latest"));
    assert_eq!(t.first_prompt.as_deref(), Some("실제 첫 지시"));
    assert_eq!(t.event_state, Some(crate::session::State::Waiting));
}

#[test]
fn transcript_finds_first_request_after_long_injected_context() {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!("waid-long-head-{}.jsonl", std::process::id()));
    let mut file = std::fs::File::create(&path).unwrap();
    writeln!(
        file,
        "{}",
        r#"{"type":"session_meta","payload":{"cwd":"project"}}"#
    )
    .unwrap();
    writeln!(file, "{}", r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"<recommended_plugins>injected</recommended_plugins>"}]}}"#).unwrap();
    writeln!(file, "{}", " ".repeat(80 * 1024)).unwrap();
    writeln!(file, "{}", r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"실제 작업 요청"}]}}"#).unwrap();
    writeln!(file, "{}", " ".repeat(80 * 1024)).unwrap();
    writeln!(
        file,
        "{}",
        r#"{"type":"event_msg","payload":{"type":"task_complete"}}"#
    )
    .unwrap();
    drop(file);
    let t = crate::transcript::read(&path, 100).unwrap();
    assert_eq!(t.cwd, Some("project".into()));
    assert_eq!(t.first_prompt.as_deref(), Some("실제 작업 요청"));
    assert_eq!(t.event_state, Some(crate::session::State::Waiting));
    assert_eq!(
        crate::transcript::read(&path, 101).unwrap().last_event_at,
        100
    );
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    writeln!(
        file,
        "{}",
        r#"{"type":"user","message":{"content":"다음 작업"}}"#
    )
    .unwrap();
    drop(file);
    let t = crate::transcript::read(&path, 102).unwrap();
    assert_eq!(t.current_prompt.as_deref(), Some("다음 작업"));
    assert_eq!(t.first_prompt.as_deref(), Some("실제 작업 요청"));
    assert_eq!(t.event_state, Some(crate::session::State::Working));
    std::fs::write(
        &path,
        r#"{"type":"user","message":{"content":"교체된 로그"}}"#,
    )
    .unwrap();
    assert_eq!(
        crate::transcript::read(&path, 103)
            .unwrap()
            .first_prompt
            .as_deref(),
        Some("교체된 로그")
    );
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn normalized_prompt_is_bounded_without_splitting_unicode() {
    assert_eq!(normalize_prompt(&"가나다".repeat(200)).chars().count(), 200);
    assert_eq!(
        normalize_prompt(&format!("# {}", "한".repeat(300)))
            .chars()
            .count(),
        200
    );
}

#[test]
fn conversation_answer_is_text_from_the_current_turn_only() {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!("waid-answer-{}.jsonl", std::process::id()));
    let mut file = std::fs::File::create(&path).unwrap();
    writeln!(
        file,
        "{}",
        r#"{"type":"user","message":{"content":"첫 요청"}}"#
    )
    .unwrap();
    for answer in [
        r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"private"},{"type":"text","text":"답변 하나"}]}}"#,
        r#"{"type":"response_item","payload":{"role":"assistant","content":[{"type":"output_text","text":"답변 둘"}]}}"#,
        r#"{"type":"assistant.message","data":{"content":"답변 셋"}}"#,
    ] {
        writeln!(file, "{answer}").unwrap();
        file.flush().unwrap();
        let t = crate::transcript::read(&path, 1).unwrap();
        assert!(t.last_answer.as_deref().unwrap().starts_with("답변"));
        assert!(!t.last_answer.as_deref().unwrap().contains("private"));
    }
    writeln!(
        file,
        "{}",
        r#"{"type":"user","message":{"content":"새 요청"}}"#
    )
    .unwrap();
    file.flush().unwrap();
    assert!(crate::transcript::read(&path, 2)
        .unwrap()
        .last_answer
        .is_none());
    writeln!(file, "{}", r#"{"type":"assistant","message":{"content":[{"type":"tool_use","input":{"text":"not an answer"}}]}}"#).unwrap();
    file.flush().unwrap();
    assert!(crate::transcript::read(&path, 3)
        .unwrap()
        .last_answer
        .is_none());
    drop(file);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn transcript_metadata_does_not_refresh_observed_activity() {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!("waid-event-time-{}.jsonl", std::process::id()));
    let now = crate::time::from_iso8601("2026-09-06T04:00:00Z").unwrap();
    let earlier = now - 3600;
    std::fs::write(&path, concat!(
        "{\"timestamp\":\"2026-09-06T03:00:00Z\",\"type\":\"user\",\"message\":{\"content\":\"real request\"}}\n",
        "{\"timestamp\":\"2026-09-06T04:00:00Z\",\"type\":\"cost-state\"}\n"
    )).unwrap();
    let t = crate::transcript::read(&path, now).unwrap();
    assert_eq!(t.last_event_at, earlier);
    let agent = crate::adapters::by_name("claude").unwrap();
    assert_eq!(
        crate::session::from_pair(None, agent, &t, now).state,
        crate::session::State::Unknown
    );
    assert_eq!(
        crate::transcript::read(&path, now + 1)
            .unwrap()
            .last_event_at,
        earlier
    );
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    writeln!(
        file,
        "{}",
        r#"{"timestamp":"2026-09-06T04:00:01Z","type":"cost-state"}"#
    )
    .unwrap();
    assert_eq!(
        crate::transcript::read(&path, now + 1)
            .unwrap()
            .last_event_at,
        earlier
    );
    writeln!(file, "{}", r#"{"timestamp":"2026-09-06T13:00:02+09:00","type":"event_msg","payload":{"type":"task_complete"}}"#).unwrap();
    let t = crate::transcript::read(&path, now + 3).unwrap();
    assert_eq!(t.last_event_at, now + 2);
    assert_eq!(t.event_state, Some(crate::session::State::Waiting));
    drop(file);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn latest_request_identity_and_auxiliary_are_from_envelopes() {
    let path = std::env::temp_dir().join(format!("waid-identity-{}.jsonl", std::process::id()));
    std::fs::write(&path, concat!(
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"stable-session\",\"cwd\":\"project\",\"source\":{\"subagent\":{\"other\":\"guardian\"}}}}\n",
        "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"first request\"}]}}\n",
        "{\"type\":\"turn_context\",\"payload\":{\"model\":\"real-model\"}}\n",
        "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"<environment_context>injected</environment_context>\\nlatest request\"}]}}\n",
        "{\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"output\":{\"cwd\":\"fake\",\"model\":\"fake\",\"role\":\"user\",\"content\":\"fake request\"}}}\n"
    )).unwrap();
    let t = crate::transcript::read(&path, 100).unwrap();
    assert_eq!(t.session_id.as_deref(), Some("stable-session"));
    assert!(t.auxiliary);
    assert_eq!(t.model.as_deref(), Some("real-model"));
    assert_eq!(t.cwd, Some("project".into()));
    assert_eq!(t.current_prompt.as_deref(), Some("latest request"));
    assert_eq!(t.first_prompt.as_deref(), Some("first request"));
    let agent = crate::adapters::by_name("codex").unwrap();
    let a = crate::session::from_pair(None, agent, &t, 100);
    assert_eq!(a.prompt.as_deref(), Some("latest request"));
    assert_eq!(a.task.text.as_deref(), Some("latest request"));
    let mut moved = t.clone();
    moved.path = "elsewhere.jsonl".into();
    assert_eq!(a.id, crate::session::from_pair(None, agent, &moved, 100).id);
    assert!(crate::session::to_json(&[a], 100, false).contains("\"alive\":null"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn document_prompts_preserve_lines_and_text_beyond_the_summary_limit() {
    let path = std::env::temp_dir().join(format!("waid-raw-prompt-{}.json", std::process::id()));
    let prompt = format!("# Request\n  {}\n\nlast line", "long prompt ".repeat(30));
    let mut writer = crate::json::Writer::new(false);
    writer.begin_obj();
    writer.field_str("role", "user");
    writer.field_str("content", &prompt);
    writer.end_obj();
    let doc = writer.buf;
    std::fs::write(&path, &doc).unwrap();
    let json = crate::transcript::read_json(&path, 100).unwrap();
    assert_eq!(json.current_prompt.as_deref(), Some(prompt.as_str()));
    let row = vec![("id".into(), "raw-prompt".into()), ("blob".into(), doc)];
    let sqlite = crate::transcript::row_transcript(&path, &row, 100).unwrap();
    assert_eq!(sqlite.current_prompt, json.current_prompt);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn worker_classification_reads_envelopes_and_late_dispatch_without_hiding_parent() {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!("waid-worker-{}.jsonl", std::process::id()));
    let meta = r#"{"type":"session_meta","payload":{"id":"worker","source":"cli"}}"#;
    let worker = r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"You are working inside Orca, a multi-agent IDE. You are a dispatched worker.\nYour task ID is: task_example\nDo the task."}]}}"#;
    std::fs::write(&path, format!("{meta}\n{{\"type\":\"user\",\"message\":{{\"content\":\"Coordinate workers\"}}}}\n")).unwrap();
    assert!(!crate::transcript::read(&path, 100).unwrap().auxiliary);
    let mut file = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
    writeln!(file, "{worker}").unwrap();
    let t = crate::transcript::read(&path, 101).unwrap();
    assert!(t.auxiliary);
    assert_eq!(t.session_id.as_deref(), Some("worker"));
    writeln!(file, "{{\"type\":\"padding\",\"text\":\"{}\"}}", "x".repeat(300_000)).unwrap();
    assert!(crate::transcript::read(&path, 102).unwrap().auxiliary);
    drop(file);
    for (metadata, expected) in [
        (r#""source":{"subagent":{"thread_spawn":{"parent_thread_id":"parent"}}}"#, true),
        (r#""source":"cli","parent_thread_id":"parent""#, true),
        (r#""source":"subagent""#, true),
        (r#""thread_source":{"subagent":"review"}"#, true),
        (r#""source":{"subagent":null},"parent_thread_id":null"#, false),
        (r#""source":"cli","forked_from_id":"parent""#, false),
    ] {
        std::fs::write(&path, format!("{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"test\",{metadata}}}}}\n")).unwrap();
        assert_eq!(crate::transcript::read(&path, 103).unwrap().auxiliary, expected, "{metadata}");
    }
    let output = format!("{meta}\n{{\"type\":\"response_item\",\"payload\":{{\"type\":\"function_call_output\",\"output\":{worker}}}}}\n");
    std::fs::write(&path, output).unwrap();
    assert!(!crate::transcript::read(&path, 104).unwrap().auxiliary);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn claude_sidechain_does_not_replace_parent_identity() {
    let path = std::env::temp_dir().join(format!("waid-sidechain-{}.jsonl", std::process::id()));
    std::fs::write(&path, r#"{"type":"user","sessionId":"shared-parent","agentId":"claude-child","isSidechain":true,"message":{"content":"child task"}}"#).unwrap();
    let t = crate::transcript::read(&path, 100).unwrap();
    assert!(t.auxiliary);
    assert_eq!(t.current_prompt.as_deref(), Some("child task"));
    assert!(t.session_id.is_none());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn undated_metadata_and_growing_replacement_do_not_reuse_false_activity() {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!("waid-undated-{}.jsonl", std::process::id()));
    std::fs::write(
        &path,
        "{\"type\":\"user\",\"sessionId\":\"old\",\"message\":{\"content\":\"first\"}}\n",
    )
    .unwrap();
    let first = crate::transcript::read(&path, 1000).unwrap();
    assert!(first.inferred_time);
    assert_eq!(first.last_event_at, 1000);
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    writeln!(file, "{}", r#"{"type":"cost-state"}"#).unwrap();
    drop(file);
    assert_eq!(
        crate::transcript::read(&path, 2000).unwrap().last_event_at,
        1000
    );
    std::fs::write(&path,format!("{{\"type\":\"user\",\"sessionId\":\"new\",\"message\":{{\"content\":\"replacement\"}}}}\n{}", " ".repeat(600))).unwrap();
    let replaced = crate::transcript::read(&path, 3000).unwrap();
    assert_eq!(replaced.session_id.as_deref(), Some("new"));
    assert_eq!(replaced.first_prompt.as_deref(), Some("replacement"));
    assert_eq!(replaced.last_event_at, 3000);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn request_marker_ignores_metadata_and_distinguishes_repeated_requests() {
    use std::io::Write;
    let path =
        std::env::temp_dir().join(format!("waid-request-marker-{}.jsonl", std::process::id()));
    std::fs::write(&path,"{\"type\":\"user\",\"timestamp\":\"2026-09-06T00:00:00Z\",\"message\":{\"content\":\"continue\"}}\n").unwrap();
    let first = crate::transcript::read(&path, 100).unwrap();
    assert!(first.request_marker.is_some());
    assert_eq!(
        first.request_at,
        crate::time::from_iso8601("2026-09-06T00:00:00Z")
    );
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    writeln!(
        file,
        "{}",
        r#"{"type":"turn_context","payload":{"model":"changed"}}"#
    )
    .unwrap();
    assert_eq!(
        crate::transcript::read(&path, 101).unwrap().request_marker,
        first.request_marker
    );
    writeln!(
        file,
        "{}",
        r#"{"type":"user","timestamp":"2026-09-06T00:01:00Z","message":{"content":"continue"}}"#
    )
    .unwrap();
    let resumed = crate::transcript::read(&path, 102).unwrap();
    assert_eq!(first.current_prompt, resumed.current_prompt);
    assert_ne!(first.request_marker, resumed.request_marker);
    assert_eq!(
        resumed.request_at,
        crate::time::from_iso8601("2026-09-06T00:01:00Z")
    );
    writeln!(file, "{}", r#"{"type":"user","timestamp":"2026-09-06T00:02:00Z","message":{"content":"<task-notification>background job done</task-notification>Read the output file"}}"#).unwrap();
    let notification = crate::transcript::read(&path, 103).unwrap();
    assert_eq!(notification.current_prompt, resumed.current_prompt);
    assert_eq!(notification.request_marker, resumed.request_marker);
    assert_eq!(notification.request_at, resumed.request_at);
    drop(file);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn other_agents_and_windows_runtime_entrypoints() {
    for (argv, agent) in [
        (vec![r"C:\Tools\GEMINI.EXE"], "gemini"),
        (
            vec!["node.exe", r"C:\npm\@google\gemini-cli\dist\index.js"],
            "gemini",
        ),
        (
            vec![
                "NODE.EXE",
                "--no-warnings",
                r"C:\npm\@github\copilot\index.js",
            ],
            "copilot",
        ),
        (vec!["bun", r"C:\npm\opencode-ai\bin\opencode"], "opencode"),
        (vec!["python.exe", "-m", "aider"], "aider"),
        (vec!["python3.12", "-u", "-m", "aider.main"], "aider"),
        (vec!["python", r"C:\venv\Scripts\aider-script.py"], "aider"),
        (vec!["cursor-agent.exe"], "cursor"),
        (vec!["goose.exe"], "goose"),
    ] {
        assert_eq!(
            matchers::identify(&p(&argv)).map(|a| a.name),
            Some(agent),
            "{argv:?}"
        );
    }
}

#[test]
fn package_mentions_and_supervisors_are_not_sessions() {
    for argv in [
        vec!["rg", "@google/gemini-cli"],
        vec!["node", "server.js", "@github/copilot"],
        vec!["node", "-e", "require('@google/gemini-cli')"],
        vec!["node", "/npm/@github/copilot-fake/index.js"],
        vec!["node", "/npm/fake@github/copilot/index.js"],
        vec!["python", "-c", "import aider"],
        vec!["npx", "@google/gemini-cli"],
        vec!["uv", "run", "aider"],
        vec!["uvx", "aider"],
    ] {
        assert!(matchers::identify(&p(&argv)).is_none(), "{argv:?}");
    }
}

#[test]
fn copilot_metadata_requests_child_events_and_resume() {
    use std::io::Write;
    let path =
        std::env::temp_dir().join(format!("waid-copilot-reader-{}.jsonl", std::process::id()));
    std::fs::write(&path, concat!(
        "{\"type\":\"session.start\",\"data\":{\"sessionId\":\"copilot-test\",\"selectedModel\":\"model-one\",\"context\":{\"cwd\":\"D:/fixture/project\"}}}\n",
        "{\"type\":\"user.message\",\"id\":\"request-one\",\"timestamp\":\"2026-09-06T00:00:00Z\",\"data\":{\"content\":\"original task\"}}\n",
        "{\"type\":\"session.idle\",\"timestamp\":\"2026-09-06T00:00:10Z\",\"data\":{}}\n"
    )).unwrap();
    let first = transcript::read(&path, 100).unwrap();
    assert_eq!(first.session_id.as_deref(), Some("copilot-test"));
    assert_eq!(first.model.as_deref(), Some("model-one"));
    assert_eq!(
        first.cwd.as_deref(),
        Some(std::path::Path::new("D:/fixture/project"))
    );
    assert_eq!(first.event_state, Some(State::Waiting));
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    writeln!(
        f,
        "{}",
        r#"{"type":"session.model_change","data":{"newModel":"model-two"}}"#
    )
    .unwrap();
    writeln!(
        f,
        "{}",
        r#"{"type":"user.message","agentId":"child","data":{"content":"child request"}}"#
    )
    .unwrap();
    writeln!(
        f,
        "{}",
        r#"{"type":"session.error","agentId":"child","data":{"message":"child error"}}"#
    )
    .unwrap();
    let metadata = transcript::read(&path, 200).unwrap();
    assert_eq!(metadata.model.as_deref(), Some("model-two"));
    assert_eq!(metadata.current_prompt.as_deref(), Some("original task"));
    assert_eq!(metadata.request_marker, first.request_marker);
    assert_eq!(metadata.last_event_at, first.last_event_at);
    assert_eq!(metadata.event_state, Some(State::Waiting));
    writeln!(
        f,
        "{}",
        r#"{"type":"session.shutdown","data":{"shutdownType":"routine"}}"#
    )
    .unwrap();
    assert_eq!(
        transcript::read(&path, 201).unwrap().event_state,
        Some(State::Idle)
    );
    write!(
        f,
        "{}",
        r#"{"type":"user.message","id":"request-two","data":{"content":"new task"}"#
    )
    .unwrap();
    assert_eq!(
        transcript::read(&path, 202).unwrap().event_state,
        Some(State::Idle)
    );
    writeln!(f, "}}").unwrap();
    let resumed = transcript::read(&path, 203).unwrap();
    assert_eq!(resumed.event_state, Some(State::Working));
    assert_eq!(resumed.current_prompt.as_deref(), Some("new task"));
    assert_eq!(resumed.first_prompt.as_deref(), Some("original task"));
    assert_ne!(resumed.request_marker, first.request_marker);
    drop(f);
    std::fs::write(
        &path,
        "{\"type\":\"session.start\",\"data\":{\"sessionId\":\"replaced\"}}\n",
    )
    .unwrap();
    let replaced = transcript::read(&path, 204).unwrap();
    assert_eq!(replaced.session_id.as_deref(), Some("replaced"));
    assert!(replaced.current_prompt.is_none());
    assert!(replaced.model.is_none());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn copilot_turn_end_is_not_session_completion() {
    for (event, expected) in [
        (r#"{"type":"assistant.turn_end","data":{}}"#, State::Working),
        (
            r#"{"type":"tool.execution_complete","data":{"success":false}}"#,
            State::Working,
        ),
        (r#"{"type":"session.idle","data":{}}"#, State::Waiting),
        (
            r#"{"type":"session.idle","data":{"aborted":true}}"#,
            State::Idle,
        ),
        (
            r#"{"type":"abort","data":{"reason":"user_initiated"}}"#,
            State::Idle,
        ),
        (r#"{"type":"session.error","data":{}}"#, State::Error),
        (
            r#"{"type":"session.shutdown","data":{"shutdownType":"error"}}"#,
            State::Error,
        ),
    ] {
        assert_eq!(
            transcript::event_state(&json::parse(event).unwrap()),
            Some(expected)
        );
    }
}

// ------------------------------------------------------- SQLite 소스 (v0.6)

#[test]
fn sqlite_reads_rows_by_column_name() {
    // OS 의 SQLite 를 빌려 쓴다. 없는 환경에서는 소스가 조용히 꺼져야 한다.
    if !crate::sqlite::available() {
        assert!(crate::sqlite::query(std::path::Path::new("none.db"), "select 1").is_err());
        return;
    }
    let path = std::env::temp_dir().join(format!("waid-sqlite-test-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    crate::sqlite::exec(
        &path,
        "create table t (id text, task text, updated_ms integer)",
    )
    .unwrap();
    crate::sqlite::exec(
        &path,
        "insert into t values ('a', '첫 요청', 1700000000000), ('b', '', 1700000001000)",
    )
    .unwrap();
    let rows =
        crate::sqlite::query(&path, "select id, task, updated_ms from t order by id").unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(crate::sqlite::get(&rows[0], "id"), Some("a"));
    assert_eq!(crate::sqlite::get(&rows[0], "task"), Some("첫 요청"));
    assert_eq!(
        crate::sqlite::get(&rows[0], "updated_ms"),
        Some("1700000000000")
    );
    // 빈 값은 없는 것으로 다룬다 — 카드에 빈 문자열을 띄우지 않는다.
    assert_eq!(crate::sqlite::get(&rows[1], "task"), None);
    assert!(crate::sqlite::query(
        &path,
        "select 1 as n union all select abs(-9223372036854775808)",
    )
    .is_err());
    let _ = std::fs::remove_file(&path);
}

// ------------------------------------------- JSONL 이 아닌 소스 (§4.1, v0.6)

#[test]
fn json_file_becomes_one_session() {
    // Cline·Roo 처럼 세션 하나를 JSON 배열 한 파일로 쓰는 형식.
    let dir = std::env::temp_dir().join(format!("waid-json-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("api_conversation_history.json");
    std::fs::write(
        &path,
        r#"[
          {"role":"user","content":"첫 요청","timestamp":1700000000000,"cwd":"D:/work"},
          {"role":"assistant","content":"작업 중","model":"claude-sonnet-5"},
          {"role":"user","content":"두 번째 요청","timestamp":1700000060000}
        ]"#,
    )
    .unwrap();
    let t = crate::transcript::read_json(&path, 1).expect("세션으로 읽혀야 한다");
    // 대표 요청은 첫 요청, 작업은 최근 요청이다 — JSONL 과 같은 규칙이다.
    assert_eq!(t.first_prompt.as_deref(), Some("첫 요청"));
    assert_eq!(t.current_prompt.as_deref(), Some("두 번째 요청"));
    assert_eq!(t.model.as_deref(), Some("sonnet-5"));
    assert_eq!(t.cwd, Some(std::path::PathBuf::from("D:/work")));
    // 파일 이름이 고정된 형식은 상위 폴더가 세션 식별자다.
    assert_eq!(
        t.session_id.as_deref(),
        dir.file_name().and_then(|n| n.to_str())
    );
    // 밀리초 타임스탬프는 초로 내려온다.
    assert_eq!(t.last_event_at, 1700000060);
    assert!(!t.inferred_time);
    assert_eq!(t.request_at, Some(1700000060));
    let marker = t.request_marker.clone();
    std::fs::write(
        &path,
        r#"{
          "updatedAt":1700000999000,
          "messages":[
            {"role":"user","content":"첫 요청","timestamp":1700000000000},
            {"role":"assistant","content":"new metadata","model":"changed"},
            {"role":"user","content":"두 번째 요청","timestamp":1700000060000,"updatedAt":1700000999000},
            {"role":"assistant","content":"later answer","timestamp":1700000999000}
          ]
        }"#,
    )
    .unwrap();
    let metadata = crate::transcript::read_json(&path, 999).unwrap();
    assert_eq!(metadata.request_marker, marker);
    std::fs::write(
        &path,
        r#"[
          {"role":"user","content":"첫 요청","timestamp":1700000000000},
          {"role":"user","content":"두 번째 요청","timestamp":1700000060000},
          {"role":"user","content":"두 번째 요청"}
        ]"#,
    )
    .unwrap();
    assert_ne!(
        crate::transcript::read_json(&path, 1000)
            .unwrap()
            .request_marker,
        marker
    );
    std::fs::write(&path, r#"[{"role":"user","content":"repeat"}]"#).unwrap();
    let repeated = crate::transcript::read_json(&path, 1001)
        .unwrap()
        .request_marker;
    std::fs::write(
        &path,
        r#"[{"role":"user","content":"repeat"},{"role":"user","content":"repeat"}]"#,
    )
    .unwrap();
    assert_ne!(
        crate::transcript::read_json(&path, 1002)
            .unwrap()
            .request_marker,
        repeated
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn json_model_uses_latest_event_metadata_not_tool_payloads() {
    let path = std::env::temp_dir().join(format!("waid-json-model-{}.json", std::process::id()));
    std::fs::write(
        &path,
        r#"{"model":"root-old","messages":[
          {"role":"assistant","content":"old","model":"event-old"},
          {"role":"user","content":"task"},
          {"role":"assistant","content":"done","model":"event-latest"},
          {"type":"response_item","payload":{"type":"function_call_output","output":{"model":"tool-fake"}}}
        ]}"#,
    )
    .unwrap();
    assert_eq!(
        crate::transcript::read_json(&path, 1)
            .unwrap()
            .model
            .as_deref(),
        Some("event-latest")
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn document_request_markers_keep_fractional_time_and_history_count() {
    let path = std::env::temp_dir().join(format!("waid-json-marker-{}.json", std::process::id()));
    std::fs::write(
        &path,
        r#"[{"role":"user","content":"repeat","timestamp":"2026-09-06T00:00:00.100Z"}]"#,
    )
    .unwrap();
    let first = crate::transcript::read_json(&path, 1).unwrap();
    std::fs::write(
        &path,
        r#"[{"role":"user","content":"repeat","timestamp":"2026-09-06T00:00:00.900Z"}]"#,
    )
    .unwrap();
    let fractional = crate::transcript::read_json(&path, 1).unwrap();
    assert_eq!(first.request_at, fractional.request_at);
    assert_ne!(first.request_marker, fractional.request_marker);
    assert!(fractional
        .request_marker
        .as_deref()
        .unwrap()
        .starts_with("r3:"));
    std::fs::write(
        &path,
        r#"[
      {"role":"user","content":"earlier","timestamp":"2026-09-05T23:59:59Z"},
      {"role":"user","content":"repeat","timestamp":"2026-09-06T00:00:00.900Z"}
    ]"#,
    )
    .unwrap();
    let history = crate::transcript::read_json(&path, 1).unwrap();
    assert_ne!(fractional.request_marker, history.request_marker);

    let row = |blob: &str| {
        crate::transcript::row_transcript(
            std::path::Path::new("state.vscdb"),
            &vec![
                ("id".into(), "session".into()),
                ("blob".into(), blob.into()),
            ],
            1,
        )
        .unwrap()
    };
    let sqlite_first = row(
        r#"{"messages":[{"role":"user","content":"repeat","timestamp":"2026-09-06T00:00:00.100Z"}]}"#,
    );
    let sqlite_fractional = row(
        r#"{"messages":[{"role":"user","content":"repeat","timestamp":"2026-09-06T00:00:00.900Z"}]}"#,
    );
    assert_ne!(
        sqlite_first.request_marker,
        sqlite_fractional.request_marker
    );
    let sqlite_history = row(
        r#"{"messages":[{"role":"user","content":"earlier"},{"role":"user","content":"repeat","timestamp":"2026-09-06T00:00:00.900Z"}]}"#,
    );
    assert_ne!(
        sqlite_fractional.request_marker,
        sqlite_history.request_marker
    );
    assert!(sqlite_history
        .request_marker
        .as_deref()
        .unwrap()
        .starts_with("r3:"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn sqlite_cache_fingerprint_tracks_subsecond_updates_and_wal_absence() {
    let path = std::env::temp_dir().join(format!("waid-cache-stamp-{}.db", std::process::id()));
    std::fs::write(&path, "same").unwrap();
    let base = std::time::UNIX_EPOCH + std::time::Duration::from_secs(2_000_000_000);
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(
            std::fs::FileTimes::new().set_modified(base + std::time::Duration::from_millis(100)),
        )
        .unwrap();
    let first = crate::transcript::sqlite_cache_fingerprint(&path);
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(
            std::fs::FileTimes::new().set_modified(base + std::time::Duration::from_millis(900)),
        )
        .unwrap();
    let later = crate::transcript::sqlite_cache_fingerprint(&path);
    assert_ne!(first, later);
    std::fs::write(&path, "different size").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(
            std::fs::FileTimes::new().set_modified(base + std::time::Duration::from_millis(900)),
        )
        .unwrap();
    let resized = crate::transcript::sqlite_cache_fingerprint(&path);
    assert_ne!(later, resized);
    let wal = std::path::PathBuf::from(format!("{}-wal", path.display()));
    std::fs::write(&wal, "wal").unwrap();
    let with_wal = crate::transcript::sqlite_cache_fingerprint(&path);
    assert_ne!(resized, with_wal);
    std::fs::remove_file(&wal).unwrap();
    assert_eq!(resized, crate::transcript::sqlite_cache_fingerprint(&path));
    let _ = std::fs::remove_file(path);
}

#[test]
fn json_without_a_user_request_is_not_a_session() {
    // 편집기 폴더에는 설정 JSON 이 굴러다닌다. 카드로 만들면 안 된다.
    let path = std::env::temp_dir().join(format!("waid-json-cfg-{}.json", std::process::id()));
    std::fs::write(&path, r#"{"version":1,"entries":{}}"#).unwrap();
    assert!(crate::transcript::read_json(&path, 1).is_none());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn file_uri_becomes_a_local_path() {
    // 편집기 계열은 경로를 URI 로 저장한다.
    assert_eq!(
        crate::transcript::local_path("file:///d%3A/work/repo"),
        Some(std::path::PathBuf::from("d:/work/repo"))
    );
    assert_eq!(
        crate::transcript::local_path("D:\\work"),
        Some(std::path::PathBuf::from("D:\\work"))
    );
    assert_eq!(
        crate::transcript::local_path("file:///D:/작업/프로젝트"),
        Some(std::path::PathBuf::from("D:/작업/프로젝트"))
    );
    assert_eq!(
        crate::transcript::local_path("file:///home/작업"),
        Some(std::path::PathBuf::from("/home/작업"))
    );
    assert_eq!(
        crate::transcript::local_path(
            "file:///D:/%EC%9E%91%EC%97%85/%ED%94%84%EB%A1%9C%EC%A0%9D%ED%8A%B8"
        ),
        Some(std::path::PathBuf::from("D:/작업/프로젝트"))
    );
    assert_eq!(
        crate::transcript::local_path("file://server/share/%EC%9E%91%EC%97%85"),
        Some(std::path::PathBuf::from(r"\\server\share\작업"))
    );
    assert_eq!(
        crate::transcript::local_path("file://localhost/D:/work"),
        Some(std::path::PathBuf::from("D:/work"))
    );
    assert_eq!(crate::transcript::local_path("file:///D:/bad%ZZ"), None);
    assert_eq!(crate::transcript::local_path("file:///D:/bad%E9"), None);
    assert_eq!(crate::transcript::local_path("file:///D:/bad%00"), None);
    assert_eq!(crate::transcript::local_path("file://../share/work"), None);
    assert_eq!(
        crate::transcript::local_path("file://server:80/share"),
        None
    );
    // 상대 경로는 기준을 알 수 없다. 틀린 프로젝트를 보여주느니 비운다.
    assert_eq!(crate::transcript::local_path("../work"), None);
    assert_eq!(crate::transcript::local_path(""), None);
}

#[test]
fn sqlite_row_becomes_a_session() {
    // 열 이름이 곧 필드다. 매핑 문법을 따로 두지 않는다.
    let db = std::path::PathBuf::from("state.vscdb");
    let row: crate::sqlite::Row = vec![
        ("id".into(), "abc".into()),
        ("updated_ms".into(), "1700000000000".into()),
        ("auxiliary".into(), "1".into()),
        (
            "blob".into(),
            r#"{"name":"제목","text":"최근 요청","fullConversationHeadersOnly":[{"grouping":{"textPreview":"첫 요청"}}]}"#
                .into(),
        ),
    ];
    let t = crate::transcript::row_transcript(&db, &row, 5).expect("행 하나가 세션 하나다");
    assert_eq!(t.session_id.as_deref(), Some("abc"));
    assert_eq!(t.current_prompt.as_deref(), Some("최근 요청"));
    // 제목보다 실제 첫 요청이 우선한다.
    assert_eq!(t.first_prompt.as_deref(), Some("첫 요청"));
    assert!(t.auxiliary);
    assert_eq!(t.last_event_at, 1700000000);
    assert!(!t.inferred_time);
    // DB 행에는 턴 경계가 없다. 상태를 지어내지 않는다.
    assert!(t.event_state.is_none());

    let mut metadata = row.clone();
    metadata
        .iter_mut()
        .find(|(key, _)| key == "updated_ms")
        .unwrap()
        .1 = "1700009999000".into();
    assert_eq!(
        crate::transcript::row_transcript(&db, &metadata, 6)
            .unwrap()
            .request_marker,
        t.request_marker
    );
    let repeated: crate::sqlite::Row = vec![
        ("id".into(), "abc".into()),
        ("task".into(), "repeat".into()),
        (
            "blob".into(),
            r#"{"messages":[{"role":"user","id":"one","content":"repeat","createdAt":1700000000000,"updatedAt":1700000001000}]}"#.into(),
        ),
    ];
    let first = crate::transcript::row_transcript(&db, &repeated, 7).unwrap();
    let mut metadata_again = repeated.clone();
    metadata_again
        .iter_mut()
        .find(|(key, _)| key == "blob")
        .unwrap()
        .1 = r#"{"model":"changed","messages":[{"role":"user","id":"one","content":"repeat","createdAt":1700000000000,"updatedAt":1700009999000},{"role":"assistant","content":"done"}]}"#.into();
    assert_eq!(
        crate::transcript::row_transcript(&db, &metadata_again, 8)
            .unwrap()
            .request_marker,
        first.request_marker
    );
    let mut repeated_again = repeated;
    repeated_again
        .iter_mut()
        .find(|(key, _)| key == "blob")
        .unwrap()
        .1 = r#"{"model":"changed","messages":[{"role":"user","id":"one","content":"repeat"},{"role":"assistant","content":"done"},{"role":"user","id":"two","content":"repeat"}]}"#.into();
    assert_ne!(
        crate::transcript::row_transcript(&db, &repeated_again, 8)
            .unwrap()
            .request_marker,
        first.request_marker
    );
    let title_only: crate::sqlite::Row = vec![
        ("id".into(), "renamed".into()),
        ("summary".into(), "session title".into()),
        ("updated_ms".into(), "1700010000000".into()),
    ];
    assert!(crate::transcript::row_transcript(&db, &title_only, 9)
        .unwrap()
        .request_marker
        .is_none());

    // 요청도 제목도 없는 빈 대화는 카드가 되지 않는다.
    let empty: crate::sqlite::Row = vec![("id".into(), "x".into()), ("blob".into(), "{}".into())];
    assert!(crate::transcript::row_transcript(&db, &empty, 5).is_none());
}

#[test]
fn sqlite_source_survives_a_changed_schema() {
    // 편집기가 스키마를 바꾸면 그 소스만 비어야 한다. 앱은 계속 돈다.
    if !crate::sqlite::available() {
        return;
    }
    let path = std::env::temp_dir().join(format!("waid-schema-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    crate::sqlite::exec(&path, "create table other (x text)").unwrap();
    assert!(crate::sqlite::query(&path, "select id from composerHeaders").is_err());
    // 같은 DB 의 다른 질의는 여전히 동작한다.
    assert!(crate::sqlite::query(&path, "select x as id from other").is_ok());
    let _ = std::fs::remove_file(&path);
}
