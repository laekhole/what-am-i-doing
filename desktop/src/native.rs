use std::{cell::{Cell, RefCell}, io, ptr::{null, null_mut}};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{Controls::*, HiDpi::*, Input::KeyboardAndMouse::SetFocus,
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::*},
};
use super::{Row, Updates};

const MINIMIZE: usize = 1;
const CLOSE: usize = 2;
const SESSIONS: usize = 3;
const TOPMOST: usize = 10;
const OPACITY: usize = 100;
const CLASS: &str = "waid.native.memo";

fn wide(text: &str) -> Vec<u16> { text.encode_utf16().chain(Some(0)).collect() }
fn alpha(percent: u8) -> u8 { ((percent as u16 * 255 + 50) / 100) as u8 }

struct Window {
    list: Cell<HWND>,
    minimize: Cell<HWND>,
    close: Cell<HWND>,
    font: Cell<HFONT>,
    heading: Cell<HFONT>,
    detail: Cell<HFONT>,
    topmost: Cell<bool>,
    opacity: Cell<u8>,
    rows: RefCell<Vec<Row>>,
    message: RefCell<String>,
    updates: Updates,
}

impl Window {
    fn new(updates: Updates) -> Self {
        Self { list: Cell::new(null_mut()), minimize: Cell::new(null_mut()),
            close: Cell::new(null_mut()), font: Cell::new(null_mut()),
            heading: Cell::new(null_mut()), detail: Cell::new(null_mut()),
            topmost: Cell::new(false), opacity: Cell::new(100),
            rows: RefCell::new(Vec::new()), message: RefCell::new("세션을 읽는 중…".into()), updates }
    }

    unsafe fn px(&self, hwnd: HWND, value: i32) -> i32 {
        value * GetDpiForWindow(hwnd).max(96) as i32 / 96
    }

