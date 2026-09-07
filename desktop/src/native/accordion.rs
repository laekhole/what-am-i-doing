//! Single-column conversation cards; native buttons and read-only edits retain keyboard access.
use super::*;

pub(super) const FEED: usize = 52;
const REQUEST: usize = 53;
const ANSWER: usize = 54;
const PROMPT: usize = 55;
const OPEN: usize = 56;
const COPY: usize = 57;
const LOGO: usize = 58;
const CARD: usize = 1000;
const HEADER: i32 = 78;
const BODY: i32 = 378;

#[derive(Default)]
pub(super) struct Feed {
    pub buttons: RefCell<Vec<HWND>>,
    pub logos: RefCell<Vec<HWND>>,
    open: RefCell<Option<String>>,
    scroll: Cell<i32>,
}

impl Window {
    fn feed_header(&self) -> i32 {
        HEADER.max(self.skin.borrow().font_size * 2 + 48)
    }
    #[cfg(test)]
    pub(super) unsafe fn check_feed(&self, hwnd: HWND) {
        *self.updates.lock().unwrap() = Some(Ok(demo_rows()));
        self.tick(hwnd);
        assert_eq!(self.feed.buttons.borrow().len(), 3);
        self.toggle_card(hwnd, 0);
        assert_eq!(
            self.feed.open.borrow().as_ref(),
            Some(&self.visible.borrow()[0].id)
        );
        assert_eq!(text(self.get(ANSWER)), self.visible.borrow()[0].last_answer);
        let mut first = RECT::default();
        let mut second = RECT::default();
        GetWindowRect(self.feed.buttons.borrow()[0], &mut first);
        GetWindowRect(self.feed.buttons.borrow()[1], &mut second);
        assert_eq!(
            second.top - first.top,
            self.px(hwnd, self.feed_header() + BODY)
        );
        self.toggle_card(hwnd, 1);
        GetWindowRect(self.feed.buttons.borrow()[1], &mut second);
        assert_eq!(second.top - first.top, self.px(hwnd, self.feed_header()));
        assert_eq!(text(self.get(REQUEST)), self.visible.borrow()[1].task);
        self.feed.scroll.set(i32::MAX);
        self.layout_feed(hwnd);
        let mut scroll = SCROLLINFO {
            cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
            fMask: SIF_ALL,
            ..Default::default()
        };
        GetScrollInfo(self.get(FEED), SB_VERT, &mut scroll);
        assert_eq!(
            self.feed.scroll.get(),
            (scroll.nMax + 1 - scroll.nPage as i32).max(0)
        );
        self.toggle_card(hwnd, 1);
        assert!(self.feed.open.borrow().is_none());
        assert_eq!(
            GetWindowLongPtrW(self.get(ANSWER), GWL_STYLE) as u32 & WS_VISIBLE,
            0
        );
        // The logo is its own native button and targets its row, not the last selection.
        assert_eq!(self.feed.logos.borrow().len(), 3);
        self.feed.scroll.set(0);
        self.layout_feed(hwnd);
        let logo = self.feed.logos.borrow()[2];
        let mut rect = RECT::default();
        GetWindowRect(logo, &mut rect);
        let mut point = POINT { x: (rect.left + rect.right) / 2, y: (rect.top + rect.bottom) / 2 };
        ScreenToClient(self.get(FEED), &mut point);
        assert_eq!(ChildWindowFromPointEx(self.get(FEED), point, CWP_SKIPINVISIBLE), logo);
        assert!(text(logo).contains("해당 세션으로 이동"));
        SendMessageW(logo, BM_CLICK, 0, 0);
        assert_eq!(self.selected().unwrap().id, self.visible.borrow()[2].id);
        assert!(self.feed.open.borrow().is_none(), "logo must not toggle the card");
        assert!(self.expanded.get(), "unlinked demo sessions show details");
        assert!(self.notice.borrow().contains("연결을 확인할 수 없어"));
        self.command(hwnd, BACK, 0);
    }

