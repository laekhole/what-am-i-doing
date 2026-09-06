//! 프로세스 열거.
//!
//! 읽기 전용이다(매니페스트 §1.1). 시그널을 보내지 않고, 프로세스를 만들지
//! 않는다. `/proc`, `ps`, Windows의 Tool Help API로 관찰한다.

#[cfg(not(windows))]
use std::fs;
use std::path::PathBuf;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(crate) use windows::snapshot as windows_snapshot;

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: i32,
    /// argv를 NUL 기준으로 분리한 것. 셸 인용부호 재파싱을 피할 수 있어서
    /// `/proc` 경로가 `ps`보다 정확하다.
    pub argv: Vec<String>,
    pub cwd: Option<PathBuf>,
}

impl Process {
    pub fn cmdline(&self) -> String {
        self.argv.join(" ")
    }
}

pub fn list() -> Vec<Process> {
    #[cfg(windows)]
    { windows_snapshot().unwrap_or_default() }
    #[cfg(not(windows))]
    if PathBuf::from("/proc/self/cmdline").exists() {
        from_procfs()
    } else {
        from_ps()
    }
}

#[cfg(not(windows))]
fn from_procfs() -> Vec<Process> {
    let mut out = Vec::new();
    let entries = match fs::read_dir("/proc") {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let pid: i32 = match name.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        // 커널이 언제든 프로세스를 치울 수 있으므로 실패는 전부 무시한다.
        let raw = match fs::read(format!("/proc/{pid}/cmdline")) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if raw.is_empty() {
            continue; // 커널 스레드
        }
        let argv: Vec<String> = raw
            .split(|b| *b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .collect();
        if argv.is_empty() {
            continue;
        }
        let cwd = fs::read_link(format!("/proc/{pid}/cwd")).ok();
        out.push(Process { pid, argv, cwd });
    }
    out
}

/// macOS 폴백. `ps`는 cwd를 주지 않는다.
///
/// cwd 없이는 트랜스크립트를 프로세스에 정확히 짝지을 수 없으므로, macOS에서는
/// 세션 목록이 디스크의 트랜스크립트 쪽에서 주도되고 프로세스는 생존 여부
/// 확인에만 쓰인다. `lsof`로 cwd를 캐낼 수는 있지만 프로세스당 수십 ms가 들고
/// 성능 예산(§8)을 깬다.
#[cfg(not(windows))]
fn from_ps() -> Vec<Process> {
    let out = match std::process::Command::new("ps").args(["-eo", "pid=,args="]).output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let (pid, rest) = line.split_once(char::is_whitespace)?;
            let pid: i32 = pid.parse().ok()?;
            let argv: Vec<String> = rest.split_whitespace().map(str::to_string).collect();
            if argv.is_empty() {
                return None;
            }
            Some(Process { pid, argv, cwd: None })
        })
        .collect()
}