    unsafe fn layout(&self, hwnd: HWND) {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect);
        let (gap, header, button) = (self.px(hwnd, 8), self.px(hwnd, 36), self.px(hwnd, 32));
        MoveWindow(self.minimize.get(), rect.right - button * 2, 0, button, header, 1);
        MoveWindow(self.close.get(), rect.right - button, 0, button, header, 1);
        MoveWindow(self.list.get(), gap, header, (rect.right - gap * 2).max(1),
            (rect.bottom - header - gap).max(1), 1);
        SendMessageW(self.list.get(), LB_SETITEMHEIGHT, 0, self.px(hwnd, 82) as isize);
    }

    unsafe fn set_font(&self, hwnd: HWND) {
        for (target, size, weight) in [(&self.font, 14, FW_NORMAL),
            (&self.heading, 14, FW_SEMIBOLD), (&self.detail, 12, FW_NORMAL)] {
            let font = CreateFontW(-self.px(hwnd, size), 0, 0, 0, weight as i32,
                0, 0, 0, DEFAULT_CHARSET as u32, 0, 0, CLEARTYPE_QUALITY as u32, 0,
                wide("Segoe UI").as_ptr());
            if !font.is_null() {
                let old = target.replace(font);
                if !old.is_null() { DeleteObject(old); }
            }
        }
        for child in [self.list.get(), self.minimize.get(), self.close.get()] {
            SendMessageW(child, WM_SETFONT, self.font.get() as usize, 1);
        }
    }

    unsafe fn update(&self, hwnd: HWND) {
        let next = self.updates.lock().unwrap_or_else(|e| e.into_inner()).take();
        let Some(next) = next else { return; };
        let (rows, message) = match next {
            Ok(rows) => (rows, "관찰 중인 세션이 없습니다.".to_string()),
            Err(message) => (Vec::new(), message),
        };
        if *self.rows.borrow() == rows && *self.message.borrow() == message { return; }
        let list = self.list.get();
        let selected = SendMessageW(list, LB_GETCURSEL, 0, 0) as usize;
        let top = SendMessageW(list, LB_GETTOPINDEX, 0, 0) as usize;
        let selected_id = self.rows.borrow().get(selected).map(|r| r.id.clone());
        let top_id = self.rows.borrow().get(top).map(|r| r.id.clone());
        let selected = rows.iter().position(|r| Some(&r.id) == selected_id.as_ref());
        let top = rows.iter().position(|r| Some(&r.id) == top_id.as_ref()).unwrap_or(0);
        let labels: Vec<_> = rows.iter().map(|r| wide(&r.accessible_text())).collect();
        SendMessageW(list, WM_SETREDRAW, 0, 0);
        SendMessageW(list, LB_RESETCONTENT, 0, 0);
        *self.rows.borrow_mut() = rows;
        *self.message.borrow_mut() = message;
        for label in &labels { SendMessageW(list, LB_ADDSTRING, 0, label.as_ptr() as isize); }
        if let Some(index) = selected { SendMessageW(list, LB_SETCURSEL, index, 0); }
        SendMessageW(list, LB_SETTOPINDEX, top, 0);
        SendMessageW(list, WM_SETREDRAW, 1, 0);
        ShowWindow(list, if labels.is_empty() { SW_HIDE } else { SW_SHOWNA });
        InvalidateRect(list, null(), 1);
        InvalidateRect(hwnd, null(), 1);
    }

    unsafe fn draw_row(&self, hwnd: HWND, item: &DRAWITEMSTRUCT) {
        let row = self.rows.borrow().get(item.itemID as usize).cloned();
        let Some(row) = row else { return; };
        let selected = item.itemState & ODS_SELECTED != 0;
        let foreground = GetSysColor(if selected { COLOR_HIGHLIGHTTEXT } else { COLOR_WINDOWTEXT });
        FillRect(item.hDC, &item.rcItem, GetSysColorBrush(
            if selected { COLOR_HIGHLIGHT } else { COLOR_WINDOW }));
        let old = SelectObject(item.hDC, self.heading.get());
        SetBkMode(item.hDC, TRANSPARENT as i32);
        SetTextColor(item.hDC, foreground);
        let mut rect = item.rcItem;
        rect.left += self.px(hwnd, 8);
        rect.right -= self.px(hwnd, 8);
        rect.top += self.px(hwnd, 8);
        rect.bottom = rect.top + self.px(hwnd, 21);
        let mut title = rect;
        title.right -= self.px(hwnd, 72);
        DrawTextW(item.hDC, wide(&row.title).as_ptr(), -1, &mut title,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX);
        SelectObject(item.hDC, self.detail.get());
        let mut status = rect;
        status.left = title.right + self.px(hwnd, 4);
        DrawTextW(item.hDC, wide(row.status()).as_ptr(), -1, &mut status,
            DT_SINGLELINE | DT_VCENTER | DT_RIGHT | DT_NOPREFIX);
        rect.top += self.px(hwnd, 24);
        rect.bottom += self.px(hwnd, 24);
        SelectObject(item.hDC, self.font.get());
        DrawTextW(item.hDC, wide(&row.task).as_ptr(), -1, &mut rect,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX);
        rect.top += self.px(hwnd, 22);
        rect.bottom += self.px(hwnd, 22);
        SelectObject(item.hDC, self.detail.get());
        SetTextColor(item.hDC, if selected { foreground } else { GetSysColor(COLOR_GRAYTEXT) });
        DrawTextW(item.hDC, wide(&format!("{} · {}", row.agent, row.model)).as_ptr(), -1, &mut rect,
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX);
        let mut line = item.rcItem;
        line.top = line.bottom - 1;
        FillRect(item.hDC, &line, GetSysColorBrush(COLOR_BTNFACE));
        if item.itemState & ODS_FOCUS != 0 { DrawFocusRect(item.hDC, &item.rcItem); }
        SelectObject(item.hDC, old);
    }

    unsafe fn command(&self, hwnd: HWND, id: usize) {
        match id {
            MINIMIZE => { ShowWindow(hwnd, SW_MINIMIZE); }
            CLOSE => { DestroyWindow(hwnd); }
            TOPMOST => {
                let enabled = !self.topmost.get();
                if SetWindowPos(hwnd, if enabled { HWND_TOPMOST } else { HWND_NOTOPMOST },
                    0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) != 0 {
                    self.topmost.set(enabled);
                    InvalidateRect(hwnd, null(), 0);
                }
            }
            id if (OPACITY + 30..=OPACITY + 100).contains(&id) && id % 10 == 0 => {
                let percent = (id - OPACITY) as u8;
                if SetLayeredWindowAttributes(hwnd, 0, alpha(percent), LWA_ALPHA) != 0 {
                    self.opacity.set(percent);
                }
            }
            _ => {}
        }
    }

    unsafe fn menu(&self, hwnd: HWND, position: LPARAM) {
        let menu = CreatePopupMenu();
        let opacity = CreatePopupMenu();
        if menu.is_null() || opacity.is_null() {
            if !menu.is_null() { DestroyMenu(menu); }
            if !opacity.is_null() { DestroyMenu(opacity); }
            return;
        }
        AppendMenuW(menu, MF_STRING | if self.topmost.get() { MF_CHECKED } else { 0 },
            TOPMOST, wide("항상 위").as_ptr());
        for percent in [100, 90, 80, 70, 60, 50, 40, 30] {
            AppendMenuW(opacity, MF_STRING | if self.opacity.get() == percent as u8 { MF_CHECKED } else { 0 },
                OPACITY + percent, wide(&format!("{percent}%")).as_ptr());
        }
        AppendMenuW(menu, MF_POPUP, opacity as usize, wide("불투명도").as_ptr());
        let point = if position == -1 {
            let mut p = POINT { x: self.px(hwnd, 10), y: self.px(hwnd, 32) };
            ClientToScreen(hwnd, &mut p);
            p
        } else { POINT { x: position as i16 as i32, y: (position >> 16) as i16 as i32 } };
        let id = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON, point.x, point.y,
            0, hwnd, null_mut()) as usize;
        DestroyMenu(menu); // 하위 메뉴도 함께 해제된다.
        self.command(hwnd, id);
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        for font in [self.font.get(), self.heading.get(), self.detail.get()] {
            if !font.is_null() { unsafe { DeleteObject(font); } }
        }
    }
}

