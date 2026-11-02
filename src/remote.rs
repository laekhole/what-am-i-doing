//! Opt-in SSH collection: run the same read-only collector on servers the user lists.
//!
//! waid opens no connection unless the user adds a server. Each connection uses the
//! system `ssh` client with the user's own `~/.ssh` configuration, keys and agent, and
//! runs only the fixed commands below. The server sends JSON snapshots back over that
//! connection; nothing goes anywhere else.
use crate::i18n::tr;
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Child, ChildStderr, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

/// Servers one app watches at most. Each one is a long-lived SSH connection.
pub const MAX_SERVERS: usize = 16;
/// Remote snapshots skip `--history` (one server measured 8.5 MB and 11 s with it,
/// 130 KB and 0.3 s without) and refresh less often than the local collector.
pub const INTERVAL_SECS: u64 = 3;
/// Where the app installs the server binary. Also the first place the stream looks.
pub const INSTALL_PATH: &str = "~/.local/bin/waid";
/// The only server platform the bundled binary supports today.
pub const SUPPORTED_PLATFORM: &str = "Linux x86_64";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_SNAPSHOT: u64 = 16 * 1024 * 1024;
const MAX_DESTINATION: usize = 255;
const RETRY_SECS: [u64; 5] = [5, 10, 30, 60, 120];
/// Lines a login shell may print before the version line (for example from `.bashrc`).
const MAX_PREAMBLE: usize = 64;

// Both scripts are passed as one `sh -c '…'` argument so they run the same under any
// login shell. They must not contain single quotes, `!` (csh history) or newlines.
const STREAM_SCRIPT: &str = r#"w="$HOME/.local/bin/waid"; [ -x "$w" ] || w="$(command -v waid)" || w=""; [ -n "$w" ] || { echo waid-missing >&2; exit 127; }; v="$("$w" --version)" || exit 126; echo "$v"; exec "$w" --json --watch --interval INTERVAL --lang LANG"#;
const INSTALL_SCRIPT: &str = r#"set -e; a="$(uname -s)/$(uname -m)"; case "$a" in Linux/x86_64|Linux/amd64) ;; *) echo "waid-unsupported $a" >&2; exit 3;; esac; d="$HOME/.local/bin"; mkdir -p "$d"; t="$d/.waid-install.$$"; trap "rm -f $t" EXIT; cat > "$t"; chmod 755 "$t"; "$t" --version > /dev/null; mv -f "$t" "$d/waid"; "$d/waid" --version"#;

/// One configured server. Disabled servers stay in the list without connecting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Server {
    pub destination: String,
    pub enabled: bool,
}

/// Checks a destination before it reaches `ssh` as an argument.
///
/// Accepts a `~/.ssh/config` host alias or `user@host`. A leading `-` would be read as
/// an `ssh` option, and spaces or shell characters never belong in a destination.
pub fn validate(destination: &str) -> Result<(), String> {
    if destination.is_empty() {
        return Err(tr("서버 이름이 비어 있습니다.", "The server name is empty.").into());
    }
    if destination.len() > MAX_DESTINATION {
        return Err(crate::trf!(
            "서버 이름은 {MAX_DESTINATION}자 이하여야 합니다: {destination}",
            "Server names must be at most {MAX_DESTINATION} characters: {destination}"
        ));
    }
    if destination.starts_with('-') {
        return Err(crate::trf!(
            "서버 이름은 '-'로 시작할 수 없습니다: {destination}",
            "Server names cannot start with '-': {destination}"
        ));
    }
    if let Some(bad) = destination
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || "._-@:[]%+".contains(*c)))
    {
        return Err(crate::trf!(
            "서버 이름에 쓸 수 없는 문자 '{bad}': {destination} (SSH 설정의 Host 이름이나 user@host를 적으세요)",
            "Invalid character '{bad}' in server name: {destination} (use an SSH config Host name or user@host)"
        ));
    }
    Ok(())
}

