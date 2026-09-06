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
}

struct Snapshot(Handle);

impl Drop for Snapshot {
    fn drop(&mut self) {
        // SAFETY: owns one valid, non-inherited snapshot handle.
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
        if let Some(process) = process(entry.pid, &entry.exe) {
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