unsafe extern "system" fn list_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM,
    id: usize, parent: usize) -> LRESULT {
    if msg == WM_CONTEXTMENU { return SendMessageW(parent as HWND, msg, wp, lp); }
    if msg == WM_NCDESTROY { RemoveWindowSubclass(hwnd, Some(list_proc), id); }
    DefSubclassProc(hwnd, msg, wp, lp)
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = &*(lp as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Window;
    if state.is_null() { return DefWindowProcW(hwnd, msg, wp, lp); }
    // 공유 참조 + Cell: SendMessage의 동기 재진입 중에도 &mut alias를 만들지 않는다.
    let state = &*state;
    match msg {
        WM_SIZE if wp != SIZE_MINIMIZED as usize => state.layout(hwnd),
        WM_TIMER => state.update(hwnd),
        WM_COMMAND => state.command(hwnd, wp & 0xffff),
        WM_CONTEXTMENU => state.menu(hwnd, lp),
        WM_SYSCOMMAND if wp & 0xfff0 == SC_MAXIMIZE as usize => return 0,
        WM_NCLBUTTONDBLCLK => return 0,
        WM_NCHITTEST => {
            let hit = DefWindowProcW(hwnd, msg, wp, lp);
            let mut point = POINT { x: lp as i16 as i32, y: (lp >> 16) as i16 as i32 };
            ScreenToClient(hwnd, &mut point);
            return if hit == HTCLIENT as isize && point.y < state.px(hwnd, 36) {
                HTCAPTION as isize
            } else { hit };
        }
        WM_GETMINMAXINFO => {
            let info = &mut *(lp as *mut MINMAXINFO);
            info.ptMinTrackSize = POINT { x: state.px(hwnd, 280), y: state.px(hwnd, 160) };
        }
        WM_DPICHANGED => {
            let rect = &*(lp as *const RECT);
            SetWindowPos(hwnd, null_mut(), rect.left, rect.top, rect.right - rect.left,
                rect.bottom - rect.top, SWP_NOZORDER | SWP_NOACTIVATE);
            state.set_font(hwnd);
            state.layout(hwnd);
        }
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut paint);
            FillRect(dc, &paint.rcPaint, GetSysColorBrush(COLOR_WINDOW));
            let old = SelectObject(dc, state.font.get());
            SetTextColor(dc, GetSysColor(COLOR_WINDOWTEXT));
            SetBkMode(dc, TRANSPARENT as i32);
            let mut rect = RECT { left: state.px(hwnd, 10), top: 0,
                right: state.px(hwnd, 180), bottom: state.px(hwnd, 36) };
            let title = wide(if state.topmost.get() { "waid · 항상 위" } else { "waid" });
            DrawTextW(dc, title.as_ptr(), -1, &mut rect, DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX);
            if state.rows.borrow().is_empty() {
                GetClientRect(hwnd, &mut rect);
                rect.left += state.px(hwnd, 16);
                rect.right -= state.px(hwnd, 16);
                rect.top += state.px(hwnd, 52);
                SetTextColor(dc, GetSysColor(COLOR_GRAYTEXT));
                DrawTextW(dc, wide(&state.message.borrow()).as_ptr(), -1, &mut rect,
                    DT_WORDBREAK | DT_NOPREFIX);
            }
            SelectObject(dc, old);
            EndPaint(hwnd, &paint);
        }
        WM_MEASUREITEM if wp == SESSIONS => {
            (*(lp as *mut MEASUREITEMSTRUCT)).itemHeight = state.px(hwnd, 82) as u32;
            return 1;
        }
        WM_DRAWITEM => {
            let item = &*(lp as *const DRAWITEMSTRUCT);
            if item.CtlID == SESSIONS as u32 { state.draw_row(hwnd, item); return 1; }
            FillRect(item.hDC, &item.rcItem, GetSysColorBrush(
                if item.itemState & ODS_SELECTED != 0 { COLOR_BTNFACE } else { COLOR_WINDOW }));
            let old = SelectObject(item.hDC, state.font.get());
            SetTextColor(item.hDC, GetSysColor(COLOR_WINDOWTEXT));
            SetBkMode(item.hDC, TRANSPARENT as i32);
            let glyph = wide(if item.CtlID == MINIMIZE as u32 { "−" } else { "×" });
            let mut rect = item.rcItem;
            DrawTextW(item.hDC, glyph.as_ptr(), -1, &mut rect, DT_SINGLELINE | DT_CENTER | DT_VCENTER);
            if item.itemState & ODS_FOCUS != 0 { DrawFocusRect(item.hDC, &rect); }
            SelectObject(item.hDC, old);
            return 1;
        }
        WM_CLOSE => { DestroyWindow(hwnd); }
        WM_DESTROY => { KillTimer(hwnd, 1); PostQuitMessage(0); }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, msg, wp, lp);
        }
        _ => return DefWindowProcW(hwnd, msg, wp, lp),
    }
    0
}

