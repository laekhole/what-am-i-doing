use super::*;
use std::{collections::BTreeSet, time::Duration};

pub(super) const WAITING: usize = 103;
pub(super) const ACKNOWLEDGE: usize = 104;
pub(super) const NOTIFICATIONS: usize = 105;
pub(super) const PAUSE: usize = 106;
pub(super) const SESSION: usize = 200;

#[derive(Default)]
struct Reply {
    marker: String,
    unread: bool,
    pending: bool,
}

#[derive(Default)]
pub(super) struct State {
    replies: BTreeMap<String, Reply>,
    due: Option<Instant>,
    paused_until: Option<Instant>,
    balloon: Vec<(String, String)>,
    pub balloon_visible: bool,
    pub tooltip: String,
}

// Use the normal recent-session policy, independent of the window's search and filters.
fn rows(settings: &Settings, source: &[Row]) -> Vec<Row> {
    let mut settings = settings.clone();
    settings.search.clear();
    settings.state.clear();
    settings.agent.clear();
    settings.show_all = false;
    settings.show_aux = false;
    settings.show_hidden = false;
    settings.visible(source)
}

impl State {
    fn observe(&mut self, source: &[Row], included: &[Row], now: Instant) {
        let included: BTreeSet<_> = included.iter().map(|r| r.id.as_str()).collect();
        let current: BTreeMap<_, _> = source.iter().map(|r| (r.id.as_str(), r)).collect();
        for (id, reply) in &mut self.replies {
            // Retain the last completed request across disappearance/reappearance.
            reply.unread &= current.get(id.as_str()).is_some_and(|r| r.state == "waiting"
                && r.request_marker == reply.marker && included.contains(id.as_str()));
            reply.pending &= reply.unread;
        }
        for row in source.iter().filter(|r| r.state == "waiting"
            && r.observed_since_launch && !r.request_marker.is_empty()
            && ["transcript", "inferred_time", "orca_hook"].contains(&r.evidence.as_str()))
        {
            let reply = self.replies.entry(row.id.clone()).or_default();
            if reply.marker != row.request_marker {
                reply.marker.clone_from(&row.request_marker);
                reply.unread = included.contains(row.id.as_str());
                reply.pending = reply.unread;
            }
        }
        if self.replies.values().any(|r| r.pending) {
            self.due.get_or_insert(now + Duration::from_secs(2));
        } else {
            self.due = None;
        }
    }

    fn unread(&self, id: &str) -> bool {
        self.replies.get(id).is_some_and(|r| r.unread)
    }

    pub fn acknowledge(&mut self, id: &str) {
        if let Some(reply) = self.replies.get_mut(id) {
            reply.unread = false;
            reply.pending = false;
        }
    }

    fn take_pending(&mut self, now: Instant) -> Vec<(String, String)> {
        if self.due.is_none_or(|due| now < due) { return Vec::new(); }
        self.due = None;
        self.replies.iter_mut().filter_map(|(id, r)| {
            std::mem::take(&mut r.pending).then(|| (id.clone(), r.marker.clone()))
        }).collect()
    }
}

// Keep UTF-16 buffers terminated without cutting a surrogate pair or accepting embedded NULs.
fn copy_text(buffer: &mut [u16], text: &str) {
    buffer.fill(0);
    let mut used = 0;
    for ch in text.chars().filter(|ch| !ch.is_control()) {
        let mut encoded = [0; 2];
        let units = ch.encode_utf16(&mut encoded);
        if used + units.len() >= buffer.len() { break; }
        buffer[used..used + units.len()].copy_from_slice(units);
        used += units.len();
    }
}

fn label(text: &str, limit: usize) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut short: String = text.chars().filter(|ch| !ch.is_control()).take(limit).collect();
    if text.chars().count() > limit { short.push('…'); }
    short.replace('&', "&&")
}

fn data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    }
}

unsafe fn notification(hwnd: HWND, title: &str, body: &str) -> bool {
    let mut icon = data(hwnd);
    icon.uFlags = NIF_INFO | NIF_REALTIME;
    icon.dwInfoFlags = NIIF_INFO | NIIF_NOSOUND | NIIF_RESPECT_QUIET_TIME;
    copy_text(&mut icon.szInfoTitle, title);
    copy_text(&mut icon.szInfo, body);
    Shell_NotifyIconW(NIM_MODIFY, &icon) != 0
}