/// Parses the editor text: one server per line, `#` turns a server off, blank lines
/// are ignored and repeated servers are merged.
pub fn parse_servers(text: &str) -> Result<Vec<Server>, String> {
    let mut servers: Vec<Server> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (enabled, destination) = match line.strip_prefix('#') {
            Some(rest) => (false, rest.trim()),
            None => (true, line),
        };
        if !enabled && destination.is_empty() {
            continue;
        }
        validate(destination)?;
        if let Some(existing) = servers.iter_mut().find(|s| s.destination == destination) {
            existing.enabled |= enabled;
            continue;
        }
        servers.push(Server { destination: destination.into(), enabled });
    }
    if servers.len() > MAX_SERVERS {
        return Err(crate::trf!(
            "서버는 최대 {MAX_SERVERS}개까지 등록할 수 있습니다.",
            "You can add up to {MAX_SERVERS} servers."
        ));
    }
    Ok(servers)
}

pub fn format_servers(servers: &[Server]) -> String {
    servers
        .iter()
        .map(|s| if s.enabled { s.destination.clone() } else { format!("# {}", s.destination) })
        .collect::<Vec<_>>()
        .join("\r\n")
}

/// Why a connection or install did not work, in words the user can act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    SshMissing,
    Auth,
    HostKey,
    Unreachable(String),
    Missing,
    Broken(String),
    Unsupported(String),
    NotBundled,
    Stopped(String),
}

impl Problem {
    pub fn message(&self, destination: &str) -> String {
        match self {
            Problem::SshMissing => tr(
                "SSH 클라이언트를 찾지 못했습니다. Windows 설정 > 시스템 > 선택적 기능에서 OpenSSH 클라이언트를 설치하세요.",
                "Could not find the SSH client. Install OpenSSH Client in Windows Settings > System > Optional features.",
            )
            .into(),
            Problem::Auth => crate::trf!(
                "키 인증으로 접속하지 못했습니다. 비밀번호 입력 없이 `ssh {destination}`가 되도록 SSH 키나 ssh-agent를 설정하세요.",
                "Key authentication failed. Set up an SSH key or ssh-agent so `ssh {destination}` works without a password prompt."
            ),
            Problem::HostKey => crate::trf!(
                "서버의 호스트 키를 확인하지 못했습니다. 터미널에서 `ssh {destination}`로 한 번 접속해 키를 확인하세요.",
                "Could not verify the server's host key. Connect once with `ssh {destination}` in a terminal and confirm the key."
            ),
            Problem::Unreachable(detail) => crate::trf!("서버에 연결하지 못했습니다: {detail}", "Could not reach the server: {detail}"),
            Problem::Missing => tr(
                "서버에 waid가 없습니다. 원격 서버 설정에서 '서버에 설치'를 누르세요.",
                "waid is not installed on the server. Choose 'Install on servers' in the remote server settings.",
            )
            .into(),
            Problem::Broken(detail) => crate::trf!(
                "서버의 waid를 실행하지 못했습니다: {detail} — 다시 설치하세요.",
                "Could not run waid on the server: {detail} — install it again."
            ),
            Problem::Unsupported(platform) => crate::trf!(
                "이 서버({platform})용 waid는 아직 없습니다. 지금은 {SUPPORTED_PLATFORM}만 지원합니다.",
                "waid is not available for this server ({platform}) yet. Only {SUPPORTED_PLATFORM} is supported."
            ),
            Problem::NotBundled => tr(
                "이 빌드에는 서버용 waid가 들어 있지 않습니다. 정식 배포판을 사용하세요.",
                "This build does not include the server binary. Use a released build.",
            )
            .into(),
            Problem::Stopped(detail) => crate::trf!("연결이 끊겼습니다: {detail}", "The connection ended: {detail}"),
        }
    }

