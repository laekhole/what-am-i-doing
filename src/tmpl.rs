//! 초소형 템플릿 엔진 (매니페스트 §6 레이어 2).
//!
//! 지원하는 문법은 넷뿐이다:
//!
//! ```text
//!   {{key}}                     값 치환 (항상 HTML 이스케이프)
//!   {{#each list}} … {{/each}}  반복
//!   {{#if key}} … {{/if}}       조건 (…{{else}}… 가능)
//!   {{#unless key}} … {{/unless}}
//! ```
//!
//! 표현식도, 필터도, 부분 템플릿도, 헬퍼 등록도 없다. 그게 부족하다고
//! 느껴지는 순간이 곧 레이어 3(`waid --json | 내 프로그램`)으로 넘어갈
//! 신호다. 이 엔진을 키우는 방향으로는 가지 않는다.
//!
//! 키는 점을 포함한 평면 문자열이다. `{{task.text}}`는 중첩 조회가 아니라
//! `"task.text"`라는 이름의 키를 찾는다. 조회 규칙이 한 줄로 끝나고,
//! 템플릿 작성자 입장에서는 차이가 보이지 않는다.

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum Val {
    Str(String),
    Bool(bool),
    List(Vec<Ctx>),
}

pub type Ctx = BTreeMap<String, Val>;

/// 빈 문자열/false/빈 목록은 거짓. `{{#if}}`의 판정 기준.
fn truthy(v: &Val) -> bool {
    match v {
        Val::Str(s) => !s.is_empty(),
        Val::Bool(b) => *b,
        Val::List(l) => !l.is_empty(),
    }
}

fn escape(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
}

/// 안쪽 스코프부터 바깥으로 훑는다. `{{#each}}` 안에서 바깥 변수도 보인다.
fn lookup<'a>(stack: &[&'a Ctx], key: &str) -> Option<&'a Val> {
    stack.iter().rev().find_map(|c| c.get(key))
}

/// `{{#kw …}}` 다음(`start`)부터 짝이 맞는 `{{/kw}}`까지를 잘라낸다.
/// 반환: (본문, else 절, 닫는 태그 다음 인덱스).
fn scan_block<'s>(src: &'s str, start: usize, kw: &str) -> (&'s str, Option<&'s str>, usize) {
    let mut blocks = vec![kw];
    let mut i = start;
    let mut alt: Option<(usize, usize)> = None;

    while let Some(rel) = src[i..].find("{{") {
        let s = i + rel;
        let c = match src[s + 2..].find("}}") {
            Some(c) => s + 2 + c,
            None => break,
        };
        let tag = src[s + 2..c].trim();
        let after = c + 2;

        let opening = tag.strip_prefix('#').and_then(|t| t.split_whitespace().next());
        if let Some(name @ ("if" | "unless" | "each")) = opening {
            blocks.push(name);
        } else if tag.strip_prefix('/') == blocks.last().copied() {
            blocks.pop();
            if blocks.is_empty() {
                return match alt {
                    Some((body_end, alt_start)) => {
                        (&src[start..body_end], Some(&src[alt_start..s]), after)
                    }
                    None => (&src[start..s], None, after),
                };
            }
        } else if tag == "else" && blocks.len() == 1 && alt.is_none() {
            alt = Some((s, after));
        }
        i = after;
    }

    // 닫는 태그가 없다. 남은 전부를 본문으로 보고 조용히 넘어간다 —
    // 템플릿 오타 하나로 대시보드가 통째로 죽는 것보다 낫다.
    (&src[start..], None, src.len())
}

pub fn render(src: &str, root: &Ctx) -> String {
    let mut out = String::with_capacity(src.len() * 2);
    render_into(src, &[root], &mut out);
    out
}

fn render_into<'a>(src: &str, stack: &[&'a Ctx], out: &mut String) {
    let mut i = 0usize;

    while let Some(rel) = src[i..].find("{{") {
        let s = i + rel;
        out.push_str(&src[i..s]);

        let c = match src[s + 2..].find("}}") {
            Some(c) => s + 2 + c,
            None => {
                // 열린 채로 끝났다. 리터럴로 취급.
                out.push_str(&src[s..]);
                return;
            }
        };
        let tag = src[s + 2..c].trim();
        let after = c + 2;

        if let Some(name) = tag.strip_prefix("#each ") {
            let (body, _, next) = scan_block(src, after, "each");
            if let Some(Val::List(items)) = lookup(stack, name.trim()) {
                for item in items {
                    let mut inner: Vec<&'a Ctx> = stack.to_vec();
                    inner.push(item);
                    render_into(body, &inner, out);
                }
            }
            i = next;
        } else if let Some(name) = tag.strip_prefix("#if ") {
            let (body, alt, next) = scan_block(src, after, "if");
            let yes = lookup(stack, name.trim()).map(truthy).unwrap_or(false);
            match (yes, alt) {
                (true, _) => render_into(body, stack, out),
                (false, Some(a)) => render_into(a, stack, out),
                (false, None) => {}
            }
            i = next;
        } else if let Some(name) = tag.strip_prefix("#unless ") {
            let (body, alt, next) = scan_block(src, after, "unless");
            let yes = lookup(stack, name.trim()).map(truthy).unwrap_or(false);
            match (yes, alt) {
                (false, _) => render_into(body, stack, out),
                (true, Some(a)) => render_into(a, stack, out),
                (true, None) => {}
            }
            i = next;
        } else if tag.starts_with('/') || tag == "else" {
            // 짝 없는 닫는 태그. 무시.
            i = after;
        } else {
            match lookup(stack, tag) {
                Some(Val::Str(v)) => escape(v, out),
                Some(Val::Bool(b)) => out.push_str(if *b { "true" } else { "false" }),
                Some(Val::List(l)) => out.push_str(&l.len().to_string()),
                // 미정의 키는 조용히 빈 문자열. 템플릿을 고치는 동안에도
                // 페이지는 계속 뜬다.
                None => {}
            }
            i = after;
        }
    }
    out.push_str(&src[i..]);
}

// ------------------------------------------------------------ 편의 생성자

pub fn s(v: impl Into<String>) -> Val {
    Val::Str(v.into())
}

pub fn b(v: bool) -> Val {
    Val::Bool(v)
}
