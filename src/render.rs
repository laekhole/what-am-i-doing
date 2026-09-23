//! 테이블 렌더러.
//!
//! 코어(§1.3)와 분리된 소비자 중 하나일 뿐이다. `session::Session`을 읽고
//! 문자열을 만든다. 그 반대 방향의 의존은 없다.

use crate::session::{Confidence, Session, State};
use crate::theme::{Column, Theme, Truncate};
use std::io::IsTerminal;

/// 터미널 표시 폭. 한글 한 글자는 두 칸을 먹는다.
///
/// 이걸 `str::len()`이나 `chars().count()`로 대충 하면 한국어 task가 들어간
/// 순간 표가 무너진다. 우리 사용자가 첫 화면에서 보게 될 버그다.
pub fn width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

fn char_width(c: char) -> usize {
    let c = c as u32;
    match c {
        // 제어 문자
        0..=0x1F | 0x7F => 0,
        // 결합 문자
        0x0300..=0x036F | 0x200B..=0x200F => 0,
        // 한글 자모
        0x1100..=0x115F => 2,
        // CJK 기호/부수, 한중일 통합 한자, 한글 음절
        0x2E80..=0x303E | 0x3041..=0x33FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF => 2,
        0xA000..=0xA4CF | 0xAC00..=0xD7A3 => 2,
        // 전각 형태, CJK 호환 한자
        0xF900..=0xFAFF | 0xFE30..=0xFE6F | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6 => 2,
        // 이모지
        0x1F300..=0x1F64F | 0x1F900..=0x1F9FF | 0x1FA70..=0x1FAFF => 2,
        _ => 1,
    }
}

fn take_width(s: &str, max: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = char_width(c);
        if w + cw > max {
            break;
        }
        out.push(c);
        w += cw;
    }
    out
}

fn take_width_rev(s: &str, max: usize) -> String {
    let mut buf: Vec<char> = Vec::new();
    let mut w = 0;
    for c in s.chars().rev() {
        let cw = char_width(c);
        if w + cw > max {
            break;
        }
        buf.push(c);
        w += cw;
    }
    buf.iter().rev().collect()
}

pub fn fit(s: &str, max: usize, mode: Truncate) -> String {
    // Logs and user themes must not inject terminal commands or split table rows.
    let s: String = s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    if max == 0 {
        return String::new();
    }
    if width(&s) <= max {
        return s;
    }
    if max <= 1 {
        return "…".into();
    }
    match mode {
        Truncate::End => format!("{}…", take_width(&s, max - 1)),
        Truncate::Start => format!("…{}", take_width_rev(&s, max - 1)),
        Truncate::Middle => {
            let left = (max - 1) / 2;
            let right = max - 1 - left;
            format!("{}…{}", take_width(&s, left), take_width_rev(&s, right))
        }
    }
}

fn pad(s: &str, to: usize) -> String {
    let w = width(s);
    if w >= to {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(to - w))
    }
}

// ------------------------------------------------------------ 색상

pub struct Style {
    pub color: bool,
}

impl Style {
    pub fn detect(force: Option<bool>) -> Self {
        if let Some(f) = force {
            return Style { color: f };
        }
        // NO_COLOR 관례를 존중한다. https://no-color.org
        if std::env::var_os("NO_COLOR").is_some() {
            return Style { color: false };
        }
        Style { color: std::io::stdout().is_terminal() }
    }

    fn hex(&self, hex: &str, s: &str) -> String {
        if !self.color {
            return s.to_string();
        }
        match rgb(hex) {
            Some((r, g, b)) => format!("\x1b[38;2;{r};{g};{b}m{s}\x1b[0m"),
            None => s.to_string(),
        }
    }