    pub fn needs_install(&self) -> bool {
        matches!(self, Problem::Missing | Problem::Broken(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Connecting,
    /// The server's waid version. It may differ from this app's [`VERSION`].
    Connected { version: String },
    Retrying { problem: Problem, retry_in: u64 },
}

#[derive(Debug)]
pub enum Event {
    Status(Status),
    /// One raw JSON snapshot line from the server's collector.
    Snapshot(Vec<u8>),
}

/// The system `ssh`. On Windows the built-in OpenSSH client sits outside some PATHs.
pub fn ssh_program() -> PathBuf {
    #[cfg(windows)]
    if let Some(root) = std::env::var_os("SystemRoot") {
        let builtin = PathBuf::from(root).join(r"System32\OpenSSH\ssh.exe");
        if builtin.is_file() {
            return builtin;
        }
    }
    PathBuf::from("ssh")
}

fn ssh(destination: &str) -> Command {
    let mut command = Command::new(ssh_program());
    command.args([
        "-T",
        "-o", "BatchMode=yes",
        "-o", "ConnectTimeout=10",
        "-o", "ServerAliveInterval=15",
        "-o", "ServerAliveCountMax=3",
        "-o", "ClearAllForwardings=yes",
        "-o", "LogLevel=ERROR",
        "--",
        destination,
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    command
}

pub fn stream_command(language: &str) -> String {
    let language = if language == "en" { "en" } else { "ko" };
    let script = STREAM_SCRIPT
        .replace("INTERVAL", &INTERVAL_SECS.to_string())
        .replace("LANG", language);
    format!("sh -c '{script}'")
}

pub fn install_command() -> String {
    format!("sh -c '{INSTALL_SCRIPT}'")
}

/// Turns an `ssh` exit and its last error lines into a [`Problem`].
pub fn classify(code: Option<i32>, stderr: &[String]) -> Problem {
    let joined = stderr.join("\n");
    let lower = joined.to_ascii_lowercase();
    let last = stderr
        .iter()
        .rev()
        .map(|line| line.trim())
        .find(|line| !line.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| match code {
            Some(code) => format!("exit {code}"),
            None => tr("종료됨", "terminated").into(),
        });
    if let Some(at) = lower.find("waid-unsupported") {
        let platform = joined[at + "waid-unsupported".len()..].lines().next().unwrap_or("").trim();
        return Problem::Unsupported(if platform.is_empty() { "?".into() } else { platform.into() });
    }
    if lower.contains("waid-missing") {
        return Problem::Missing;
    }
    if code == Some(126) {
        return Problem::Broken(last);
    }
    if code == Some(255) {
        if lower.contains("host key verification failed")
            || lower.contains("remote host identification has changed")
            || lower.contains("no matching host key")
        {
            return Problem::HostKey;
        }
        if lower.contains("permission denied")
            || lower.contains("too many authentication failures")
            || lower.contains("no more authentication methods")
        {
            return Problem::Auth;
        }
        return Problem::Unreachable(last);
    }
    Problem::Stopped(last)
}

enum Frame {
    Line(Vec<u8>),
    Oversized,
}

fn read_frame(reader: &mut impl BufRead, limit: u64) -> io::Result<Option<Frame>> {
    let mut line = Vec::new();
    if (&mut *reader).take(limit + 1).read_until(b'\n', &mut line)? == 0 {
        return Ok(None);
    }
    if line.len() as u64 > limit {
        if line.last() != Some(&b'\n') {
            loop {
                let buffer = reader.fill_buf()?;
                if buffer.is_empty() {
                    break;
                }
                let newline = buffer.iter().position(|byte| *byte == b'\n');
                let used = newline.map_or(buffer.len(), |at| at + 1);
                reader.consume(used);
                if newline.is_some() {
                    break;
                }
            }
        }
        return Ok(Some(Frame::Oversized));
    }
    while matches!(line.last(), Some(b'\n' | b'\r')) {
        line.pop();
    }
    Ok(Some(Frame::Line(line)))
}

fn version_line(line: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(line).ok()?.trim();
    let version = text.strip_prefix("waid ")?;
    (!version.is_empty()
        && version.len() <= 40
        && version.chars().all(|c| c.is_ascii_alphanumeric() || ".-+".contains(c)))
    .then(|| version.to_string())
}

/// Keeps the last few error lines. The OpenSSH post-quantum notice is not an error.
fn stderr_tail(stderr: ChildStderr) -> Vec<String> {
    let mut tail = Vec::new();
    for line in BufReader::new(stderr.take(1024 * 1024)).split(b'\n').map_while(Result::ok) {
        let line = String::from_utf8_lossy(&line).trim().chars().take(300).collect::<String>();
        if line.is_empty() || line.starts_with("** ") {
            continue;
        }
        tail.push(line);
        if tail.len() > 8 {
            tail.remove(0);
        }
    }
    tail
}

struct Shared {
    stop: AtomicBool,
    wake: AtomicBool,
    child: Mutex<Option<Child>>,
}

impl Shared {
    fn kill(&self) {
        if let Some(child) = self.child.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            let _ = child.kill();
        }
    }
}

/// A server connection that reconnects until dropped. Dropping it ends the `ssh`
/// process; the server's collector exits when its output closes.
pub struct Link {
    shared: Arc<Shared>,
}

impl Link {
    pub fn start(
        destination: &str,
        language: &str,
        on_event: impl FnMut(Event) + Send + 'static,
    ) -> Link {
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            wake: AtomicBool::new(false),
            child: Mutex::new(None),
        });
        let worker = Arc::clone(&shared);
        let destination = destination.to_string();
        let command = stream_command(language);
        thread::spawn(move || {
            let mut on_event = on_event;
            supervise(&worker, &destination, &command, &mut on_event);
        });
        Link { shared }
    }

    /// Reconnects now instead of waiting for the next retry (for example after install).
    pub fn reconnect(&self) {
        self.shared.wake.store(true, Ordering::SeqCst);
        self.shared.kill();
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        self.shared.kill();
    }
}

fn supervise(shared: &Shared, destination: &str, command: &str, emit: &mut dyn FnMut(Event)) {
    let mut failures = 0usize;
    while !shared.stop.load(Ordering::SeqCst) {
        emit(Event::Status(Status::Connecting));
        let started = Instant::now();
        let (problem, connected) = attempt(shared, destination, command, emit);
        if shared.stop.load(Ordering::SeqCst) {
            return;
        }
        // A connection that worked for a while starts the backoff over.
        if connected && started.elapsed() >= Duration::from_secs(60) {
            failures = 0;
        }
        let retry_in = RETRY_SECS[failures.min(RETRY_SECS.len() - 1)];
        failures += 1;
        emit(Event::Status(Status::Retrying { problem, retry_in }));
        let until = Instant::now() + Duration::from_secs(retry_in);
        while Instant::now() < until {
            if shared.stop.load(Ordering::SeqCst) {
                return;
            }
            if shared.wake.swap(false, Ordering::SeqCst) {
                failures = 0;
                break;
            }
            thread::sleep(Duration::from_millis(200));
        }
    }
}

fn attempt(shared: &Shared, destination: &str, command: &str, emit: &mut dyn FnMut(Event)) -> (Problem, bool) {
    shared.wake.store(false, Ordering::SeqCst);
    let mut child = match ssh(destination)
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return (Problem::SshMissing, false),
        Err(error) => return (Problem::Stopped(error.to_string()), false),
    };
    let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
        let _ = child.kill();
        let _ = child.wait();
        return (Problem::Stopped("missing ssh pipes".into()), false);
    };
    *shared.child.lock().unwrap_or_else(|e| e.into_inner()) = Some(child);
    if shared.stop.load(Ordering::SeqCst) {
        shared.kill();
    }
    let errors = thread::spawn(move || stderr_tail(stderr));
    let mut reader = BufReader::new(stdout);
    let mut version = None;
    let mut preamble = 0;
    let mut connected = false;
    loop {
        match read_frame(&mut reader, MAX_SNAPSHOT) {
            Ok(Some(Frame::Line(line))) if version.is_none() => {
                if let Some(found) = version_line(&line) {
                    version = Some(found.clone());
                    if !shared.stop.load(Ordering::SeqCst) {
                        emit(Event::Status(Status::Connected { version: found }));
                    }
                } else {
                    preamble += 1;
                    if preamble > MAX_PREAMBLE {
                        shared.kill();
                    }
                }
            }
            Ok(Some(Frame::Line(line))) => {
                connected = true;
                if !shared.stop.load(Ordering::SeqCst) {
                    emit(Event::Snapshot(line));
                }
            }
            Ok(Some(Frame::Oversized)) => {}
            Ok(None) | Err(_) => break,
        }
    }
    let code = shared
        .child
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
        .and_then(|mut child| child.wait().ok())
        .and_then(|status| status.code());
    let lines = errors.join().unwrap_or_default();
    (classify(code, &lines), connected)
}

