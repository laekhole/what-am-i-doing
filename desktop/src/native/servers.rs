//! Remote server (SSH) settings window. Opt-in: an empty list opens no connection.
use super::*;
use crate::remote::Remotes;
use std::sync::{
    mpsc::{self, Receiver, TryRecvError},
    Arc,
};
use waid::remote::{self, Problem};

/// Main window button and tray item that open this window.
pub(super) const OPEN: usize = 63;
const CLASS: &str = "waid.servers.v1";
const INTRO: usize = 1;
const TEXT: usize = 2;
const STATUS: usize = 3;
const SAVE: usize = 4;
const INSTALL: usize = 5;
const CLOSE: usize = 6;
const WIDTH: i32 = 560;
const HEIGHT: i32 = 500;

type Installs = (usize, Receiver<(String, Result<String, Problem>)>);

pub(super) struct State {
    remotes: RefCell<Option<Remotes>>,
    hwnd: Cell<HWND>,
    shown: RefCell<String>,
    notice: RefCell<String>,
    installs: RefCell<Option<Installs>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            remotes: RefCell::new(None),
            hwnd: Cell::new(null_mut()),
            shown: RefCell::new(String::new()),
            notice: RefCell::new(String::new()),
            installs: RefCell::new(None),
        }
    }
}

impl State {
    pub(super) fn hwnd(&self) -> HWND {
        self.hwnd.get()
    }
}

