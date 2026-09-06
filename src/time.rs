//! 시간 유틸.
//!
//! chrono를 끌어오지 않는다. 필요한 건 두 가지뿐이다: 지금 몇 시인가,
//! 그리고 그걸 ISO 8601로 어떻게 쓰는가.
//!
//! 출력은 항상 UTC(`Z`)다. 로컬 타임존 변환은 libc의 tzset이나 tzdata 파싱을
//! 요구하는데, 둘 다 의존성 예산에 비해 값이 비싸다. 표에 보이는 시각은
//! 어차피 "3분 전" 같은 상대 표현이라 사용자가 UTC를 볼 일은 거의 없다.

use std::time::{SystemTime, UNIX_EPOCH};

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn mtime(path: &std::path::Path) -> Option<i64> {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

/// Howard Hinnant의 civil_from_days 알고리즘.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn to_iso8601(epoch: i64) -> String {
    let days = epoch.div_euclid(86_400);
    let secs = epoch.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m,
        d,
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// 사람이 읽는 상대 시각. 표에서 `since` 컬럼 대신 쓴다.
pub fn ago(seconds: i64) -> String {
    match seconds {
        s if s < 0 => "0s".into(),
        s if s < 60 => format!("{s}s"),
        s if s < 3600 => format!("{}m", s / 60),
        s if s < 86_400 => format!("{}h", s / 3600),
        s => format!("{}d", s / 86_400),
    }
}

/// Parse the timestamp envelope used by transcript events (RFC 3339).
/// Fractions are validated and truncated to the snapshot's second precision.
pub fn from_iso8601(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    let number = |from: usize, to: usize| -> Option<i64> {
        b.get(from..to)?.iter().try_fold(0i64, |n, c| {
            c.is_ascii_digit().then(|| n * 10 + (c - b'0') as i64)
        })
    };
    if b.len() < 20 || b[4] != b'-' || b[7] != b'-'
        || !matches!(b[10], b'T' | b't') || b[13] != b':' || b[16] != b':' { return None; }
    let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
    let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month { 2 => if leap { 29 } else { 28 }, 4 | 6 | 9 | 11 => 30, 1..=12 => 31, _ => return None };
    if !(1..=days).contains(&day) || hour > 23 || minute > 59 || second > 59 { return None; }
    let mut zone = 19;
    if b.get(zone) == Some(&b'.') {
        zone += 1;
        let start = zone;
        while b.get(zone).is_some_and(u8::is_ascii_digit) { zone += 1; }
        if zone == start { return None; }
    }
    let offset = match b.get(zone)? {
        b'Z' | b'z' if b.len() == zone + 1 => 0,
        sign @ (b'+' | b'-') if b.len() == zone + 6 && b[zone + 3] == b':' => {
            let (h, m) = (number(zone + 1, zone + 3)?, number(zone + 4, zone + 6)?);
            if h > 23 || m > 59 { return None; }
            (h * 3600 + m * 60) * if *sign == b'+' { 1 } else { -1 }
        }
        _ => return None,
    };
    // Inverse of civil_from_days, including the proleptic Gregorian leap-year rule.
    let y = year - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * shifted_month + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((era * 146097 + doe - 719468) * 86400 + hour * 3600 + minute * 60 + second - offset)
}

#[cfg(test)]
mod timestamp_tests {
    use super::*;
    #[test]
    fn parses_event_timestamps_without_timezone_drift() {
        assert_eq!(from_iso8601("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(from_iso8601("1970-01-01T09:00:00.123456+09:00"), Some(0));
        assert_eq!(from_iso8601("1969-12-31T19:00:00-05:00"), Some(0));
        for epoch in [-2203891200, -1, 0, 951782400, 1709164800, 1788660000, 4107542400] {
            assert_eq!(from_iso8601(&to_iso8601(epoch)), Some(epoch));
        }
        for invalid in ["", "2026-02-29T00:00:00Z", "1900-02-29T00:00:00Z",
            "2026-01-00T00:00:00Z", "2026-13-01T00:00:00Z", "2026-01-01T24:00:00Z",
            "2026-01-01T00:00:00.Z", "2026-01-01T00:00:00+24:00", "2026-01-01T00:00:00Zextra"] {
            assert_eq!(from_iso8601(invalid), None, "{invalid}");
        }
    }
}
