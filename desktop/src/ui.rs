//! Presentation and user preferences. Never writes agent data.
use crate::Row;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

pub const STATES: &[&str] = &["waiting", "working", "error", "idle", "done", "unknown"];
pub const DEFAULT: &str = include_str!("../templates/daylight.json");
pub const NIGHT: &str = include_str!("../templates/midnight.json");
pub const MAX_CONFIG: u64 = 128 * 1024;
const MAX_SETTINGS: u64 = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct Skin {
    pub compact: bool,
    pub name: String,
    pub background: u32,
    pub surface: u32,
    pub foreground: u32,
    pub muted: u32,
    pub selection: u32,
    pub accent: u32,
    pub font_size: i32,
    pub padding: i32,
    pub line_gap: i32,
    pub state_colors: BTreeMap<String, u32>,
    pub fields: Vec<String>,
    pub icons: BTreeMap<String, String>,
}

pub fn color(text: &str) -> Result<u32, String> {
    if text.len() != 7 || !text.starts_with('#') {
        return Err("색상은 #RRGGBB 형식이어야 합니다.".into());
    }
    let n = u32::from_str_radix(&text[1..], 16).map_err(|_| "색상이 올바르지 않습니다.")?;
    Ok(((n >> 16) & 255) | (n & 0xff00) | ((n & 255) << 16))
}
fn contrast(a: u32, b: u32) -> f64 {
    let lum = |v: u32| {
        let c = |shift: u32| {
            let s = ((v >> shift) & 255u32) as f64 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * c(0) + 0.7152 * c(8) + 0.0722 * c(16)
    };
    let (a, b) = (lum(a), lum(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
impl Skin {
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() as u64 > MAX_CONFIG {
            return Err("템플릿은 128 KiB 이하로 작성하세요.".into());
        }
        let v: Value = serde_json::from_str(text).map_err(|e| format!("JSON 오류: {e}"))?;
        if v["version"] != 1 {
            return Err("지원하는 템플릿 version은 1입니다.".into());
        }
        let name = v["name"]
            .as_str()
            .filter(|s| {
                !s.trim().is_empty() && !s.chars().any(char::is_control) && s.chars().count() <= 48
            })
            .ok_or("name은 1~48자입니다.")?
            .to_string();
        let c = |key: &str| {
            color(
                v["colors"][key]
                    .as_str()
                    .ok_or_else(|| format!("colors.{key}가 필요합니다."))?,
            )
        };
        let n = |key: &str, min: i64, max: i64| {
            v[key]
                .as_i64()
                .filter(|n| (min..=max).contains(n))
                .map(|n| n as i32)
                .ok_or_else(|| format!("{key}는 {min}~{max}입니다."))
        };
        let mut fields = Vec::new();
        for item in v["fields"].as_array().ok_or("fields 배열이 필요합니다.")? {
            let field = item.as_str().ok_or("fields는 문자열 배열입니다.")?;
            if !["task", "project", "status", "agent", "model", "activity"].contains(&field)
                || fields.iter().any(|f| f == field)
            {
                return Err(format!("알 수 없거나 중복된 필드: {field}"));
            }
            fields.push(field.to_string());
        }
        if !["task", "project", "status"]
            .iter()
            .all(|f| fields.iter().any(|x| x == f))
        {
            return Err("task, project, status는 필수입니다.".into());
        }
        let mut icons = BTreeMap::new();
        for state in STATES {
            let icon = v["icons"][state]
                .as_str()
                .filter(|s| {
                    !s.is_empty() && s.chars().count() <= 4 && !s.chars().any(char::is_control)
                })
                .ok_or_else(|| format!("icons.{state}는 1~4자의 기호입니다."))?;
            icons.insert(state.to_string(), icon.to_string());
        }
        let mut state_colors = BTreeMap::new();
        for state in STATES {
            state_colors.insert(
                state.to_string(),
                color(
                    v["state_colors"][state]
                        .as_str()
                        .ok_or_else(|| format!("state_colors.{state}가 필요합니다."))?,
                )?,
            );
        }
        let skin = Self {
            compact: v["compact"].as_bool().unwrap_or(true),
            state_colors,
            name,
            background: c("background")?,
            surface: c("surface")?,
            foreground: c("text")?,
            muted: c("muted")?,
            selection: c("selection")?,
            accent: c("accent")?,
            font_size: n("font_size", 12, 24)?,
            padding: n("padding", 4, 24)?,
            line_gap: n("line_gap", 2, 12)?,
            fields,
            icons,
        };
        for foreground in [skin.foreground, skin.muted, skin.accent]
            .into_iter()
            .chain(skin.state_colors.values().copied())
        {
            for background in [skin.background, skin.surface, skin.selection] {
                if contrast(foreground, background) < 4.5 {
                    return Err(
                        "텍스트·강조색과 목록 배경의 명암비는 4.5:1 이상이어야 합니다.".into(),
                    );
                }
            }
        }
        Ok(skin)
    }
    pub fn row_height(&self) -> i32 {
        12 + self.padding * 2
            + (self.lines().len() as i32 + 1) * (self.font_size + 4 + self.line_gap)
    }
    pub fn lines(&self) -> Vec<Vec<&str>> {
        if self.compact {
            let mut header = vec!["project"];
            header.extend(
                self.fields
                    .iter()
                    .map(String::as_str)
                    .filter(|f| *f == "agent"),
            );
            let mut lines = vec![header, vec!["task"]];
            if self.fields.iter().any(|f| f == "activity") {
                lines.push(vec!["activity"]);
            }
            return lines;
        }
        self.fields
            .iter()
            .map(|field| vec![field.as_str()])
            .collect()
    }
    pub fn text(&self, row: &Row, field: &str, pinned: bool) -> String {
        match field {
            "task" => format!("{}{}", if pinned { "★ " } else { "" }, row.task),
            "project" => row.project().into(),
            "agent" if self.compact && row.agent == "Claude Code" => "Claude".into(),
            "agent" => row.agent.clone(),
            "model" => row.model.clone(),
            "status" => format!(
                "{} {}{}",
                self.icons.get(&row.state).unwrap_or(&self.icons["unknown"]),
                row.status(),
                if row.auxiliary {
                    " · 보조 세션"
                } else {
                    ""
                }
            ),
            "activity" => format!("마지막 기록  {}", row.short_date()),
            _ => String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub two_columns: bool,
    pub always_on_top: bool,
    pub opacity: u8,
    pub closed: BTreeMap<String, Row>,
    pub show_all: bool,
    pub pinned: BTreeSet<String>,
    pub hidden: BTreeSet<String>,
    pub session_targets: BTreeMap<String, Value>,
    pub search: String,
    pub state: String,
    pub agent: String,
    pub show_aux: bool,
    pub show_hidden: bool,
    pub template: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            two_columns: false,
            always_on_top: false,
            opacity: 100,
            closed: BTreeMap::new(),
            show_all: false,
            pinned: BTreeSet::new(),
            hidden: BTreeSet::new(),
            session_targets: BTreeMap::new(),
            search: String::new(),
            state: String::new(),
            agent: String::new(),
            show_aux: false,
            show_hidden: false,
            template: DEFAULT.into(),
        }
    }
}
impl Settings {
    pub fn visible(&self, rows: &[Row]) -> Vec<Row> {
        self.visible_at(rows, crate::time::now())
    }

    fn visible_at(&self, rows: &[Row], now: i64) -> Vec<Row> {
        let query = self.search.to_lowercase();
        let mut candidates = rows.to_vec();
        if self.show_all {
            for (id, row) in &self.closed {
                if !rows.iter().any(|r| &r.id == id) {
                    candidates.push(row.clone());
                }
            }
        }
        for row in &mut candidates {
            if self.closed.contains_key(&row.id) {
                row.state = "done".into();
            }
        }
        let mut visible: Vec<Row> = candidates
            .iter()
            .filter(|r| {
                (self.show_aux || !r.auxiliary)
                    && (self.show_all || self.pinned.contains(&r.id) || r.observed_since_launch
                        || crate::time::from_iso8601(&r.since)
                            .is_none_or(|at| now.saturating_sub(at) <= 24 * 3600))
                    && (self.show_all || (!self.closed.contains_key(&r.id) && r.state != "done"))
                    && (self.show_all || self.show_hidden || !self.hidden.contains(&r.id))
                    && (self.state.is_empty() || self.state == r.state)
                    && (self.agent.is_empty() || self.agent == r.agent)
                    && (query.is_empty()
                        || format!("{} {} {} {} {}", r.task, r.title, r.agent, r.model, r.cwd)
                            .to_lowercase()
                            .contains(&query))
            })
            .cloned()
            .collect();
        visible.sort_by(|a, b| {
            self.closed
                .contains_key(&a.id)
                .cmp(&self.closed.contains_key(&b.id))
                .then(
                    self.pinned
                        .contains(&b.id)
                        .cmp(&self.pinned.contains(&a.id))
                        .then(
                            STATES
                                .iter()
                                .position(|s| *s == a.state)
                                .cmp(&STATES.iter().position(|s| *s == b.state)),
                        )
                        .then_with(|| crate::time::from_iso8601(&b.since)
                            .cmp(&crate::time::from_iso8601(&a.since)))
                        .then(a.title.cmp(&b.title))
                        .then(a.id.cmp(&b.id)),
                )
        });
        visible
    }
    pub fn close(&mut self, row: &Row) -> Result<(), String> {
        if !self.closed.contains_key(&row.id) && self.closed.len() >= 1024 {
            return Err("종결 기록은 최대 1,024개입니다.".into());
        }
        let mut row = row.clone();
        if row.request_marker.is_empty() {
            row.request_marker = "none".into();
        }
        self.closed.insert(row.id.clone(), row);
        Ok(())
    }
    pub fn reconcile(&mut self, rows: &[Row]) -> (usize, bool) {
        let mut migrated = self.migrate_ids(rows);
        let before = self.closed.len();
        for row in rows {
            let revive = if let Some(closed) = self.closed.get_mut(&row.id) {
                let old_version = closed.request_marker.split_once(':').map(|p| p.0);
                let new_version = row.request_marker.split_once(':').map(|p| p.0);
                let migration = closed.request_marker.is_empty()
                    // A restarted core only has Orca's latest hook, not its submit time.
                    || (row.task_source == "orca_hook" && row.request_at.is_none() && row.task == closed.task
                        && row.request_marker != closed.request_marker)
                    || matches!((old_version, new_version), (Some(a), Some(b)) if a.starts_with('r') && b.starts_with('r') && a != b);
                if !row.request_marker.is_empty() && migration {
                    closed.request_marker = row.request_marker.clone();
                    closed.request_at = row.request_at;
                    migrated = true;
                } else if row.request_marker.is_empty() && old_version == Some("r1") {
                    closed.request_marker = "none".into();
                    migrated = true;
                }
                let new_request = !migration
                    && !row.request_marker.is_empty()
                    && row.request_marker != closed.request_marker;
                // The turn may finish or be interrupted between polls; the request is the evidence.
                new_request
            } else {
                false
            };
            if revive {
                self.closed.remove(&row.id);
            }
        }
        (before - self.closed.len(), migrated)
    }

    fn migrate_ids(&mut self, rows: &[Row]) -> bool {
        let mut candidates: BTreeMap<&str, Vec<&Row>> = BTreeMap::new();
        for row in rows.iter().filter(|r| !r.legacy_id.is_empty() && r.id != r.legacy_id) {
            candidates.entry(&row.legacy_id).or_default().push(row);
        }
        let mut changed = false;
        for (old, rows) in candidates {
            let saved = self.closed.get(old);
            let matches: Vec<_> = rows.iter().filter(|row| saved.is_none_or(|saved|
                (saved.session_id.is_empty() || saved.session_id == row.session_id)
                    && (saved.agent_id.is_empty() || saved.agent_id == row.agent_id)))
                .collect();
            // A legacy hash can name several sessions. Never transfer organization to a guess.
            let [row] = matches.as_slice() else { continue };
            for set in [&mut self.pinned, &mut self.hidden] {
                if set.remove(old) {
                    set.insert(row.id.clone());
                    changed = true;
                }
            }
            if let Some(mut saved) = self.closed.remove(old) {
                saved.id = row.id.clone();
                saved.legacy_id = old.into();
                self.closed.entry(row.id.clone()).or_insert(saved);
                changed = true;
            }
            if let Some(target) = self.session_targets.remove(old) {
                self.session_targets.entry(row.id.clone()).or_insert(target);
                changed = true;
            }
        }
        changed
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let text = read_bounded(path, MAX_SETTINGS)?;
        let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if v["version"] != 1 {
            return Err("설정 파일 버전을 확인할 수 없습니다.".into());
        }
        let mut s = Self::default();
        for (key, target) in [("pinned", &mut s.pinned), ("hidden", &mut s.hidden)] {
            if let Some(a) = v[key].as_array() {
                for id in a
                    .iter()
                    .take(1024)
                    .filter_map(Value::as_str)
                    .filter(|id| !id.is_empty())
                {
                    target.insert(id.to_string());
                }
            }
        }
        if let Some(targets) = v["session_targets"].as_object() {
            s.session_targets = targets.iter().take(1024)
                .filter(|(id, target)| !id.is_empty() && target.is_object())
                .map(|(id, target)| (id.clone(), target.clone())).collect();
        }
        s.always_on_top = v["always_on_top"].as_bool().unwrap_or(false);
        s.two_columns = v["two_columns"].as_bool().unwrap_or(false);
        s.opacity = v["opacity"]
            .as_u64()
            .filter(|n| (40..=100).contains(n))
            .unwrap_or(100) as u8;
        s.show_all = v["show_all"].as_bool().unwrap_or(false);
        if let Some(closed) = v["closed"].as_array() {
            for value in closed.iter().take(1024) {
                if let Some(row) = saved_row(value) {
                    s.closed.insert(row.id.clone(), row);
                }
            }
        }
        s.search = v["search"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(200)
            .collect();
        s.state = v["state"]
            .as_str()
            .filter(|s| STATES.contains(s))
            .unwrap_or("")
            .into();
        s.agent = v["agent"].as_str().unwrap_or("").chars().take(48).collect();
        s.show_aux = v["show_aux"].as_bool().unwrap_or(false);
        s.show_hidden = v["show_hidden"].as_bool().unwrap_or(false);
        s.template = v["template"].as_str().unwrap_or(DEFAULT).into();
        Skin::parse(&s.template)?;
        Ok(s)
    }
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let closed: Vec<Value> = self.closed.values().map(row_value).collect();
        let v = json!({"version":1,"two_columns":self.two_columns,"always_on_top":self.always_on_top,"opacity":self.opacity,"closed":closed,"show_all":self.show_all,"pinned":self.pinned,"hidden":self.hidden,"search":self.search,"state":self.state,"agent":self.agent,"show_aux":self.show_aux,"show_hidden":self.show_hidden,"template":self.template,"session_targets":self.session_targets});
        let text = serde_json::to_string_pretty(&v)?;
        if text.len() as u64 > MAX_SETTINGS {
            return Err(io::Error::other("설정 파일 크기 상한 8 MiB를 초과했습니다."));
        }
        atomic_write(path, &text)
    }
}
pub fn data_file() -> PathBuf {
    let root = std::env::var_os("WAID_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("waid")
        });
    root.join("settings.json")
}
pub fn read_text(path: &Path) -> Result<String, String> {
    read_bounded(path, MAX_CONFIG)
}
fn read_bounded(path: &Path, limit: u64) -> Result<String, String> {
    use std::io::Read;
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut text = String::new();
    file.take(limit + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() as u64 > limit {
        return Err(format!("파일 크기 상한: {} KiB", limit / 1024));
    }
    Ok(text.trim_start_matches('\u{feff}').to_string())
}
pub fn atomic_write(path: &Path, text: &str) -> io::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = fs::File::create(&tmp)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remote_hook_restart_does_not_revive_a_dismissed_request() {
        let mut row = Row { id: "orca:remote".into(), task: "same request".into(),
            task_source: "orca_hook".into(), request_marker: "orca-request:100:same request".into(),
            request_at: Some(100), ..Row::default() };
        let mut settings = Settings::default();
        settings.close(&row).unwrap();
        row.request_at = None;
        row.request_marker = "orca-request:110:same request".into();
        assert_eq!(settings.reconcile(&[row.clone()]), (0, true));
        assert_eq!(settings.reconcile(&[row.clone()]), (0, false));
        row.request_at = Some(120);
        row.request_marker = "orca-request:120:same request".into();
        assert_eq!(settings.reconcile(&[row.clone()]).0, 1);
        settings.close(&row).unwrap();
        row.request_at = None;
        row.task = "different request".into();
        row.request_marker = "orca-request:130:different request".into();
        assert_eq!(settings.reconcile(&[row]).0, 1);
    }

    #[test]
    fn recent_list_keeps_pins_live_work_and_unknown_dates_and_all_restores_history() {
        let now = crate::time::from_iso8601("2026-09-08T12:00:00Z").unwrap();
        let row = |id: &str, age| Row {
            id:id.into(), since:crate::time::to_iso8601(now - age),
            state:"unknown".into(), ..Row::default()
        };
        let mut rows = vec![row("recent", 10), row("boundary", 86400), row("old", 86401),
            row("pinned", 172800), row("long-work", 172800), Row { id:"undated".into(), ..Row::default() }];
        rows[4].observed_since_launch = true;
        let mut settings = Settings::default();
        settings.pinned.insert("pinned".into());
        let shown = settings.visible_at(&rows, now);
        assert_eq!(shown.len(), 5);
        assert_eq!(shown[0].id, "pinned");
        assert!(!shown.iter().any(|r| r.id == "old"));
        assert!(!settings.visible_at(&rows, now + 1).iter().any(|r| r.id == "boundary"));
        settings.show_all = true;
        assert_eq!(settings.visible_at(&rows, now).len(), 6);
        settings.search = "old".into();
        rows[2].task = "old request".into();
        assert_eq!(settings.visible_at(&rows, now)[0].id, "old");
    }

    #[test]
    fn full_ids_migrate_organization_without_guessing_legacy_collisions() {
        let path = std::env::temp_dir().join(format!("waid-id-migration-{}.json", std::process::id()));
        let old = Row { id:"adee487a".into(), session_id:"source-one".into(), agent_id:"codex".into(),
            request_marker:"r2:request".into(), ..Row::default() };
        let current = Row { id:"full-identity:".repeat(40), legacy_id:old.id.clone(), ..old.clone() };
        let mut settings = Settings::default();
        settings.pinned.insert(old.id.clone());
        settings.hidden.insert(old.id.clone());
        settings.session_targets.insert(old.id.clone(), json!({"kind":"chatgpt_link","url":"codex://threads/source-one"}));
        settings.close(&old).unwrap();
        assert_eq!(settings.reconcile(&[current.clone()]), (0, true));
        assert!(settings.pinned.contains(&current.id));
        assert!(settings.hidden.contains(&current.id));
        assert!(settings.session_targets.contains_key(&current.id));
        settings.save(&path).unwrap();
        let restored = Settings::read(&path).unwrap();
        assert!(restored.closed.contains_key(&current.id));
        assert_eq!(restored.session_targets, settings.session_targets);
        let saved = fs::read(&path).unwrap();
        let mut oversized = settings.clone();
        oversized.search = "x".repeat(MAX_SETTINGS as usize);
        assert!(oversized.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), saved);
        let other = Row { id:"another-full-id".into(), session_id:"source-two".into(), ..current.clone() };
        let mut ambiguous = Settings::default();
        ambiguous.pinned.insert(old.id.clone());
        assert_eq!(ambiguous.reconcile(&[current.clone(), other.clone()]), (0, false));
        assert!(ambiguous.pinned.contains(&old.id));
        ambiguous.close(&old).unwrap();
        assert_eq!(ambiguous.reconcile(&[current.clone(), other]), (0, true));
        assert!(ambiguous.pinned.contains(&current.id));
        assert_eq!(ambiguous.reconcile(&[current]), (0, false));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn skins_validate_and_are_distinct() {
        let a = Skin::parse(DEFAULT).unwrap();
        let b = Skin::parse(NIGHT).unwrap();
        assert_ne!(a, b);
        assert_ne!(a.background, b.background);
        assert_ne!(a.fields, b.fields);
        assert_eq!(
            a.lines(),
            vec![vec!["project", "agent"], vec!["task"]]
        );
        assert_eq!(a.row_height(), 92);
        let mut expanded = a.clone();
        expanded.compact = false;
        assert_eq!(expanded.lines().len(), expanded.fields.len());
        let mut activity = a.clone();
        activity.fields.push("activity".into());
        assert_eq!(activity.lines().last().unwrap(), &["activity"]);
        assert_eq!(activity.row_height(), 112);
        let mut v: Value = serde_json::from_str(DEFAULT).unwrap();
        v["fields"] = json!(["task", "project"]);
        assert!(Skin::parse(&v.to_string()).is_err());
        v = serde_json::from_str(DEFAULT).unwrap();
        v["colors"]["text"] = v["colors"]["surface"].clone();
        assert!(Skin::parse(&v.to_string()).is_err());
        assert!(Skin::parse("{").is_err());
    }

    #[test]
    fn closed_sessions_persist_and_only_new_requests_revive_them() {
        let mut row = Row {
            id: "stable".into(),
            task: "same request".into(),
            state: "waiting".into(),
            request_marker: "request-a".into(),
            ..Row::default()
        };
        let mut settings = Settings::default();
        assert_eq!(settings.visible(&[row.clone()]).len(), 1);
        settings.close(&row).unwrap();
        assert!(settings.visible(&[row.clone()]).is_empty());
        settings.show_all = true;
        assert_eq!(settings.visible(&[])[0].state, "done");
        let path = std::env::temp_dir().join(format!("waid-closed-{}.json", std::process::id()));
        settings.save(&path).unwrap();
        let mut settings = Settings::read(&path).unwrap();
        assert!(settings.closed.contains_key("stable"));
        settings.show_all = false;
        row.model = "changed".into();
        row.since = "newer metadata".into();
        for state in ["working", "waiting", "idle", "unknown"] {
            row.state = state.into();
            assert_eq!(settings.reconcile(&[row.clone()]), (0, false));
        }
        row.request_marker = "request-b".into();
        row.state = "working".into();
        assert_eq!(settings.reconcile(&[row.clone()]), (1, false));
        assert_eq!(settings.visible(&[row.clone()]).len(), 1);
        settings.close(&row).unwrap();
        row.request_marker = "request-c".into();
        row.state = "waiting".into();
        assert_eq!(
            settings.reconcile(&[row.clone()]),
            (1, false),
            "a fast completed turn must not be missed between polls"
        );
        settings.close(&row).unwrap();
        row.request_marker = "request-d".into();
        row.state = "unknown".into();
        assert_eq!(settings.reconcile(&[row.clone()]), (1, false));
        settings.close(&row).unwrap();
        row.request_marker = "request-e".into();
        row.state = "idle".into();
        assert_eq!(settings.reconcile(&[row.clone()]), (1, false));
        settings.close(&row).unwrap();
        settings.closed.get_mut("stable").unwrap().request_marker = "r1:legacy".into();
        row.request_marker = "r2:current".into();
        assert_eq!(settings.reconcile(&[row.clone()]), (0, true));
        settings.save(&path).unwrap();
        assert_eq!(
            Settings::read(&path).unwrap().closed["stable"].request_marker,
            "r2:current"
        );
        row.request_marker = "r2:next".into();
        assert_eq!(settings.reconcile(&[row.clone()]), (1, false));
        settings.close(&row).unwrap();
        row.request_marker.clear();
        assert_eq!(settings.reconcile(&[row]), (0, false));
        let mut markerless = Row { id: "markerless".into(), ..Row::default() };
        settings.close(&markerless).unwrap();
        assert_eq!(settings.closed["markerless"].request_marker, "none");
        markerless.request_marker = "r2:first-request".into();
        assert_eq!(settings.reconcile(&[markerless]), (1, false));
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn organization_filters_and_settings_roundtrip() {
        let rows = vec![
            Row {
                id: "a".into(),
                title: "한글 프로젝트".into(),
                state: "working".into(),
                agent: "Codex".into(),
                ..Row::default()
            },
            Row {
                id: "b".into(),
                state: "waiting".into(),
                ..Row::default()
            },
            Row {
                id: "c".into(),
                auxiliary: true,
                ..Row::default()
            },
        ];
        let mut s = Settings::default();
        assert!(!s.two_columns);
        assert_eq!(s.visible(&rows).len(), 2);
        s.show_all = true;
        assert_eq!(s.visible(&rows).len(), 2);
        s.show_aux = true;
        assert_eq!(s.visible(&rows).len(), 3);
        s.show_all = false;
        s.show_aux = false;
        s.two_columns = false;
        s.pinned.insert("a".into());
        assert_eq!(s.visible(&rows)[0].id, "a");
        s.search = "한글".into();
        assert_eq!(s.visible(&rows).len(), 1);
        s.hidden.insert("a".into());
        assert!(s.visible(&rows).is_empty());
        s.show_hidden = true;
        assert_eq!(s.visible(&rows).len(), 1);
        let path = std::env::temp_dir().join(format!("waid-settings-{}.json", std::process::id()));
        s.save(&path).unwrap();
        s.save(&path).unwrap();
        let loaded = Settings::read(&path).unwrap();
        assert_eq!(loaded.search, s.search);
        assert_eq!(loaded.hidden, s.hidden);
        assert_eq!(loaded.template, s.template);
        assert!(!loaded.two_columns);
        fs::remove_file(path).unwrap();
    }
}

fn row_value(row: &Row) -> Value {
    let cut = |s: &str, n: usize| s.chars().take(n).collect::<String>();
    json!({"last_answer":cut(&row.last_answer,1000),"session_id":row.session_id,"agent_id":row.agent_id,"host":row.host,"id":row.id,"legacy_id":row.legacy_id,"logged_state":row.logged_state,"request_marker":row.request_marker,"request_at":row.request_at,"title":row.title,"agent":row.agent,"model":row.model,"state":row.state,"since":row.since,"evidence":row.evidence,"task_source":row.task_source,"auxiliary":row.auxiliary,"task":cut(&row.task,4000),"summary":cut(&row.summary,200),"cwd":cut(&row.cwd,512)})
}
fn saved_row(v: &Value) -> Option<Row> {
    let id = v["id"]
        .as_str()
        .filter(|s| !s.is_empty())?
        .to_string();
    Some(Row {
        legacy_id: v["legacy_id"].as_str().unwrap_or("").to_string(),
        logged_state: v["logged_state"].as_str().unwrap_or("").to_string(),
        observed_since_launch: false,
        request_at: v["request_at"].as_i64(),
        last_answer: crate::field(&v["last_answer"], 4000),
        session_id: v["session_id"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        agent_id: v["agent_id"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(48)
            .collect(),
        host: v["host"].as_str().unwrap_or("").to_string(),
        id,
        request_marker: v["request_marker"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(128)
            .collect(),
        title: crate::field(&v["title"], 160),
        agent: crate::field(&v["agent"], 48),
        model: crate::field(&v["model"], 80),
        state: crate::field(&v["state"], 16),
        since: crate::field(&v["since"], 48),
        evidence: crate::field(&v["evidence"], 48),
        task_source: crate::field(&v["task_source"], 48),
        auxiliary: v["auxiliary"].as_bool().unwrap_or(false),
        task: v["task"].as_str().unwrap_or("—").chars().filter(|c| *c != '\0').take(4000).collect(),
        summary: crate::field(&v["summary"], 200),
        cwd: crate::field(&v["cwd"], 512),
    })
}
