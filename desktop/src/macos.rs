//! AppKit owns presentation; Rust owns collection, organization and persistence.
use crate::{activate, ui::{self, Settings, Skin}, Row, Updates};
use serde_json::{json, Value};
use std::{ffi::{c_char, c_void, CStr, CString}, path::PathBuf, sync::mpsc};

extern "C" {
    fn waid_app_run(context: *mut c_void, callback: extern "C" fn(*mut c_void, *const c_char) -> *const c_char);
    fn waid_app_error(message: *const c_char);
}

struct App {
    settings: Settings,
    path: PathBuf,
    rows: Vec<Row>,
    updates: Updates,
    error: String,
    warnings: Vec<String>,
    notice: String,
    preview: Option<String>,
    activation: Option<mpsc::Receiver<Result<String, String>>>,
    response: CString,
}

impl App {
    fn poll(&mut self) {
        let update = self.updates.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(update) = update {
            if let Some(snapshot) = update.snapshot {
                let (revived, migrated) = self.settings.reconcile(&snapshot.rows);
                self.rows = snapshot.rows;
                self.warnings = snapshot.warnings;
                if revived > 0 || migrated { self.save(); }
            }
            self.error = update.error.unwrap_or_default();
        }
        if let Some(rx) = &self.activation {
            match rx.try_recv() {
                Ok(result) => {
                    self.notice = result.unwrap_or_else(|e| e);
                    self.activation = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.notice = "Session return stopped unexpectedly. Try again.".into();
                    self.activation = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
    }

    fn save(&mut self) {
        if let Err(e) = self.settings.save(&self.path) {
            self.notice = format!("Settings were not saved: {e}. Check folder permissions and retry.");
        }
    }

    fn action(&mut self, action: &Value) -> Result<(), String> {
        let name = action["action"].as_str().ok_or("Missing action")?;
        if name == "poll" { return Ok(()); }
        let value = action["value"].as_str().unwrap_or_default();
        let before = self.settings.clone();
        match name {
            "search" => self.settings.search = value.chars().take(200).collect(),
            "state" => {
                if !value.is_empty() && !ui::STATES.contains(&value) { return Err("Invalid status".into()); }
                self.settings.state = value.into();
            }
            "agent" => self.settings.agent = value.chars().take(48).collect(),
            "all" => self.settings.show_all = !self.settings.show_all,
            "aux" => self.settings.show_aux = !self.settings.show_aux,
            "top" => self.settings.always_on_top = !self.settings.always_on_top,
            "opacity" => self.settings.opacity = value.parse::<u8>().ok().filter(|v| (40..=100).contains(v)).ok_or("Opacity must be 40–100")?,
            "preview" | "template" => {
                Skin::parse(value)?;
                if name == "preview" { self.preview = Some(value.into()); return Ok(()); }
                self.settings.template = value.into();
                self.preview = None;
            }
            "cancel_preview" => { self.preview = None; return Ok(()); }
            "save" => {}
            "pin" | "hide" | "dismiss" | "connect" | "disconnect" | "open" => {
                let id = action["id"].as_str().ok_or("Select a session")?;
                let row = self.settings.visible(&self.rows).into_iter().find(|r| r.id == id)
                    .ok_or("This session is no longer in the list. Select it again.")?;
                match name {
                    "pin" | "hide" => {
                        let set = if name == "pin" { &mut self.settings.pinned } else { &mut self.settings.hidden };
                        if !set.remove(id) {
                            if set.len() >= 1024 { return Err("At most 1,024 entries can be organized this way.".into()); }
                            set.insert(id.into());
                        }
                    }
                    "dismiss" => {
                        if self.settings.closed.remove(id).is_some() { self.settings.hidden.remove(id); }
                        else { self.settings.close(&row)?; }
                    }
                    "connect" => {
                        activate::validate_chatgpt_link(&row, value)?;
                        if self.settings.session_targets.len() >= 1024 && !self.settings.session_targets.contains_key(id) {
                            return Err("At most 1,024 connections can be saved.".into());
                        }
                        self.settings.session_targets.insert(id.into(), json!({"kind":"chatgpt_link","url":value}));
                    }
                    "disconnect" => { self.settings.session_targets.remove(id); }
                    "open" => {
                        if self.activation.is_some() { return Err("Session return is already in progress.".into()); }
                        let target = self.settings.session_targets.get(id).cloned();
                        let (tx, rx) = mpsc::channel();
                        self.activation = Some(rx);
                        self.notice = "Checking the existing session…".into();
                        std::thread::spawn(move || {
                            let result = match target {
                                Some(target) => activate::open_associated(&row, &target),
                                None => activate::open(&row).map(|_| "Selected the existing Orca session.".into()),
                            };
                            let _ = tx.send(result);
                        });
                        return Ok(());
                    }
                    _ => unreachable!(),
                }
            }
            _ => return Err("Unknown action".into()),
        }
        if let Err(error) = self.settings.save(&self.path) {
            self.settings = before;
            return Err(format!("Settings were not saved; change reverted: {error}"));
        }
        self.notice.clear();
        Ok(())
    }

    fn view(&self) -> Value {
        let template = self.preview.as_deref().unwrap_or(&self.settings.template);
        let skin = Skin::parse(template).expect("validated template");
        let rows: Vec<_> = self.settings.visible(&self.rows).iter().map(|row| {
            let pinned = self.settings.pinned.contains(&row.id);
            let mut lines: Vec<String> = skin.lines().iter().map(|line| line.iter()
                .map(|field| skin.text(row, field, pinned).split_whitespace().collect::<Vec<_>>().join(" "))
                .collect::<Vec<_>>().join(" · ")).collect();
            if skin.compact {
                lines.push(skin.text(row, "status", pinned));
                if skin.fields.iter().any(|f| f == "model") { lines.push(row.model.clone()); }
            }
            json!({"id":row.id,"lines":lines,"status":skin.text(row,"status",pinned),
                "state":row.state,"model":row.model,"prompt":row.task,
                "detail":format!("{}\n\nPrompt\n{}\n\nLast answer\n{}\n\nFirst prompt\n{}\n\nFolder\n{}\n\nLast record (UTC)\n{}\n\n{}",row.accessible_text(),row.task,row.last_answer,row.summary,row.cwd,row.since,row.status_context()),
                "pinned":pinned,"hidden":self.settings.hidden.contains(&row.id),
                "dismissed":self.settings.closed.contains_key(&row.id),
                "connected":self.settings.session_targets.contains_key(&row.id)})
        }).collect();
        let mut agents: Vec<_> = self.rows.iter().map(|r| r.agent.clone()).collect();
        agents.push(self.settings.agent.clone());
        agents.retain(|a| !a.is_empty()); agents.sort(); agents.dedup();
        json!({"rows":rows,"agents":agents,"search":self.settings.search,"state":self.settings.state,
            "agent":self.settings.agent,"all":self.settings.show_all,"aux":self.settings.show_aux,
            "top":self.settings.always_on_top,"opacity":self.settings.opacity,
            "template":self.settings.template,"skin":serde_json::from_str::<Value>(template).unwrap(),
            "error":self.error,"notice":self.notice,"warnings":self.warnings,"busy":self.activation.is_some()})
    }
}

// The pointer stays valid until the next synchronous callback. Swift copies it
// immediately; callbacks and all App mutation run exclusively on AppKit's thread.
extern "C" fn callback(context: *mut c_void, request: *const c_char) -> *const c_char {
    let app = unsafe { &mut *(context as *mut App) };
    app.poll();
    let result = unsafe { CStr::from_ptr(request) }.to_str().map_err(|e| e.to_string())
        .and_then(|text| serde_json::from_str::<Value>(text).map_err(|e| e.to_string()))
        .and_then(|action| app.action(&action));
    if let Err(error) = result { app.notice = error; }
    app.response = CString::new(app.view().to_string()).expect("JSON escapes NUL");
    app.response.as_ptr()
}

pub fn run() {
    let result = (|| -> Result<(), String> {
        let path = ui::data_file();
        let settings = if path.exists() { Settings::read(&path)? } else { Settings::default() };
        let (_core, updates) = crate::start_core(&std::env::current_exe().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let mut app = App { settings, path, rows: Vec::new(), updates, error: String::new(),
            warnings: Vec::new(), notice: String::new(), preview: None, activation: None,
            response: CString::default() };
        unsafe { waid_app_run(&mut app as *mut App as *mut c_void, callback); }
        Ok(()) // Drop kills and joins only our collector child.
    })();
    if let Err(error) = result {
        let message = CString::new(format!("waid could not start: {}", error.replace('\0', ""))).unwrap();
        unsafe { waid_app_error(message.as_ptr()); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_actions_persist_filter_restore_and_validate_templates_and_links() {
        let dir = std::env::temp_dir().join(format!("waid-mac-actions-{}", std::process::id()));
        let mut app = App { settings: Settings::default(), path: dir.join("settings.json"),
            rows: vec![Row { id:"one".into(), session_id:"session-one".into(), agent_id:"codex".into(),
                task:"find me".into(), request_marker:"r2:first".into(), ..Row::default() }],
            updates: Updates::default(), error:String::new(), warnings:vec![], notice:String::new(),
            preview:None, activation:None, response:CString::default() };
        app.action(&json!({"action":"pin","id":"one"})).unwrap();
        app.action(&json!({"action":"dismiss","id":"one"})).unwrap();
        assert!(app.view()["rows"].as_array().unwrap().is_empty());
        app.settings = Settings::read(&app.path).unwrap();
        assert!(app.settings.pinned.contains("one"));
        app.action(&json!({"action":"all"})).unwrap();
        assert_eq!(app.view()["rows"][0]["dismissed"], true);
        app.action(&json!({"action":"dismiss","id":"one"})).unwrap();
        assert!(app.action(&json!({"action":"connect","id":"one","value":"codex://threads/other"})).is_err());
        app.action(&json!({"action":"connect","id":"one","value":"codex://threads/session-one"})).unwrap();
        app.action(&json!({"action":"search","value":"absent"})).unwrap();
        assert!(app.view()["rows"].as_array().unwrap().is_empty());
        app.action(&json!({"action":"search","value":"find"})).unwrap();
        assert_eq!(app.view()["rows"].as_array().unwrap().len(), 1);
        assert!(app.action(&json!({"action":"template","value":"{}"})).is_err());
        app.action(&json!({"action":"preview","value":ui::NIGHT})).unwrap();
        assert_eq!(Settings::read(&app.path).unwrap().template, ui::DEFAULT);
        app.action(&json!({"action":"template","value":ui::NIGHT})).unwrap();
        assert_eq!(Settings::read(&app.path).unwrap().template, ui::NIGHT);
        app.action(&json!({"action":"dismiss","id":"one"})).unwrap();
        let mut row = app.rows[0].clone(); row.request_marker = "r2:next".into();
        crate::publish_update(&app.updates, Ok(crate::Snapshot { rows:vec![row], warnings:vec![] }));
        app.poll();
        assert!(Settings::read(&app.path).unwrap().closed.is_empty());
        crate::publish_update(&app.updates, Err("collector failure".into())); app.poll();
        assert_eq!(app.rows.len(), 1); assert_eq!(app.error, "collector failure");
        app.path = dir.clone(); // Saving over a directory must fail and roll back.
        assert!(app.action(&json!({"action":"search","value":"lost"})).is_err());
        assert_eq!(app.settings.search, "find");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