// A classic masked icon lets GDI add an opaque badge without damaging the logo's alpha.
unsafe fn badged_icon(base: HICON, warning: bool) -> HICON {
    let size = GetSystemMetrics(SM_CXSMICON).max(16);
    let screen = GetDC(null_mut());
    let dc = CreateCompatibleDC(screen);
    let color = CreateCompatibleBitmap(screen, size, size);
    let mask = CreateBitmap(size, size, 1, 1, null());
    ReleaseDC(null_mut(), screen);
    if dc.is_null() || color.is_null() || mask.is_null() {
        if !color.is_null() { DeleteObject(color); }
        if !mask.is_null() { DeleteObject(mask); }
        if !dc.is_null() { DeleteDC(dc); }
        return null_mut();
    }
    let old = SelectObject(dc, color);
    PatBlt(dc, 0, 0, size, size, BLACKNESS);
    DrawIconEx(dc, 0, 0, base, size, size, 0, null_mut(), DI_NORMAL);
    let badge = RECT { left: size / 2, top: size / 2, right: size, bottom: size };
    fill(dc, &badge, if warning { 0x00344dde } else { 0x00c76c13 });
    draw(dc, if warning { "!" } else { "•" }, badge, GetStockObject(DEFAULT_GUI_FONT) as HFONT,
        0x00ffffff, DT_SINGLELINE | DT_CENTER | DT_VCENTER);
    SelectObject(dc, mask);
    PatBlt(dc, 0, 0, size, size, WHITENESS);
    DrawIconEx(dc, 0, 0, base, size, size, 0, null_mut(), DI_MASK);
    fill(dc, &badge, 0);
    SelectObject(dc, old);
    let result = CreateIconIndirect(&ICONINFO {
        fIcon: 1, hbmColor: color, hbmMask: mask, ..Default::default()
    });
    DeleteObject(color);
    DeleteObject(mask);
    DeleteDC(dc);
    result
}

impl Window {
    #[cfg(test)]
    pub(super) unsafe fn check_tray_updates(&self, hwnd: HWND) {
        let mut sample = demo_rows();
        sample[0].observed_since_launch = true;
        sample[0].evidence = "transcript".into();
        self.settings.borrow_mut().search = "unrelated filter".into();
        publish_update(&self.updates, Ok(Snapshot { rows: sample.clone(), ..Snapshot::default() }));
        self.tick(hwnd);
        assert_eq!(IsWindowVisible(hwnd), 0, "a completion must not restore the hidden window");
        assert!(self.tray.borrow().tooltip.contains(tr("새 답변 1 · 내 차례 1 · 작업 중 1", "New 1 · Waiting 1 · Working 1")));
        let menu = CreatePopupMenu();
        let sessions = self.append_tray_menu(menu);
        let mut menu_text = [0u16; 256];
        let count = GetMenuStringW(menu, SESSION as u32, menu_text.as_mut_ptr(), menu_text.len() as i32, MF_BYCOMMAND);
        assert!(String::from_utf16_lossy(&menu_text[..count as usize]).starts_with(tr("새 답변 ·", "New answer ·")));
        assert_eq!(sessions[0].id, sample[0].id);
        DestroyMenu(menu);
        self.tray_command(hwnd, PAUSE, &[]);
        assert!(self.tray.borrow().tooltip.contains(tr("알림 쉬는 중", "Notifications paused")));
        self.tray_command(hwnd, NOTIFICATIONS, &[]);
        self.settings.borrow_mut().tray_hint_seen = true;
        self.save();
        let saved = Settings::read(&self.path).unwrap();
        assert!(!saved.notifications && saved.tray_hint_seen);
        assert!(self.tray.borrow().tooltip.contains(tr("알림 꺼짐", "Notifications off")));
        self.tray.borrow_mut().balloon = vec![(sample[0].id.clone(), sample[0].request_marker.clone())];
        window_proc(hwnd, TRAY_MESSAGE, 0, NIN_BALLOONUSERCLICK as isize);
        assert_ne!(IsWindowVisible(hwnd), 0);
        assert_eq!(self.selected().unwrap().id, sample[0].id);
        assert!(self.settings.borrow().search.is_empty());
        assert!(text(self.get(DETAIL)).contains(&sample[0].last_answer));
        assert!(!self.tray.borrow().unread(&sample[0].id));
        assert_eq!(self.selected().unwrap().state, "waiting");
        window_proc(hwnd, WM_CLOSE, 0, 0);
        sample[0].request_marker = "second-answer".into();
        publish_update(&self.updates, Ok(Snapshot { rows: sample, ..Snapshot::default() }));
        self.tick(hwnd);
        assert!(self.tray.borrow().tooltip.contains(tr("새 답변 1", "New 1")));
        publish_update(&self.updates, Err("collector stopped".into()));
        self.tick(hwnd);
        assert!(self.tray.borrow().tooltip.contains(tr("수집 중단/오류", "Collection failed")));
        self.tray_command(hwnd, ACKNOWLEDGE, &[]);
        assert!(self.tray.borrow().tooltip.contains(tr("새 답변 0", "New 0")));
        assert!(self.tray.borrow().tooltip.contains(tr("내 차례 1", "Waiting 1")));
        // Changed counts repeatedly replace the owned icon, without leaking GDI objects.
        #[link(name = "user32")]
        extern "system" { fn GetGuiResources(process: HANDLE, flags: u32) -> u32; }
        let before = GetGuiResources(-1isize as HANDLE, 0);
        for _ in 0..100 {
            self.tray.borrow_mut().tooltip.clear();
            self.refresh_tray(hwnd);
        }
        GdiFlush();
        assert!(GetGuiResources(-1isize as HANDLE, 0) <= before + 2);
    }