    pub(super) unsafe fn create_feed(&self, hwnd: HWND) -> io::Result<()> {
        let name = wide("waid.conversation-cards");
        let class = WNDCLASSW {
            lpfnWndProc: Some(feed_proc),
            hInstance: GetModuleHandleW(null()),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            lpszClassName: name.as_ptr(),
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
            return Err(io::Error::last_os_error());
        }
        let feed = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            name.as_ptr(),
            wide("세션 대화 목록").as_ptr(),
            WS_CHILD | WS_VSCROLL | WS_CLIPCHILDREN,
            0,
            0,
            1,
            1,
            hwnd,
            FEED as HMENU,
            GetModuleHandleW(null()),
            self as *const _ as _,
        );
        if feed.is_null() {
            return Err(io::Error::last_os_error());
        }
        self.controls.borrow_mut().insert(FEED, feed);
        for (id, label) in [
            (REQUEST, "내가 시킨 일"),
            (ANSWER, "답변"),
            (PROMPT, "Prompt · 첫 요청"),
        ] {
            self.add(
                feed,
                id,
                "EDIT",
                label,
                (ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY) as u32 | WS_VSCROLL,
            )?;
        }
        self.add(
            feed,
            OPEN,
            "BUTTON",
            "이 프로젝트에서 이어서 요청하기  ↗",
            BS_OWNERDRAW as u32,
        )?;
        self.add(feed, COPY, "BUTTON", "복사", BS_OWNERDRAW as u32)?;
        Ok(())
    }

    pub(super) unsafe fn rebuild_feed(&self, hwnd: HWND) {
        let rows = self.visible.borrow();
        if self
            .feed
            .open
            .borrow()
            .as_ref()
            .is_some_and(|id| !rows.iter().any(|r| &r.id == id))
        {
            self.feed.open.borrow_mut().take();
        }
        let mut buttons = self.feed.buttons.borrow_mut();
        let mut logos = self.feed.logos.borrow_mut();
        while buttons.len() > rows.len() {
            DestroyWindow(buttons.pop().unwrap());
            DestroyWindow(logos.pop().unwrap());
        }
        // ponytail: two native buttons per session; virtualize if thousands of cards exhaust USER handles.
        for (index, row) in rows.iter().enumerate() {
            if index == buttons.len() {
                let button = CreateWindowExW(
                    0,
                    wide("BUTTON").as_ptr(),
                    null(),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_CLIPSIBLINGS | (BS_OWNERDRAW | BS_NOTIFY) as u32,
                    0,
                    0,
                    1,
                    1,
                    self.get(FEED),
                    (CARD + index) as HMENU,
                    GetModuleHandleW(null()),
                    null(),
                );
                if button.is_null() {
                    break;
                }
                let logo = CreateWindowExW(
                    0, wide("BUTTON").as_ptr(), null(),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | (BS_OWNERDRAW | BS_NOTIFY) as u32,
                    0, 0, 1, 1, self.get(FEED), LOGO as HMENU,
                    GetModuleHandleW(null()), null(),
                );
                if logo.is_null() {
                    DestroyWindow(button);
                    break;
                }
                buttons.push(button);
                logos.push(logo);
                send(button, WM_SETFONT, self.body.get() as usize, 0);
            }
            let logo_label = format!("{} · 해당 세션으로 이동", row.accessible_text());
            if text(logos[index]) != logo_label {
                set_text(logos[index], &logo_label);
            }
            let open = self.feed.open.borrow().as_ref() == Some(&row.id);
            let label = format!(
                "{} · {}",
                row.accessible_text(),
                if open { "접기" } else { "펼치기" }
            );
            if text(buttons[index]) != label {
                set_text(buttons[index], &label);
            }
            if open {
                for (id, value) in [
                    (REQUEST, row.task.as_str()),
                    (ANSWER, row.last_answer.as_str()),
                    (PROMPT, row.summary.as_str()),
                ] {
                    let value = if value.is_empty() || value == "—" {
                        "아직 읽은 기록이 없습니다."
                    } else {
                        value
                    };
                    if text(self.get(id)).replace("\r\n", "\n") != value {
                        set_text(self.get(id), value);
                    }
                }
            }
        }
        drop(buttons);
        drop(logos);
        drop(rows);
        self.layout_feed(hwnd);
    }

    pub(super) unsafe fn layout_feed(&self, hwnd: HWND) {
        let feed = self.get(FEED);
        if feed.is_null() {
            return;
        }
        let p = |n| self.px(hwnd, n);
        let mut bounds = RECT::default();
        GetClientRect(feed, &mut bounds);
        let rows = self.visible.borrow();
        let open = self.feed.open.borrow();
        let open_index = rows.iter().position(|r| Some(&r.id) == open.as_ref());
        let total = p(
            rows.len() as i32 * self.feed_header() + if open_index.is_some() { BODY } else { 0 }
        );
        let scroll = self
            .feed
            .scroll
            .get()
            .clamp(0, (total - bounds.bottom).max(0));
        self.feed.scroll.set(scroll);
        let info = SCROLLINFO {
            cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
            fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
            nMin: 0,
            nMax: total.saturating_sub(1),
            nPage: bounds.bottom.max(0) as u32,
            nPos: scroll,
            ..Default::default()
        };
        SetScrollInfo(feed, SB_VERT, &info, 1);
        GetClientRect(feed, &mut bounds);
        for (index, button) in self.feed.buttons.borrow().iter().enumerate() {
            let y = p(index as i32 * self.feed_header()
                + if open_index.is_some_and(|i| index > i) {
                    BODY
                } else {
                    0
                })
                - scroll;
            MoveWindow(*button, 0, y, bounds.right, p(self.feed_header()), 1);
            SetWindowPos(self.feed.logos.borrow()[index], HWND_TOP,
                p(16), y + p(15), p(42), p(42), SWP_NOACTIVATE);
        }
        for id in [REQUEST, ANSWER, PROMPT, OPEN, COPY] {
            ShowWindow(
                self.get(id),
                if open_index.is_some() {
                    SW_SHOWNA
                } else {
                    SW_HIDE
                },
            );
        }
        if let Some(index) = open_index {
            let y = p(index as i32 * self.feed_header() + self.feed_header()) - scroll;
            for (id, top, height) in [(REQUEST, 30, 66), (ANSWER, 130, 88), (PROMPT, 252, 48)] {
                MoveWindow(
                    self.get(id),
                    p(56),
                    y + p(top),
                    (bounds.right - p(86)).max(1),
                    p(height),
                    1,
                );
                let mut rect = RECT::default();
                GetClientRect(self.get(id), &mut rect);
                rect.left += p(8);
                rect.right -= p(5);
                rect.top += p(5);
                send(self.get(id), EM_SETRECT, 0, &rect as *const _ as isize);
                ShowScrollBar(
                    self.get(id),
                    SB_VERT,
                    if send(self.get(id), EM_GETLINECOUNT, 0, 0)
                        * p(self.skin.borrow().font_size + 3) as isize
                        > (rect.bottom - rect.top) as isize
                    {
                        1
                    } else {
                        0
                    },
                );
            }
            MoveWindow(
                self.get(OPEN),
                p(18),
                y + p(326),
                (bounds.right - p(36)).max(1),
                p(36),
                1,
            );
            MoveWindow(
                self.get(COPY),
                bounds.right - p(72),
                y + p(226),
                p(44),
                p(24),
                1,
            );
        }
        InvalidateRect(feed, null(), 1);
    }

    pub(super) unsafe fn toggle_card(&self, hwnd: HWND, index: usize) {
        let Some(row) = self.visible.borrow().get(index).cloned() else {
            return;
        };
        self.busy.set(true);
        send(self.get(LIST), LB_SETCURSEL, index, 0);
        self.busy.set(false);
        let next = if self.feed.open.borrow().as_ref() == Some(&row.id) {
            None
        } else {
            Some(row.id)
        };
        *self.feed.open.borrow_mut() = next;
        // Keep the clicked header in view when another open card above it collapses.
        self.feed.scroll.set(
            self.feed
                .scroll
                .get()
                .min(self.px(hwnd, index as i32 * self.feed_header())),
        );
        self.rebuild(hwnd);
    }

    unsafe fn draw_card_header(&self, hwnd: HWND, item: &DRAWITEMSTRUCT) {
        let index = item.CtlID as usize - CARD;
        let Some(row) = self.visible.borrow().get(index).cloned() else {
            return;
        };
        let skin = self.skin.borrow();
        let p = |n| self.px(hwnd, n);
        let dc = item.hDC;
        let open = self.feed.open.borrow().as_ref() == Some(&row.id);
        fill(dc, &item.rcItem, skin.background);
        let card = RECT {
            left: p(3),
            top: p(2),
            right: item.rcItem.right - p(3),
            bottom: item.rcItem.bottom - if open { 0 } else { p(8) },
        };
        visual::rounded(
            dc,
            RECT {
                top: card.top + p(2),
                bottom: card.bottom + p(2),
                ..card
            },
            p(20),
            visual::blend(skin.background, skin.foreground, 4),
            None,
        );
        visual::rounded(
            dc,
            card,
            p(20),
            skin.surface,
            Some(if item.itemState & ODS_FOCUS != 0 {
                skin.accent
            } else {
                visual::blend(skin.surface, skin.foreground, 6)
            }),
        );
        if open {
            fill(
                dc,
                &RECT {
                    top: card.bottom - p(20),
                    ..card
                },
                skin.surface,
            );
        }
        let line = p(skin.font_size + 7);
        let arrow = RECT {
            left: card.right - p(40),
            right: card.right - p(10),
            top: p(18),
            bottom: p(48),
        };
        visual::rounded(dc, arrow, p(15), skin.background, None);
        // Draw chevrons with strokes instead of font-dependent Unicode symbols.
        let pen = CreatePen(PS_SOLID, p(2).max(1), skin.foreground);
        let old = SelectObject(dc, pen);
        let cx = (arrow.left + arrow.right) / 2;
        let cy = (arrow.top + arrow.bottom) / 2;
        let sign = if open { -1 } else { 1 };
        MoveToEx(dc, cx - p(4), cy - sign * p(2), null_mut());
        LineTo(dc, cx, cy + sign * p(2));
        LineTo(dc, cx + p(4), cy - sign * p(2));
        SelectObject(dc, old);
        DeleteObject(pen);
        let status_right = arrow.left - p(8);
        let status_left = status_right - p((skin.font_size * 8).max(96));
        let badge_width = p(skin.font_size * 4 + 8);
        let badge_left = (status_left + status_right - badge_width) / 2;
        let badge = RECT {
            left: badge_left,
            right: badge_left + badge_width,
            top: p(22),
            bottom: p(22 + skin.font_size + 6),
        };
        let color = *skin.state_colors.get(&row.state).unwrap_or(&skin.muted);
        visual::rounded(
            dc,
            badge,
            p(11),
            visual::blend(skin.surface, color, 7),
            None,
        );
        draw(
            dc,
            &format!(
                "● {}",
                if row.state == "idle" {
                    "유휴"
                } else {
                    row.status()
                }
            ),
            badge,
            self.body.get(),
            color,
            DT_SINGLELINE | DT_CENTER | DT_VCENTER,
        );
        draw(
            dc,
            &row.model,
            RECT {
                left: status_left,
                right: status_right,
                top: badge.bottom + p(2),
                bottom: badge.bottom + p(2) + line,
            },
            self.body.get(),
            skin.muted,
            DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_END_ELLIPSIS,
        );
        let title = if self.settings.borrow().pinned.contains(&row.id) {
            format!("★ {}", row.project())
        } else {
            row.project().into()
        };
        let date_rect = RECT {
            left: status_left - p(skin.font_size * 5 + 8),
            right: status_left - p(8),
            top: p(22),
            bottom: p(44),
        };
        draw(
            dc,
            &row.date_label(open, crate::time::now()),
            date_rect,
            self.body.get(),
            skin.muted,
            DT_SINGLELINE | DT_RIGHT | DT_VCENTER | DT_END_ELLIPSIS,
        );
        draw(
            dc,
            &title,
            RECT {
                left: p(72),
                top: p(15),
                right: date_rect.left - p(8),
                bottom: p(15) + line,
            },
            self.heading.get(),
            skin.foreground,
            DT_SINGLELINE | DT_END_ELLIPSIS | DT_VCENTER,
        );
        draw(
            dc,
            &row.task,
            RECT {
                left: p(72),
                top: p(17) + line,
                right: status_left - p(8),
                bottom: p(19) + line * 2,
            },
            self.body.get(),
            skin.foreground,
            DT_SINGLELINE | DT_END_ELLIPSIS | DT_VCENTER,
        );
    }

    unsafe fn draw_card_logo(&self, hwnd: HWND, item: &DRAWITEMSTRUCT) {
        let Some(index) = self.feed.logos.borrow().iter().position(|h| *h == item.hwndItem) else { return };
        let rows = self.visible.borrow();
        let Some(row) = rows.get(index) else { return };
        let skin = self.skin.borrow();
        let p = |n| self.px(hwnd, n);
        fill(item.hDC, &item.rcItem, skin.surface);
        visual::rounded(item.hDC, item.rcItem, p(12),
            if item.itemState & ODS_SELECTED != 0 { skin.selection } else { 0xffffff },
            Some(if item.itemState & ODS_FOCUS != 0 { skin.accent }
                else { visual::blend(0xffffff, skin.muted, 8) }));
        if !self.visuals.logo(item.hDC, &row.agent_id, p(5), p(5), p(32)) {
            draw(item.hDC, "›_", item.rcItem, self.heading.get(), 0x343020,
                DT_SINGLELINE | DT_CENTER | DT_VCENTER);
        }
    }

    unsafe fn paint_feed(&self, hwnd: HWND, feed: HWND) {
        let mut ps = PAINTSTRUCT::default();
        let dc = BeginPaint(feed, &mut ps);
        let skin = self.skin.borrow();
        fill(dc, &ps.rcPaint, skin.background);
        let mut bounds = RECT::default();
        GetClientRect(feed, &mut bounds);
        let p = |n| self.px(hwnd, n);
        let rows = self.visible.borrow();
        if let Some((index, row)) = rows
            .iter()
            .enumerate()
            .find(|(_, r)| Some(&r.id) == self.feed.open.borrow().as_ref())
        {
            let y =
                p(index as i32 * self.feed_header() + self.feed_header()) - self.feed.scroll.get();
            let card = RECT {
                left: p(3),
                right: bounds.right - p(3),
                top: y - p(22),
                bottom: y + p(BODY - 8),
            };
            visual::rounded(
                dc,
                card,
                p(20),
                skin.surface,
                Some(visual::blend(skin.surface, skin.foreground, 6)),
            );
            let inner = RECT {
                left: p(12),
                right: bounds.right - p(12),
                top: y,
                bottom: y + p(BODY - 18),
            };
            visual::rounded(dc, inner, p(14), skin.background, None);
            for (label, icon, top, height) in [
                ("내가 시킨 일", "♙", 0, 96),
                ("답변", "✦", 100, 118),
                ("Prompt · 첫 요청", ">_", 222, 78),
            ] {
                let avatar = RECT {
                    left: p(20),
                    right: p(48),
                    top: y + p(top + 2),
                    bottom: y + p(top + 30),
                };
                visual::rounded(dc, avatar, p(14), skin.selection, None);
                draw(
                    dc,
                    icon,
                    avatar,
                    self.heading.get(),
                    skin.accent,
                    DT_SINGLELINE | DT_CENTER | DT_VCENTER,
                );
                draw(
                    dc,
                    label,
                    RECT {
                        left: p(58),
                        right: bounds.right - p(85),
                        top: y + p(top),
                        bottom: y + p(top + 28),
                    },
                    self.heading.get(),
                    skin.foreground,
                    DT_SINGLELINE | DT_VCENTER,
                );
                visual::rounded(
                    dc,
                    RECT {
                        left: p(54),
                        right: bounds.right - p(26),
                        top: y + p(top + 29),
                        bottom: y + p(top + height + 2),
                    },
                    p(12),
                    skin.surface,
                    None,
                );
            }
            // The source supplies session activity time, not individual message timestamps.
            draw(
                dc,
                &format!("{} · 마지막 기록 {}", row.agent, row.short_date()),
                RECT {
                    left: p(58),
                    right: bounds.right - p(28),
                    top: y + p(303),
                    bottom: y + p(323),
                },
                self.body.get(),
                skin.muted,
                DT_SINGLELINE | DT_END_ELLIPSIS | DT_VCENTER,
            );
        }
        EndPaint(feed, &ps);
    }
}

