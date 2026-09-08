use super::{
    ui::{self, Settings, Skin},
    visual::{self, Visuals},
    publish_update, Row, Snapshot, Updates,
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    io,
    path::PathBuf,
    ptr::{null, null_mut},
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::{Dwm::*, Gdi::*},
    System::{DataExchange::*, LibraryLoader::GetModuleHandleW, Memory::*},
    UI::{
        Controls::{Dialogs::*, *},
        HiDpi::*,
        Input::KeyboardAndMouse::*,
        Shell::*,
        WindowsAndMessaging::*,
    },
};

mod accordion;
const SEARCH: usize = 10;
const STATUS: usize = 11;
const AGENT: usize = 12;
const PIN: usize = 13;
const HIDE: usize = 14;
const AUX: usize = 15;
const HIDDEN: usize = 16;
const LIST: usize = 20;
const DETAIL: usize = 21;
const EDITOR: usize = 30;
const PREVIEW: usize = 31;
const APPLY: usize = 32;
const RESET: usize = 33;
const IMPORT: usize = 34;
const EXPORT: usize = 35;
const DAY: usize = 36;
const NIGHT: usize = 37;
const CLOSE_EDITOR: usize = 38;
const ALL: usize = 40;
const MORE: usize = 41;
const CLOSE_SESSION: usize = 42;
const BACK: usize = 43;
const SHOW_DETAIL: usize = 44;
const FILTER: usize = 45;
const TBM_GETPOS: u32 = WM_USER; // commctrl.h: trackbar position
const TOPMOST: usize = 46;
const OPACITY: usize = 47;
const OPACITY_SLIDER: usize = 48;
const MINIMIZE: usize = 49;
const CLOSE_WINDOW: usize = 50;
const TWO_COLUMNS: usize = 51;
const ASSOCIATE: usize = 60;
const CLASS: &str = "waid.sessions.v2";
const TRAY_MESSAGE: u32 = WM_APP + 1;
const TRAY_ID: u32 = 1;
const TRAY_OPEN: usize = 100;
const TRAY_EXIT: usize = 101;
const TRAY_LICENSE: usize = 102;
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
unsafe fn text(hwnd: HWND) -> String {
    let n = GetWindowTextLengthW(hwnd).max(0) as usize;
    let mut b = vec![0u16; n + 1];
    let used = GetWindowTextW(hwnd, b.as_mut_ptr(), b.len() as i32).max(0) as usize;
    String::from_utf16_lossy(&b[..used])
}
unsafe fn set_text(hwnd: HWND, s: &str) {
    let windows_text = s.replace("\r\n", "\n").replace('\n', "\r\n");
    SetWindowTextW(hwnd, wide(&windows_text).as_ptr());
}
unsafe fn clipboard_text(hwnd: HWND) -> Result<String, String> {
    if OpenClipboard(hwnd) == 0 {
        return Err("클립보드를 읽을 수 없습니다. 링크를 복사한 뒤 다시 시도하세요.".into());
    }
    let result = (|| {
        let handle = GetClipboardData(13); // CF_UNICODETEXT
        let bytes = if handle.is_null() { 0 } else { GlobalSize(handle) };
        if bytes < 2 || bytes > 4096 || bytes % 2 != 0 {
            return Err("ChatGPT 세션 링크를 텍스트로 복사하세요.".into());
        }
        let data = GlobalLock(handle) as *const u16;
        if data.is_null() { return Err("복사한 링크를 읽을 수 없습니다.".into()); }
        let text = std::slice::from_raw_parts(data, bytes / 2);
        let result = text.iter().position(|c| *c == 0)
            .ok_or_else(|| "복사한 링크가 올바른 텍스트가 아닙니다.".to_string())
            .and_then(|end| String::from_utf16(&text[..end]).map_err(|_| "복사한 링크가 올바른 텍스트가 아닙니다.".into()));
        GlobalUnlock(handle);
        result.map(|text| text.trim().to_owned())
    })();
    CloseClipboard();
    result
}
unsafe fn fill(dc: HDC, rect: &RECT, color: u32) {
    let brush = CreateSolidBrush(color);
    FillRect(dc, rect, brush);
    DeleteObject(brush);
}
unsafe fn draw(dc: HDC, s: &str, mut rect: RECT, font: HFONT, color: u32, flags: u32) {
    let old = SelectObject(dc, font);
    SetBkMode(dc, TRANSPARENT as i32);
    SetTextColor(dc, color);
    DrawTextW(dc, wide(s).as_ptr(), -1, &mut rect, flags | DT_NOPREFIX);
    SelectObject(dc, old);
}