    pub(super) fn tray_rows(&self) -> Vec<Row> {
        rows(&self.settings.borrow(), &self.rows.borrow())
    }

    pub(super) unsafe fn refresh_tray(&self, hwnd: HWND) {
        let rows = self.tray_rows();
        let now = Instant::now();
        self.tray.borrow_mut().observe(&self.rows.borrow(), &rows, now);
        let (unread, paused) = {
            let tray = self.tray.borrow();
            (rows.iter().filter(|r| tray.unread(&r.id)).count(),
                tray.paused_until.is_some_and(|until| now < until))
        };
        let count = |state| rows.iter().filter(|r| r.state == state).count();
        let warning = !self.core_error.borrow().is_empty()
            || !self.collection_warnings.borrow().is_empty() || count("error") > 0;
        let mut summary = waid::trf!("waid · 새 답변 {unread} · 내 차례 {} · 작업 중 {} · 오류 {} · 미확인 {}", "waid · New {unread} · Waiting {} · Working {} · Error {} · Unknown {}",
            count("waiting"), count("working"), count("error"), count("unknown"));
        if self.loading.get() { summary = tr("waid · 세션을 가져오는 중", "waid · Loading sessions").into(); }
        if !self.core_error.borrow().is_empty() { summary.push_str(tr(" · 수집 중단/오류 (마지막 기록)", " · Collection failed (last snapshot)")); }
        else if !self.collection_warnings.borrow().is_empty() { summary.push_str(tr(" · 수집 경고", " · Collection warning")); }
        if !self.settings.borrow().notifications { summary.push_str(tr(" · 알림 꺼짐", " · Notifications off")); }
        else if paused { summary.push_str(tr(" · 알림 쉬는 중", " · Notifications paused")); }
        if self.tray.borrow().tooltip == summary { return; }
        let base = LoadImageW(GetModuleHandleW(null()), 1usize as _, IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON), LR_SHARED) as HICON;
        if base.is_null() { return; }
        let badge = if warning || unread > 0 { badged_icon(base, warning) } else { null_mut() };
        let mut icon = data(hwnd);
        icon.uFlags = NIF_TIP | NIF_ICON | NIF_SHOWTIP;
        icon.hIcon = if badge.is_null() { base } else { badge };
        copy_text(&mut icon.szTip, &summary);
        let success = Shell_NotifyIconW(NIM_MODIFY, &icon) != 0;
        if !badge.is_null() { DestroyIcon(badge); }
        if success { self.tray.borrow_mut().tooltip = summary; }
    }

    pub(super) unsafe fn tray_tick(&self, hwnd: HWND) {
        let now = Instant::now();
        if self.tray.borrow().paused_until.is_some_and(|until| now >= until) {
            self.tray.borrow_mut().paused_until = None;
            self.refresh_tray(hwnd);
        }
        // Shell callbacks have no notification ID; keep one visible balloon's target stable.
        if self.tray.borrow().balloon_visible { return; }
        let pending = self.tray.borrow_mut().take_pending(now);
        if pending.is_empty() { return; }
        let paused = self.tray.borrow().paused_until.is_some_and(|until| now < until);
        let mut availability = QUNS_NOT_PRESENT;
        if self.demo || !self.settings.borrow().notifications || paused
            || (GetForegroundWindow() == hwnd && IsWindowVisible(hwnd) != 0 && IsIconic(hwnd) == 0)
            || !self.core_error.borrow().is_empty()
            || SHQueryUserNotificationState(&mut availability) < 0
            || availability != QUNS_ACCEPTS_NOTIFICATIONS
        { return; }
        let rows = self.tray_rows();
        let message = if pending.len() == 1 {
            let Some(row) = rows.iter().find(|r| r.id == pending[0].0) else { return; };
            waid::trf!("{} · {}의 답변이 도착했습니다. 눌러서 세션 열기", "{} · New answer from {}. Click to open the session",
                label(row.project(), 40).replace("&&", "&"), label(&row.agent, 24).replace("&&", "&"))
        } else {
            waid::trf!("{}개 세션에 새 답변이 도착했습니다. 눌러서 내 차례 목록 보기", "New answers in {} sessions. Click to view waiting sessions", pending.len())
        };
        if notification(hwnd, tr("waid · 답변 도착", "waid · New answer"), &message) {
            self.tray.borrow_mut().balloon = pending;
        } else {
            *self.notice.borrow_mut() = tr("답변 알림을 표시하지 못했습니다 · 트레이에서 새 답변을 확인하세요", "Could not show the notification · Check the tray for new answers").into();
            InvalidateRect(hwnd, null(), 0);
        }
    }

    pub(super) unsafe fn show_tray_waiting(&self, hwnd: HWND) {
        restore_window(self, hwnd);
        if self.editing.get() { return; }
        self.busy.set(true);
        {
            let mut settings = self.settings.borrow_mut();
            settings.search.clear();
            settings.agent.clear();
            settings.state = "waiting".into();
            settings.show_all = false;
            settings.show_hidden = false;
            settings.show_aux = false;
        }
        set_text(self.get(SEARCH), "");
        send(self.get(AGENT), CB_SETCURSEL, 0, 0);
        let status = ui::STATES.iter().position(|s| *s == "waiting").unwrap() + 1;
        send(self.get(STATUS), CB_SETCURSEL, status, 0);
        for id in [AUX, HIDDEN] { send(self.get(id), BM_SETCHECK, BST_UNCHECKED as usize, 0); }
        self.busy.set(false);
        self.changed();
        self.rebuild(hwnd);
    }

    pub(super) unsafe fn open_tray_session(&self, hwnd: HWND, id: &str) {
        self.show_tray_waiting(hwnd);
        if self.editing.get() { return; }
        let index = self.visible.borrow().iter().position(|r| r.id == id);
        if let Some(index) = index {
            send(self.get(LIST), LB_SETCURSEL, index, 0);
            self.rebuild(hwnd);
            self.activate_selected(hwnd);
        } else {
            *self.notice.borrow_mut() = tr("이 세션은 더 이상 내 차례가 아닙니다 · 현재 목록을 표시합니다", "This session is no longer waiting · Showing the current list").into();
        }
    }

    pub(super) unsafe fn open_tray_notification(&self, hwnd: HWND) {
        self.tray.borrow_mut().balloon_visible = false;
        let balloon = std::mem::take(&mut self.tray.borrow_mut().balloon);
        if let [(id, marker)] = balloon.as_slice() {
            let current = self.rows.borrow().iter().any(|r| &r.id == id
                && &r.request_marker == marker && r.state == "waiting");
            if current { self.open_tray_session(hwnd, id); return; }
        }
        self.show_tray_waiting(hwnd);
    }

    pub(super) unsafe fn append_tray_menu(&self, menu: HMENU) -> Vec<Row> {
        let mut waiting: Vec<_> = self.tray_rows().into_iter().filter(|r| r.state == "waiting").collect();
        let tray = self.tray.borrow();
        waiting.sort_by_key(|r| !tray.unread(&r.id));
        AppendMenuW(menu, MF_STRING | MF_DISABLED, 0, wide(&tray.tooltip).as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        for (i, row) in waiting.iter().take(8).enumerate() {
            let text = format!("{} · {} · {} · {}",
                if tray.unread(&row.id) { tr("새 답변", "New answer") } else { tr("내 차례", "Waiting") },
                label(row.project(), 22), label(&row.agent, 16), label(&row.task, 38));
            AppendMenuW(menu, MF_STRING, SESSION + i, wide(&text).as_ptr());
        }
        let text = if waiting.is_empty() { tr("내 차례인 세션이 없습니다", "No waiting sessions") } else { tr("내 차례 모두 보기", "Show all waiting sessions") };
        AppendMenuW(menu, MF_STRING, WAITING, wide(text).as_ptr());
        AppendMenuW(menu, MF_STRING | if tray.replies.values().any(|r| r.unread) { 0 } else { MF_GRAYED },
            ACKNOWLEDGE, wide(tr("새 답변 표시 모두 지우기", "Clear new-answer markers")).as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING | if self.settings.borrow().notifications { MF_CHECKED } else { 0 },
            NOTIFICATIONS, wide(tr("답변 도착 알림", "Reply notifications")).as_ptr());
        let paused = tray.paused_until.is_some_and(|until| Instant::now() < until);
        AppendMenuW(menu, MF_STRING, PAUSE, wide(if paused { tr("알림 다시 받기", "Resume notifications") } else { tr("알림 1시간 쉬기", "Pause notifications for 1 hour") }).as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        waiting.truncate(8);
        waiting
    }

    pub(super) unsafe fn tray_command(&self, hwnd: HWND, command: usize, sessions: &[Row]) {
        match command {
            WAITING => self.show_tray_waiting(hwnd),
            ACKNOWLEDGE => {
                let mut tray = self.tray.borrow_mut();
                for reply in tray.replies.values_mut() { reply.unread = false; reply.pending = false; }
            }
            NOTIFICATIONS => {
                let enabled = !self.settings.borrow().notifications;
                self.settings.borrow_mut().notifications = enabled;
                self.changed();
            }
            PAUSE => {
                let now = Instant::now();
                let mut tray = self.tray.borrow_mut();
                tray.paused_until = if tray.paused_until.is_some_and(|until| now < until) {
                    None
                } else { Some(now + Duration::from_secs(3600)) };
            }
            id if id >= SESSION => {
                if let Some(row) = sessions.get(id - SESSION) { self.open_tray_session(hwnd, &row.id); }
            }
            _ => {}
        }
        self.refresh_tray(hwnd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "shows a sample Windows notification; requires an interactive desktop accepting notifications"]
    fn windows_notification_reports_display_through_shell_callback() {
        let _desktop = super::super::tests::DESKTOP_TEST.lock().unwrap_or_else(|e| e.into_inner());
        let state = Window::new(Updates::default(), std::env::temp_dir().join(
            format!("waid-tray-notification-check-{}.json", std::process::id())), false);
        unsafe {
            let hwnd = create_window(&state).unwrap();
            add_tray_icon(hwnd).unwrap();
            ShowWindow(hwnd, SW_HIDE);
            let mut availability = QUNS_NOT_PRESENT;
            assert_eq!(SHQueryUserNotificationState(&mut availability), 0);
            eprintln!("Windows notification availability: {availability}");
            assert_eq!(availability, QUNS_ACCEPTS_NOTIFICATIONS);
            let row = Row { id: "notification-check".into(), title: "샘플 검증".into(),
                agent: "테스트 에이전트".into(), request_marker: "sample-request".into(),
                state: "waiting".into(), evidence: "transcript".into(), observed_since_launch: true,
                ..Row::default() };
            publish_update(&state.updates, Ok(Snapshot { rows: vec![row], ..Snapshot::default() }));
            state.tick(hwnd);
            state.tray.borrow_mut().due = Some(Instant::now());
            state.tick(hwnd);
            assert_eq!(state.tray.borrow().balloon.len(), 1, "the normal hidden-window delivery path must send the notification");
            let deadline = Instant::now() + Duration::from_secs(12);
            let mut displayed = false;
            while Instant::now() < deadline && !displayed {
                let mut message = MSG::default();
                while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                    displayed |= state.tray.borrow().balloon_visible;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            DestroyWindow(hwnd);
            eprintln!("Shell NIN_BALLOONSHOW observed: {displayed}");
            assert!(displayed, "Windows accepted the request but did not report a displayed notification");
        }
    }

    #[test]
    fn replies_are_per_request_observed_eligible_and_acknowledged_separately() {
        let now = Instant::now();
        let mut tray = State::default();
        let mut row = Row { id: "a".into(), request_marker: "r1".into(), state: "waiting".into(),
            evidence: "transcript".into(), ..Row::default() };
        let observe = |tray: &mut State, row: &Row, included: bool| {
            tray.observe(std::slice::from_ref(row), if included { std::slice::from_ref(row) } else { &[] }, now);
        };
        observe(&mut tray, &row, true);
        assert!(!tray.unread("a"), "pre-launch answers are not new");
        row.observed_since_launch = true;
        observe(&mut tray, &row, true);
        assert!(tray.unread("a"));
        assert!(tray.take_pending(now).is_empty());
        assert_eq!(tray.take_pending(now + Duration::from_secs(2)).len(), 1);
        observe(&mut tray, &row, true);
        assert!(tray.take_pending(now + Duration::from_secs(3)).is_empty());
        tray.acknowledge("a");
        observe(&mut tray, &row, true);
        assert!(!tray.unread("a"));
        assert_eq!(row.state, "waiting");
        row.request_marker = "r2".into();
        observe(&mut tray, &row, true);
        assert!(tray.unread("a"), "waiting to waiting with a new request is a new reply");
        row.state = "working".into();
        observe(&mut tray, &row, true);
        assert!(!tray.unread("a"));
        assert!(tray.take_pending(now + Duration::from_secs(3)).is_empty());
        row.state = "waiting".into();
        row.request_marker = "r3".into();
        observe(&mut tray, &row, false);
        observe(&mut tray, &row, true);
        assert!(!tray.unread("a"), "unhiding does not replay an old completion");
        row.request_marker = "r4".into();
        row.evidence = "terminal_screen".into();
        observe(&mut tray, &row, true);
        assert!(!tray.unread("a"));
        row.evidence = "orca_hook".into();
        let other = Row { id: "b".into(), ..row.clone() };
        tray.observe(&[row.clone(), other.clone()], &[row.clone(), other], now);
        assert_eq!(tray.take_pending(now + Duration::from_secs(2)).len(), 2);
        tray.observe(&[], &[], now);
        observe(&mut tray, &row, true);
        assert!(!tray.unread("a"), "reappearing logs do not repeat a reply");
    }

    #[test]
    fn tray_scope_ignores_filters_and_text_is_bounded() {
        let mut settings = Settings::default();
        settings.search = "does not match".into();
        settings.state = "working".into();
        settings.agent = "other".into();
        settings.show_all = true;
        settings.show_aux = true;
        settings.show_hidden = true;
        let normal = Row { id: "normal".into(), state: "waiting".into(), ..Row::default() };
        let hidden = Row { id: "hidden".into(), ..normal.clone() };
        settings.hidden.insert(hidden.id.clone());
        let closed = Row { id: "closed".into(), ..normal.clone() };
        settings.close(&closed).unwrap();
        let aux = Row { id: "aux".into(), auxiliary: true, ..normal.clone() };
        assert_eq!(rows(&settings, &[normal.clone(), hidden, closed, aux]), vec![normal]);
        assert_eq!(label("a&b\r\nsecret", 4), "a&&b …");
        let mut buffer = [0u16; 4];
        copy_text(&mut buffer, "한😀글");
        assert_eq!(String::from_utf16(&buffer[..3]).unwrap(), "한😀");
        copy_text(&mut buffer[..3], "한😀글");
        assert_eq!(buffer[0], '한' as u16);
        assert_eq!(buffer[1], 0);
    }
}