/// Copies `binary` to [`INSTALL_PATH`] on the server over one SSH connection and
/// returns the installed version. The server needs no internet access.
pub fn install(destination: &str, binary: &[u8]) -> Result<String, Problem> {
    if binary.is_empty() {
        return Err(Problem::NotBundled);
    }
    validate(destination).map_err(Problem::Stopped)?;
    let mut child = match ssh(destination)
        .arg(install_command())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Err(Problem::SshMissing),
        Err(error) => return Err(Problem::Stopped(error.to_string())),
    };
    let stdin = child.stdin.take();
    let data = binary.to_vec();
    let writer = thread::spawn(move || {
        if let Some(mut stdin) = stdin {
            let _ = stdin.write_all(&data);
        }
    });
    let output = child.wait_with_output().map_err(|e| Problem::Stopped(e.to_string()))?;
    let _ = writer.join();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.lines().rev().find_map(|line| version_line(line.as_bytes()));
    if output.status.success() {
        return version.ok_or_else(|| Problem::Broken(tr("설치 후 버전을 확인하지 못했습니다", "could not confirm the installed version").into()));
    }
    let stderr: Vec<String> = String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(|line| line.trim().chars().take(300).collect::<String>())
        .filter(|line| !line.is_empty() && !line.starts_with("** "))
        .collect();
    let tail = &stderr[stderr.len().saturating_sub(8)..];
    Err(match classify(output.status.code(), tail) {
        // A failed install step is not "missing"; show what the server said.
        Problem::Stopped(detail) => Problem::Broken(detail),
        problem => problem,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destinations_reject_options_and_shell_characters() {
        for ok in ["dbes-server", "appadm@172.21.100.14", "user@[fe80::1%eth0]", "a.b_c+d"] {
            assert!(validate(ok).is_ok(), "{ok}");
        }
        for bad in ["", "-oProxyCommand=calc", "a b", "host;rm", "host`x`", "h$(x)", "호스트", &"a".repeat(256)] {
            assert!(validate(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn server_text_keeps_order_disables_with_hash_and_merges_repeats() {
        let servers = parse_servers("dbes-server\r\n\r\n# staging\nuser@10.0.0.2\n#\ndbes-server\n# user@10.0.0.2\n").unwrap();
        assert_eq!(
            servers,
            vec![
                Server { destination: "dbes-server".into(), enabled: true },
                Server { destination: "staging".into(), enabled: false },
                Server { destination: "user@10.0.0.2".into(), enabled: true },
            ]
        );
        assert_eq!(format_servers(&servers), "dbes-server\r\n# staging\r\nuser@10.0.0.2");
        assert!(parse_servers("ok\nbad host").is_err());
        let many: String = (0..=MAX_SERVERS).map(|i| format!("h{i}\n")).collect();
        assert!(parse_servers(&many).is_err());
    }

    #[test]
    fn scripts_survive_any_login_shell() {
        for command in [stream_command("ko"), stream_command("en"), install_command()] {
            let body = command.strip_prefix("sh -c '").and_then(|c| c.strip_suffix('\'')).unwrap();
            assert!(!body.contains('\'') && !body.contains('!') && !body.contains('\n'), "{command}");
        }
        assert!(stream_command("en").contains("--lang en") && stream_command("x").contains("--lang ko"));
        assert!(stream_command("ko").contains("--interval 3") && !stream_command("ko").contains("--history"));
    }

    #[test]
    fn exits_map_to_actionable_problems() {
        let lines = |text: &str| text.lines().map(str::to_string).collect::<Vec<_>>();
        assert_eq!(classify(Some(255), &lines("appadm@host: Permission denied (publickey,password).")), Problem::Auth);
        assert_eq!(classify(Some(255), &lines("Host key verification failed.")), Problem::HostKey);
        assert_eq!(
            classify(Some(255), &lines("ssh: connect to host 10.0.0.9 port 22: Connection timed out")),
            Problem::Unreachable("ssh: connect to host 10.0.0.9 port 22: Connection timed out".into())
        );
        assert_eq!(classify(Some(127), &lines("waid-missing")), Problem::Missing);
        assert_eq!(classify(Some(3), &lines("waid-unsupported Linux/aarch64")), Problem::Unsupported("Linux/aarch64".into()));
        assert_eq!(classify(Some(126), &lines("sh: waid: cannot execute binary file")), Problem::Broken("sh: waid: cannot execute binary file".into()));
        assert_eq!(classify(Some(1), &[]), Problem::Stopped("exit 1".into()));
        assert!(Problem::Missing.needs_install() && Problem::Broken(String::new()).needs_install());
        assert!(!Problem::Auth.needs_install());
    }

    #[test]
    fn frames_trim_newlines_and_skip_oversized_lines() {
        let mut input = io::Cursor::new(b"waid 0.2.1\r\n0123456789abc\n{\"a\":1}\n".to_vec());
        let mut lines = Vec::new();
        while let Some(frame) = read_frame(&mut input, 12).unwrap() {
            lines.push(match frame {
                Frame::Line(line) => String::from_utf8(line).unwrap(),
                Frame::Oversized => "<oversized>".into(),
            });
        }
        assert_eq!(lines, ["waid 0.2.1", "<oversized>", "{\"a\":1}"]);
        assert_eq!(version_line(b"waid 0.2.1"), Some("0.2.1".into()));
        assert_eq!(version_line(b"Welcome to the server"), None);
        assert_eq!(version_line(b"waid 0.2.1; rm -rf"), None);
    }

    /// `WAID_TEST_SSH=<host> cargo test --lib remote_live -- --ignored --nocapture`
    #[test]
    #[ignore = "needs a reachable SSH server with key authentication"]
    fn remote_live_stream() {
        let destination = std::env::var("WAID_TEST_SSH").expect("WAID_TEST_SSH");
        let (tx, rx) = std::sync::mpsc::channel();
        let link = Link::start(&destination, "ko", move |event| {
            let _ = tx.send(event);
        });
        let deadline = Instant::now() + Duration::from_secs(60);
        while let Ok(event) = rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            match event {
                Event::Status(status) => {
                    println!("status: {status:?}");
                    if let Status::Retrying { problem, .. } = status {
                        panic!("{}", problem.message(&destination));
                    }
                }
                Event::Snapshot(line) => {
                    let text = String::from_utf8_lossy(&line);
                    println!("snapshot: {} bytes, starts {}", line.len(), &text[..text.len().min(40)]);
                    assert!(text.starts_with('{') && text.contains("\"sessions\""));
                    drop(link);
                    return;
                }
            }
        }
        panic!("no snapshot within 60 s");
    }

    /// `WAID_TEST_SSH=<host> WAID_TEST_INSTALL=<linux binary> cargo test --lib remote_live_install -- --ignored`
    #[test]
    #[ignore = "installs ~/.local/bin/waid on the test server"]
    fn remote_live_install() {
        let destination = std::env::var("WAID_TEST_SSH").expect("WAID_TEST_SSH");
        let binary = std::fs::read(std::env::var("WAID_TEST_INSTALL").expect("WAID_TEST_INSTALL")).unwrap();
        let version = install(&destination, &binary).unwrap_or_else(|p| panic!("{}", p.message(&destination)));
        println!("installed waid {version}");
        assert_eq!(version, VERSION);
    }
}