struct Window {
    taskbar_created: u32,
    date_tick: Cell<i64>,
    feed: accordion::Feed,
    activation: RefCell<Option<(String, std::sync::mpsc::Receiver<Result<String, String>>)>>,
    visuals: Visuals,
    opacity_open: Cell<bool>,
    expanded: Cell<bool>,
    menu_open: Cell<bool>,
    compact_rect: Cell<RECT>,
    tile_height: Cell<i32>,
    force_rebuild: Cell<bool>,
    controls: RefCell<BTreeMap<usize, HWND>>,
    ready: Cell<bool>,
    busy: Cell<bool>,
    editing: Cell<bool>,
    body: Cell<HFONT>,
    heading: Cell<HFONT>,
    surface: Cell<HBRUSH>,
    rows: RefCell<Vec<Row>>,
    visible: RefCell<Vec<Row>>,
    settings: RefCell<Settings>,
    skin: RefCell<Skin>,
    updates: Updates,
    notice: RefCell<String>,
    core_error: RefCell<String>,
    collection_warnings: RefCell<Vec<String>>,
    loading: Cell<bool>,
    path: PathBuf,
    dirty: Cell<Option<Instant>>,
    demo: bool,
}
impl Window {
    fn new(updates: Updates, path: PathBuf, demo: bool) -> Self {
        let (settings, notice) = match Settings::read(&path) {
            Ok(s) => (s, String::new()),
            Err(e) if path.exists() => (
                Settings::default(),
                format!("설정을 읽지 못해 기본값으로 열었습니다: {e}"),
            ),
            Err(_) => (Settings::default(), String::new()),
        };
        let skin = Skin::parse(&settings.template).expect("bundled default template");
        Self {
            taskbar_created: unsafe { RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()) },
            date_tick: Cell::new(0),
            feed: accordion::Feed::default(),
            activation: RefCell::new(None),
            visuals: Visuals::new(),
            opacity_open: Cell::new(false),
            expanded: Cell::new(false),
            menu_open: Cell::new(false),
            compact_rect: Cell::new(RECT::default()),
            tile_height: Cell::new(100),
            force_rebuild: Cell::new(true),
            controls: RefCell::new(BTreeMap::new()),
            ready: Cell::new(false),
            busy: Cell::new(false),
            editing: Cell::new(false),
            body: Cell::new(null_mut()),
            heading: Cell::new(null_mut()),
            surface: Cell::new(null_mut()),
            rows: RefCell::new(Vec::new()),
            visible: RefCell::new(Vec::new()),
            settings: RefCell::new(settings),
            skin: RefCell::new(skin),
            updates,
            notice: RefCell::new(notice),
            core_error: RefCell::new(String::new()),
            collection_warnings: RefCell::new(Vec::new()),
            loading: Cell::new(true),
            path,
            dirty: Cell::new(None),
            demo,
        }
    }
    fn get(&self, id: usize) -> HWND {
        self.controls
            .borrow()
            .get(&id)
            .copied()
            .unwrap_or(null_mut())
    }
    unsafe fn px(&self, hwnd: HWND, n: i32) -> i32 {
        n * GetDpiForWindow(hwnd).max(96) as i32 / 96
    }
    unsafe fn add(
        &self,
        hwnd: HWND,
        id: usize,
        class: &str,
        label: &str,
        style: u32,
    ) -> io::Result<()> {
        let child = CreateWindowExW(
            0,
            wide(class).as_ptr(),
            wide(label).as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | style,
            0,
            0,
            1,
            1,
            hwnd,
            id as HMENU,
            GetModuleHandleW(null()),
            null(),
        );
        if child.is_null() {
            return Err(io::Error::last_os_error());
        }
        self.controls.borrow_mut().insert(id, child);
        Ok(())
    }
    unsafe fn fonts(&self, hwnd: HWND) {
        let skin = self.skin.borrow().clone();
        let mut old_fonts = Vec::new();
        for (slot, size, weight, face, resource) in [
            (
                &self.body,
                skin.font_size,
                FW_SEMIBOLD,
                "Pretendard SemiBold",
                self.visuals.fonts[1],
            ),
            (
                &self.heading,
                skin.font_size + 3,
                FW_BOLD,
                "Pretendard",
                self.visuals.fonts[0],
            ),
        ] {
            let font = CreateFontW(
                -self.px(hwnd, size),
                0,
                0,
                0,
                weight as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                OUT_TT_PRECIS as u32,
                0,
                CLEARTYPE_QUALITY as u32,
                0,
                wide(if resource.is_null() {
                    "Malgun Gothic"
                } else {
                    face
                })
                .as_ptr(),
            );
            if !font.is_null() {
                let old = slot.replace(font);
                if !old.is_null() {
                    old_fonts.push(old);
                }
            }
        }
        let children: Vec<HWND> = self.controls.borrow().values().copied().collect();
        for child in children {
            send(child, WM_SETFONT, self.body.get() as usize, 1);
        }
        let buttons = self.feed.buttons.borrow().clone();
        for child in buttons {
            send(child, WM_SETFONT, self.body.get() as usize, 1);
        }
        for font in old_fonts {
            DeleteObject(font);
        }
        let old = self.surface.replace(CreateSolidBrush(skin.surface));
        if !old.is_null() {
            DeleteObject(old);
        }
        self.row_height(self.px(hwnd, skin.row_height()));
        send(self.get(LIST), LVM_SETBKCOLOR, 0, skin.background as isize);
        self.force_rebuild.set(true);
    }
    unsafe fn row_height(&self, height: i32) {
        self.tile_height.set(height);
        self.fit_list();
    }
    unsafe fn fit_list(&self) {
        let list = self.get(LIST);
        let mut bounds = RECT::default();
        GetWindowRect(list, &mut bounds);
        // Reserve the scrollbar gutter so adding sessions never changes the column count.
        let info = LVTILEVIEWINFO {
            cbSize: std::mem::size_of::<LVTILEVIEWINFO>() as u32,
            dwMask: LVTVIM_TILESIZE,
            dwFlags: LVTVIF_FIXEDSIZE,
            sizeTile: SIZE {
                cx: ((bounds.right
                    - bounds.left
                    - GetSystemMetricsForDpi(SM_CXVSCROLL, GetDpiForWindow(list))
                    - 8)
                    / if self.settings.borrow().two_columns {
                        2
                    } else {
                        1
                    })
                .max(1),
                cy: self.tile_height.get() + 2, // Native tile bounds exclude a two-pixel gap.
            },
            ..Default::default()
        };
        send(list, LVM_SETTILEVIEWINFO, 0, &info as *const _ as isize);
        send(list, LVM_ARRANGE, LVA_DEFAULT as usize, 0);
    }
    unsafe fn layout(&self, hwnd: HWND) {
        if !self.ready.get() {
            return;
        }
        let mut r = RECT::default();
        GetClientRect(hwnd, &mut r);
        let dpi = GetDpiForWindow(hwnd).max(96) as i32;
        let w = r.right * 96 / dpi;
        let h = r.bottom * 96 / dpi;
        let mv = |id: usize, x: i32, y: i32, width: i32, height: i32| {
            MoveWindow(
                self.get(id),
                self.px(hwnd, x),
                self.px(hwnd, y),
                self.px(hwnd, width.max(1)),
                self.px(hwnd, height.max(1)),
                1,
            );
        };
        let expanded = self.expanded.get();
        let two_columns = self.settings.borrow().two_columns;
        ShowWindow(self.get(accordion::FEED), if !expanded && !two_columns { SW_SHOWNA } else { SW_HIDE });
        mv(TOPMOST, w - 184, 10, 88, 30);
        mv(MINIMIZE, w - 82, 10, 30, 30);
        mv(CLOSE_WINDOW, w - 46, 10, 30, 30);
        mv(TWO_COLUMNS, w - 88, 62, 60, 24);
        mv(OPACITY, 16, 104, 104, 32);
        let extra = if self.opacity_open.get() { 36 } else { 0 };
        ShowWindow(
            self.get(OPACITY_SLIDER),
            if extra > 0 { SW_SHOWNA } else { SW_HIDE },
        );
        mv(OPACITY_SLIDER, 128, 146, w - 148, 24);
        for id in [
            SEARCH,
            STATUS,
            AGENT,
            PIN,
            CLOSE_SESSION,
            AUX,
            HIDDEN,
            EDITOR,
            DETAIL,
            BACK,
            ASSOCIATE,
        ] {
            ShowWindow(self.get(id), if expanded { SW_SHOWNA } else { SW_HIDE });
        }
        ShowWindow(self.get(HIDE), SW_HIDE);
        for id in [
            PREVIEW,
            APPLY,
            RESET,
            IMPORT,
            EXPORT,
            DAY,
            NIGHT,
            CLOSE_EDITOR,
        ] {
            ShowWindow(
                self.get(id),
                if expanded && self.editing.get() {
                    SW_SHOWNA
                } else {
                    SW_HIDE
                },
            );
        }
        ShowWindow(self.get(MORE), if expanded { SW_HIDE } else { SW_SHOWNA });
        if !expanded {
            mv(ALL, w - 156, 104, 96, 32);
            mv(MORE, w - 52, 104, 36, 32);
            ShowWindow(self.get(FILTER), SW_SHOWNA);
            mv(FILTER, 128, 104, (w - 292).min(76), 32);
            let menu_open = self.menu_open.get();
            let actions = [SHOW_DETAIL, PIN, CLOSE_SESSION, ASSOCIATE, EDITOR];
            for (i, id) in actions.into_iter().enumerate() {
                ShowWindow(self.get(id), if menu_open { SW_SHOWNA } else { SW_HIDE });
                let width = (w - 50) / actions.len() as i32;
                mv(id, 16 + i as i32 * (width + 6), 146 + extra, width, 32);
            }
            let top = if menu_open { 190 } else { 148 } + extra;
            mv(LIST, 13, top, w - 26, h - top - 36);
            mv(accordion::FEED, 13, top, w - 26, h - top - 36);
            self.layout_feed(hwnd);
            ShowWindow(self.get(LIST), if two_columns { SW_SHOWNA } else { SW_HIDE });
            self.fit_list();
            InvalidateRect(hwnd, null(), 1);
            return;
        }
        for id in [SHOW_DETAIL, FILTER] {
            ShowWindow(self.get(id), SW_HIDE);
        }
        mv(ALL, w - 222, 104, 96, 32);
        mv(BACK, w - 116, 104, 96, 32);
        let search = (w - 370).max(220);
        mv(SEARCH, 32, 156 + extra, search - 24, 24);
        mv(STATUS, search + 32, 150 + extra, 142, 300);
        mv(AGENT, search + 186, 150 + extra, w - search - 206, 300);
        mv(PIN, 20, 196 + extra, 90, 32);
        mv(CLOSE_SESSION, 118, 196 + extra, 100, 32);
        mv(AUX, 232, 196 + extra, 125, 32);
        mv(HIDDEN, 366, 196 + extra, 125, 32);
        mv(ASSOCIATE, w - 224, 196 + extra, 56, 32);
        mv(EDITOR, w - 160, 196 + extra, 140, 32);
        let top = 244 + extra;
        let bottom = h - 40;
        if w >= 1000 {
            let left = (w * 56 / 100).max(450);
            mv(LIST, 20, top, left - 32, bottom - top);
            mv(
                DETAIL,
                left + 8,
                top,
                w - left - 28,
                bottom - top - if self.editing.get() { 86 } else { 0 },
            );
        } else {
            let detail_h =
                (bottom - top - 116).clamp(100, if self.editing.get() { 220 } else { 160 });
            mv(
                LIST,
                20,
                top,
                w - 40,
                (bottom - top - detail_h - 16).max(100),
            );
            mv(
                DETAIL,
                20,
                (bottom - detail_h).max(top + 116),
                w - 40,
                detail_h - if self.editing.get() { 86 } else { 0 },
            );
        }
        let mut detail_bounds = RECT::default();
        GetClientRect(self.get(DETAIL), &mut detail_bounds);
        let mut detail_window = RECT::default();
        GetWindowRect(self.get(DETAIL), &mut detail_window);
        let region = CreateRoundRectRgn(
            0,
            0,
            detail_window.right - detail_window.left + 1,
            detail_window.bottom - detail_window.top + 1,
            self.px(hwnd, 32),
            self.px(hwnd, 32),
        );
        if SetWindowRgn(self.get(DETAIL), region, 1) == 0 {
            DeleteObject(region);
        }
        let inset = self.px(hwnd, 14);
        detail_bounds.left += inset;
        detail_bounds.top += inset;
        detail_bounds.right -= inset;
        detail_bounds.bottom -= inset;
        send(
            self.get(DETAIL),
            EM_SETRECT,
            0,
            &detail_bounds as *const _ as isize,
        );
        self.fit_list();
        let mut detail = RECT::default();
        GetWindowRect(self.get(DETAIL), &mut detail);
        let mut p = POINT {
            x: detail.left,
            y: detail.bottom,
        };
        ScreenToClient(hwnd, &mut p);
        let x = p.x * 96 / dpi;
        let y = p.y * 96 / dpi + 8;
        let available = (w - x - 20).max(360);
        let bw = (available - 18) / 4;
        for (i, id) in [PREVIEW, APPLY, RESET, CLOSE_EDITOR]
            .into_iter()
            .enumerate()
        {
            mv(id, x + i as i32 * (bw + 6), y, bw, 30);
        }
        for (i, id) in [DAY, NIGHT, IMPORT, EXPORT].into_iter().enumerate() {
            mv(id, x + i as i32 * (bw + 6), y + 38, bw, 30);
        }
        InvalidateRect(hwnd, null(), 1);
    }
    unsafe fn expand(&self, hwnd: HWND, open: bool) {
        if self.expanded.replace(open) == open {
            return;
        }
        if open {
            self.menu_open.set(false);
            let mut rect = RECT::default();
            GetWindowRect(hwnd, &mut rect);
            self.compact_rect.set(rect);
            SetWindowPos(
                hwnd,
                null_mut(),
                0,
                0,
                self.px(hwnd, 800),
                self.px(hwnd, 760),
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        } else {
            let rect = self.compact_rect.get();
            SetWindowPos(
                hwnd,
                null_mut(),
                rect.left,
                rect.top,
                (rect.right - rect.left).max(self.px(hwnd, 480)),
                (rect.bottom - rect.top).max(self.px(hwnd, 440)),
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        self.layout(hwnd);
        if !open {
            SetFocus(self.session_focus());
        }
    }
    unsafe fn menu(&self, hwnd: HWND) {
        self.menu_open.set(!self.menu_open.get());
        self.layout(hwnd);
        SetFocus(self.get(if self.menu_open.get() {
            SHOW_DETAIL
        } else {
            LIST
        }));
    }
    unsafe fn selected(&self) -> Option<Row> {
        let index = send(self.get(LIST), LB_GETCURSEL, 0, 0);
        if index < 0 {
            None
        } else {
            self.visible.borrow().get(index as usize).cloned()
        }
    }
    unsafe fn session_focus(&self) -> HWND {
        if self.expanded.get() || self.settings.borrow().two_columns {
            self.get(LIST)
        } else {
            let index = send(self.get(LIST), LB_GETCURSEL, 0, 0).max(0) as usize;
            self.feed.buttons.borrow().get(index)
                .copied().unwrap_or(self.get(FILTER))
        }
    }
    unsafe fn activate_selected(&self, hwnd: HWND) {
        if self.activation.borrow().is_some() || self.editing.get() {
            return;
        }
        let Some(row) = self.selected() else { return };
        let target = self.settings.borrow().session_targets.get(&row.id).cloned();
        if self.demo
            || (row.session_id.is_empty() && target.is_none())
            || self.settings.borrow().closed.contains_key(&row.id)
        {
            *self.notice.borrow_mut() =
                "열린 창·탭과의 연결을 확인할 수 없어 상세 내용을 표시합니다.".into();
            self.command(hwnd, SHOW_DETAIL, 0);
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        *self.activation.borrow_mut() = Some((row.id.clone(), rx));
        *self.notice.borrow_mut() = "세션이 열린 창·탭을 확인하고 있습니다…".into();
        InvalidateRect(hwnd, null(), 1);
        std::thread::spawn(move || {
            let result = if let Some(target) = target {
                super::activate::open_associated(&row, &target)
            } else {
                super::activate::open(&row).map(|()| "기존 세션 탭으로 이동했습니다.".into())
            };
            let _ = tx.send(result);
        });
    }
    fn set_session_target(&self, row: &Row, target: Option<serde_json::Value>) -> Result<(), String> {
        let mut settings = self.settings.borrow_mut();
        if let Some(target) = target {
            if settings.session_targets.len() >= 1024 && !settings.session_targets.contains_key(&row.id) {
                return Err("창 연결은 최대 1024개까지 저장할 수 있습니다.".into());
            }
            settings.session_targets.insert(row.id.clone(), target);
        } else {
            settings.session_targets.remove(&row.id);
        }
        self.changed();
        Ok(())
    }
    unsafe fn association_menu(&self, hwnd: HWND) {
        let Some(row) = self.selected() else { return };
        if self.demo || row.state == "done" {
            *self.notice.borrow_mut() = "실제 세션을 선택한 뒤 창이나 링크를 연결하세요.".into();
            InvalidateRect(hwnd, null(), 0);
            return;
        }
        // ponytail: explicit menu scan can take three seconds; move to a worker if it becomes disruptive.
        let windows = super::activate::windows();
        let menu = CreatePopupMenu();
        if menu.is_null() { return; }
        AppendMenuW(menu, MF_STRING | MF_DISABLED, 0, wide("직접 연결 · 창 안의 대화는 직접 확인하세요").as_ptr());
        AppendMenuW(menu, MF_STRING, 1, wide("복사한 ChatGPT 세션 링크 연결").as_ptr());
        AppendMenuW(menu, MF_STRING, 2, wide("연결 해제 · Orca 자동 연결 사용").as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        for (index, target) in windows.iter().enumerate() {
            AppendMenuW(menu, MF_STRING, index + 3, wide(&target["label"].as_str().unwrap_or("앱 창").replace('&', "&&")).as_ptr());
        }
        if windows.is_empty() {
            AppendMenuW(menu, MF_STRING | MF_DISABLED, 0, wide("연결 가능한 앱·PowerShell 창이 없습니다").as_ptr());
        }
        let mut rect = RECT::default();
        GetWindowRect(self.get(ASSOCIATE), &mut rect);
        let command = TrackPopupMenu(menu, TPM_RETURNCMD, rect.left, rect.bottom, 0, hwnd, null()) as usize;
        DestroyMenu(menu);
        let result = match command {
            0 => return,
            1 => clipboard_text(hwnd).and_then(|url| {
                super::activate::validate_chatgpt_link(&row, &url)?;
                self.set_session_target(&row, Some(serde_json::json!({"kind":"chatgpt_link", "url":url})))?;
                Ok("ChatGPT 세션 링크를 연결했습니다. 로고를 누르면 앱에 링크를 전달합니다.")
            }),
            2 => self.set_session_target(&row, None).map(|()| "직접 연결을 해제했습니다. Orca의 정확한 세션 연결을 자동으로 확인합니다."),
            _ => {
                let Some(target) = windows.get(command - 3) else { return };
                self.set_session_target(&row, Some(target.clone())).map(|()| "앱 창을 직접 연결했습니다. 로고를 눌러 이동한 뒤 대화를 확인하세요.")
            }
        };
        *self.notice.borrow_mut() = result.map(str::to_owned).unwrap_or_else(|error| error);
        InvalidateRect(hwnd, null(), 0);
    }
    unsafe fn detail(&self) {
        if self.editing.get() {
            return;
        }
        let content = if let Some(row) = self.selected() {
            let evidence = match row.evidence.as_str() {
                "before_launch" => "앱 실행 후 새 사용자 요청이 관측되지 않은 세션",
                "awaiting_completion" => "앱 실행 후 요청을 확인했으며 아직 응답 종료 기록이 없음",
                "process_only" => "프로세스 이름만 확인됨 · 세션 상태 미확인",
                "stale" => "작업 기록이 20초 이상 없어 현재 상태 미확인",
                "inferred_time" => "이벤트 시각 없음 · 파일 시각에서 추정, 현재 생존 여부 미확인",
                "unknown" => "해석할 수 있는 상태 기록 없음",
                _ => "트랜스크립트의 마지막 기록 · 현재 생존 여부 미확인",
            };
            format!("{}\r\n\r\n{}\r\n\r\n상태  {}\r\n에이전트  {}\r\n모델  {}\r\n마지막 활동 (UTC)  {}\r\n\r\n{}\r\n\r\n작업 출처  {}\r\n대표 요청\r\n{}\r\n\r\n폴더\r\n{}",row.project(),row.task,row.status_context(),row.agent,row.model,row.since,evidence,if row.task_source=="transcript_first_prompt"{"첫 요청 (현재 요청은 읽기 범위 밖)"}else{"최근 요청 또는 사용자 라벨"},row.summary,row.cwd)
        } else {
            "세션을 선택하면 전체 작업 내용과 근거를 확인할 수 있습니다.\r\n\r\n이전 세션도 수집합니다. 종결한 세션은 전체 보기에서 확인하고, 새 요청이 감지되면 기본 목록으로 돌아옵니다.".into()
        };
        if text(self.get(DETAIL)) != content {
            set_text(self.get(DETAIL), &content);
        }
    }
    unsafe fn rebuild(&self, hwnd: HWND) {
        self.busy.set(true);
        let list = self.get(LIST);
        let selected = self.selected().map(|r| r.id);
        let top = send(list, LB_GETTOPINDEX, 0, 0).max(0) as usize;
        let top_id = self.visible.borrow().get(top).map(|r| r.id.clone());
        let next = self.settings.borrow().visible(&self.rows.borrow());
        if self.force_rebuild.replace(false) || *self.visible.borrow() != next {
            send(list, WM_SETREDRAW, 0, 0);
            send(list, LB_RESETCONTENT, 0, 0);
            for row in &next {
                send(
                    list,
                    LB_ADDSTRING,
                    0,
                    wide(&row.accessible_text()).as_ptr() as isize,
                );
            }
            let selection = next
                .iter()
                .position(|r| Some(&r.id) == selected.as_ref())
                .unwrap_or(0);
            let top = next
                .iter()
                .position(|r| Some(&r.id) == top_id.as_ref())
                .unwrap_or(top.min(next.len().saturating_sub(1)));
            *self.visible.borrow_mut() = next;
            send(list, LB_SETCURSEL, selection, 0);
            send(list, LB_SETTOPINDEX, top, 0);
            send(list, WM_SETREDRAW, 1, 0);
        }
        let settings = self.settings.borrow().clone();
        set_text(
            self.get(ALL),
            if settings.show_all {
                "기본 보기"
            } else {
                "전체 보기"
            },
        );
        set_text(
            self.get(CLOSE_SESSION),
            if self
                .selected()
                .is_some_and(|r| settings.closed.contains_key(&r.id))
            {
                "되살리기"
            } else {
                "종결"
            },
        );
        set_text(
            self.get(PIN),
            if self
                .selected()
                .is_some_and(|r| settings.pinned.contains(&r.id))
            {
                "고정 해제"
            } else {
                "고정"
            },
        );
        set_text(
            self.get(HIDE),
            if self
                .selected()
                .is_some_and(|r| settings.hidden.contains(&r.id))
            {
                "숨김 해제"
            } else {
                "숨기기"
            },
        );
        self.detail();
        self.fit_list();
        self.rebuild_feed(hwnd);
        InvalidateRect(list, null(), 1);
        InvalidateRect(hwnd, null(), 1);
        self.busy.set(false);
    }
    unsafe fn agents(&self) {
        let mut names: Vec<String> = self.rows.borrow().iter().map(|r| r.agent.clone()).collect();
        names.sort();
        names.dedup();
        let selected = self.settings.borrow().agent.clone();
        if !selected.is_empty() && !names.contains(&selected) {
            names.push(selected.clone());
        }
        let combo = self.get(AGENT);
        self.busy.set(true);
        send(combo, CB_RESETCONTENT, 0, 0);
        send(
            combo,
            CB_ADDSTRING,
            0,
            wide("모든 에이전트").as_ptr() as isize,
        );
        for name in &names {
            send(combo, CB_ADDSTRING, 0, wide(name).as_ptr() as isize);
        }
        send(
            combo,
            CB_SETCURSEL,
            names
                .iter()
                .position(|n| *n == selected)
                .map(|i| i + 1)
                .unwrap_or(0),
            0,
        );
        self.busy.set(false);
    }
    fn changed(&self) {
        self.dirty.set(Some(Instant::now()));
    }
    fn save(&self) {
        if self.dirty.take().is_some() {
            if let Err(e) = self.settings.borrow().save(&self.path) {
                *self.notice.borrow_mut() = format!("설정 저장 실패: {e}");
            }
        }
    }
    unsafe fn tick(&self, hwnd: HWND) {
        let second = crate::time::now();
        let previous = self.date_tick.replace(second);
        // Age labels advance even when no new log record arrives; repaint only changed labels.
        if previous != second && IsWindowVisible(self.get(accordion::FEED)) != 0 {
            for (button, row) in self.feed.buttons.borrow().iter().zip(self.visible.borrow().iter()) {
                if row.date_label(false, previous) != row.date_label(false, second) {
                    InvalidateRect(*button, null(), 0);
                }
            }
        }
        let activated =
            self.activation
                .borrow()
                .as_ref()
                .and_then(|(id, rx)| match rx.try_recv() {
                    Ok(result) => Some((id.clone(), result)),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        Some((id.clone(), Err("창·탭 연결 확인이 중단됐습니다.".into())))
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => None,
                });
        if let Some((id, result)) = activated {
            self.activation.borrow_mut().take();
            match result {
                Ok(message) => *self.notice.borrow_mut() = message,
                Err(message) => {
                    *self.notice.borrow_mut() = message;
                    if self.selected().is_some_and(|r| r.id == id) && !self.editing.get() {
                        self.command(hwnd, SHOW_DETAIL, 0);
                    }
                }
            }
            InvalidateRect(hwnd, null(), 1);
        }
        let next = self
            .updates
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some(next) = next {
            self.loading.set(false);
            InvalidateRect(hwnd, null(), 1);
            if let Some(Snapshot { rows, warnings }) = next.snapshot {
                *self.collection_warnings.borrow_mut() = warnings;
                self.core_error.borrow_mut().clear();
                let (revived, marker_migrated) = self.settings.borrow_mut().reconcile(&rows);
                if revived > 0 || marker_migrated {
                    self.changed();
                }
                if revived > 0 {
                    *self.notice.borrow_mut() =
                        format!("새 대화 {revived}개 · 목록에 복귀했습니다");
                }
                if *self.rows.borrow() != rows || revived > 0 {
                    *self.rows.borrow_mut() = rows;
                    self.agents();
                    self.rebuild(hwnd);
                }
            }
            if let Some(error) = next.error {
                *self.core_error.borrow_mut() = error;
            }
        }
        if second.div_euclid(60) != previous.div_euclid(60) {
            let next = self.settings.borrow().visible(&self.rows.borrow());
            if next != *self.visible.borrow() {
                self.rebuild(hwnd);
            }
        }
        if self
            .dirty
            .get()
            .is_some_and(|t| t.elapsed().as_millis() >= 600)
        {
            self.save();
            InvalidateRect(hwnd, null(), 1);
        }
    }
    unsafe fn command(&self, hwnd: HWND, id: usize, notification: usize) {
        if self.busy.get() || !self.ready.get() {
            return;
        }
        if [SHOW_DETAIL, FILTER, PIN, CLOSE_SESSION, EDITOR].contains(&id)
            && self.menu_open.replace(false)
        {
            self.layout(hwnd);
        }
        match id {
            TWO_COLUMNS => {
                let top = send(self.get(LIST), LB_GETTOPINDEX, 0, 0);
                self.settings.borrow_mut().two_columns =
                    send(self.get(TWO_COLUMNS), BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                self.fit_list();
                send(self.get(LIST), LB_SETTOPINDEX, top as usize, 0);
                InvalidateRect(self.get(LIST), null(), 1);
                self.changed();
                self.layout(hwnd);
            }
            TOPMOST => {
                let next = !self.settings.borrow().always_on_top;
                self.settings.borrow_mut().always_on_top = next;
                self.window_preferences(hwnd);
                self.changed();
            }
            OPACITY => {
                self.opacity_open.set(!self.opacity_open.get());
                self.layout(hwnd);
            }
            MINIMIZE => {
                ShowWindow(hwnd, SW_MINIMIZE);
            }
            CLOSE_WINDOW => {
                send(hwnd, WM_CLOSE, 0, 0);
            }

            ALL => {
                let show = !self.settings.borrow().show_all;
                self.settings.borrow_mut().show_all = show;
                self.changed();
                self.rebuild(hwnd);
            }
            MORE => self.menu(hwnd),
            BACK => {
                if self.editing.get() {
                    self.template_command(hwnd, CLOSE_EDITOR);
                }
                self.expand(hwnd, false);
            }
            SHOW_DETAIL => {
                self.expand(hwnd, true);
                self.detail();
                SetFocus(self.get(DETAIL));
            }
            FILTER => {
                self.expand(hwnd, true);
                SetFocus(self.get(SEARCH));
            }
            ASSOCIATE => self.association_menu(hwnd),
            CLOSE_SESSION => {
                if let Some(row) = self.selected() {
                    let mut settings = self.settings.borrow_mut();
                    if settings.closed.remove(&row.id).is_some() {
                        *self.notice.borrow_mut() = "목록으로 되살렸습니다".into();
                    } else if let Err(e) = settings.close(&row) {
                        *self.notice.borrow_mut() = e;
                    } else {
                        *self.notice.borrow_mut() =
                            "종결 · 전체 보기에서 확인할 수 있습니다".into();
                    }
                    drop(settings);
                    self.changed();
                    self.rebuild(hwnd);
                }
            }
            SEARCH if notification == EN_CHANGE as usize => {
                self.settings.borrow_mut().search = text(self.get(SEARCH));
                self.changed();
                self.rebuild(hwnd);
            }
            STATUS if notification == CBN_SELCHANGE as usize => {
                let i = send(self.get(STATUS), CB_GETCURSEL, 0, 0);
                self.settings.borrow_mut().state = ui::STATES
                    .get(i.wrapping_sub(1) as usize)
                    .unwrap_or(&"")
                    .to_string();
                self.changed();
                self.rebuild(hwnd);
            }
            AGENT if notification == CBN_SELCHANGE as usize => {
                self.settings.borrow_mut().agent = if send(self.get(AGENT), CB_GETCURSEL, 0, 0) == 0
                {
                    String::new()
                } else {
                    text(self.get(AGENT))
                };
                self.changed();
                self.rebuild(hwnd);
            }
            LIST if notification == LBN_SELCHANGE as usize => {
                self.rebuild(hwnd);
            }
            PIN | HIDE => {
                if let Some(row) = self.selected() {
                    let mut s = self.settings.borrow_mut();
                    let set = if id == PIN {
                        &mut s.pinned
                    } else {
                        &mut s.hidden
                    };
                    if !set.remove(&row.id) {
                        if set.len() < 1024 {
                            set.insert(row.id);
                        }
                    }
                    drop(s);
                    self.changed();
                    self.rebuild(hwnd);
                }
            }
            AUX | HIDDEN => {
                let checked = send(self.get(id), BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                if id == AUX {
                    self.settings.borrow_mut().show_aux = checked
                } else {
                    self.settings.borrow_mut().show_hidden = checked
                }
                self.changed();
                self.rebuild(hwnd);
            }
            EDITOR => {
                if !self.editing.get() {
                    self.editor(hwnd, true)
                } else {
                    SetFocus(self.get(DETAIL));
                }
            }
            PREVIEW | APPLY | RESET | DAY | NIGHT | CLOSE_EDITOR | IMPORT | EXPORT => {
                self.template_command(hwnd, id)
            }
            _ => {}
        }
    }
    unsafe fn window_preferences(&self, hwnd: HWND) {
        let (opacity, always_on_top) = {
            let settings = self.settings.borrow();
            (settings.opacity, settings.always_on_top)
        };
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        if style & WS_EX_LAYERED == 0 {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, (style | WS_EX_LAYERED) as isize);
        }
        SetLayeredWindowAttributes(
            hwnd,
            0,
            (opacity as u32 * 255 / 100) as u8,
            LWA_ALPHA,
        );
        // Apply Z-order after the layered style; do not overwrite the OS-owned TOPMOST bit.
        SetWindowPos(
            hwnd,
            if always_on_top {
                HWND_TOPMOST
            } else {
                HWND_NOTOPMOST
            },
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
        set_text(
            self.get(TOPMOST),
            if always_on_top {
                "항상 위 ✓"
            } else {
                "항상 위"
            },
        );
        set_text(
            self.get(OPACITY),
            &format!("투명도 {}%", 100 - opacity),
        );
        send(
            self.get(OPACITY_SLIDER),
            TBM_SETPOS,
            1,
            (100 - opacity) as isize,
        );
        InvalidateRect(hwnd, null(), 0);
    }
    unsafe fn opacity_changed(&self, hwnd: HWND) {
        let transparency = send(self.get(OPACITY_SLIDER), TBM_GETPOS, 0, 0).clamp(0, 60) as u8;
        self.settings.borrow_mut().opacity = 100 - transparency;
        self.window_preferences(hwnd);
        self.changed();
    }
    unsafe fn draw_button(&self, hwnd: HWND, item: &DRAWITEMSTRUCT) {
        let skin = self.skin.borrow();
        fill(item.hDC, &item.rcItem, skin.background);
        let id = item.CtlID as usize;
        let active = id == TOPMOST && self.settings.borrow().always_on_top
            || id == ALL && self.settings.borrow().show_all
            || id == OPACITY && self.opacity_open.get()
            || id == MORE && self.menu_open.get();
        let pressed = item.itemState & ODS_SELECTED != 0;
        let background = if active || pressed {
            skin.selection
        } else {
            skin.surface
        };
        let border = if item.itemState & ODS_FOCUS != 0 {
            Some(skin.accent)
        } else {
            None
        };
        visual::rounded(item.hDC, item.rcItem, self.px(hwnd, 15), background, border);
        let label = match id {
            MINIMIZE => "−".into(),
            CLOSE_WINDOW => "×".into(),
            _ => text(item.hwndItem),
        };
        draw(
            item.hDC,
            &label,
            item.rcItem,
            if id == CLOSE_WINDOW || id == MINIMIZE {
                self.heading.get()
            } else {
                self.body.get()
            },
            if item.itemState & ODS_DISABLED != 0 {
                visual::blend(skin.background, skin.muted, 60)
            } else if active {
                skin.accent
            } else {
                skin.foreground
            },
            DT_SINGLELINE | DT_CENTER | DT_VCENTER,
        );
    }
    unsafe fn draw_row(&self, hwnd: HWND, item: &DRAWITEMSTRUCT) {
        let row = self.visible.borrow().get(item.itemID as usize).cloned();
        let Some(row) = row else {
            return;
        };
        let skin = self.skin.borrow().clone();
        let p = |n| self.px(hwnd, n);
        let selected = item.itemState & ODS_SELECTED != 0;
        fill(item.hDC, &item.rcItem, skin.background);
        let card = RECT {
            left: item.rcItem.left + p(3),
            top: item.rcItem.top + p(2),
            right: item.rcItem.right - p(3),
            bottom: item.rcItem.bottom - p(10),
        };
        let shadow = RECT {
            top: card.top + p(2),
            bottom: card.bottom + p(2),
            ..card
        };
        visual::rounded(
            item.hDC,
            shadow,
            p(22),
            visual::blend(skin.background, skin.foreground, 4),
            None,
        );
        visual::rounded(
            item.hDC,
            card,
            p(22),
            if selected {
                skin.selection
            } else {
                skin.surface
            },
            Some(if selected {
                visual::blend(skin.selection, skin.accent, 30)
            } else {
                visual::blend(skin.surface, skin.foreground, 4)
            }),
        );
        let pad = p(skin.padding);
        let tile = RECT {
            left: card.left + pad,
            top: card.top + pad,
            right: card.left + pad + p(24),
            bottom: card.top + pad + p(24),
        };
        visual::rounded(item.hDC, tile, p(14), 0xffffff, None);
        if !self.visuals.logo(
            item.hDC,
            row.logo_id(),
            tile.left + p(2),
            tile.top + p(2),
            p(20),
        ) {
            draw(
                item.hDC,
                "›_",
                tile,
                self.heading.get(),
                0x343020,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
        }
        let line = p(skin.font_size + 4 + skin.line_gap);
        let pinned = self.settings.borrow().pinned.contains(&row.id);
        // Reserve a column beside the task for the status and model beneath it.
        let status_width = p(skin.font_size * 7 + 8).min((card.right - card.left) / 3);
        if skin.compact {
            let badge = RECT {
                left: card.right - pad - status_width,
                right: card.right - pad,
                top: card.top + pad + line,
                bottom: card.top + pad + line * 2,
            };
            let color = *skin.state_colors.get(&row.state).unwrap_or(&skin.accent);
            visual::rounded(item.hDC, badge, line / 2, skin.background, None);
            draw(
                item.hDC,
                if row.state == "idle" {
                    "유휴"
                } else {
                    row.status()
                },
                badge,
                self.body.get(),
                color,
                DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_END_ELLIPSIS,
            );
            if skin.fields.iter().any(|field| field == "model") {
                draw(
                    item.hDC,
                    &row.model,
                    RECT { top: badge.bottom + p(2), bottom: badge.bottom + line, ..badge },
                    self.body.get(),
                    skin.muted,
                    DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_END_ELLIPSIS,
                );
            }
        }
        let mut y = card.top + pad;
        for (i, fields) in skin.lines().iter().enumerate() {
            let task = fields.contains(&"task");
            let status = fields.contains(&"status");
            let mut rect = RECT {
                left: if i == 0 || skin.compact {
                    tile.right + p(8)
                } else {
                    card.left + pad
                },
                right: card.right - pad,
                top: y,
                bottom: y + if task { line * 2 } else { line },
            };
            y = rect.bottom;
            if skin.compact && i == 0 {
                let mut labels = Vec::new();
                for field in fields {
                    let label = skin.text(&row, field, pinned);
                    let font = if *field == "project" {
                        self.heading.get()
                    } else {
                        self.body.get()
                    };
                    let old = SelectObject(item.hDC, font);
                    let mut measured = RECT::default();
                    DrawTextW(
                        item.hDC,
                        wide(&label).as_ptr(),
                        -1,
                        &mut measured,
                        DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX,
                    );
                    SelectObject(item.hDC, old);
                    labels.push((label, font, measured.right.max(1)));
                }
                let gap = p(8);
                let available = (rect.right - rect.left - gap * (labels.len() as i32 - 1)).max(1);
                let total: i32 = labels.iter().map(|(_, _, width)| width).sum();
                if total > available {
                    let mut remaining = available;
                    for (_, _, width) in labels.iter_mut().skip(1) {
                        *width = (*width).min(available / 3);
                        remaining -= *width;
                    }
                    labels[0].2 = remaining;
                }
                for (index, (label, font, width)) in labels.iter().enumerate() {
                    let part = RECT {
                        right: rect.left + width,
                        ..rect
                    };
                    draw(
                        item.hDC,
                        label,
                        part,
                        *font,
                        if index == 0 {
                            skin.foreground
                        } else {
                            skin.muted
                        },
                        DT_SINGLELINE | DT_END_ELLIPSIS | DT_VCENTER,
                    );
                    rect.left = part.right + gap;
                }
                continue;
            }
            if skin.compact {
                rect.right -= status_width + p(8);
            }
            if status {
                let label = skin.text(&row, "status", pinned);
                let mut measured = RECT::default();
                let old = SelectObject(item.hDC, self.body.get());
                DrawTextW(
                    item.hDC,
                    wide(&label).as_ptr(),
                    -1,
                    &mut measured,
                    DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX,
                );
                SelectObject(item.hDC, old);
                let available = rect.right - rect.left;
                let width = (measured.right + p(20)).min(if fields.len() > 1 {
                    available * 3 / 5
                } else {
                    available
                });
                let badge = RECT {
                    left: rect.right - width,
                    bottom: rect.bottom - p(1),
                    ..rect
                };
                let color = *skin.state_colors.get(&row.state).unwrap_or(&skin.accent);
                visual::rounded(item.hDC, badge, line / 2, skin.background, None);
                draw(
                    item.hDC,
                    &label,
                    badge,
                    self.body.get(),
                    color,
                    DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_END_ELLIPSIS,
                );
                rect.right = badge.left - p(8);
            }
            draw(
                item.hDC,
                &fields
                    .iter()
                    .filter(|f| **f != "status")
                    .map(|f| skin.text(&row, f, pinned))
                    .collect::<Vec<_>>()
                    .join(" · "),
                rect,
                if task && !skin.compact {
                    self.heading.get()
                } else {
                    self.body.get()
                },
                if task {
                    skin.foreground
                } else {
                    skin.muted
                },
                if task {
                    DT_WORDBREAK | DT_END_ELLIPSIS | DT_EDITCONTROL
                } else {
                    DT_SINGLELINE | DT_END_ELLIPSIS | DT_VCENTER
                },
            );
        }
    }
    unsafe fn paint(&self, hwnd: HWND) {
        let mut ps = PAINTSTRUCT::default();
        let dc = BeginPaint(hwnd, &mut ps);
        let skin = self.skin.borrow().clone();
        fill(dc, &ps.rcPaint, skin.background);
        let mut r = RECT::default();
        GetClientRect(hwnd, &mut r);
        let p = |n| self.px(hwnd, n);
        DrawIconEx(
            dc,
            p(16),
            p(9),
            self.visuals.mascot,
            p(32),
            p(32),
            0,
            null_mut(),
            DI_NORMAL,
        );
        draw(
            dc,
            if self.demo { "waid / 샘플" } else { "waid" },
            RECT {
                left: p(56),
                top: p(10),
                right: r.right - p(188),
                bottom: p(40),
            },
            self.heading.get(),
            skin.foreground,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
        );
        let hero = RECT {
            left: p(16),
            top: p(54),
            right: r.right - p(16),
            bottom: p(94),
        };
        visual::rounded(dc, hero, p(24), skin.selection, None);
        let rows = self.visible.borrow();
        let waiting = rows.iter().filter(|row| row.state == "waiting").count();
        let summary = if self.loading.get() {
            "AI 친구들의 소식을 가져오는 중".into()
        } else {
            format!("세션 {}개 · 내 차례 {}개", rows.len(), waiting)
        };
        drop(rows);
        draw(
            dc,
            &summary,
            RECT {
                left: p(32),
                top: p(60),
                right: hero.right - p(84),
                bottom: p(88),
            },
            self.body.get(),
            skin.muted,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
        );
        if self.expanded.get() {
            let extra = if self.opacity_open.get() { 36 } else { 0 };
            let w = r.right * 96 / GetDpiForWindow(hwnd).max(96) as i32;
            visual::rounded(
                dc,
                RECT {
                    left: p(20),
                    top: p(148 + extra),
                    right: p(20 + (w - 370).max(220)),
                    bottom: p(188 + extra),
                },
                p(16),
                skin.surface,
                Some(visual::blend(skin.surface, skin.muted, 18)),
            );
        }
        if self.opacity_open.get() {
            draw(
                dc,
                "선명 ↔ 투명",
                RECT {
                    left: p(16),
                    top: p(146),
                    right: p(126),
                    bottom: p(170),
                },
                self.body.get(),
                skin.muted,
                DT_SINGLELINE | DT_VCENTER,
            );
        }
        let count = self.visible.borrow().len();
        let total = self.rows.borrow().len();
        let two_columns = self.settings.borrow().two_columns;
        ShowWindow(self.get(accordion::FEED), if count > 0 && !self.expanded.get() && !two_columns { SW_SHOWNA } else { SW_HIDE });
        if count == 0 {
            let list = self.get(LIST);
            let mut rect = RECT::default();
            GetWindowRect(list, &mut rect);
            let mut at = POINT {
                x: rect.left,
                y: rect.top,
            };
            ScreenToClient(hwnd, &mut at);
            ShowWindow(list, SW_HIDE);
            draw(
                dc,
                if self.loading.get() {
                    "조금만 기다려 주세요.\nAI 친구들의 소식을 가져오고 있어요."
                } else if total == 0 {
                    "아직 조용하네요.\nAI와 작업을 시작하면 여기에 모아드릴게요."
                } else {
                    "찾는 세션이 없어요.\n검색어나 필터를 바꾸거나 전체 보기를 눌러보세요."
                },
                RECT {
                    left: at.x + p(16),
                    right: at.x + rect.right - rect.left - p(16),
                    top: at.y + p(20),
                    bottom: at.y + p(140),
                },
                self.body.get(),
                skin.foreground,
                DT_WORDBREAK,
            );
        } else {
            if ShowWindow(self.get(LIST), if self.expanded.get() || two_columns { SW_SHOWNA } else { SW_HIDE }) == 0 {
                self.fit_list();
            }
        }
        let collection_message = self.collection_message();
        let message = if !collection_message.is_empty() {
            collection_message
        } else if !self.notice.borrow().is_empty() {
            self.notice.borrow().clone()
        } else {
            if self.demo {
                "샘플 미리보기 · 실제 AI 세션이 아니에요".into()
            } else if self.expanded.get() {
                "Ctrl+F 검색 · Ctrl+D 종결 · Esc 간단히".into()
            } else {
                "2초마다 새 소식 · 카드를 눌러 대화 펼치기".into()
            }
        };
        draw(
            dc,
            &message,
            RECT {
                left: p(20),
                top: r.bottom - p(30),
                right: r.right - p(20),
                bottom: r.bottom - p(6),
            },
            self.body.get(),
            skin.muted,
            DT_SINGLELINE | DT_END_ELLIPSIS | DT_VCENTER,
        );
        EndPaint(hwnd, &ps);
    }
    fn collection_message(&self) -> String {
        let mut messages = Vec::new();
        if !self.core_error.borrow().is_empty() {
            messages.push(format!("갱신 오류 · 마지막 기록 표시 · {}", self.core_error.borrow()));
        }
        if !self.collection_warnings.borrow().is_empty() {
            messages.push(format!("일부 수집 경고 · {}", self.collection_warnings.borrow().join(" · ")));
        }
        messages.join(" · ")
    }
    unsafe fn editor(&self, hwnd: HWND, open: bool) {
        if open {
            self.expand(hwnd, true);
        }
        self.editing.set(open);
        send(
            self.get(DETAIL),
            EM_SETREADONLY,
            if open { 0 } else { 1 },
            0,
        );
        for id in [
            PREVIEW,
            APPLY,
            RESET,
            IMPORT,
            EXPORT,
            DAY,
            NIGHT,
            CLOSE_EDITOR,
        ] {
            ShowWindow(self.get(id), if open { SW_SHOWNA } else { SW_HIDE });
        }
        if open {
            let template = self.settings.borrow().template.clone();
            set_text(self.get(DETAIL), &template);
        } else {
            self.detail();
        }
        self.layout(hwnd);
        SetFocus(self.get(DETAIL));
    }
    unsafe fn template_command(&self, hwnd: HWND, id: usize) {
        match id {
            CLOSE_EDITOR => {
                let template = self.settings.borrow().template.clone();
                self.set_skin(hwnd, &template);
                self.editor(hwnd, false);
            }
            IMPORT | EXPORT => {
                if let Some(path) = choose_template_file(hwnd, id == EXPORT) {
                    if id == IMPORT {
                        self.import_template(hwnd, &path);
                    } else {
                        self.export_template(&path);
                    }
                }
            }
            DAY | NIGHT => {
                set_text(
                    self.get(DETAIL),
                    if id == DAY { ui::DEFAULT } else { ui::NIGHT },
                );
            }
            RESET => {
                set_text(self.get(DETAIL), ui::DEFAULT);
                self.set_skin(hwnd, ui::DEFAULT);
                self.settings.borrow_mut().template = ui::DEFAULT.into();
                self.changed();
                *self.notice.borrow_mut() = "기본 템플릿으로 복구했습니다.".into();
            }
            PREVIEW | APPLY => {
                let template = text(self.get(DETAIL));
                if self.set_skin(hwnd, &template) && id == APPLY {
                    self.settings.borrow_mut().template = template;
                    self.changed();
                    *self.notice.borrow_mut() = "템플릿을 적용했습니다.".into();
                }
            }
            _ => {}
        }
        InvalidateRect(hwnd, null(), 1);
    }
    unsafe fn import_template(&self, hwnd: HWND, path: &std::path::Path) -> bool {
        match ui::read_text(path).and_then(|text| Skin::parse(&text).map(|_| text)) {
            Ok(template) => {
                set_text(self.get(DETAIL), &template);
                *self.notice.borrow_mut() = "가져왔습니다. 미리보기 후 적용하세요.".into();
                InvalidateRect(hwnd, null(), 1);
                true
            }
            Err(e) => {
                *self.notice.borrow_mut() = format!("가져오기 실패: {e}");
                InvalidateRect(hwnd, null(), 1);
                false
            }
        }
    }
    unsafe fn export_template(&self, path: &std::path::Path) -> bool {
        let template = text(self.get(DETAIL));
        let result = Skin::parse(&template).and_then(|_| {
            if path
                .extension()
                .and_then(|s| s.to_str())
                .is_none_or(|s| !s.eq_ignore_ascii_case("json"))
                || path == self.path
            {
                return Err("별도의 .json 파일로 저장하세요.".into());
            }
            ui::atomic_write(path, &template).map_err(|e| e.to_string())
        });
        match result {
            Ok(()) => {
                *self.notice.borrow_mut() = "템플릿을 내보냈습니다.".into();
                true
            }
            Err(e) => {
                *self.notice.borrow_mut() = format!("내보내기 실패: {e}");
                false
            }
        }
    }
    unsafe fn set_skin(&self, hwnd: HWND, template: &str) -> bool {
        match Skin::parse(template) {
            Ok(skin) => {
                *self.skin.borrow_mut() = skin;
                self.notice.borrow_mut().clear();
                self.fonts(hwnd);
                self.layout(hwnd);
                self.rebuild(hwnd);
                true
            }
            Err(e) => {
                *self.notice.borrow_mut() = e;
                InvalidateRect(hwnd, null(), 1);
                false
            }
        }
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            for font in [self.body.get(), self.heading.get()] {
                if !font.is_null() {
                    DeleteObject(font);
                }
            }
            if !self.surface.get().is_null() {
                DeleteObject(self.surface.get());
            }
        }
    }
}

unsafe fn add_tray_icon(hwnd: HWND) -> io::Result<()> {
    let icon = LoadImageW(
        GetModuleHandleW(null()),
        1usize as _,
        IMAGE_ICON,
        GetSystemMetrics(SM_CXSMICON),
        GetSystemMetrics(SM_CYSMICON),
        LR_DEFAULTCOLOR | LR_SHARED,
    ) as HICON;
    if icon.is_null() {
        return Err(io::Error::last_os_error());
    }
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: TRAY_MESSAGE,
        hIcon: icon,
        ..Default::default()
    };
    data.szTip[..4].copy_from_slice(&wide("waid")[..4]);
    if Shell_NotifyIconW(NIM_ADD, &data) == 0 && Shell_NotifyIconW(NIM_MODIFY, &data) == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    let data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    };
    Shell_NotifyIconW(NIM_DELETE, &data);
}

unsafe fn restore_window(s: &Window, hwnd: HWND) {
    ShowWindow(hwnd, SW_RESTORE);
    SetForegroundWindow(hwnd);
    SetFocus(s.session_focus());
}

unsafe fn tray_menu(s: &Window, hwnd: HWND) {
    let menu = CreatePopupMenu();
    if menu.is_null() {
        return;
    }
    AppendMenuW(menu, MF_STRING, TRAY_OPEN, wide("열기").as_ptr());
    AppendMenuW(menu, MF_STRING, TRAY_LICENSE, wide("폰트 라이선스").as_ptr());
    AppendMenuW(menu, MF_SEPARATOR, 0, null());
    AppendMenuW(menu, MF_STRING, TRAY_EXIT, wide("종료").as_ptr());
    let mut point = POINT::default();
    GetCursorPos(&mut point);
    SetForegroundWindow(hwnd);
    let command = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD,
        point.x,
        point.y,
        0,
        hwnd,
        null(),
    );
    DestroyMenu(menu);
    match command as usize {
        TRAY_OPEN => restore_window(s, hwnd),
        TRAY_LICENSE => {
            let result = (|| -> io::Result<()> {
                let path = ui::data_file().with_file_name("FONT-LICENSE.txt");
                std::fs::create_dir_all(path.parent().unwrap())?;
                std::fs::write(&path, super::FONT_LICENSE)?;
                let file: Vec<u16> = {
                    use std::os::windows::ffi::OsStrExt;
                    path.as_os_str().encode_wide().chain(Some(0)).collect()
                };
                if ShellExecuteW(hwnd, wide("open").as_ptr(), file.as_ptr(), null(), null(), SW_SHOWNORMAL) as isize <= 32 {
                    return Err(io::Error::other("폰트 라이선스 파일을 열지 못했습니다."));
                }
                Ok(())
            })();
            if let Err(error) = result { show_error(&error.to_string()); }
        }
        TRAY_EXIT => {
            s.save();
            DestroyWindow(hwnd);
        }
        _ => {}
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        SetWindowLongPtrW(
            hwnd,
            GWLP_USERDATA,
            (*(lp as *const CREATESTRUCTW)).lpCreateParams as isize,
        );
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Window;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let s = &*ptr;
    match msg {
        _ if s.taskbar_created != 0 && msg == s.taskbar_created => {
            if let Err(error) = add_tray_icon(hwnd) {
                *s.notice.borrow_mut() = format!("트레이 아이콘 복구 실패: {error}");
                restore_window(s, hwnd);
            }
        }
        WM_NCCALCSIZE => return 0,
        WM_NCPAINT => return 0,
        WM_NCACTIVATE => return 1,
        WM_NCHITTEST => {
            let mut point = POINT {
                x: lp as i16 as i32,
                y: (lp >> 16) as i16 as i32,
            };
            ScreenToClient(hwnd, &mut point);
            let mut rect = RECT::default();
            GetClientRect(hwnd, &mut rect);
            let border = s.px(hwnd, 6);
            let left = point.x < border;
            let right = point.x >= rect.right - border;
            let top = point.y < border;
            let bottom = point.y >= rect.bottom - border;
            let hit = match (left, right, top, bottom) {
                (true, _, true, _) => HTTOPLEFT,
                (_, true, true, _) => HTTOPRIGHT,
                (true, _, _, true) => HTBOTTOMLEFT,
                (_, true, _, true) => HTBOTTOMRIGHT,
                (true, _, _, _) => HTLEFT,
                (_, true, _, _) => HTRIGHT,
                (_, _, true, _) => HTTOP,
                (_, _, _, true) => HTBOTTOM,
                _ if point.y < s.px(hwnd, 44) => HTCAPTION,
                _ => HTCLIENT,
            };
            return hit as isize;
        }
        WM_NCLBUTTONDBLCLK if wp == HTCAPTION as usize => return 0,
        WM_SYSCOMMAND if wp & 0xfff0 == SC_MAXIMIZE as usize => return 0,
        WM_SIZE => {
            if wp != SIZE_MINIMIZED as usize {
                s.layout(hwnd);
            }
        }
        TRAY_MESSAGE => match lp as u32 {
            WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                restore_window(s, hwnd);
            }
            WM_RBUTTONUP => tray_menu(s, hwnd),
            _ => {}
        },
        WM_HSCROLL if lp as HWND == s.get(OPACITY_SLIDER) => s.opacity_changed(hwnd),
        WM_TIMER => s.tick(hwnd),
        WM_COMMAND => s.command(hwnd, wp & 0xffff, (wp >> 16) & 0xffff),
        WM_GETMINMAXINFO => {
            (*(lp as *mut MINMAXINFO)).ptMinTrackSize = POINT {
                x: s.px(hwnd, if s.expanded.get() { 720 } else { 480 }),
                y: s.px(hwnd, if s.expanded.get() { 620 } else { 440 }),
            };
        }
        WM_DPICHANGED => {
            let r = &*(lp as *const RECT);
            SetWindowPos(
                hwnd,
                null_mut(),
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            s.fonts(hwnd);
            s.layout(hwnd);
        }
        WM_CONTEXTMENU => s.menu(hwnd),
        WM_PAINT => s.paint(hwnd),
        WM_ERASEBKGND => return 1,
        WM_NOTIFY if wp == OPACITY_SLIDER => {
            let info = &*(lp as *const NMCUSTOMDRAW);
            if info.hdr.code == NM_CUSTOMDRAW && info.dwDrawStage == CDDS_PREPAINT {
                let skin = s.skin.borrow();
                let mut bounds = RECT::default();
                GetClientRect(info.hdr.hwndFrom, &mut bounds);
                fill(info.hdc, &bounds, skin.background);
                let mut thumb = RECT::default();
                send(
                    s.get(OPACITY_SLIDER),
                    TBM_GETTHUMBRECT,
                    0,
                    &mut thumb as *mut _ as isize,
                );
                let center = (bounds.top + bounds.bottom) / 2;
                let track = RECT {
                    left: bounds.left + s.px(hwnd, 8),
                    right: bounds.right - s.px(hwnd, 8),
                    top: center - s.px(hwnd, 2),
                    bottom: center + s.px(hwnd, 2),
                };
                visual::rounded(
                    info.hdc,
                    track,
                    s.px(hwnd, 2),
                    visual::blend(skin.background, skin.muted, 25),
                    None,
                );
                let center_x = (thumb.left + thumb.right) / 2;
                visual::rounded(
                    info.hdc,
                    RECT {
                        right: center_x,
                        ..track
                    },
                    s.px(hwnd, 2),
                    skin.accent,
                    None,
                );
                let radius = s.px(hwnd, 7);
                visual::rounded(
                    info.hdc,
                    RECT {
                        left: center_x - radius,
                        right: center_x + radius,
                        top: center - radius,
                        bottom: center + radius,
                    },
                    radius,
                    skin.surface,
                    Some(skin.accent),
                );
                return CDRF_SKIPDEFAULT as isize;
            }
            return CDRF_DODEFAULT as isize;
        }
        WM_NOTIFY if wp == LIST => {
            let header = &*(lp as *const NMHDR);
            if header.code == NM_CUSTOMDRAW {
                let item = &*(lp as *const NMLVCUSTOMDRAW);
                if item.nmcd.dwDrawStage == CDDS_PREPAINT {
                    return CDRF_NOTIFYITEMDRAW as isize;
                }
                if item.nmcd.dwDrawStage == CDDS_ITEMPREPAINT {
                    let mut rect = RECT {
                        left: LVIR_BOUNDS as i32,
                        ..Default::default()
                    };
                    send(
                        s.get(LIST),
                        LVM_GETITEMRECT,
                        item.nmcd.dwItemSpec,
                        &mut rect as *mut _ as isize,
                    );
                    s.draw_row(
                        hwnd,
                        &DRAWITEMSTRUCT {
                            itemID: item.nmcd.dwItemSpec as u32,
                            itemState: if send(
                                s.get(LIST),
                                LVM_GETITEMSTATE,
                                item.nmcd.dwItemSpec,
                                LVIS_SELECTED as isize,
                            ) != 0
                            {
                                ODS_SELECTED
                            } else {
                                0
                            },
                            hDC: item.nmcd.hdc,
                            rcItem: rect,
                            ..Default::default()
                        },
                    );
                    return CDRF_SKIPDEFAULT as isize;
                }
                return CDRF_DODEFAULT as isize;
            }
            if (header.code == NM_CLICK || header.code == NM_DBLCLK)
                && (*(lp as *const NMITEMACTIVATE)).iItem >= 0
            {
                s.activate_selected(hwnd);
            }
            if header.code == LVN_ITEMCHANGED && !s.busy.get() && s.ready.get() {
                s.rebuild(hwnd);
            }
        }
        WM_DRAWITEM if wp != LIST => {
            s.draw_button(hwnd, &*(lp as *const DRAWITEMSTRUCT));
            return 1;
        }
        WM_DRAWITEM if wp == LIST => {
            s.draw_row(hwnd, &*(lp as *const DRAWITEMSTRUCT));
            return 1;
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC | WM_CTLCOLORLISTBOX => {
            let skin = s.skin.borrow();
            SetTextColor(wp as HDC, skin.foreground);
            if lp as HWND == s.get(TWO_COLUMNS) {
                SetBkColor(wp as HDC, skin.selection);
                SetDCBrushColor(wp as HDC, skin.selection);
                return GetStockObject(DC_BRUSH) as isize;
            }
            if [s.get(AUX), s.get(HIDDEN)].contains(&(lp as HWND)) {
                SetBkColor(wp as HDC, skin.background);
                SetDCBrushColor(wp as HDC, skin.background);
                return GetStockObject(DC_BRUSH) as isize;
            }
            SetBkColor(wp as HDC, skin.surface);
            return s.surface.get() as isize;
        }
        WM_CLOSE => {
            s.save();
            ShowWindow(hwnd, SW_HIDE);
        }
        WM_DESTROY => {
            remove_tray_icon(hwnd);
            KillTimer(hwnd, 1);
            PostQuitMessage(0);
        }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, msg, wp, lp);
        }
        _ => return DefWindowProcW(hwnd, msg, wp, lp),
    }
    0
}
unsafe fn create_window(s: &Window) -> io::Result<HWND> {
    InitCommonControlsEx(&INITCOMMONCONTROLSEX {
        dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_LISTVIEW_CLASSES | ICC_BAR_CLASSES,
    });
    let instance = GetModuleHandleW(null());
    let class = wide(CLASS);
    let wc = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        hIcon: s.visuals.app,
        hCursor: LoadCursorW(null_mut(), IDC_ARROW),
        lpszClassName: class.as_ptr(),
        ..Default::default()
    };
    if RegisterClassW(&wc) == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
        return Err(io::Error::last_os_error());
    }
    let dpi = GetDpiForSystem().max(96) as i32;
    let mut work = RECT::default();
    SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut work as *mut _ as *mut _, 0);
    let always_on_top = s.settings.borrow().always_on_top;
    let hwnd = CreateWindowExW(
        WS_EX_APPWINDOW
            | WS_EX_CONTROLPARENT
            | WS_EX_LAYERED
            | if always_on_top {
                WS_EX_TOPMOST
            } else {
                0
            },
        class.as_ptr(),
        wide("waid · AI 세션").as_ptr(),
        WS_POPUP | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN,
        work.left + 32 * dpi / 96,
        work.top + 64 * dpi / 96,
        (600 * dpi / 96).min((work.right - work.left).max(480)),
        (780 * dpi / 96).min((work.bottom - work.top).max(440)),
        null_mut(),
        null_mut(),
        instance,
        s as *const _ as *const _,
    );
    if hwnd.is_null() {
        return Err(io::Error::last_os_error());
    }
    // The class icon only covers windows created after registration; set both sizes here.
    SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, s.visuals.app as isize);
    SendMessageW(
        hwnd,
        WM_SETICON,
        ICON_SMALL as usize,
        s.visuals.app_small as isize,
    );
    let corner: u32 = DWMWCP_ROUND as u32;
    DwmSetWindowAttribute(
        hwnd,
        DWMWA_WINDOW_CORNER_PREFERENCE as u32,
        &corner as *const _ as _,
        std::mem::size_of_val(&corner) as u32,
    );
    DeleteMenu(GetSystemMenu(hwnd, 0), SC_MAXIMIZE, MF_BYCOMMAND);
    let setup = (|| -> io::Result<()> {
        s.add(hwnd, SEARCH, "EDIT", "", ES_AUTOHSCROLL as u32)?;
        send(
            s.get(SEARCH),
            EM_SETCUEBANNER,
            1,
            wide("검색: 작업, 프로젝트, 모델").as_ptr() as isize,
        );
        send(s.get(SEARCH), EM_SETLIMITTEXT, 200, 0);
        s.add(
            hwnd,
            STATUS,
            "COMBOBOX",
            "",
            CBS_DROPDOWNLIST as u32 | WS_VSCROLL,
        )?;
        for label in [
            "모든 상태",
            "내 차례",
            "작업 중",
            "오류",
            "유휴",
            "종결",
            "미확인",
        ] {
            send(
                s.get(STATUS),
                CB_ADDSTRING,
                0,
                wide(label).as_ptr() as isize,
            );
        }
        s.add(
            hwnd,
            AGENT,
            "COMBOBOX",
            "",
            CBS_DROPDOWNLIST as u32 | WS_VSCROLL,
        )?;
        for (id, label) in [
            (TOPMOST, "항상 위"),
            (OPACITY, "투명도 0%"),
            (MINIMIZE, "최소화"),
            (CLOSE_WINDOW, "닫기"),
            (SHOW_DETAIL, "상세"),
            (FILTER, "검색"),
            (ALL, "전체 보기"),
            (MORE, "···"),
            (CLOSE_SESSION, "종결"),
            (BACK, "간단히"),
            (PIN, "고정"),
            (HIDE, "숨기기"),
            (EDITOR, "템플릿"),
            (ASSOCIATE, "연결"),
            (PREVIEW, "미리보기"),
            (APPLY, "적용"),
            (RESET, "기본값 복구"),
            (IMPORT, "가져오기"),
            (EXPORT, "내보내기"),
            (DAY, "밝은 기본"),
            (NIGHT, "어두운 기본"),
            (CLOSE_EDITOR, "편집 닫기"),
        ] {
            s.add(hwnd, id, "BUTTON", label, BS_OWNERDRAW as u32)?;
        }
        s.add(
            hwnd,
            OPACITY_SLIDER,
            "msctls_trackbar32",
            "투명도 조절",
            TBS_HORZ | TBS_NOTICKS,
        )?;
        send(s.get(OPACITY_SLIDER), TBM_SETRANGEMAX, 1, 60);
        send(s.get(OPACITY_SLIDER), TBM_SETPAGESIZE, 0, 10);
        s.add(hwnd, AUX, "BUTTON", "보조 포함", BS_AUTOCHECKBOX as u32)?;
        s.add(hwnd, HIDDEN, "BUTTON", "숨김 포함", BS_AUTOCHECKBOX as u32)?;
        s.add(hwnd, TWO_COLUMNS, "BUTTON", "2열", BS_AUTOCHECKBOX as u32)?;
        s.add(
            hwnd,
            LIST,
            "SysListView32",
            "세션 목록",
            LVS_ICON | LVS_AUTOARRANGE | LVS_SINGLESEL | LVS_SHOWSELALWAYS,
        )?;
        if send(s.get(LIST), LVM_SETVIEW, LV_VIEW_TILE as usize, 0) == -1 {
            return Err(io::Error::other("Windows 타일 목록을 열지 못했습니다."));
        }
        send(
            s.get(LIST),
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            0,
            LVS_EX_DOUBLEBUFFER as isize,
        );
        s.add(
            hwnd,
            DETAIL,
            "EDIT",
            "",
            WS_VSCROLL | (ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY) as u32,
        )?;
        s.create_feed(hwnd)?;
        send(s.get(DETAIL), EM_SETLIMITTEXT, ui::MAX_CONFIG as usize, 0);
        if SetTimer(hwnd, 1, 250, None) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    })();
    if let Err(e) = setup {
        DestroyWindow(hwnd);
        return Err(e);
    }
    s.fonts(hwnd);
    let settings = s.settings.borrow().clone();
    send(
        s.get(TWO_COLUMNS),
        BM_SETCHECK,
        if settings.two_columns {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        } as usize,
        0,
    );
    set_text(s.get(SEARCH), &settings.search);
    send(
        s.get(STATUS),
        CB_SETCURSEL,
        ui::STATES
            .iter()
            .position(|v| *v == settings.state)
            .map(|i| i + 1)
            .unwrap_or(0),
        0,
    );
    send(
        s.get(AUX),
        BM_SETCHECK,
        if settings.show_aux {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        } as usize,
        0,
    );
    send(
        s.get(HIDDEN),
        BM_SETCHECK,
        if settings.show_hidden {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        } as usize,
        0,
    );
    SetWindowPos(
        hwnd,
        null_mut(),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE,
    );
    s.ready.set(true);
    s.window_preferences(hwnd);
    s.agents();
    s.editor(hwnd, false);
    s.rebuild(hwnd);
    Ok(hwnd)
}

