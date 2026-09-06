//! HTML 렌더러 (매니페스트 §6 레이어 2).
//!
//! 코어는 스냅샷을 평면 키/값으로 펼쳐 템플릿에 넘기는 데서 임무를 끝낸다.
//! 그 다음의 모든 것 — 레이아웃, 색, 타이포, 애니메이션 — 은 템플릿 작성자
//! 소유다. waid는 스타일시트를 강제하지 않는다.
//!
//! 기본 템플릿은 "설정 없이 동작한다"(§1.4)를 위한 것일 뿐 규범이 아니다.
//! `waid --html --eject > mine.html` 로 통째로 꺼내 자기 것으로 만들면 된다.

use crate::session::{Confidence, Session, State};
use crate::time;
use crate::tmpl::{self, b, s, Ctx, Val};

pub const DEFAULT_TEMPLATE: &str = include_str!("default.html");

fn state_label(st: State) -> &'static str {
    match st {
        State::Waiting => "내 차례",
        State::Working => "작업 중",
        State::Error => "오류",
        State::Idle => "유휴",
        State::Done => "완료",
        State::Unknown => "미확인",
    }
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

fn session_ctx(x: &Session, now: i64) -> Ctx {
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
    c.insert("status.label".into(), s(state_label(x.state)));
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
    let mut root = Ctx::new();
    let items: Vec<Ctx> = sessions.iter().map(|x| session_ctx(x, now)).collect();

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
            .map_err(|e| format!("템플릿 '{p}' 을 읽을 수 없습니다: {e}")),
    }
}

pub fn render(sessions: &[Session], now: i64, tpl: &str) -> String {
    tmpl::render(tpl, &context(sessions, now))
}
