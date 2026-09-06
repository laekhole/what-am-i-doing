//! Read-only process enumeration without a subprocess or external crates.
//! ABI: https://learn.microsoft.com/windows/win32/api/tlhelp32/ns-tlhelp32-processentry32w

use super::Process;
use std::{ffi::c_void, io, mem::size_of};

type Handle = *mut c_void;
const INVALID_HANDLE_VALUE: Handle = -1isize as Handle;
const TH32CS_SNAPPROCESS: u32 = 0x00000002;
const ERROR_NO_MORE_FILES: i32 = 18;

#[repr(C)]
struct ProcessEntry {
    size: u32,
    usage: u32,
    pid: u32,
    default_heap: usize,
    module: u32,
    threads: u32,
    parent_pid: u32,
    priority: i32,
    flags: u32,
    exe: [u16; 260],
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> Handle;
    fn Process32FirstW(snapshot: Handle, entry: *mut ProcessEntry) -> i32;
    fn Process32NextW(snapshot: Handle, entry: *mut ProcessEntry) -> i32;
    fn CloseHandle(handle: Handle) -> i32;
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> Handle;
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn GetProcAddress(module: Handle, name: *const u8) -> *mut c_void;
    fn LocalFree(memory: Handle) -> Handle;
}

#[link(name = "shell32")]
extern "system" {
    fn CommandLineToArgvW(command: *const u16, count: *mut i32) -> *mut *mut u16;
}

// ProcessCommandLineInformation (60) returns a local UNICODE_STRING.
// https://github.com/winsiderss/phnt/blob/master/ntpsapi.h
// Resolve dynamically: unsupported/denied queries fall back to the Tool Help name.
fn arguments(pid: u32) -> Option<Vec<String>> {
    type Query = unsafe extern "system" fn(Handle, u32, *mut c_void, u32, *mut u32) -> i32;
    static QUERY: std::sync::OnceLock<Option<Query>> = std::sync::OnceLock::new();
    let query = (*QUERY.get_or_init(|| unsafe {
        let name: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
        let module = GetModuleHandleW(name.as_ptr());
        if module.is_null() { return None; }
        let address = GetProcAddress(module, b"NtQueryInformationProcess\0".as_ptr());
        if address.is_null() { None } else { Some(std::mem::transmute::<*mut c_void, Query>(address)) }
    }))?;
    // SAFETY: read-only limited query rights, no VM read/write or debug privilege.
    let handle = unsafe { OpenProcess(0x1000, 0, pid) };
    if handle.is_null() { return None; }
    let handle = Snapshot(handle);
    let mut needed = 0;
    unsafe { query(handle.0, 60, std::ptr::null_mut(), 0, &mut needed); }
    if !(size_of::<UnicodeString>() as u32..=128 * 1024).contains(&needed) { return None; }
    let mut buffer = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
    // SAFETY: aligned owned buffer, capacity supplied in bytes; kernel fills it.
    if unsafe { query(handle.0, 60, buffer.as_mut_ptr().cast(), needed, &mut needed) } < 0 { return None; }
    // SAFETY: the allocated buffer fits the header; validate the returned range before reading.
    let value = unsafe { &*buffer.as_ptr().cast::<UnicodeString>() };
    let start = buffer.as_ptr() as usize;
    let end = start.checked_add(buffer.len() * size_of::<usize>())?;
    let text = value.buffer as usize;
    if value.length == 0 || value.length % 2 != 0 || text % 2 != 0 || text < start
        || text.checked_add(value.length as usize)? > end { return None; }
    let mut command = unsafe { std::slice::from_raw_parts(value.buffer, value.length as usize / 2) }.to_vec();
    command.push(0);
    split_arguments(&command)
}

#[repr(C)]
struct UnicodeString { length: u16, maximum_length: u16, buffer: *const u16 }