unsafe fn create_window(state: &Window) -> io::Result<HWND> {
    let instance = GetModuleHandleW(null());
    let class_name = wide(CLASS);
    let class = WNDCLASSW { lpfnWndProc: Some(window_proc), hInstance: instance,
        hCursor: LoadCursorW(null_mut(), IDC_ARROW), hbrBackground: GetSysColorBrush(COLOR_WINDOW),
        lpszClassName: class_name.as_ptr(), ..Default::default() };
    if RegisterClassW(&class) == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
        return Err(io::Error::last_os_error());
    }
    let dpi = GetDpiForSystem().max(96) as i32;
    // 기본 caption에는 비활성 최대화 버튼도 남으므로 얇은 헤더와 표준 버튼 두 개만 둔다.
    let hwnd = CreateWindowExW(WS_EX_APPWINDOW | WS_EX_LAYERED | WS_EX_CONTROLPARENT,
        class_name.as_ptr(), wide("waid").as_ptr(),
        WS_POPUP | WS_THICKFRAME | WS_MINIMIZEBOX | WS_CLIPCHILDREN,
        CW_USEDEFAULT, CW_USEDEFAULT, 460 * dpi / 96, 360 * dpi / 96,
        null_mut(), null_mut(), instance, state as *const Window as *const _);
    if hwnd.is_null() { return Err(io::Error::last_os_error()); }
    state.list.set(CreateWindowExW(0, wide("LISTBOX").as_ptr(), wide("관찰 중인 세션").as_ptr(),
        WS_CHILD | WS_VSCROLL | WS_TABSTOP |
        (LBS_OWNERDRAWFIXED | LBS_HASSTRINGS | LBS_NOINTEGRALHEIGHT | LBS_NOTIFY) as u32,
        0, 0, 0, 0, hwnd, SESSIONS as HMENU, instance, null()));
    for (id, name, target) in [(MINIMIZE, "최소화", &state.minimize), (CLOSE, "닫기", &state.close)] {
        target.set(CreateWindowExW(0, wide("BUTTON").as_ptr(), wide(name).as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_OWNERDRAW as u32,
            0, 0, 0, 0, hwnd, id as HMENU, instance, null()));
    }
    if state.list.get().is_null() || state.minimize.get().is_null() || state.close.get().is_null()
        || SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA) == 0
        || SetWindowSubclass(state.list.get(), Some(list_proc), 1, hwnd as usize) == 0
        || SetTimer(hwnd, 1, 250, None) == 0 {
        let error = io::Error::last_os_error();
        DestroyWindow(hwnd);
        return Err(error);
    }
    state.set_font(hwnd);
    state.layout(hwnd);
    Ok(hwnd)
}