    fn dim(&self, s: &str) -> String {
        if self.color {
            format!("\x1b[2m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }

    fn bold(&self, s: &str) -> String {
        if self.color {
            format!("\x1b[1m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
}

fn rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 || !h.is_ascii() {
        return None;
    }
    Some((
        u8::from_str_radix(&h[0..2], 16).ok()?,
        u8::from_str_radix(&h[2..4], 16).ok()?,
        u8::from_str_radix(&h[4..6], 16).ok()?,
    ))
}

// ------------------------------------------------------------ 표

/// 셀 내용과, 색을 입히기 전의 순수 텍스트를 함께 들고 다닌다.
/// 정렬은 순수 텍스트 폭으로 계산하고 색은 마지막에 입힌다. 이 순서를
/// 뒤집으면 ANSI 시퀀스가 폭 계산에 섞여 들어가 표가 어긋난다.
struct Cell {
    plain: String,
    styled: String,
}

fn cell(s: &Session, col: &Column, theme: &Theme, st: &Style) -> Cell {
    match col.key.as_str() {
        "status" => {
            let sym = theme.symbols.get(s.state.id()).cloned().unwrap_or_else(|| "•".into());
            let plain = format!("{} {}", sym, s.state.id());
            let plain = fit(&plain, col.width, col.truncate);
            let styled = match theme.colors.get(s.state.id()) {
                Some(c) => st.hex(c, &plain),
                None => plain.clone(),
            };
            Cell { plain, styled }
        }
        "title" => {
            let plain = fit(&s.title, col.width, col.truncate);
            let styled = if s.state == State::Waiting { st.bold(&plain) } else { plain.clone() };
            Cell { plain, styled }
        }
        "agent" => {
            let plain = fit(s.agent.display, col.width, col.truncate);
            Cell { styled: plain.clone(), plain }
        }
        "llm" => {
            let raw = s.llm_display.clone().unwrap_or_else(|| "—".into());
            let plain = fit(&raw, col.width, col.truncate);
            let styled =
                if s.llm_display.is_none() { st.dim(&plain) } else { plain.clone() };
            Cell { plain, styled }
        }
        "task" => {
            let raw = s.task.text.clone().unwrap_or_else(|| "—".into());
            let plain = fit(&raw, col.width, col.truncate);
            // 추론한 값을 확실한 값처럼 보이게 만들지 않는다(매니페스트 §3.1).
            let inferred = s.task.confidence != Confidence::Explicit;
            let styled = if theme.dim_when_inferred && inferred {
                st.dim(&plain)
            } else {
                plain.clone()
            };
            Cell { plain, styled }
        }
        _ => Cell { plain: String::new(), styled: String::new() },
    }
}

pub fn table(sessions: &[Session], theme: &Theme, st: &Style) -> String {
    if sessions.is_empty() {
        return crate::i18n::tr("실행 중인 코딩 에이전트가 없습니다.\n", "No coding agents are running.\n").to_string();
    }

    let mut ordered: Vec<&Session> = sessions.iter().collect();
    ordered.sort_by_key(|s| theme.status_priority.iter()
        .position(|state| state == s.state.id()).unwrap_or(usize::MAX));
    let rows: Vec<Vec<Cell>> = ordered
        .iter()
        .map(|s| theme.columns.iter().map(|c| cell(s, c, theme, st)).collect())
        .collect();

    // 실제 내용에 맞춰 컬럼을 좁힌다. 설정한 너비는 상한이지 고정값이 아니다.
    let widths: Vec<usize> = theme
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let content = rows.iter().map(|r| width(&r[i].plain)).max().unwrap_or(0);
            content.max(width(&header(&c.key))).min(c.width)
        })
        .collect();

    let mut out = String::new();
    let head: Vec<String> = theme
        .columns
        .iter()
        .zip(&widths)
        .map(|(c, w)| pad(&fit(&header(&c.key), *w, c.truncate), *w))
        .collect();
    out.push_str(&st.dim(head.join("  ").trim_end()));
    out.push('\n');

    for row in &rows {
        let line: Vec<String> = row
            .iter()
            .zip(&widths)
            .map(|(c, w)| {
                let padding = w.saturating_sub(width(&c.plain));
                format!("{}{}", c.styled, " ".repeat(padding))
            })
            .collect();
        out.push_str(line.join("  ").trim_end());
        out.push('\n');
    }
    out
}

fn header(key: &str) -> String {
    match key {
        "status" => "STATUS",
        "title" => "TITLE",
        "agent" => "AGENT",
        "llm" => "LLM",
        "task" => "TASK",
        other => other,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_colors_and_log_controls_are_safe() {
        let style = Style { color: true };
        for invalid in ["#a가bc", "#가나다", "#12345g", ""] {
            assert_eq!(style.hex(invalid, "status"), "status");
        }
        assert_eq!(rgb("#f5a623"), Some((245, 166, 35)));
        for mode in [Truncate::Start, Truncate::Middle, Truncate::End] {
            for max in 0..30 {
                let fitted = fit("로그\n\r\t\x1b]0;changed\x07\u{009b}31m", max, mode);
                assert!(width(&fitted) <= max);
                assert!(!fitted.chars().any(char::is_control));
            }
        }
    }
}
