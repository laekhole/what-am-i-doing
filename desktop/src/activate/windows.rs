//! Manual associations are window returns, never proof of a conversation.
use serde_json::{json, Value};
use std::{io::Write, mem::size_of, os::windows::{io::FromRawHandle, process::CommandExt}, process::Command};
use windows_sys::Win32::{
    Foundation::{CloseHandle, FILETIME, HWND, INVALID_HANDLE_VALUE, LPARAM},
    System::{
        Console::{AttachConsole, FreeConsole, GetConsoleProcessList, GetConsoleWindow, GetStdHandle, STD_OUTPUT_HANDLE},
        Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS},
        Threading::{GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION},
    },
    UI::WindowsAndMessaging::{EnumWindows, GetClassNameW, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindowAsync, SW_RESTORE},
};

fn process(pid: u32) -> Option<(u64, String)> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() { return None; }
        let mut created: FILETIME = std::mem::zeroed();
        let mut exit = created;
        let mut kernel = created;
        let mut user = created;
        let mut path = [0u16; 32768];
        let mut len = path.len() as u32;
        let ok = GetProcessTimes(handle, &mut created, &mut exit, &mut kernel, &mut user) != 0
            && exit.dwLowDateTime == 0 && exit.dwHighDateTime == 0
            && QueryFullProcessImageNameW(handle, 0, path.as_mut_ptr(), &mut len) != 0;
        CloseHandle(handle);
        ok.then(|| (((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64,
            String::from_utf16_lossy(&path[..len as usize])))
    }
}

fn basename(path: &str) -> &str { path.rsplit(['\\', '/']).next().unwrap_or(path) }

fn app_kind(path: &str) -> Option<&'static str> {
    if basename(path).eq_ignore_ascii_case("ChatGPT.exe") { Some("chatgpt") }
    else if basename(path).eq_ignore_ascii_case("claude.exe") { Some("claude") }
    else { None }
}

fn shell_pids() -> (Vec<u32>, Vec<u32>) {
    let hosts = classic_console_owners();
    if hosts.is_empty() { return (Vec::new(), Vec::new()); }
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE { return (Vec::new(), Vec::new()); }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut more = Process32FirstW(snapshot, &mut entry);
        let mut pids = Vec::new();
        let mut candidates = Vec::new();
        while more != 0 {
            let n = entry.szExeFile.iter().position(|c| *c == 0).unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..n]);
            if name.eq_ignore_ascii_case("powershell.exe") || name.eq_ignore_ascii_case("pwsh.exe") {
                pids.push(entry.th32ProcessID);
                // Classic console HWND ownership may name the shell directly,
                // including consoles explicitly launched through conhost.exe.
                if hosts.contains(&entry.th32ProcessID) {
                    candidates.push(entry.th32ProcessID);
                }
            }
            if hosts.contains(&entry.th32ProcessID)
                && (name.eq_ignore_ascii_case("conhost.exe") || name.eq_ignore_ascii_case("OpenConsole.exe")) {
                candidates.push(entry.th32ParentProcessID);
            }
            more = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
        candidates.retain(|pid| pids.contains(pid));
        candidates.sort_unstable();
        candidates.dedup();
        (pids, candidates)
    }
}

fn window(hwnd: HWND, kind: &str) -> Option<Value> {
    unsafe {
        if IsWindowVisible(hwnd) == 0 { return None; }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        let (created, exe) = process(pid)?;
        if kind != "powershell" && app_kind(&exe) != Some(kind) { return None; }
        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
        if len <= 0 { return None; }
        Some(json!({"kind":kind, "hwnd":hwnd as usize, "pid":pid, "created":created,
            "exe":exe, "label":format!("{} — {} (PID {})", kind, String::from_utf16_lossy(&title[..len as usize]), pid)}))
    }
}