unsafe extern "system" fn feed_proc(feed: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        SetWindowLongPtrW(
            feed,
            GWLP_USERDATA,
            (*(lp as *const CREATESTRUCTW)).lpCreateParams as isize,
        );
    }
    let ptr = GetWindowLongPtrW(feed, GWLP_USERDATA) as *const Window;
    if ptr.is_null() {
        return DefWindowProcW(feed, msg, wp, lp);
    }
    let s = &*ptr;
    let hwnd = GetParent(feed);
    match msg {
        WM_PAINT => s.paint_feed(hwnd, feed),
        WM_ERASEBKGND => return 1,
        WM_SETCURSOR if s.feed.logos.borrow().contains(&(wp as HWND)) => {
            SetCursor(LoadCursorW(null_mut(), IDC_HAND));
            return 1;
        }
        WM_DRAWITEM if wp == LOGO => s.draw_card_logo(hwnd, &*(lp as *const DRAWITEMSTRUCT)),
        WM_DRAWITEM if wp >= CARD => s.draw_card_header(hwnd, &*(lp as *const DRAWITEMSTRUCT)),
        WM_DRAWITEM | WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
            return SendMessageW(hwnd, msg, wp, lp)
        }
        WM_COMMAND => {
            let id = wp & 0xffff;
            let notification = (wp >> 16) & 0xffff;
            let logo_index = if id == LOGO {
                s.feed.logos.borrow().iter().position(|h| *h == lp as HWND)
            } else { None };
            if let Some(index) = logo_index.filter(|_| notification == BN_CLICKED as usize) {
                s.busy.set(true);
                send(s.get(LIST), LB_SETCURSEL, index, 0);
                s.busy.set(false);
                s.activate_selected(hwnd);
            }
            if id >= CARD && notification == BN_CLICKED as usize {
                s.toggle_card(hwnd, id - CARD);
            }
            if (id >= CARD || logo_index.is_some()) && notification == BN_SETFOCUS as usize {
                s.busy.set(true);
                send(s.get(LIST), LB_SETCURSEL, logo_index.unwrap_or_else(|| id - CARD), 0);
                s.busy.set(false);
                let mut rect = RECT::default();
                GetWindowRect(s.feed.buttons.borrow()[logo_index.unwrap_or_else(|| id - CARD)], &mut rect);
                let mut point = POINT { x: 0, y: rect.top };
                ScreenToClient(feed, &mut point);
                let mut bounds = RECT::default();
                GetClientRect(feed, &mut bounds);
                let delta = if point.y < 0 {
                    point.y
                } else {
                    (point.y + s.px(hwnd, s.feed_header()) - bounds.bottom).max(0)
                };
                if delta != 0 {
                    s.feed.scroll.set(s.feed.scroll.get() + delta);
                    s.layout_feed(hwnd);
                }
            }
            if id == OPEN {
                let index = s
                    .visible
                    .borrow()
                    .iter()
                    .position(|r| Some(&r.id) == s.feed.open.borrow().as_ref());
                if let Some(index) = index {
                    s.busy.set(true);
                    send(s.get(LIST), LB_SETCURSEL, index, 0);
                    s.busy.set(false);
                    s.activate_selected(hwnd);
                }
            }
            if id == COPY {
                send(s.get(PROMPT), EM_SETSEL, 0, -1);
                SendMessageW(s.get(PROMPT), WM_COPY, 0, 0);
            }
        }
        WM_VSCROLL | WM_MOUSEWHEEL => {
            let mut info = SCROLLINFO {
                cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                fMask: SIF_ALL,
                ..Default::default()
            };
            GetScrollInfo(feed, SB_VERT, &mut info);
            let position = if msg == WM_MOUSEWHEEL {
                info.nPos - ((wp >> 16) as i16 as i32) * s.px(hwnd, 48) / 120
            } else {
                match (wp & 0xffff) as i32 {
                    SB_LINEUP => info.nPos - s.px(hwnd, 32),
                    SB_LINEDOWN => info.nPos + s.px(hwnd, 32),
                    SB_PAGEUP => info.nPos - info.nPage as i32,
                    SB_PAGEDOWN => info.nPos + info.nPage as i32,
                    SB_THUMBTRACK | SB_THUMBPOSITION => info.nTrackPos,
                    SB_TOP => 0,
                    SB_BOTTOM => info.nMax,
                    _ => info.nPos,
                }
            };
            s.feed.scroll.set(position);
            s.layout_feed(hwnd);
        }
        _ => return DefWindowProcW(feed, msg, wp, lp),
    }
    0
}