fn labels() -> [(usize, &'static str); 3] {
    [
        (SAVE, tr("저장하고 연결", "Save and connect")),
        (INSTALL, tr("서버에 설치", "Install on servers")),
        (CLOSE, tr("닫기", "Close")),
    ]
}

fn intro() -> String {
    waid::trf!(
        "SSH로 서버에 접속해 그 서버의 Claude Code·Codex 세션도 함께 봅니다.\r\n\
         한 줄에 서버 하나씩, ~/.ssh/config의 Host 이름이나 user@host를 적으세요. 앞에 #을 붙이면 연결하지 않습니다.\r\n\
         비밀번호 입력 없이 접속되는 키 인증이 필요합니다. 서버에는 {}에 waid가 있어야 하며, 없으면 '서버에 설치'로 넣을 수 있습니다({}).",
        "Also show Claude Code and Codex sessions from servers you reach over SSH.\r\n\
         Enter one server per line: a Host name from ~/.ssh/config or user@host. Start a line with # to keep it off.\r\n\
         Key authentication without a password prompt is required. The server needs waid at {}; 'Install on servers' copies it there ({}).",
        remote::INSTALL_PATH,
        remote::SUPPORTED_PLATFORM
    )
}

unsafe fn message(owner: HWND, text: &str, title: &str, flags: MESSAGEBOX_STYLE) -> i32 {
    MessageBoxW(owner, wide(text).as_ptr(), wide(title).as_ptr(), flags)
}

impl Window {
    /// Starts links for saved servers. Called once after the local collector starts.
    pub(super) fn start_remotes(&self, hub: Arc<crate::Hub>) {
        let servers = self.settings.borrow().remote_servers.clone();
        let mut remotes = Remotes::new(hub);
        remotes.apply(&servers);
        *self.servers.remotes.borrow_mut() = Some(remotes);
    }

    pub(super) unsafe fn open_servers(&self, owner: HWND) {
        let existing = self.servers.hwnd.get();
        if !existing.is_null() {
            ShowWindow(existing, SW_SHOWNORMAL);
            SetForegroundWindow(existing);
            return;
        }
        let instance = GetModuleHandleW(null());
        let class = wide(CLASS);
        let wc = WNDCLASSW {
            lpfnWndProc: Some(servers_proc),
            hInstance: instance,
            hIcon: self.visuals.app,
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_BTNFACE + 1) as usize as HBRUSH,
            lpszClassName: class.as_ptr(),
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
            *self.notice.borrow_mut() = io::Error::last_os_error().to_string();
            return;
        }
        let dpi = GetDpiForWindow(owner).max(96);
        let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU;
        let mut frame = RECT {
            left: 0,
            top: 0,
            right: WIDTH * dpi as i32 / 96,
            bottom: HEIGHT * dpi as i32 / 96,
        };
        AdjustWindowRectExForDpi(&mut frame, style, 0, 0, dpi);
        let mut anchor = RECT::default();
        GetWindowRect(owner, &mut anchor);
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            wide(tr("원격 서버 (SSH)", "Remote servers (SSH)")).as_ptr(),
            style,
            anchor.left + 24 * dpi as i32 / 96,
            anchor.top + 48 * dpi as i32 / 96,
            frame.right - frame.left,
            frame.bottom - frame.top,
            owner,
            null_mut(),
            instance,
            self as *const _ as *const _,
        );
        if hwnd.is_null() {
            *self.notice.borrow_mut() = io::Error::last_os_error().to_string();
            return;
        }
        self.servers.hwnd.set(hwnd);
        let px = |n: i32| n * dpi as i32 / 96;
        let font = self.body.get();
        let add = |id: usize, class: &str, text: &str, style: u32, x: i32, y: i32, w: i32, h: i32, ex: u32| {
            let child = CreateWindowExW(
                ex,
                wide(class).as_ptr(),
                wide(text).as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                px(x),
                px(y),
                px(w),
                px(h),
                hwnd,
                id as HMENU,
                instance,
                null(),
            );
            if !font.is_null() {
                SendMessageW(child, WM_SETFONT, font as usize, 1);
            }
            child
        };
        // SS_LEFT is 0: a plain left-aligned, word-wrapped static label.
        add(INTRO, "STATIC", &intro(), 0, 16, 12, WIDTH - 32, 92, 0);
        let editor = add(
            TEXT,
            "EDIT",
            &remote::format_servers(&self.settings.borrow().remote_servers),
            WS_TABSTOP | WS_VSCROLL | (ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32,
            16,
            110,
            WIDTH - 32,
            130,
            WS_EX_CLIENTEDGE,
        );
        send(editor, EM_SETLIMITTEXT, 8 * 1024, 0);
        add(
            STATUS,
            "EDIT",
            "",
            WS_TABSTOP | WS_VSCROLL | (ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY) as u32,
            16,
            252,
            WIDTH - 32,
            184,
            WS_EX_CLIENTEDGE,
        );
        for (i, (id, label)) in labels().into_iter().enumerate() {
            let kind = if id == SAVE { BS_DEFPUSHBUTTON } else { BS_PUSHBUTTON };
            add(id, "BUTTON", label, WS_TABSTOP | kind as u32, 16 + i as i32 * 180, 450, 168, 34, 0);
        }
        self.servers.shown.borrow_mut().clear();
        self.servers.notice.borrow_mut().clear();
        self.refresh_servers();
        ShowWindow(hwnd, SW_SHOWNORMAL);
        SetFocus(editor);
    }

    pub(super) unsafe fn close_servers(&self) {
        let hwnd = self.servers.hwnd.get();
        if !hwnd.is_null() {
            DestroyWindow(hwnd);
        }
    }

    fn servers_text(&self) -> String {
        let servers = self.settings.borrow().remote_servers.clone();
        let mut lines = Vec::new();
        if self.demo {
            lines.push(tr("데모 모드에서는 서버에 연결하지 않습니다.", "Demo mode does not connect to servers.").to_string());
        } else if servers.is_empty() {
            lines.push(tr(
                "등록된 서버가 없습니다. waid는 어떤 서버에도 접속하지 않습니다.",
                "No servers added. waid does not connect to any server.",
            ).to_string());
        }
        let live = self
            .servers
            .remotes
            .borrow()
            .as_ref()
            .map(Remotes::statuses)
            .unwrap_or_default();
        for server in &servers {
            let line = if !server.enabled {
                waid::trf!("○ {} — 꺼짐", "○ {} — Off", server.destination)
            } else if let Some((_, status, sessions, _)) = live.iter().find(|(d, ..)| *d == server.destination) {
                let status = if status.is_empty() { tr("연결 준비 중…", "Preparing…").to_string() } else { status.clone() };
                if *sessions > 0 {
                    waid::trf!("● {} — {status} · 세션 {sessions}개", "● {} — {status} · {sessions} sessions", server.destination)
                } else {
                    format!("● {} — {status}", server.destination)
                }
            } else {
                waid::trf!("● {} — 연결 준비 중…", "● {} — Preparing…", server.destination)
            };
            lines.push(line);
        }
        let notice = self.servers.notice.borrow();
        if !notice.is_empty() {
            lines.push(String::new());
            lines.push(notice.clone());
        }
        lines.join("\r\n")
    }

    unsafe fn refresh_servers(&self) {
        let hwnd = self.servers.hwnd.get();
        if hwnd.is_null() {
            return;
        }
        let text = self.servers_text();
        if *self.servers.shown.borrow() != text {
            set_text(GetDlgItem(hwnd, STATUS as i32), &text);
            *self.servers.shown.borrow_mut() = text;
        }
    }

    /// Polls installs and refreshes the status text. Runs on the main window timer.
    pub(super) unsafe fn servers_tick(&self) {
        let mut finished = Vec::new();
        let mut done = false;
        if let Some((pending, rx)) = self.servers.installs.borrow_mut().as_mut() {
            loop {
                match rx.try_recv() {
                    Ok(result) => {
                        finished.push(result);
                        *pending = pending.saturating_sub(1);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        *pending = 0;
                        break;
                    }
                }
            }
            done = *pending == 0;
        }
        if done {
            self.servers.installs.borrow_mut().take();
        }
        for (destination, result) in finished {
            let line = match result {
                Ok(version) => {
                    if let Some(remotes) = self.servers.remotes.borrow().as_ref() {
                        remotes.reconnect(&destination);
                    }
                    waid::trf!("{destination}: waid {version} 설치 완료", "{destination}: Installed waid {version}")
                }
                Err(problem) => waid::trf!(
                    "{destination}: 설치 실패 — {}",
                    "{destination}: Install failed — {}",
                    problem.message(&destination)
                ),
            };
            let mut notice = self.servers.notice.borrow_mut();
            if notice.starts_with(tr("설치 중", "Installing")) {
                notice.clear();
            }
            if !notice.is_empty() {
                notice.push_str("\r\n");
            }
            notice.push_str(&line);
        }
        self.refresh_servers();
    }

    unsafe fn save_servers(&self, hwnd: HWND) {
        let text = text(GetDlgItem(hwnd, TEXT as i32));
        let servers = match remote::parse_servers(&text) {
            Ok(servers) => servers,
            Err(error) => {
                message(hwnd, &error, tr("서버 목록을 확인하세요", "Check the server list"), MB_OK | MB_ICONWARNING);
                return;
            }
        };
        let first_enable = !self.settings.borrow().remote_servers.iter().any(|s| s.enabled)
            && servers.iter().any(|s| s.enabled);
        if first_enable && !self.demo {
            let consent = message(
                hwnd,
                tr(
                    "원격 서버 연결을 켭니다.\n\nwaid가 이 PC의 SSH 설정과 키로 등록한 서버에 접속해, 그 서버에서 waid를 읽기 전용으로 실행하고 세션 정보(요청·답변 포함)를 이 PC로 가져옵니다.\n가져온 내용은 이 PC 화면에만 표시하며 개발자나 다른 서버로 보내지 않습니다. 목록을 비우거나 #으로 끄면 연결을 끊습니다.\n\n계속할까요?",
                    "Turn on remote server connections?\n\nwaid will connect to the listed servers with this PC's SSH settings and keys, run waid there read-only, and bring session information (including requests and answers) to this PC.\nIt is shown only on this PC and is not sent to the developer or any other server. Clear the list or add # to disconnect.\n\nContinue?",
                ),
                tr("원격 서버 연결", "Remote server connections"),
                MB_OKCANCEL | MB_ICONINFORMATION,
            );
            if consent != IDOK {
                return;
            }
        }
        self.settings.borrow_mut().remote_servers = servers.clone();
        self.changed();
        self.save();
        if let Some(remotes) = self.servers.remotes.borrow_mut().as_mut() {
            remotes.apply(&servers);
        }
        set_text(GetDlgItem(hwnd, TEXT as i32), &remote::format_servers(&servers));
        *self.servers.notice.borrow_mut() = tr("저장했습니다.", "Saved.").into();
        self.refresh_servers();
    }

    unsafe fn install_servers(&self, hwnd: HWND) {
        if self.servers.installs.borrow().is_some() {
            return;
        }
        let binary = crate::remote::bundled();
        if binary.is_empty() {
            message(hwnd, &Problem::NotBundled.message(""), tr("서버에 설치", "Install on servers"), MB_OK | MB_ICONWARNING);
            return;
        }
        let enabled: Vec<String> = self
            .settings
            .borrow()
            .remote_servers
            .iter()
            .filter(|s| s.enabled)
            .map(|s| s.destination.clone())
            .collect();
        if enabled.is_empty() {
            message(
                hwnd,
                tr("먼저 서버를 적고 '저장하고 연결'을 누르세요.", "Add a server and choose 'Save and connect' first."),
                tr("서버에 설치", "Install on servers"),
                MB_OK | MB_ICONINFORMATION,
            );
            return;
        }
        let flagged: Vec<String> = self
            .servers
            .remotes
            .borrow()
            .as_ref()
            .map(Remotes::statuses)
            .unwrap_or_default()
            .into_iter()
            .filter(|(destination, _, _, needs)| *needs && enabled.contains(destination))
            .map(|(destination, ..)| destination)
            .collect();
        let targets = if flagged.is_empty() { enabled } else { flagged };
        let list: String = targets.iter().map(|d| format!("\n  • {d}")).collect();
        let answer = message(
            hwnd,
            &waid::trf!(
                "다음 서버의 {}에 waid {}({})를 설치하거나 업데이트합니다. 다른 파일과 설정은 바꾸지 않습니다.\n{list}\n\n계속할까요?",
                "Install or update waid {1} ({2}) at {0} on these servers. No other files or settings change.\n{list}\n\nContinue?",
                remote::INSTALL_PATH,
                remote::VERSION,
                remote::SUPPORTED_PLATFORM
            ),
            tr("서버에 설치", "Install on servers"),
            MB_OKCANCEL | MB_ICONQUESTION,
        );
        if answer != IDOK {
            return;
        }
        let (tx, rx) = mpsc::channel();
        for destination in &targets {
            let tx = tx.clone();
            let destination = destination.clone();
            std::thread::spawn(move || {
                let result = remote::install(&destination, binary);
                let _ = tx.send((destination, result));
            });
        }
        *self.servers.installs.borrow_mut() = Some((targets.len(), rx));
        *self.servers.notice.borrow_mut() = waid::trf!("설치 중… ({}대)", "Installing… ({} servers)", targets.len());
        self.refresh_servers();
    }
}

unsafe extern "system" fn servers_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*(lp as *const CREATESTRUCTW)).lpCreateParams as isize);
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Window;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let s = &*ptr;
    match msg {
        // Buttons only; edit-control notifications share WM_COMMAND.
        WM_COMMAND if (wp >> 16) & 0xffff == BN_CLICKED as usize => match wp & 0xffff {
            SAVE => s.save_servers(hwnd),
            INSTALL => s.install_servers(hwnd),
            CLOSE => {
                DestroyWindow(hwnd);
            }
            _ => {}
        },
        WM_CLOSE => {
            DestroyWindow(hwnd);
        }
        WM_DESTROY => {
            s.servers.hwnd.set(null_mut());
            s.servers.shown.borrow_mut().clear();
        }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return DefWindowProcW(hwnd, msg, wp, lp);
        }
        _ => return DefWindowProcW(hwnd, msg, wp, lp),
    }
    0
}
