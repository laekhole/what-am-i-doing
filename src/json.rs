//! 최소 JSON 구현.
//!
//! 외부 크레이트를 쓰지 않기로 했으므로(매니페스트 §5) 직접 만든다.
//! 요구사항이 좁다는 점을 이용한다: 트랜스크립트 한 줄을 읽고, 스냅샷 한 벌을
//! 쓴다. 그 이상은 하지 않는다.

use std::collections::BTreeMap;
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(m) => m.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(v) => Some(v),
            _ => None,
        }
    }

    /// 키 하나를 깊이 우선으로 재귀 탐색해 첫 문자열 값을 찾는다.
    ///
    /// 이게 방어적으로 보이는 데는 이유가 있다. 에이전트 CLI들의 트랜스크립트
    /// 스키마는 문서화되어 있지 않고 릴리스마다 조용히 바뀐다. `model`이
    /// 최상위에 있을 때도, `message.model`일 때도, `payload.info.model`일
    /// 때도 있다. 경로를 고정하면 다음 마이너 업데이트에서 깨진다.
    pub fn find_str(&self, keys: &[&str]) -> Option<String> {
        match self {
            Json::Obj(m) => {
                for k in keys {
                    if let Some(Json::Str(s)) = m.get(*k) {
                        if !s.is_empty() {
                            return Some(s.clone());
                        }
                    }
                }
                for v in m.values() {
                    if let Some(found) = v.find_str(keys) {
                        return Some(found);
                    }
                }
                None
            }
            Json::Arr(a) => a.iter().find_map(|v| v.find_str(keys)),
            _ => None,
        }
    }

    /// 이 값(또는 그 하위)이 주어진 키에 대해 특정 문자열 값을 갖는지.
    pub fn has_kv(&self, key: &str, val: &str) -> bool {
        match self {
            Json::Obj(m) => {
                if let Some(Json::Str(s)) = m.get(key) {
                    if s == val {
                        return true;
                    }
                }
                m.values().any(|v| v.has_kv(key, val))
            }
            Json::Arr(a) => a.iter().any(|v| v.has_kv(key, val)),
            _ => false,
        }
    }

    /// 사람이 읽을 수 있는 텍스트를 뽑는다.
    ///
    /// content는 평문 문자열일 수도, `[{type:"text", text:"..."}]` 배열일
    /// 수도 있다. 후자에서 이미지/툴 블록은 건너뛴다.
    pub fn text_content(&self) -> Option<String> {
        match self {
            Json::Str(s) => Some(s.clone()),
            Json::Arr(a) => {
                let parts: Vec<String> = a
                    .iter()
                    .filter_map(|b| match b {
                        Json::Str(s) => Some(s.clone()),
                        Json::Obj(m) => match m.get("type").and_then(|t| t.as_str()) {
                            Some("text" | "input_text" | "output_text") | None => {
                                m.get("text").and_then(|t| t.as_str()).map(str::to_string)
                            }
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(" "))
                }
            }
            Json::Obj(m) => m.get("text").and_then(|t| t.as_str()).map(str::to_string),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------- 파서

pub fn parse(input: &str) -> Result<Json, String> {
    let b = input.as_bytes();
    let mut p = Parser { b, i: 0 };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.i != b.len() {
        return Err(format!("trailing input at byte {}", p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn eat(&mut self, c: u8) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected {:?} at byte {}", c as char, self.i))
        }
    }

    fn lit(&mut self, s: &str, v: Json) -> Result<Json, String> {
        if self.b[self.i..].starts_with(s.as_bytes()) {
            self.i += s.len();
            Ok(v)
        } else {
            Err(format!("bad literal at byte {}", self.i))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b't') => self.lit("true", Json::Bool(true)),
            Some(b'f') => self.lit("false", Json::Bool(false)),
            Some(b'n') => self.lit("null", Json::Null),
            Some(_) => self.number(),
            None => Err("unexpected end of input".into()),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.eat(b'{')?;
        let mut m = BTreeMap::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Json::Obj(m));
        }
        loop {
            self.ws();
            let k = self.string()?;
            self.ws();
            self.eat(b':')?;
            self.ws();
            let v = self.value()?;
            m.insert(k, v);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Obj(m));
                }
                _ => return Err(format!("bad object at byte {}", self.i)),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.eat(b'[')?;
        let mut a = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Json::Arr(a));
        }
        loop {
            self.ws();
            a.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::Arr(a));
                }
                _ => return Err(format!("bad array at byte {}", self.i)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let mut s = String::new();
        loop {
            let c = self.peek().ok_or("unterminated string")?;
            self.i += 1;
            match c {
                b'"' => return Ok(s),
                b'\\' => {
                    let e = self.peek().ok_or("unterminated escape")?;
                    self.i += 1;
                    match e {
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        b'/' => s.push('/'),
                        b'b' => s.push('\u{8}'),
                        b'f' => s.push('\u{c}'),
                        b'n' => s.push('\n'),
                        b'r' => s.push('\r'),
                        b't' => s.push('\t'),
                        b'u' => s.push(self.unicode_escape()?),
                        _ => return Err(format!("bad escape at byte {}", self.i)),
                    }
                }
                // UTF-8 연속 바이트를 문자 경계까지 모아서 넣는다.
                _ => {
                    let start = self.i - 1;
                    while self.i < self.b.len() && (self.b[self.i] & 0xC0) == 0x80 {
                        self.i += 1;
                    }
                    match std::str::from_utf8(&self.b[start..self.i]) {
                        Ok(part) => s.push_str(part),
                        Err(_) => s.push('\u{fffd}'),
                    }
                }
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        if self.i + 4 > self.b.len() {
            return Err("truncated \\u escape".into());
        }
        let hex = std::str::from_utf8(&self.b[self.i..self.i + 4]).map_err(|_| "bad \\u")?;
        let n = u32::from_str_radix(hex, 16).map_err(|_| "bad \\u")?;
        self.i += 4;
        Ok(n)
    }

    fn unicode_escape(&mut self) -> Result<char, String> {
        let n = self.hex4()?;
        // 서로게이트 페어. 한국어 텍스트에서는 드물지만, 이모지가 섞인
        // 프롬프트에서는 실제로 나온다.
        if (0xD800..0xDC00).contains(&n) {
            if self.peek() == Some(b'\\') && self.b.get(self.i + 1) == Some(&b'u') {
                self.i += 2;
                let lo = self.hex4()?;
                if (0xDC00..0xE000).contains(&lo) {
                    let c = 0x10000 + ((n - 0xD800) << 10) + (lo - 0xDC00);
                    return char::from_u32(c).ok_or_else(|| "bad surrogate pair".into());
                }
            }
            return Ok('\u{fffd}');
        }
        char::from_u32(n).ok_or_else(|| "bad code point".into())
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || matches!(c, b'.' | b'e' | b'E' | b'+' | b'-') {
                self.i += 1;
            } else {
                break;
            }
        }
        std::str::from_utf8(&self.b[start..self.i])
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .map(Json::Num)
            .ok_or_else(|| format!("bad number at byte {}", start))
    }
}