fn split_arguments(command: &[u16]) -> Option<Vec<String>> {
    if command.last() != Some(&0) { return None; }
    let mut count = 0;
    // SAFETY: terminated input. Returned allocation owns both pointer array and strings.
    let argv = unsafe { CommandLineToArgvW(command.as_ptr(), &mut count) };
    if argv.is_null() { return None; }
    let mut out = Vec::new();
    for i in 0..count as usize {
        unsafe {
            let arg = *argv.add(i);
            let mut len = 0;
            while *arg.add(len) != 0 { len += 1; }
            out.push(String::from_utf16_lossy(std::slice::from_raw_parts(arg, len)));
        }
    }
    unsafe { LocalFree(argv.cast()); }
    (!out.is_empty()).then_some(out)
}

struct Snapshot(Handle);

impl Drop for Snapshot {
    fn drop(&mut self) {
        // SAFETY: owns one valid, non-inherited kernel handle.
        unsafe { CloseHandle(self.0); }
    }
}

pub(crate) fn snapshot() -> io::Result<Vec<Process>> {
    // SAFETY: only requests the process list; no pointers, process access or mutation.
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let snapshot = Snapshot(handle);
    // SAFETY: all members are integers/arrays. Windows requires dwSize before enumeration.
    let mut entry: ProcessEntry = unsafe { std::mem::zeroed() };
    entry.size = size_of::<ProcessEntry>() as u32;
    let mut processes = Vec::new();
    // SAFETY: snapshot stays alive, and entry has the documented C layout and buffer size.
    let mut found = unsafe { Process32FirstW(snapshot.0, &mut entry) };
    loop {
        if found == 0 {
            let error = io::Error::last_os_error();
            return if error.raw_os_error() == Some(ERROR_NO_MORE_FILES) {
                Ok(processes)
            } else {
                Err(error)
            };
        }
        if let Some(mut process) = process(entry.pid, &entry.exe) {
            if crate::matchers::needs_arguments(&process.argv[0]) {
                if let Some(argv) = arguments(entry.pid) { process.argv = argv; }
            }
            processes.push(process);
        }
        // SAFETY: same owned snapshot and initialized entry as the first call.
        found = unsafe { Process32NextW(snapshot.0, &mut entry) };
    }
}

fn process(pid: u32, name: &[u16]) -> Option<Process> {
    let pid = i32::try_from(pid).ok().filter(|pid| *pid > 0)?;
    let end = name.iter().position(|ch| *ch == 0).unwrap_or(name.len());
    if end == 0 { return None; }
    Some(Process {
        pid,
        argv: vec![String::from_utf16_lossy(&name[..end])],
        cwd: None, // Tool Help provides neither cwd nor command-line arguments.
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_snapshot_contains_current_process() {
        let processes = snapshot().expect("Windows process snapshot");
        let current = processes.iter().find(|p| p.pid as u32 == std::process::id())
            .expect("first snapshot must include this test process without a warm-up");
        let exe = std::env::current_exe().unwrap();
        assert_eq!(current.argv, [exe.file_name().unwrap().to_string_lossy()]);
        assert!(current.cwd.is_none());
        assert!(super::super::list().iter().any(|p| p.pid == current.pid));
    }

    #[test]
    fn command_line_query_matches_actual_current_arguments() {
        assert_eq!(arguments(std::process::id()).expect("own process query"), std::env::args().collect::<Vec<_>>());
        assert!(arguments(u32::MAX).is_none());
        let raw: Vec<u16> = r#""C:\Program Files\node.exe" "C:\한글 폴더\@google\gemini-cli\index.js" "two words""#.encode_utf16().chain([0]).collect();
        assert_eq!(split_arguments(&raw).unwrap(), [r"C:\Program Files\node.exe", r"C:\한글 폴더\@google\gemini-cli\index.js", "two words"]);
    }

    #[test]
    fn names_are_utf16_and_unknown_fields_stay_unknown() {
        let name: Vec<u16> = "한글, 🦀.exe\0ignored".encode_utf16().collect();
        let p = process(123, &name).unwrap();
        assert_eq!(p.argv, ["한글, 🦀.exe"]);
        assert!(p.cwd.is_none());
        assert!(process(0, &name).is_none());
        assert!(process(u32::MAX, &name).is_none());
        assert!(process(1, &[0]).is_none());
    }
}
