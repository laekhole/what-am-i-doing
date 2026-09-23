#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_diag;

mod adapters;
mod diag;
mod html;
pub mod i18n;
mod json;
mod matchers;
mod orca;
pub use orca::path as orca_hook_path;
mod proc;
mod render;
mod serve;
mod session;
mod sqlite;
mod theme;
mod tmpl;
mod time;
mod transcript;
#[cfg(windows)]
mod terminal;

pub use transcript::ContextUsage;

use render::Style;
use i18n::{tr, set_language, Language};

const USAGE_KO: &str = concat!("waid ", env!("CARGO_PKG_VERSION"), " — what am I doing?

사용법:
  waid [옵션]
  waid doctor

출력 (§6 커스터마이징 레이어):
  -1, --once            표 한 번 출력하고 종료 (기본값)      레이어 1: theme.toml
      --html            HTML 대시보드                        레이어 2: 템플릿
  -j, --json            JSON 스냅샷                          레이어 3: 파이프

옵션:
      --lang ko|en      표시 언어 (기본: 시스템 언어)
      --watch           지속 갱신 (--html 이면 로컬 서버)
      --history         날짜 제한 없이 기존 세션도 수집
      --template FILE   HTML 템플릿 (기본: 내장)
      --eject           기본 템플릿을 stdout 으로 꺼낸다
      --keys            템플릿에서 쓸 수 있는 키 목록
      --port N          --html --watch 의 포트 (기본 7423, 0은 자동 배정)
  -i, --interval SEC    폴링 주기 (기본 2)
      --agent NAME      특정 에이전트만
      --waiting         내 입력을 기다리는 세션만
      --color / --no-color
  -h, --help            이 도움말
  -V, --version

관찰만 합니다. 에이전트를 실행하거나 종료하거나 명령을 보내지 않습니다.
");

const USAGE_EN: &str = concat!("waid ", env!("CARGO_PKG_VERSION"), " — what am I doing?

Usage:
  waid [options]
  waid doctor

Output:
  -1, --once            Print a table once and exit (default)
      --html            HTML dashboard
  -j, --json            JSON snapshot

Options:
      --lang ko|en      Display language (default: system language)
      --watch           Keep updating (--html starts a localhost server)
      --history         Include sessions without a date limit
      --template FILE   HTML template (default: bundled)
      --eject           Write the bundled HTML template to stdout
      --keys            List available template keys
      --port N          --html --watch port (default 7423, 0 assigns a free port)
  -i, --interval SEC    Polling interval (default 2)
      --agent NAME      Show only this agent
      --waiting         Show only sessions waiting for your input
      --color / --no-color
  -h, --help            Show this help
  -V, --version

Read-only: waid never starts, stops, or sends commands to agents.
");

struct Args {
    action: Option<&'static str>,
    json: bool,
    html: bool,
    template: Option<String>,
    port: u16,
    watch: bool,
    history: bool,
    interval: u64,
    agent: Option<String>,
    waiting_only: bool,
    color: Option<bool>,
    doctor: bool,
}

fn parse_args(mut it: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut a = Args {
        action: None,
        json: false,
        html: false,
        template: None,
        port: 7423,
        watch: false,
        history: false,
        interval: 2,
        agent: None,
        waiting_only: false,
        color: None,
        doctor: false,
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => a.action = Some("help"),
            "-V" | "--version" => a.action = Some("version"),
            "doctor" => a.doctor = true,
            "-1" | "--once" => {}
            "-j" | "--json" => a.json = true,
            "--html" => a.html = true,
            "--eject" => a.action = Some("eject"),
            "--keys" => a.action = Some("keys"),
            "--lang" => {
                let v = it.next().ok_or(tr("--lang 에 ko 또는 en이 필요합니다", "--lang requires ko or en"))?;
                let language = Language::from_code(&v)
                    .filter(|_| matches!(v.as_str(), "ko" | "en"))
                    .ok_or(tr("--lang 은 ko 또는 en이어야 합니다", "--lang must be ko or en"))?;
                set_language(language);
            }
            "--template" => {
                a.template = Some(it.next().ok_or(tr("--template 에 파일 경로가 필요합니다", "--template requires a file path"))?);
                a.html = true;
            }
            "--port" => {
                let v = it.next().ok_or(tr("--port 에 값이 필요합니다", "--port requires a value"))?;
                a.port = v.parse::<u16>().map_err(|_| tr("--port 는 0..65535 여야 합니다", "--port must be between 0 and 65535"))?;
            }
            "--watch" => a.watch = true,
            "--history" => a.history = true,
            "--waiting" => a.waiting_only = true,
            "--color" => a.color = Some(true),
            "--no-color" => a.color = Some(false),
            "-i" | "--interval" => {
                let v = it.next().ok_or(tr("--interval 에 값이 필요합니다", "--interval requires a value"))?;
                a.interval = v.parse::<u64>().map_err(|_| tr("--interval 은 숫자여야 합니다", "--interval must be a positive integer"))?.max(1);
            }
            "--agent" => {
                let v = it.next().ok_or(tr("--agent 에 값이 필요합니다", "--agent requires a value"))?;
                if matchers::by_name(&v).is_none() {
                    let known: Vec<&str> = matchers::all().iter().map(|a| a.name).collect();
                    return Err(crate::trf!("알 수 없는 에이전트 '{v}'. 가능: {}", "Unknown agent '{v}'. Available: {}", known.join(", ")));
                }
                a.agent = Some(v);
            }
            other => return Err(crate::trf!("알 수 없는 옵션 '{other}' — `waid --help`", "Unknown option '{other}' — `waid --help`")),
        }
    }
    Ok(a)
}

fn snapshot(args: &Args) -> (Vec<session::Session>, i64) {
    let now = time::now();
    let mut s = session::collect_with_history(now, args.history);
    if let Some(name) = &args.agent {
        s.retain(|x| x.agent.name == name);
    }
    if args.waiting_only {
        s.retain(|x| x.state == session::State::Waiting);
    }
    (s, now)
}

/// Run the CLI, also used by the desktop executable's internal collector mode.
pub fn run(args: impl Iterator<Item = String>) {
    set_language(Language::detect());
    let args = match parse_args(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("waid: {e}");
            std::process::exit(2);
        }
    };

    if let Some(action) = args.action {
        match action {
            "help" => print!("{}", tr(USAGE_KO, USAGE_EN)),
            "version" => println!("waid {}", env!("CARGO_PKG_VERSION")),
            "eject" => print!("{}", html::DEFAULT_TEMPLATE),
            "keys" => print_keys(),
            _ => unreachable!(),
        }
        return;
    }

    if args.doctor {
        doctor();
        return;
    }

    if args.json && args.html {
        eprintln!("{}", tr("waid: --json 과 --html 은 함께 쓸 수 없습니다", "waid: --json and --html cannot be used together"));
        std::process::exit(2);
    }

    let theme = theme::load();
    let style = Style::detect(args.color);

    // --html --watch 는 로컬 서버가 된다. 나머지 조합은 전부 stdout.
    if args.html && args.watch {
        let tpl = match html::template(args.template.as_deref()) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("waid: {e}");
                std::process::exit(1);
            }
        };
        let port = args.port;
        let interval = args.interval;
        let snap = std::sync::Arc::new(move || snapshot(&args));
        if let Err(e) = serve::serve(port, tpl, interval, snap) {
            eprintln!("waid: {e}");
            std::process::exit(1);
        }
        return;
    }

    if !args.watch {
        let (sessions, now) = snapshot(&args);
        if args.json {
            print!("{}", session::to_json(&sessions, now, true));
        } else if args.html {
            match html::template(args.template.as_deref()) {
                Ok(tpl) => print!("{}", html::render(&sessions, now, &tpl)),
                Err(e) => {
                    eprintln!("waid: {e}");
                    std::process::exit(1);
                }
            }
        } else {
            print!("{}", render::table(&sessions, &theme, &style));
        }
        return;
    }

    loop {
        let (sessions, now) = snapshot(&args);
        if args.json {
            // 줄 단위 JSON. 커스터마이징 레이어 3의 입력이다(§6).
            print!("{}", session::to_json(&sessions, now, false));
        } else {
            // 화면 지우고 커서 원점으로. 대체 화면 버퍼는 쓰지 않는다 —
            // Ctrl-C 후에도 마지막 표가 스크롤백에 남는 편이 낫다.
            print!("\x1b[2J\x1b[H{}", render::table(&sessions, &theme, &style));
        }
        use std::io::Write;
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_secs(args.interval));
    }
}

