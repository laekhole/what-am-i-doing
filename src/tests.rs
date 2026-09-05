//! 테스트.
//!
//! 여기서 검증하려는 것은 대체로 "실제로 만들면 반드시 밟는 함정" 목록이다.
//! 한글 폭 계산, 색상값 안의 `#`, 하네스가 주입한 가짜 첫 프롬프트,
//! `vim claude.md` 오탐 같은 것들.

use crate::json::{self, Json};
use crate::matchers;
use crate::proc::Process;
use crate::render::{self, width};
use crate::theme::{self, Truncate, Val};
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
    assert_eq!(v.get("a").unwrap().get("b").unwrap().as_array().unwrap().len(), 4);
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
    let v = json::parse(
        r#"{"content":[{"type":"image"},{"type":"text","text":"인증 리팩터링"}]}"#,
    )
    .unwrap();
    assert_eq!(v.get("content").unwrap().text_content().unwrap(), "인증 리팩터링");
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
    assert_eq!(matchers::identify(&p(&["/usr/local/bin/codex", "--yolo"])).unwrap().name, "codex");
}

#[test]
fn detects_windows_shims() {
    assert_eq!(matchers::identify(&p(&["C:\\npm\\claude.cmd"])).unwrap().name, "claude");
    assert_eq!(matchers::identify(&p(&["claude.exe"])).unwrap().name, "claude");
}

#[test]
fn detects_through_launcher() {
    let proc = p(&["/usr/bin/node", "/home/mk/.nvm/versions/node/v22/bin/claude", "--resume"]);
    assert_eq!(matchers::identify(&proc).unwrap().name, "claude");
}

#[test]
fn detects_via_package_marker() {
    // 파일명이 cli.js라서 basename으로는 안 잡히는 경우.
    let proc = p(&["node", "/usr/lib/node_modules/@anthropic-ai/claude-code/cli.js"]);
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
    for s in ["api-server", "인증 리팩터링 작업", "a", "가나다라마바사아자차", "🚀 배포"] {
        for max in 1..20 {
            let out = render::fit(s, max, Truncate::End);
            assert!(width(&out) <= max, "fit({s:?},{max}) = {out:?} 폭 {}", width(&out));
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
    assert_eq!(normalize_model("gemini-3-pro"), "gemini-3.pro".replace("3.pro", "3-pro"));
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
    assert_eq!(kv.get("columns.order"), Some(&Val::List(vec!["status".into(), "title".into()])));
    assert_eq!(kv.get("columns.title.width"), Some(&Val::Int(30)));
    assert_eq!(kv.get("columns.task.dim_when_inferred"), Some(&Val::Bool(false)));
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
    let mut states = vec![State::Done, State::Idle, State::Working, State::Waiting, State::Error];
    states.sort_by_key(|s| s.rank());
    assert_eq!(states[0], State::Waiting);
    assert_eq!(states[1], State::Working);
}

// ------------------------------------------------------- 템플릿 엔진 (§6 레이어 2)

use crate::tmpl::{self, b, s, Ctx, Val as TVal};

fn ctx(pairs: &[(&str, TVal)]) -> Ctx {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
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
        TVal::List(vec![ctx(&[("title", s("api"))]), ctx(&[("title", s("web"))])]),
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
fn default_template_renders_without_leftover_tags() {
    // 기본 템플릿과 컨텍스트가 어긋나면 화면에 {{...}} 가 그대로 남는다.
    let out = crate::html::render(&[], crate::time::now(), crate::html::DEFAULT_TEMPLATE);
    assert!(!out.contains("{{"), "미치환 태그 잔존: {out}");
    assert!(out.contains("data-waid-root"), "갱신 훅이 사라졌다");
    assert!(out.contains("돌고 있는 코딩 에이전트가 없습니다"), "빈 상태가 안 나온다");
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
    let home = std::env::var("HOME").expect("HOME");
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
    let expected = format!("{home}/.mytool/sessions");
    assert!(
        d.transcript_dirs.iter().any(|p| p.to_string_lossy() == expected),
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
    assert!(adapters::parse_def_for_test("[adapter]\nname = \"my tool\"\nexec = [\"x\"]\n").is_err());
}

#[test]
fn adapter_markers_alone_are_enough() {
    // exec 없이 패키지 표지만으로도 성립한다 (`node .../cli.js` 형태).
    let d =
        adapters::parse_def_for_test("[adapter]\nname = \"x\"\nmarkers = [\"@vendor/x\"]\n").unwrap();
    assert!(d.exec.is_empty());
    assert_eq!(d.markers, vec!["@vendor/x"]);
}

#[test]
fn builtin_table_is_intact_after_refactor() {
    let names: Vec<&str> = adapters::all().iter().map(|a| a.name).collect();
    for expected in ["claude", "codex", "gemini", "opencode", "aider", "cursor", "copilot", "goose"] {
        assert!(names.contains(&expected), "{expected} 가 사라졌다");
    }
    // 트랜스크립트 리더는 claude/codex 만.
    assert!(adapters::by_name("claude").unwrap().has_reader);
    assert!(!adapters::by_name("aider").unwrap().has_reader);
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
    let prefixes: std::collections::HashSet<&str> =
        ids.iter().map(|i| &i[..4]).collect();
    assert_eq!(prefixes.len(), ids.len(), "앞 4자리가 충돌한다: {ids:?}");
}

#[test]
fn dedupe_extends_suffix_until_titles_are_unique() {
    use crate::session::{dedupe_titles, Confidence, State, Task};
    use crate::adapters::Agent;

    let agent = Agent { name: "claude", display: "Claude Code", has_reader: true };
    let mk = |id: &str| crate::session::Session {
        id: id.to_string(),
        title: "api".into(),
        agent,
        llm_id: None,
        llm_display: None,
        task: Task { text: None, source: "none", confidence: Confidence::None },
        state: State::Idle,
        since: 0,
        cwd: None,
        branch: None,
        pid: None,
    };

    // 앞 4자리가 일부러 겹치는 id 들.
    let mut sessions = vec![mk("aaaa0001"), mk("aaaa0002"), mk("bbbb0003")];
    dedupe_titles(&mut sessions);

    let titles: std::collections::HashSet<&String> =
        sessions.iter().map(|s| &s.title).collect();
    assert_eq!(titles.len(), 3, "구분되지 않았다: {:?}", 
        sessions.iter().map(|s| &s.title).collect::<Vec<_>>());
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
    let home = std::env::var("HOME").expect("HOME");
    let roots = adapters::transcript_dirs("claude");
    let want = format!("{home}/.claude/projects");
    assert!(
        roots.iter().any(|p| p.to_string_lossy() == want),
        "내 홈 루트가 빠졌다: {roots:?}"
    );
}

#[test]
fn agent_without_transcript_root_has_no_reader() {
    let roots = adapters::transcript_dirs("aider");
    assert!(roots.is_empty());
    assert!(!adapters::by_name("aider").unwrap().has_reader);
    // 없는 에이전트를 물어도 패닉하지 않는다.
    assert!(adapters::transcript_dirs("nope").is_empty());
}