// ------------------------------------------------------------ 직렬화

pub fn escape(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// 출력 쪽은 트리를 만들지 않고 문자열을 직접 쌓는다.
/// 스냅샷 구조가 고정되어 있으니 중간 표현이 필요 없다.
pub struct Writer {
    pub buf: String,
    stack: Vec<bool>, // 각 깊이에서 원소를 이미 하나 썼는지
    pretty: bool,
}

impl Writer {
    pub fn new(pretty: bool) -> Self {
        Writer { buf: String::new(), stack: Vec::new(), pretty }
    }

    fn sep(&mut self) {
        if let Some(written) = self.stack.last_mut() {
            if *written {
                self.buf.push(',');
            }
            *written = true;
        }
        if self.pretty && !self.stack.is_empty() {
            self.buf.push('\n');
            for _ in 0..self.stack.len() {
                self.buf.push_str("  ");
            }
        }
    }

    fn close(&mut self, c: char) {
        let had = self.stack.pop().unwrap_or(false);
        if self.pretty && had {
            self.buf.push('\n');
            for _ in 0..self.stack.len() {
                self.buf.push_str("  ");
            }
        }
        self.buf.push(c);
    }

    pub fn begin_obj(&mut self) {
        self.sep();
        self.buf.push('{');
        self.stack.push(false);
    }

    pub fn end_obj(&mut self) {
        self.close('}');
    }

    pub fn begin_arr(&mut self) {
        self.sep();
        self.buf.push('[');
        self.stack.push(false);
    }

    pub fn end_arr(&mut self) {
        self.close(']');
    }

    fn key(&mut self, k: &str) {
        self.sep();
        escape(k, &mut self.buf);
        self.buf.push(':');
        if self.pretty {
            self.buf.push(' ');
        }
    }

    pub fn field_obj(&mut self, k: &str) {
        self.key(k);
        self.buf.push('{');
        self.stack.push(false);
    }

    pub fn field_arr(&mut self, k: &str) {
        self.key(k);
        self.buf.push('[');
        self.stack.push(false);
    }

    pub fn field_str(&mut self, k: &str, v: &str) {
        self.key(k);
        escape(v, &mut self.buf);
    }

    pub fn field_opt_str(&mut self, k: &str, v: Option<&str>) {
        match v {
            Some(s) => self.field_str(k, s),
            None => {
                self.key(k);
                self.buf.push_str("null");
            }
        }
    }

    pub fn field_num(&mut self, k: &str, v: i64) {
        self.key(k);
        let _ = write!(self.buf, "{}", v);
    }

    pub fn field_bool(&mut self, k: &str, v: bool) {
        self.key(k);
        self.buf.push_str(if v { "true" } else { "false" });
    }
}