pub fn run() -> io::Result<()> {
    let exe = std::env::current_exe()?.with_file_name("waid.exe");
    let (_core, updates) = super::start_core(&exe)?;
    let state = Box::new(Window::new(updates));
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let hwnd = create_window(&state)?;
        ShowWindow(hwnd, SW_SHOWNORMAL);
        SetFocus(state.minimize.get());
        let mut message = MSG::default();
        loop {
            match GetMessageW(&mut message, null_mut(), 0, 0) {
                0 => break,
                -1 => {
                    let error = io::Error::last_os_error();
                    DestroyWindow(hwnd);
                    return Err(error);
                }
                _ => {
                    if IsDialogMessageW(hwnd, &message) == 0 {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn show_error(message: &str) {
    unsafe { MessageBoxW(null_mut(), wide(message).as_ptr(), wide("waid 시작 실패").as_ptr(),
        MB_OK | MB_ICONERROR); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_window_has_only_requested_controls() {
        let state = Box::new(Window::new(Updates::default()));
        unsafe {
            let hwnd = create_window(&state).unwrap();
            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            assert_eq!(style & (WS_CAPTION | WS_MAXIMIZEBOX), 0);
            assert_ne!(GetWindowLongW(state.list.get(), GWL_STYLE) as u32 & LBS_HASSTRINGS as u32, 0);
            state.command(hwnd, TOPMOST);
            assert_ne!(GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST, 0);
            state.command(hwnd, TOPMOST);
            assert_eq!(GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST, 0);
            state.command(hwnd, OPACITY + 80);
            let mut actual = 0;
            assert_ne!(GetLayeredWindowAttributes(hwnd, null_mut(), &mut actual, null_mut()), 0);
            assert_eq!(actual, 204);
            state.command(hwnd, OPACITY); // 완전 투명해져 창을 잃어버리지 않는다.
            assert_eq!(state.opacity.get(), 80);
            SendMessageW(hwnd, WM_SYSCOMMAND, SC_MAXIMIZE as usize, 0);
            assert_eq!(IsZoomed(hwnd), 0);
            assert_eq!(IsWindowVisible(hwnd), 0);
            ShowWindow(hwnd, SW_SHOWNORMAL);
            let visible = IsWindowVisible(hwnd);
            let mut rect = RECT::default();
            GetWindowRect(hwnd, &mut rect);
            state.command(hwnd, MINIMIZE);
            let minimized = IsIconic(hwnd);
            ShowWindow(hwnd, SW_RESTORE);
            let row = Row { id: "1".into(), title: "repo".into(), agent: "Codex".into(),
                model: "—".into(), task: "≈ 작업".into(), state: "waiting".into() };
            *state.updates.lock().unwrap() = Some(Ok(vec![row.clone()]));
            state.update(hwnd);
            UpdateWindow(hwnd);
            UpdateWindow(state.list.get());
            assert_eq!(SendMessageW(state.list.get(), LB_GETCOUNT, 0, 0), 1);
            assert_eq!(SendMessageW(state.list.get(), LB_GETITEMHEIGHT, 0, 0), state.px(hwnd, 82) as isize);
            let length = SendMessageW(state.list.get(), LB_GETTEXTLEN, 0, 0) as usize;
            let mut label = vec![0u16; length + 1];
            SendMessageW(state.list.get(), LB_GETTEXT, 0, label.as_mut_ptr() as isize);
            assert_eq!(String::from_utf16_lossy(&label[..length]), row.accessible_text());
            SendMessageW(state.list.get(), LB_SETCURSEL, 0, 0);
            *state.updates.lock().unwrap() = Some(Ok(vec![Row { id: "2".into(), ..row.clone() }, row]));
            state.update(hwnd);
            UpdateWindow(state.list.get());
            assert_eq!(SendMessageW(state.list.get(), LB_GETCURSEL, 0, 0), 1);
            *state.updates.lock().unwrap() = Some(Ok(Vec::new()));
            state.update(hwnd);
            assert_eq!(SendMessageW(state.list.get(), LB_GETCOUNT, 0, 0), 0);
            assert_eq!(IsWindowVisible(state.list.get()), 0);
            *state.updates.lock().unwrap() = Some(Err("test error".into()));
            state.update(hwnd);
            assert_eq!(*state.message.borrow(), "test error");
            DestroyWindow(hwnd);
            assert_ne!(visible, 0, "startup must display the memo");
            assert_ne!(minimized, 0, "minimize must minimize the memo");
            assert!(rect.right > rect.left && rect.bottom > rect.top,
                "window bounds: {},{} {},{}", rect.left, rect.top, rect.right, rect.bottom);
        }
        assert_eq!(alpha(30), 77);
        assert_eq!(alpha(100), 255);
    }
}