/// 템플릿에서 쓸 수 있는 키. 문서를 따로 두면 코드와 어긋나므로
/// 실제 컨텍스트를 만들어서 그대로 나열한다.
fn print_keys() {
    let ctx = html::key_context();
    println!("{}", tr("전역 키\n", "Global keys\n"));
    for key in ctx.keys().filter(|key| *key != "sessions") {
        println!("  {{{{{key}}}}}");
    }
    println!("{}", tr("\n{{#each sessions}} 안에서 쓸 수 있는 키\n", "\nKeys inside {{#each sessions}}\n"));
    if let Some(tmpl::Val::List(items)) = ctx.get("sessions") {
        for key in items[0].keys() {
            println!("  {{{{{key}}}}}");
        }
    }
}

/// 감지 경로를 진단한다. "왜 내 세션이 안 보이지"에 답하기 위한 유일한 서브커맨드.
fn doctor() {
    println!("{}", crate::trf!("waid {} — 진단\n", "waid {} — diagnostics\n", env!("CARGO_PKG_VERSION")));

    let procfs = std::path::Path::new("/proc/self/cmdline").exists();
    println!(
        "{} {}", tr("프로세스 열거  ", "Processes     "),
        if cfg!(windows) { tr("Windows Tool Help API (첫 조회부터 수집, 지원 프로세스 인자 조회, cwd 없음)", "Windows Tool Help API (collected on first scan; arguments queried for supported processes, no cwd)") }
        else if procfs { tr("/proc (정확: cwd 확보 가능)", "/proc (cwd available)") } else { tr("ps 폴백 (cwd 없음)", "ps fallback (no cwd)") }
    );

    println!(
        "SQLite          {}",
        if sqlite::available() {
            tr("OS 라이브러리 사용 가능 (편집기 DB 읽기 O)", "OS library available (editor databases readable)")
        } else {
            tr("없음 — SQLite 소스는 꺼집니다", "Unavailable — SQLite sources disabled")
        }
    );

    let home = std::env::var("HOME").unwrap_or_else(|_| "(unset)".into());
    println!("HOME            {home}");
    if let Some(home) = adapters::home() {
        println!("{} {}", tr("사용 홈        ", "Resolved home  "), home.display());
    }
    match theme::path() {
        Some(p) => println!(
            "{} {} {}", tr("테마           ", "Theme          "),
            p.display(),
            if p.exists() { tr("(적용됨)", "(loaded)") } else { tr("(없음 — 기본값)", "(missing — defaults)") }
        ),
        None => println!("{}", tr("테마            (경로 확인 불가 — 기본값)", "Theme           (path unavailable — defaults)")),
    }

    match adapters::dir() {
        Some(d) => {
            let n = adapters::table()
                .iter()
                .filter(|x| !matches!(x.origin, adapters::Origin::Builtin))
                .count();
            println!("{}", crate::trf!(
                "어댑터          {} ({}개 로드됨)", "Adapters        {} ({} loaded)",
                d.display(),
                n
            ));
        }
        None => println!("{}", tr("어댑터          (경로 확인 불가)", "Adapters        (path unavailable)")),
    }

    // 어댑터 오타는 조용히 무시되지만, 여기서는 반드시 보여준다.
    // "왜 내 어댑터가 안 먹지"의 답이 여기 있어야 한다.
    let problems = adapters::problems();
    if !problems.is_empty() {
        println!("{}", tr("\n어댑터 문제", "\nAdapter problems"));
        for p in problems {
            println!("  ! {p}");
        }
    }

    println!("{}", tr("\n에이전트", "\nAgents"));
    #[cfg(windows)]
    let procs = match proc::windows_snapshot() {
        Ok(processes) => processes,
        Err(error) => {
            println!("{}", crate::trf!("  ! 프로세스 열거 실패: {error}", "  ! Process enumeration failed: {error}"));
            Vec::new()
        }
    };
    #[cfg(not(windows))]
    let procs = proc::list();
    for d in adapters::table() {
        let a = d.agent;
        let n = procs
            .iter()
            .filter(|p| matchers::identify(p).map(|x| x.name) == Some(a.name))
            .count();
        let origin = match &d.origin {
            adapters::Origin::Builtin => tr("내장", "bundled").to_string(),
            adapters::Origin::User(p) => format!(
                "← {}",
                p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default()
            ),
        };
        let mut kinds: Vec<&str> = Vec::new();
        if !d.transcript_dirs.is_empty() {
            kinds.push("jsonl");
        }
        if !d.json_dirs.is_empty() {
            kinds.push("json");
        }
        if !d.queries.is_empty() {
            kinds.push("sqlite");
        }
        let reader = if kinds.is_empty() {
            tr("읽기 X", "no reader").to_string()
        } else {
            kinds.join("+")
        };
        println!("{}", crate::trf!("  {:<12} {:<16} 실행 중 {:<3} {}", "  {:<12} {:<16} running {:<3} {}", a.name, reader, n, origin));
    }

    let now = time::now();
    println!("{}", tr("\n트랜스크립트 (최근 24시간)", "\nTranscripts (last 24 hours)"));
    for d in adapters::table().iter().filter(|d| d.agent.has_reader) {
        let a = d.agent;
        let ts = transcript::discover(a.name, now);
        println!("{}", crate::trf!("  {:<12} {} 개", "  {:<12} {} found", a.name, ts.len()));
        // 루트를 전부 보여주고, 각각 실제로 존재하는지 표시한다.
        // "어느 셸에서 띄웠든 한 화면에"가 안 될 때 여기서 원인이 보여야 한다.
        for root in d.transcript_dirs.iter().chain(d.json_dirs.iter()) {
            let mark = if root.is_dir() { "O" } else { tr("· 없음", "· missing") };
            println!("      [{mark}] {}", root.display());
        }
        for q in &d.queries {
            let mark = if q.file.is_file() { "O" } else { tr("· 없음", "· missing") };
            println!("      [{mark}] {} (sqlite)", q.file.display());
        }
        for t in ts.iter().take(3) {
            println!(
                "      {} | cwd={} | model={}",
                time::ago(now - t.last_event_at),
                t.cwd.as_ref().map(|c| c.display().to_string()).unwrap_or_else(|| "?".into()),
                t.model.clone().unwrap_or_else(|| "?".into())
            );
        }
    }

    if let Some(path) = orca::path() {
        println!("{}", crate::trf!("\nOrca SSH hook  {} 개 (최근 24시간)\n  {}", "\nOrca SSH hook  {} found (last 24 hours)\n  {}", orca::collect(now, false).len(), path.display()));
    }
    #[cfg(windows)]
    println!("{}", crate::trf!("\nSSH 터미널 화면 관찰  {} 개 (보이는 Codex·Claude 화면, 상태 미확인)",
        "\nSSH terminal screen observations  {} found (visible Codex/Claude screens, status unknown)", terminal::collect(&procs, now).len()));

    // 위 목록을 만들며 읽지 못한 것들. "왜 내 세션이 안 보이지"의 답이다.
    // JSON 스냅샷의 `warnings` 와 같은 출처를 쓴다 — 진단이 두 벌이 되면
    // 둘 중 하나는 반드시 낡는다.
    let source_problems = transcript::source_problems();
    if !source_problems.is_empty() {
        println!("{}", tr("\n소스 문제", "\nSource problems"));
        for p in &source_problems {
            println!("  ! {p}");
        }
    }
}