fn console_window(pid: u32, all_shells: &[u32]) -> Option<Value> {
    let identity = process(pid)?;
    if !["powershell.exe", "pwsh.exe"].iter().any(|name| basename(&identity.1).eq_ignore_ascii_case(name)) {
        return None;
    }
    unsafe {
        if !GetConsoleWindow().is_null() || AttachConsole(pid) == 0 { return None; }
        let hwnd = GetConsoleWindow();
        let mut members = [0u32; 1024];
        let count = GetConsoleProcessList(members.as_mut_ptr(), members.len() as u32) as usize;
        let mut class = [0u16; 64];
        let len = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32).max(0) as usize;
        let classic = String::from_utf16_lossy(&class[..len]) == "ConsoleWindowClass";
        // Pseudoconsole handles are hidden message windows, not terminal tabs.
        let target = if classic && count > 0 && count <= members.len()
            && console_members_match(pid, all_shells, &members[..count])
        { window(hwnd, "powershell") } else { None };
        FreeConsole();
        let mut target = target?;
        if target["created"].as_u64()? < identity.0 { return None; }
        if process(pid).as_ref() != Some(&identity) { return None; }
        target["shell_pid"] = json!(pid);
        target["shell_created"] = json!(identity.0);
        target["shell_exe"] = json!(identity.1);
        Some(target)
    }
}

fn console_members_match(pid: u32, shells: &[u32], members: &[u32]) -> bool {
    members.contains(&pid) && shells.contains(&pid)
        && shells.iter().filter(|p| members.contains(p)).count() == 1
}

unsafe extern "system" fn enumerate(hwnd: HWND, data: LPARAM) -> i32 {
    let out = &mut *(data as *mut Vec<Value>);
    if IsWindowVisible(hwnd) == 0 { return 1; }
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if let Some((_, path)) = process(pid) {
        if let Some(kind) = app_kind(&path) {
            if let Some(target) = window(hwnd, kind) { out.push(target); }
        }
    }
    1
}

/// Read-only candidates; title helps the user choose but is never identity proof.
pub fn windows() -> Vec<Value> {
    let mut out = Vec::new();
    unsafe { EnumWindows(Some(enumerate), &mut out as *mut Vec<Value> as LPARAM); }
    out.extend(console_windows());
    out.sort_by_key(|v| v["label"].as_str().unwrap_or_default().to_string());
    out
}

fn console_windows() -> Vec<Value> {
    if classic_console_owners().is_empty() { return Vec::new(); }
    let Ok(exe) = std::env::current_exe() else { return Vec::new(); };
    let mut command = Command::new(exe);
    // DETACHED_PROCESS: the helper starts without any console, including in a
    // debug build. Console control events cannot terminate the desktop process.
    command.arg("--waid-console-probe").creation_flags(0x00000008);
    super::command_json(command).and_then(|v| v["result"]["windows"].as_array().cloned()).unwrap_or_default()
}

fn classic_console_owners() -> Vec<u32> {
    unsafe extern "system" fn find(hwnd: HWND, data: LPARAM) -> i32 {
        let mut class = [0u16; 64];
        let len = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32).max(0) as usize;
        if IsWindowVisible(hwnd) != 0 && String::from_utf16_lossy(&class[..len]) == "ConsoleWindowClass" {
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            (*(data as *mut Vec<u32>)).push(pid);
        }
        1
    }
    let mut found = Vec::new();
    unsafe { EnumWindows(Some(find), &mut found as *mut Vec<u32> as LPARAM); }
    found
}

/// Call before normal app initialization. This helper only reads native metadata.
pub fn probe_requested() -> bool {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_none_or(|arg| arg != "--waid-console-probe") { return false; }
    if args.len() != 1 { return true; }
    // Preserve the inherited pipe before AttachConsole changes standard handles.
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE { return true; }
    let mut output = unsafe { std::fs::File::from_raw_handle(handle) };
    // Only direct classic console hosts are supported. Parent metadata narrows
    // candidates; AttachConsole membership and creation times prove the binding.
    // No generic scan/attachment of other apps' pseudoconsole processes.
    let (shells, pids) = shell_pids();
    let candidates: Vec<_> = pids.iter().filter_map(|pid| console_window(*pid, &shells)).collect();
    let _ = write!(output, "{}", json!({"ok":true,"result":{"windows":candidates}}));
    true
}

fn same_window(saved: &Value, current: &Value) -> bool {
    let Some(kind) = saved["kind"].as_str() else { return false; };
    if !["chatgpt", "claude", "powershell"].contains(&kind) { return false; }
    let fields: &[&str] = if kind == "powershell" {
        &["kind", "hwnd", "pid", "created", "exe", "shell_pid", "shell_created", "shell_exe"]
    } else { &["kind", "hwnd", "pid", "created", "exe"] };
    fields.iter().all(|field| !saved[*field].is_null() && saved[*field] == current[*field])
}

