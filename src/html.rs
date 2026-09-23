//! HTML 렌더러 (매니페스트 §6 레이어 2).
//!
//! 코어는 스냅샷을 평면 키/값으로 펼쳐 템플릿에 넘기는 데서 임무를 끝낸다.
//! 그 다음의 모든 것 — 레이아웃, 색, 타이포, 애니메이션 — 은 템플릿 작성자
//! 소유다. waid는 스타일시트를 강제하지 않는다.
//!
//! 기본 템플릿은 "설정 없이 동작한다"(§1.4)를 위한 것일 뿐 규범이 아니다.
//! `waid --html --eject > mine.html` 로 통째로 꺼내 자기 것으로 만들면 된다.

use crate::session::{Confidence, Session, State};
use crate::i18n::{language, Language};
use crate::time;
use crate::tmpl::{self, b, s, Ctx, Val};

pub const DEFAULT_TEMPLATE: &str = include_str!("default.html");

fn state_label(st: State, language: Language) -> &'static str {
    let (ko, en) = match st {
        State::Waiting => ("내 차례", "Waiting"),
        State::Working => ("작업 중", "Working"),
        State::Error => ("오류", "Error"),
        State::Idle => ("유휴", "Idle"),
        State::Done => ("완료", "Done"),
        State::Unknown => ("미확인", "Unknown"),
    };
    language.text(ko, en)
}

fn state_symbol(st: State) -> &'static str {
    match st {
        State::Waiting => "◐",
        State::Working => "●",
        State::Error => "✗",
        State::Idle => "○",
        State::Done => "✓",
        State::Unknown => "?",
    }
}

fn session_ctx(x: &Session, now: i64, language: Language) -> Ctx {
    let mut c = Ctx::new();
    c.insert("id".into(), s(&x.id));
    c.insert("title".into(), s(&x.title));

    c.insert("agent.name".into(), s(x.agent.name));
    c.insert("agent.display".into(), s(x.agent.display));

    let llm = x.llm_display.clone().or_else(|| x.llm_id.clone());
    c.insert("llm.present".into(), b(llm.is_some()));
    c.insert("llm.display".into(), s(llm.unwrap_or_default()));

    let task = x.task.text.clone().unwrap_or_default();
    c.insert("task.present".into(), b(!task.is_empty()));
    c.insert("task.text".into(), s(task));
    c.insert("task.source".into(), s(x.task.source));
    c.insert(
        "task.inferred".into(),
        b(x.task.confidence == Confidence::Inferred),
    );
    c.insert(
        "task.explicit".into(),
        b(x.task.confidence == Confidence::Explicit),
    );

    c.insert("status.state".into(), s(x.state.id()));
    c.insert("status.label".into(), s(state_label(x.state, language)));
    c.insert("status.symbol".into(), s(state_symbol(x.state)));
    c.insert(
        "status.since".into(),
        s(if x.since > 0 {
            time::ago(now - x.since)
        } else {
            String::new()
        }),
    );
    // 상태별 불리언. `{{#if status.waiting}}` 형태로 분기하기 위한 것.
    for st in [
        State::Waiting,
        State::Working,
        State::Error,
        State::Idle,
        State::Done,
        State::Unknown,
    ] {
        c.insert(format!("status.{}", st.id()), b(x.state == st));
    }

    c.insert(
        "cwd".into(),
        s(x.cwd
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()),
    );
    let branch = x.branch.clone().unwrap_or_default();
    c.insert("branch.present".into(), b(!branch.is_empty()));
    c.insert("branch".into(), s(branch));
    c.insert(
        "pid".into(),
        s(x.pid.map(|p| p.to_string()).unwrap_or_default()),
    );
    c.insert("alive".into(), b(x.pid.is_some()));
    c
}

pub fn context(sessions: &[Session], now: i64) -> Ctx {
    context_language(sessions, now, language())
}

/// Derive --keys from the renderer without collecting the user's sessions.
pub(crate) fn key_context() -> Ctx {
    let session = Session {
        context: crate::ContextUsage::default(),
        prompt: None,
        last_answer: None,
        session_id: None,
        summary: None,
        request_marker: None,
        request_at: None,
        auxiliary: false,
        evidence: "unknown",
        id: String::new(),
        legacy_id: String::new(),
        title: String::new(),
        agent: crate::adapters::Agent { name: "", display: "", has_reader: false },
        llm_id: None,
        llm_display: None,
        task: crate::session::Task { text: None, source: "none", confidence: Confidence::None },
        state: State::Unknown,
        since: 0,
        cwd: None,
        branch: None,
        pid: None,
    };
    context(&[session], 0)
}

fn context_language(sessions: &[Session], now: i64, language: Language) -> Ctx {
    let mut root = Ctx::new();
    let items: Vec<Ctx> = sessions.iter().map(|x| session_ctx(x, now, language)).collect();

    root.insert("language".into(), s(language.code()));
    root.insert("language.ko".into(), b(language == Language::Korean));
    root.insert("language.en".into(), b(language == Language::English));
    for (key, ko, en) in [
        ("sessions", "개 세션", "sessions"),
        ("waiting", "내 차례", "Waiting"),
        ("working", "작업 중", "Working"),
        ("idle", "유휴", "Idle"),
        ("error", "오류", "Error"),
        ("no_task", "— 지시 내용 없음", "— No request recorded"),
        ("empty", "돌고 있는 코딩 에이전트가 없습니다.", "No coding agents are running."),
        ("doctor", "감지 경로를 확인하려면", "Check collection paths with"),
        ("live", "◉ 실시간", "◉ live"),
        ("stopped", "○ 정지", "○ disconnected"),
        ("apply", "적용", "Apply"),
    ] {
        root.insert(format!("ui.{key}"), s(language.text(ko, en)));
    }

    let count = |st: State| sessions.iter().filter(|x| x.state == st).count();
    for st in [
        State::Waiting,
        State::Working,
        State::Error,
        State::Idle,
        State::Done,
        State::Unknown,
    ] {
        root.insert(format!("count.{}", st.id()), s(count(st).to_string()));
        root.insert(format!("any.{}", st.id()), b(count(st) > 0));
    }

    root.insert("count".into(), s(sessions.len().to_string()));
    root.insert("any".into(), b(!sessions.is_empty()));
    root.insert("generated_at".into(), s(time::to_iso8601(now)));
    root.insert("version".into(), s(env!("CARGO_PKG_VERSION")));
    root.insert("sessions".into(), Val::List(items));
    root
}

/// 템플릿 파일을 읽는다. 없으면 내장 기본값.
pub fn template(path: Option<&str>) -> Result<String, String> {
    match path {
        None => Ok(DEFAULT_TEMPLATE.to_string()),
        Some(p) => std::fs::read_to_string(p)
            .map_err(|e| crate::trf!("템플릿 '{p}' 을 읽을 수 없습니다: {e}", "Cannot read template '{p}': {e}")),
    }
}

pub fn render(sessions: &[Session], now: i64, tpl: &str) -> String {
    render_language(sessions, now, tpl, language())
}

pub fn render_language(sessions: &[Session], now: i64, tpl: &str, language: Language) -> String {
    tmpl::render(tpl, &context_language(sessions, now, language))
}