unsafe fn choose_template_file(owner: HWND, save: bool) -> Option<PathBuf> {
    let mut file = vec![0u16; 32768];
    if save {
        let name = wide("my-waid-template.json");
        file[..name.len()].copy_from_slice(&name);
    }
    let filter = wide("waid UI template (*.json)\0*.json\0");
    let ext = wide("json");
    let title = wide(if save {
        "템플릿 내보내기"
    } else {
        "템플릿 가져오기"
    });
    let mut options = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: filter.as_ptr(),
        nFilterIndex: 1,
        lpstrFile: file.as_mut_ptr(),
        nMaxFile: file.len() as u32,
        lpstrDefExt: ext.as_ptr(),
        lpstrTitle: title.as_ptr(),
        Flags: OFN_EXPLORER
            | OFN_NOCHANGEDIR
            | OFN_PATHMUSTEXIST
            | if save {
                OFN_OVERWRITEPROMPT
            } else {
                OFN_FILEMUSTEXIST
            },
        ..Default::default()
    };
    let result = if save {
        GetSaveFileNameW(&mut options)
    } else {
        GetOpenFileNameW(&mut options)
    };
    if result == 0 {
        let error = CommDlgExtendedError();
        if error != 0 {
            show_error(&format!("파일 선택 창 오류: {error}"));
        }
        return None;
    }
    use std::os::windows::ffi::OsStringExt;
    let end = file.iter().position(|c| *c == 0).unwrap_or(file.len());
    Some(PathBuf::from(std::ffi::OsString::from_wide(&file[..end])))
}