/// Activate only a still-existing user-selected window. No app launching or input.
pub fn open_window(saved: &Value) -> Result<(), String> {
    let stale = "연결한 창이 종료되었거나 바뀌었습니다. 창을 다시 연결하세요.";
    let hwnd = saved["hwnd"].as_u64().and_then(|v| usize::try_from(v).ok()).ok_or(stale)? as HWND;
    let kind = saved["kind"].as_str().ok_or(stale)?;
    let current = if kind == "powershell" {
        let pid = saved["shell_pid"].as_u64().and_then(|v| u32::try_from(v).ok()).ok_or(stale)?;
        console_windows().into_iter().find(|v| v["shell_pid"].as_u64() == Some(pid as u64))
    } else { window(hwnd, kind) }.ok_or(stale)?;
    if !same_window(saved, &current) { return Err(stale.into()); }
    unsafe {
        if IsIconic(hwnd) != 0 { ShowWindowAsync(hwnd, SW_RESTORE); }
        if SetForegroundWindow(hwnd) == 0 {
            return Err("Windows에서 창 전환을 허용하지 않았습니다. 작업 표시줄에서 선택하세요.".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_window_identity_rejects_reuse_and_ignores_titles() {
        let saved = json!({"kind":"powershell","hwnd":7,"pid":10,"created":100,"exe":"conhost.exe",
            "shell_pid":11,"shell_created":99,"shell_exe":"powershell.exe","label":"old"});
        let mut current = saved.clone();
        current["label"] = json!("new title");
        assert!(same_window(&saved, &current));
        for field in ["hwnd", "pid", "created", "exe", "shell_pid", "shell_created", "shell_exe", "kind"] {
            let mut changed = current.clone();
            changed[field] = json!("changed");
            assert!(!same_window(&saved, &changed), "{field}");
        }
        assert!(!same_window(&json!({}), &json!({})));
    }

    #[test]
    fn native_process_identity_and_stale_binding_are_read_only() {
        let identity = process(std::process::id()).expect("own process metadata");
        assert!(identity.0 > 0);
        assert!(!identity.1.is_empty());
        assert!(process(u32::MAX).is_none());
        assert!(open_window(&json!({"kind":"chatgpt","hwnd":0})).is_err());
        // A message-only fixture never appears on screen or takes focus. A title
        // resembling an app must not bypass verification of the owner process.
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, HWND_MESSAGE, WS_VISIBLE};
            let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
            let title: Vec<u16> = "ChatGPT\0".encode_utf16().collect();
            let hwnd = CreateWindowExW(0, class.as_ptr(), title.as_ptr(), WS_VISIBLE,
                0, 0, 1, 1, HWND_MESSAGE, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null());
            assert!(!hwnd.is_null());
            assert!(window(hwnd, "chatgpt").is_none());
            DestroyWindow(hwnd);
        }
    }

    #[test]
    fn console_selection_requires_one_attached_powershell() {
        assert!(console_members_match(7, &[7, 8], &[7, 99]));
        assert!(!console_members_match(7, &[7, 8], &[7, 8, 99]));
        assert!(!console_members_match(7, &[7, 8], &[8, 99]));
        assert!(!console_members_match(7, &[8], &[7, 99]));
    }

    #[test]
    #[ignore = "read-only inventory of installed native app windows"]
    fn installed_windows_have_revalidatable_identity() {
        let candidates = windows();
        assert!(!candidates.is_empty(), "no supported native windows running");
        for candidate in candidates {
            println!("{} PID {}", candidate["kind"], candidate["pid"]);
            let identity = process(candidate["pid"].as_u64().unwrap() as u32).unwrap();
            assert_eq!(candidate["created"].as_u64(), Some(identity.0));
            assert_eq!(candidate["exe"].as_str(), Some(identity.1.as_str()));
        }
    }

    #[test]
    #[ignore = "set WAID_TEST_CONSOLE_PID to an existing classic PowerShell console"]
    fn classic_console_owner_is_discovered() {
        let pid = std::env::var("WAID_TEST_CONSOLE_PID")
            .expect("classic console fixture PID").parse::<u32>().unwrap();
        assert!(classic_console_owners().contains(&pid), "fixture must own a visible classic console");
        let (shells, candidates) = shell_pids();
        assert!(shells.contains(&pid));
        assert!(candidates.contains(&pid), "visible PowerShell console was excluded");
    }
}
