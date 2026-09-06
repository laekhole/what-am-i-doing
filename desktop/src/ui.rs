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
        10 + self.padding * 2 + self.lines().len() as i32 * (self.font_size + 4 + self.line_gap)
    }
    pub fn lines(&self) -> Vec<Vec<&str>> {
        let mut lines: Vec<Vec<&str>> = Vec::new();
        for field in &self.fields {
            let separate = field == "task" || field == "activity" || !self.compact;
            if !separate
                && lines
                    .last()
                    .is_some_and(|line| line.len() == 1 && !["task", "activity"].contains(&line[0]))
            {
                lines.last_mut().unwrap().push(field);
            } else {
                lines.push(vec![field]);
            }
        }
        lines
    }
    pub fn text(&self, row: &Row, field: &str, pinned: bool) -> String {
        match field {
            "task" => format!("{}{}", if pinned { "★ " } else { "" }, row.task),
            "project" => row.title.clone(),
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
            "activity" => format!("마지막 기록  {}", row.since),
            _ => String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub always_on_top: bool,
    pub opacity: u8,
    pub closed: BTreeMap<String, Row>,
    pub show_all: bool,
    pub pinned: BTreeSet<String>,
    pub hidden: BTreeSet<String>,
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
            always_on_top: false,
            opacity: 100,
            closed: BTreeMap::new(),
            show_all: false,
            pinned: BTreeSet::new(),
            hidden: BTreeSet::new(),
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
                (self.show_all || self.show_aux || !r.auxiliary)
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
        self.closed.insert(row.id.clone(), row.clone());
        Ok(())
    }
    pub fn reconcile(&mut self, rows: &[Row]) -> usize {
        let before = self.closed.len();
        for row in rows {
            if let Some(closed) = self.closed.get(&row.id) {
                let new_request =
                    !row.request_marker.is_empty() && row.request_marker != closed.request_marker;
                // A request can finish between polls: a fresh waiting/error record still revives it.
                let observed = ["working", "waiting", "error"].contains(&row.state.as_str());
                if new_request && observed {
                    self.closed.remove(&row.id);
                }
            }
        }
        before - self.closed.len()
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        let text = read_bounded(path, 8 * 1024 * 1024)?;
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
                    .filter(|id| id.len() <= 128)
                {
                    target.insert(id.to_string());
                }
            }
        }
        s.always_on_top = v["always_on_top"].as_bool().unwrap_or(false);
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
        let v = json!({"version":1,"always_on_top":self.always_on_top,"opacity":self.opacity,"closed":closed,"show_all":self.show_all,"pinned":self.pinned,"hidden":self.hidden,"search":self.search,"state":self.state,"agent":self.agent,"show_aux":self.show_aux,"show_hidden":self.show_hidden,"template":self.template});
        atomic_write(path, &serde_json::to_string_pretty(&v)?)
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
    fn skins_validate_and_are_distinct() {
        let a = Skin::parse(DEFAULT).unwrap();
        let b = Skin::parse(NIGHT).unwrap();
        assert_ne!(a, b);
        assert_ne!(a.background, b.background);
        assert_ne!(a.fields, b.fields);
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
            assert_eq!(settings.reconcile(&[row.clone()]), 0);
        }
        row.request_marker = "request-b".into();
        row.state = "working".into();
        assert_eq!(settings.reconcile(&[row.clone()]), 1);
        assert_eq!(settings.visible(&[row.clone()]).len(), 1);
        settings.close(&row).unwrap();
        row.request_marker = "request-c".into();
        row.state = "waiting".into();
        assert_eq!(
            settings.reconcile(&[row.clone()]),
            1,
            "a fast completed turn must not be missed between polls"
        );
        settings.close(&row).unwrap();
        row.request_marker.clear();
        assert_eq!(settings.reconcile(&[row]), 0);
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
        fs::remove_file(path).unwrap();
    }
}

fn row_value(row: &Row) -> Value {
    let cut = |s: &str, n: usize| s.chars().take(n).collect::<String>();
    json!({"agent_id":row.agent_id,"id":row.id,"request_marker":row.request_marker,"title":row.title,"agent":row.agent,"model":row.model,"state":row.state,"since":row.since,"evidence":row.evidence,"task_source":row.task_source,"auxiliary":row.auxiliary,"task":cut(&row.task,512),"summary":cut(&row.summary,200),"cwd":cut(&row.cwd,512)})
}
fn saved_row(v: &Value) -> Option<Row> {
    let id = v["id"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 128)?
        .to_string();
    Some(Row {
        agent_id: v["agent_id"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(48)
            .collect(),
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
        task: crate::field(&v["task"], 512),
        summary: crate::field(&v["summary"], 200),
        cwd: crate::field(&v["cwd"], 512),
    })
}