pub fn run() -> io::Result<()> {
    let demo = std::env::args().any(|a| a == "--demo");
    let (core, updates) = if demo {
        (None, Updates::default())
    } else {
        let (core, updates) =
            super::start_core(&std::env::current_exe()?)?;
        (Some(core), updates)
    };
    let state = Box::new(Window::new(updates, ui::data_file(), demo));
    if demo {
        publish_update(&state.updates, Ok(Snapshot { rows: demo_rows(), ..Snapshot::default() }));
    }
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let hwnd = create_window(&state)?;
        if let Err(error) = add_tray_icon(hwnd) {
            DestroyWindow(hwnd);
            return Err(error);
        }
        ShowWindow(hwnd, SW_SHOWNORMAL);
        state.window_preferences(hwnd);
        state.tick(hwnd);
        if demo && !state.settings.borrow().two_columns {
            state.toggle_card(hwnd, 0);
        }
        SetFocus(state.session_focus());
        let mut msg = MSG::default();
        loop {
            match GetMessageW(&mut msg, null_mut(), 0, 0) {
                0 => break,
                -1 => {
                    DestroyWindow(hwnd);
                    return Err(io::Error::last_os_error());
                }
                _ => {
                    if msg.message == WM_KEYDOWN && GetKeyState(VK_CONTROL as i32) < 0 {
                        match msg.wParam as u16 {
                            0x46 => {
                                state.command(hwnd, FILTER, 0);
                                continue;
                            }
                            0x44 => {
                                if !state.editing.get() {
                                    state.command(hwnd, CLOSE_SESSION, 0);
                                    continue;
                                }
                            }
                            0x50 => {
                                state.command(hwnd, PIN, 0);
                                continue;
                            }
                            0x48 => {
                                state.command(hwnd, HIDE, 0);
                                continue;
                            }
                            _ => {}
                        }
                    }
                    if msg.message == WM_KEYDOWN
                        && msg.wParam == VK_ESCAPE as usize
                        && state.expanded.get()
                    {
                        state.command(hwnd, BACK, 0);
                        continue;
                    }
                    if msg.message == WM_KEYDOWN
                        && msg.wParam == VK_RETURN as usize
                        && state.feed.logos.borrow().contains(&msg.hwnd)
                    {
                        SendMessageW(msg.hwnd, BM_CLICK, 0, 0);
                        continue;
                    }
                    if msg.message == WM_KEYDOWN
                        && msg.wParam == VK_RETURN as usize
                        && msg.hwnd == state.get(LIST)
                    {
                        state.activate_selected(hwnd);
                        continue;
                    }
                    if IsDialogMessageW(hwnd, &msg) == 0 {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }
        }
    }
    drop(core);
    Ok(())
}
pub fn show_error(message: &str) {
    unsafe {
        MessageBoxW(
            null_mut(),
            wide(message).as_ptr(),
            wide("waid 시작 실패").as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}
fn demo_rows() -> Vec<Row> {
    [
        (
            "checkout",
            "결제 화면 접근성 개선 — 키보드 이동과 오류 안내 확인",
            "waiting",
            "Claude Code",
            "opus",
        ),
        (
            "api",
            "로그인 세션 만료와 재시도 처리",
            "working",
            "Codex",
            "model",
        ),
        (
            "docs",
            "설정 방법 문서를 한국어로 정리",
            "idle",
            "Claude Code",
            "model",
        ),
    ]
    .into_iter()
    .map(|(id, task, state, agent, model)| Row {
        id: format!("demo-{id}"),
        request_marker: format!("demo-{id}-request-1"),
        title: format!("sample/{id}"),
        task: task.into(),
        state: state.into(),
        agent_id: if agent == "Codex" { "codex" } else { "claude" }.into(),
        agent: agent.into(),
        model: model.into(),
        summary: "샘플 데이터 · 실제 AI 세션이 아닙니다.".into(),
        last_answer: "키보드만으로 모든 입력란을 이동하고 오류 안내를 확인할 수 있도록 개선했습니다. 다음 작업은 원래 세션에서 이어서 요청할 수 있습니다.".into(),
        cwd: "샘플 프로젝트".into(),
        since: crate::time::to_iso8601(crate::time::now()),
        evidence: "demo".into(),
        ..Row::default()
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    // These tests manipulate process-wide window activation and desktop Z-order.
    static DESKTOP_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());
    #[test]
    fn displayed_feed_populates_refreshes_and_removes_focused_cards() {
        let _desktop = DESKTOP_TEST.lock().unwrap_or_else(|e| e.into_inner());
        let path = std::env::temp_dir().join(format!("waid-displayed-feed-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let state = Window::new(Updates::default(), path.clone(), true);
        unsafe {
            let hwnd = create_window(&state).unwrap();
            state.check_displayed_feed(hwnd);
            state.menu(hwnd);
            assert_ne!(IsWindowVisible(state.get(ASSOCIATE)), 0);
            state.expand(hwnd, true);
            assert_ne!(IsWindowVisible(state.get(ASSOCIATE)), 0);
            let row = Row { id: "association-test".into(), agent_id: "codex".into(), session_id: "thread-test".into(), ..Row::default() };
            let target = serde_json::json!({"kind":"chatgpt_link", "url":"codex://threads/thread-test"});
            state.set_session_target(&row, Some(target.clone())).unwrap();
            state.save();
            assert_eq!(Settings::read(&path).unwrap().session_targets[&row.id], target);
            // Preserve the adapter's honest notice instead of claiming exact conversation activation.
            let (tx, rx) = std::sync::mpsc::channel();
            *state.activation.borrow_mut() = Some((row.id.clone(), rx));
            tx.send(Ok("manual window only".into())).unwrap();
            state.tick(hwnd);
            assert_eq!(&*state.notice.borrow(), "manual window only");
            state.set_session_target(&row, None).unwrap();
            state.save();
            assert!(Settings::read(&path).unwrap().session_targets.is_empty());
            DestroyWindow(hwnd);
        }
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn tray_icon_hides_and_restores_window() {
        let _desktop = DESKTOP_TEST.lock().unwrap_or_else(|e| e.into_inner());
        let path = std::env::temp_dir().join(format!("waid-tray-{}.json", std::process::id()));
        let state = Window::new(Updates::default(), path.clone(), true);
        unsafe {
            let hwnd = create_window(&state).unwrap();
            add_tray_icon(hwnd).unwrap();
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            // A shell/taskbar minimize must remain available for normal restoration.
            ShowWindow(hwnd, SW_MINIMIZE);
            assert_ne!(IsIconic(hwnd), 0);
            assert_ne!(IsWindowVisible(hwnd), 0, "system minimize must not hide the window");
            ShowWindow(hwnd, SW_RESTORE);
            assert_eq!(IsIconic(hwnd), 0);
            assert_ne!(IsWindowVisible(hwnd), 0);
            window_proc(hwnd, WM_CLOSE, 0, 0);
            assert_eq!(IsWindowVisible(hwnd), 0);
            // Simulate Explorer losing this icon, without restarting the user's shell.
            remove_tray_icon(hwnd);
            let data = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: TRAY_ID,
                ..Default::default()
            };
            assert_eq!(Shell_NotifyIconW(NIM_MODIFY, &data), 0);
            let taskbar_created = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());
            assert_ne!(taskbar_created, 0);
            SendMessageW(hwnd, taskbar_created, 0, 0);
            assert_ne!(Shell_NotifyIconW(NIM_MODIFY, &data), 0);
            SendMessageW(hwnd, taskbar_created, 0, 0);
            assert_eq!(IsWindowVisible(hwnd), 0);
            window_proc(hwnd, TRAY_MESSAGE, 0, WM_LBUTTONUP as isize);
            assert_ne!(IsWindowVisible(hwnd), 0);
            DestroyWindow(hwnd);
        }
        let _ = std::fs::remove_file(path);
    }
    #[test]
    fn window_controls_persist_and_maximize_is_unavailable() {
        let _desktop = DESKTOP_TEST.lock().unwrap_or_else(|e| e.into_inner());
        let path =
            std::env::temp_dir().join(format!("waid-window-prefs-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let state = Window::new(Updates::default(), path.clone(), true);
        unsafe {
            assert!(!state.visuals.app.is_null(), "waid logo must decode");
            assert!(!state.visuals.mascot.is_null(), "header mascot must decode");
            assert!(!state.visuals.codex.is_null(), "OpenAI logo must decode");
            assert!(!state.visuals.claude.is_null(), "Claude logo must decode");
            assert!(!state.visuals.orca.is_null(), "Orca logo must decode");
            let hwnd = create_window(&state).unwrap();
            assert!(
                state.visuals.fonts.iter().all(|font| !font.is_null()),
                "bundled fonts load"
            );
            let dc = GetDC(hwnd);
            for (font, weight) in [(state.body.get(), FW_SEMIBOLD), (state.heading.get(), FW_BOLD)] {
                let old = SelectObject(dc, font);
                let mut metrics = TEXTMETRICW::default();
                assert_ne!(GetTextMetricsW(dc, &mut metrics), 0);
                assert_eq!(metrics.tmWeight, weight as i32, "selected font has the requested weight");
                let mut face = [0u16; 64];
                let len = GetTextFaceW(dc, face.len() as i32, face.as_mut_ptr());
                assert!(
                    String::from_utf16_lossy(&face[..len.saturating_sub(1) as usize])
                        .starts_with("Pretendard"),
                    "requested font is selected instead of a fallback"
                );
                SelectObject(dc, old);
            }
            ReleaseDC(hwnd, dc);
            assert_ne!(
                send(hwnd, WM_GETICON, ICON_SMALL as usize, 0),
                0,
                "window carries the waid icon"
            );
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            assert_eq!(
                GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_MAXIMIZEBOX,
                0
            );
            assert_ne!(
                GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_MINIMIZEBOX,
                0
            );
            assert_eq!(
                GetMenuState(GetSystemMenu(hwnd, 0), SC_MAXIMIZE, MF_BYCOMMAND),
                u32::MAX
            );
            send(hwnd, WM_SYSCOMMAND, SC_MAXIMIZE as usize, 0);
            assert_eq!(IsZoomed(hwnd), 0);
            assert_eq!(
                GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_CAPTION,
                0,
                "no duplicate native title bar"
            );
            state.command(hwnd, TOPMOST, 0);
            assert_ne!(
                GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST,
                0
            );
            send(state.get(OPACITY_SLIDER), TBM_SETPOS, 1, 60);
            send(
                hwnd,
                WM_HSCROLL,
                TB_THUMBPOSITION as usize,
                state.get(OPACITY_SLIDER) as isize,
            );
            let mut alpha = 0;
            let mut flags = 0;
            assert_ne!(
                GetLayeredWindowAttributes(hwnd, null_mut(), &mut alpha, &mut flags),
                0
            );
            assert_eq!(alpha, 102);
            assert_eq!(flags, LWA_ALPHA);
            assert_eq!(state.settings.borrow().opacity, 40);
            // Trackbar PREPAINT does not provide usable bounds on this Windows build.
            // Paint an empty custom-draw rect into a real memory DC and check the control's background.
            let slider = state.get(OPACITY_SLIDER);
            let mut bounds = RECT::default();
            GetClientRect(slider, &mut bounds);
            let screen = GetDC(hwnd);
            let dc = CreateCompatibleDC(screen);
            let bitmap = CreateCompatibleBitmap(screen, bounds.right, bounds.bottom);
            let old = SelectObject(dc, bitmap);
            fill(dc, &bounds, 0xff00ff);
            let info = NMCUSTOMDRAW {
                hdr: NMHDR {
                    hwndFrom: slider,
                    idFrom: OPACITY_SLIDER,
                    code: NM_CUSTOMDRAW,
                },
                dwDrawStage: CDDS_PREPAINT,
                hdc: dc,
                ..Default::default()
            };
            assert_eq!(
                send(hwnd, WM_NOTIFY, OPACITY_SLIDER, &info as *const _ as isize),
                CDRF_SKIPDEFAULT as isize
            );
            assert_eq!(GetPixel(dc, 0, 0), state.skin.borrow().background);
            SelectObject(dc, old);
            DeleteObject(bitmap);
            DeleteDC(dc);
            ReleaseDC(hwnd, screen);
            state.command(hwnd, CLOSE_WINDOW, 0);
            assert_ne!(IsWindow(hwnd), 0);
            assert_eq!(IsWindowVisible(hwnd), 0);
            let saved = Settings::read(&path).unwrap();
            assert!(saved.always_on_top);
            assert_eq!(saved.opacity, 40);
            DestroyWindow(hwnd);
            // A restart uses a fresh UI thread after the window is explicitly destroyed.
            for _ in 0..3 {
                let restart_path = path.clone();
                std::thread::spawn(move || {
                    let restored = Window::new(Updates::default(), restart_path, true);
                    let hwnd = create_window(&restored).unwrap();
                    ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                    restored.window_preferences(hwnd);
                    assert_ne!(
                        GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST,
                        0
                    );
                    assert_ne!(
                        GetLayeredWindowAttributes(hwnd, null_mut(), &mut alpha, &mut flags),
                        0
                    );
                    assert_eq!(alpha, 102);
                    restored.command(hwnd, TOPMOST, 0);
                    assert_eq!(
                        GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST,
                        0
                    );
                    send(restored.get(OPACITY_SLIDER), TBM_SETPOS, 1, 0);
                    restored.opacity_changed(hwnd);
                    assert_eq!(restored.settings.borrow().opacity, 100);
                    restored.command(hwnd, MINIMIZE, 0);
                    assert_ne!(IsIconic(hwnd), 0);
                    assert_ne!(IsWindowVisible(hwnd), 0);
                    ShowWindow(hwnd, SW_RESTORE);
                    assert_eq!(IsIconic(hwnd), 0);
                    assert_ne!(IsWindowVisible(hwnd), 0);
                    DestroyWindow(hwnd);
                })
                .join()
                .unwrap();
            }
        }
        std::fs::write(&path, r#"{"version":1,"opacity":0}"#).unwrap();
        assert_eq!(
            Settings::read(&path).unwrap().opacity,
            100,
            "invalid opacity cannot hide the app"
        );
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn native_filters_selection_and_organization() {
        let _desktop = DESKTOP_TEST.lock().unwrap_or_else(|e| e.into_inner());
        let path = std::env::temp_dir().join(format!("waid-native-{}.json", std::process::id()));
        let state = Window::new(Updates::default(), path.clone(), true);
        unsafe {
            let hwnd = create_window(&state).unwrap();
            state.check_feed(hwnd);
            state.settings.borrow_mut().two_columns = true;
            state.layout(hwnd);
            let mut bounds = RECT::default();
            GetWindowRect(hwnd, &mut bounds);
            assert_eq!(bounds.right - bounds.left, state.px(hwnd, 600));
            assert_eq!(
                GetWindowLongPtrW(state.get(DETAIL), GWL_STYLE) as u32 & WS_VISIBLE,
                0
            );
            assert!(state.skin.borrow().row_height() <= 108);
            for height in [153, 306, 600] {
                state.row_height(height as i32);
                state.force_rebuild.set(true);
                *state.rows.borrow_mut() = demo_rows();
                state.rebuild(hwnd);
                assert!(
                    send(state.get(LIST), LB_GETITEMHEIGHT, 0, 0) >= height,
                    "height {height}, actual {}, view {}",
                    send(state.get(LIST), LB_GETITEMHEIGHT, 0, 0),
                    send(state.get(LIST), LVM_GETVIEW, 0, 0)
                );
            }
            state.fonts(hwnd);
            publish_update(&state.updates, Ok(Snapshot { rows: demo_rows(), ..Snapshot::default() }));
            state.tick(hwnd);
            assert_eq!(send(state.get(LIST), LB_GETCOUNT, 0, 0), 3);
            let mut list_bounds = RECT::default();
            GetClientRect(state.get(LIST), &mut list_bounds);
            let tile = |index| {
                let mut rect = RECT {
                    left: LVIR_BOUNDS as i32,
                    ..Default::default()
                };
                assert_ne!(
                    send(
                        state.get(LIST),
                        LVM_GETITEMRECT,
                        index,
                        &mut rect as *mut _ as isize
                    ),
                    0
                );
                rect
            };
            assert_eq!(
                tile(0).top,
                tile(1).top,
                "first two sessions share a grid row"
            );
            assert!(tile(1).left >= tile(0).right, "columns do not overlap");
            assert!(
                tile(2).top >= tile(0).bottom,
                "third session starts the next row"
            );
            let mut many = Vec::new();
            // A real card notification falls back to details; blank space does nothing.
            let selection = state.selected().unwrap().id;
            let mut click = NMITEMACTIVATE {
                hdr: NMHDR { hwndFrom: state.get(LIST), idFrom: LIST, code: NM_CLICK },
                iItem: -1,
                ..Default::default()
            };
            SendMessageW(hwnd, WM_NOTIFY, LIST, &click as *const _ as isize);
            assert!(!state.expanded.get());
            click.iItem = 0;
            SendMessageW(hwnd, WM_NOTIFY, LIST, &click as *const _ as isize);
            assert!(state.expanded.get());
            assert_eq!(state.selected().unwrap().id, selection);
            assert!(state.notice.borrow().contains("연결을 확인할 수 없어"));
            state.command(hwnd, BACK, 0);
            let (tx, rx) = std::sync::mpsc::channel();
            *state.activation.borrow_mut() = Some((selection, rx));
            tx.send(Err("연결 종료".into())).unwrap();
            state.tick(hwnd);
            assert!(state.activation.borrow().is_none());
            assert!(state.expanded.get());
            assert_eq!(&*state.notice.borrow(), "연결 종료");
            state.command(hwnd, BACK, 0);
            for i in 0..30 {
                let mut row = demo_rows()[0].clone();
                row.id = format!("sample-{i:02}");
                row.title = format!("sample/{i:02}");
                many.push(row);
            }
            publish_update(&state.updates, Ok(Snapshot { rows: many.clone(), ..Snapshot::default() }));
            state.tick(hwnd);
            GetClientRect(state.get(LIST), &mut list_bounds);
            assert!(
                tile(1).right <= list_bounds.right,
                "second column fits with a scrollbar"
            );
            assert!(
                tile(5).bottom <= list_bounds.bottom,
                "six sessions fit without scrolling"
            );
            state.expand(hwnd, true);
            assert!(
                tile(1).left >= tile(0).right,
                "resizing repositions both columns"
            );
            state.expand(hwnd, false);
            let selected_before_layout = state.selected().unwrap().id;
            send(
                state.get(TWO_COLUMNS),
                BM_SETCHECK,
                BST_UNCHECKED as usize,
                0,
            );
            state.command(hwnd, TWO_COLUMNS, 0);
            assert!(!state.settings.borrow().two_columns);
            assert!(tile(1).top >= tile(0).bottom, "unchecked means one column");
            assert_eq!(state.selected().unwrap().id, selected_before_layout);
            state.save();
            assert!(!Settings::read(&path).unwrap().two_columns);
            send(state.get(TWO_COLUMNS), BM_SETCHECK, BST_CHECKED as usize, 0);
            state.command(hwnd, TWO_COLUMNS, 0);
            assert_eq!(tile(0).top, tile(1).top, "checked means two columns");
            assert_eq!(state.selected().unwrap().id, selected_before_layout);
            state.busy.set(true);
            send(state.get(LIST), LB_SETCURSEL, 8, 0);
            send(state.get(LIST), LB_SETTOPINDEX, 7, 0);
            state.busy.set(false);
            let selected = state.selected().unwrap().id;
            let top = send(state.get(LIST), LB_GETTOPINDEX, 0, 0);
            let top_id = state.visible.borrow()[top as usize].id.clone();
            many[0].task = "changed sample".into();
            publish_update(&state.updates, Ok(Snapshot { rows: many, ..Snapshot::default() }));
            state.tick(hwnd);
            assert_eq!(state.selected().unwrap().id, selected);
            assert_eq!(
                state.visible.borrow()[send(state.get(LIST), LB_GETTOPINDEX, 0, 0) as usize].id,
                top_id
            );
            publish_update(&state.updates, Ok(Snapshot {
                rows: state.rows.borrow().clone(), warnings: vec!["sample source unavailable".into()],
            }));
            state.tick(hwnd);
            assert_eq!(state.visible.borrow().len(), 30);
            assert!(state.core_error.borrow().is_empty());
            assert!(state.collection_message().contains("sample source unavailable"));
            publish_update(&state.updates, Err("sample collection error".into()));
            state.tick(hwnd);
            assert_eq!(state.visible.borrow().len(), 30);
            assert!(!state.core_error.borrow().is_empty());
            assert!(state.collection_message().contains("sample collection error"));
            assert!(state.collection_message().contains("sample source unavailable"));
            publish_update(&state.updates, Ok(Snapshot { rows: demo_rows(), ..Snapshot::default() }));
            state.tick(hwnd);
            assert!(state.core_error.borrow().is_empty());
            assert!(state.collection_warnings.borrow().is_empty());
            assert!(state.collection_message().is_empty());
            let closing = state.selected().unwrap();
            state.command(hwnd, CLOSE_SESSION, 0);
            assert!(state.settings.borrow().closed.contains_key(&closing.id));
            assert_eq!(state.visible.borrow().len(), 2);
            state.settings.borrow_mut().closed.get_mut(&closing.id).unwrap().request_marker = "r1:legacy".into();
            let mut migrated = demo_rows();
            migrated.iter_mut().find(|row| row.id == closing.id).unwrap().request_marker = "r2:current".into();
            state.dirty.set(None);
            state.notice.borrow_mut().clear();
            publish_update(&state.updates, Ok(Snapshot { rows: migrated, ..Snapshot::default() }));
            state.tick(hwnd);
            assert!(state.dirty.get().is_some(), "marker migration must schedule a save");
            assert!(state.notice.borrow().is_empty(), "migration is not a revival");
            state.save();
            assert_eq!(Settings::read(&path).unwrap().closed[&closing.id].request_marker, "r2:current");
            state.command(hwnd, ALL, 0);
            assert_eq!(state.visible.borrow().len(), 3);
            assert!(state
                .visible
                .borrow()
                .iter()
                .any(|r| r.id == closing.id && r.state == "done"));
            state.command(hwnd, ALL, 0);
            let mut resumed = demo_rows();
            let row = resumed.iter_mut().find(|r| r.id == closing.id).unwrap();
            row.request_marker = "request-after-close".into();
            row.state = "working".into();
            publish_update(&state.updates, Ok(Snapshot { rows: resumed, ..Snapshot::default() }));
            state.tick(hwnd);
            assert!(!state.settings.borrow().closed.contains_key(&closing.id));
            assert_eq!(state.visible.borrow().len(), 3);
            let chosen = state.selected().unwrap();
            state.command(hwnd, PIN, 0);
            assert!(state.settings.borrow().pinned.contains(&chosen.id));
            set_text(state.get(SEARCH), "로그인");
            state.command(hwnd, SEARCH, EN_CHANGE as usize);
            assert_eq!(state.visible.borrow().len(), 1);
            assert!(text(state.get(DETAIL)).contains("로그인"));
            state.command(hwnd, HIDE, 0);
            assert!(state.visible.borrow().is_empty());
            send(state.get(HIDDEN), BM_SETCHECK, BST_CHECKED as usize, 0);
            state.command(hwnd, HIDDEN, 0);
            assert_eq!(state.visible.borrow().len(), 1);
            state.save();
            assert!(Settings::read(&path).unwrap().show_hidden);
            SetWindowPos(
                hwnd,
                null_mut(),
                0,
                0,
                state.px(hwnd, 720),
                state.px(hwnd, 500),
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
            state.editor(hwnd, true);
            assert_eq!(
                send(state.get(DETAIL), EM_LINEINDEX, 1, 0),
                3,
                "template editor must preserve separate JSON lines"
            );
            let mut client = RECT::default();
            GetClientRect(hwnd, &mut client);
            for id in [LIST, DETAIL, PREVIEW, APPLY, RESET, EXPORT] {
                let mut rect = RECT::default();
                GetWindowRect(state.get(id), &mut rect);
                let mut point = POINT {
                    x: rect.right,
                    y: rect.bottom,
                };
                ScreenToClient(hwnd, &mut point);
                assert!(
                    point.x <= client.right && point.y <= client.bottom - state.px(hwnd, 30),
                    "control {id} outside compact window"
                );
            }
            let before = state.skin.borrow().clone();
            set_text(state.get(DETAIL), "{");
            state.template_command(hwnd, PREVIEW);
            assert_eq!(*state.skin.borrow(), before);
            let export = path.with_extension("template.json");
            set_text(state.get(DETAIL), ui::NIGHT);
            assert!(state.export_template(&export));
            assert!(state.import_template(hwnd, &export));
            state.template_command(hwnd, PREVIEW);
            assert_eq!(state.skin.borrow().name, "Midnight");
            assert_ne!(state.settings.borrow().template, ui::NIGHT);
            state.template_command(hwnd, CLOSE_EDITOR);
            assert_eq!(*state.skin.borrow(), before);
            state.editor(hwnd, true);
            assert!(state.import_template(hwnd, &export));
            state.template_command(hwnd, APPLY);
            state.save();
            assert_eq!(
                Skin::parse(&Settings::read(&path).unwrap().template)
                    .unwrap()
                    .name,
                "Midnight"
            );
            state.template_command(hwnd, RESET);
            state.save();
            assert_eq!(Settings::read(&path).unwrap().template, ui::DEFAULT);
            std::fs::write(&export, "invalid").unwrap();
            assert!(!state.import_template(hwnd, &export));
            assert_eq!(state.skin.borrow().name, "Daylight");
            std::fs::remove_file(export).unwrap();
            // Repeated previews replace owned GDI resources rather than accumulating them.
            #[link(name = "user32")]
            extern "system" {
                fn GetGuiResources(process: HANDLE, flags: u32) -> u32;
            }
            let mut counts = Vec::new();
            for _ in 0..4 {
                for i in 0..100 {
                    assert!(state.set_skin(hwnd, if i % 2 == 0 { ui::NIGHT } else { ui::DEFAULT }));
                }
                let mut msg = MSG::default();
                while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                GdiFlush();
                counts.push(GetGuiResources(-1isize as HANDLE, 0));
            }
            eprintln!("GDI per 100 previews: {counts:?}");
            assert!(counts[3] <= counts[2] + 2);
            DestroyWindow(hwnd);
        }
        std::fs::write(&path, "invalid saved settings").unwrap();
        let recovered = Window::new(Updates::default(), path.clone(), true);
        assert_eq!(recovered.skin.borrow().name, "Daylight");
        assert!(!recovered.notice.borrow().is_empty());
        let _ = std::fs::remove_file(path);
    }
}

unsafe fn send(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        LB_GETCOUNT => SendMessageW(hwnd, LVM_GETITEMCOUNT, 0, 0),
        LB_RESETCONTENT => SendMessageW(hwnd, LVM_DELETEALLITEMS, 0, 0),
        LB_ADDSTRING => {
            let item = LVITEMW {
                mask: LVIF_TEXT,
                iItem: SendMessageW(hwnd, LVM_GETITEMCOUNT, 0, 0) as i32,
                pszText: lp as *mut u16,
                ..Default::default()
            };
            SendMessageW(hwnd, LVM_INSERTITEMW, 0, &item as *const _ as isize)
        }
        LB_GETCURSEL => SendMessageW(hwnd, LVM_GETNEXTITEM, usize::MAX, LVNI_SELECTED as isize),
        LB_SETCURSEL => {
            let clear = LVITEMW {
                stateMask: LVIS_SELECTED | LVIS_FOCUSED,
                ..Default::default()
            };
            SendMessageW(
                hwnd,
                LVM_SETITEMSTATE,
                usize::MAX,
                &clear as *const _ as isize,
            );
            let item = LVITEMW {
                stateMask: LVIS_SELECTED | LVIS_FOCUSED,
                state: LVIS_SELECTED | LVIS_FOCUSED,
                ..Default::default()
            };
            SendMessageW(hwnd, LVM_SETITEMSTATE, wp, &item as *const _ as isize)
        }
        LB_GETTOPINDEX => {
            // ponytail: linear scan of session tiles; cache the scroll origin for thousands of sessions.
            let count = SendMessageW(hwnd, LVM_GETITEMCOUNT, 0, 0);
            for index in 0..count {
                let mut rect = RECT {
                    left: LVIR_BOUNDS as i32,
                    ..Default::default()
                };
                SendMessageW(
                    hwnd,
                    LVM_GETITEMRECT,
                    index as usize,
                    &mut rect as *mut _ as isize,
                );
                if rect.bottom > 0 {
                    return index;
                }
            }
            0
        }
        LB_SETTOPINDEX => {
            let mut r = RECT {
                left: LVIR_BOUNDS as i32,
                ..Default::default()
            };
            SendMessageW(hwnd, LVM_GETITEMRECT, wp, &mut r as *mut _ as isize);
            SendMessageW(hwnd, LVM_SCROLL, 0, r.top as isize)
        }
        LB_GETITEMHEIGHT => {
            let mut r = RECT {
                left: LVIR_BOUNDS as i32,
                ..Default::default()
            };
            SendMessageW(hwnd, LVM_GETITEMRECT, wp, &mut r as *mut _ as isize);
            (r.bottom - r.top) as isize
        }
        _ => SendMessageW(hwnd, msg, wp, lp),
    }
}
