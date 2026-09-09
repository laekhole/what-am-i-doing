<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/waid-on-dark.png">
    <img src="assets/waid-horizontal.png" alt="waid — what am I doing?" width="440">
  </picture>
</p>

# waid — what am I doing?

Korean version: [README.ko.md](README.ko.md).

A native desktop app that shows the **current task, project, agent/model, and last observed status** of multiple coding AI sessions in one place. It reads existing coding-agent logs and does not send commands to agents.

**v0.2.0 is the Windows milestone, with stabilization ongoing.** Both Rust packages are version `0.2.0`. The native macOS app is development work toward v0.3.0; Android, iOS/iPadOS and waidaway LAN access are planned. See the [v0.2.0 release notes](releases/v0.2.0.md) for changes from v0.1.0 and upgrade instructions.

[Download and verify](#download-and-verify-a-release) · [Controls](#using-the-app) · [Agent support](#agent-support) · [Troubleshooting](#troubleshooting-missing-sessions) · [Mac development](MACOS.md)

## Features at a glance

| Feature | What you can do |
|---|---|
| Session overview | See project, latest request, agent, model and observed status together, refreshed every two seconds. Claude Code and Codex are the priority sources. |
| Conversation previews | Expand a card for the latest prompt, last logged answer and first prompt; copy the prompt without switching apps. |
| Context visibility | See the latest logged usage and compaction evidence. When 25% or less remains, the Windows card/list emphasizes the remaining percentage; observed compaction is also emphasized. |
| Session organization | Search and filter, pin, hide, exclude and restore sessions. A distinguishable new user request automatically restores an excluded conversation. |
| Return to work | Open an exact existing local Orca session, an associated ChatGPT link, or a manually connected supported Windows window. [Target-specific limits](#session-return) apply. |
| Desktop convenience | Expandable one-column cards or a two-column list, always on top, 0–60% transparency, taskbar minimize, tray hide/restore and keyboard shortcuts on Windows. |
| Custom appearance | Daylight/Midnight JSON templates with edit, preview, apply, import and export; settings persist across restarts. |
| Local tools | CLI tables, streaming JSON, a localhost HTML dashboard, custom log adapters and `waid doctor` diagnostics. |

Windows controls and status labels are currently primarily Korean; English names in this guide explain their function. Mac uses native AppKit controls with some shared Korean status/detail text. A language selector is not implemented.

## Before you start

- **This is a log viewer, not proof that a session is currently open.** Old working/waiting history starts **Unknown** each launch. Status changes depend on readable request/response events; delayed logs delay the display.
- **Collection and session return have different support levels.** Some registered agents have only synthetic tests or process detection. JSON/SQLite sources have no reliable turn-end events and remain Unknown.
- **Context is the last logged measurement, not a live token count or account quota.** Claude transcripts do not establish a context limit, so their percentage is unknown. Missing compaction evidence does not mean compaction never happened.
- **Excluding a session only organizes waid.** It neither deletes the conversation nor stops its agent. Reopening the original app alone does not restore an excluded entry. Search and filters still apply in Show all; auxiliary sessions require Include auxiliary.
- **Local settings can contain saved prompts and answers.** Use one waid window per settings file, back up settings before upgrading, and check exported JSON/HTML or screenshots before sharing them.
- **Release signing verifies origin and integrity.** Sigstore is not Windows Authenticode; Smart App Control or SmartScreen may still block the EXE. macOS installation/signing acceptance, accessibility, mixed-DPI and long-duration real use remain validation work.

Latest code validation (`47f4317`, 2026-09-09): [Windows CI](https://github.com/laekhole/what-am-i-doing/actions/runs/34308905362) passed **130 checks**; [Mac CI](https://github.com/laekhole/what-am-i-doing/actions/runs/34308905356) passed **121 Rust checks per architecture** plus AppKit smoke/restart checks on Apple Silicon and Intel. Environment-dependent tests remain ignored: three on Windows and one on each Mac architecture. These results do not complete the real-device walkthroughs in [VALIDATION.md](VALIDATION.md) and [MACOS.md](MACOS.md).

## Privacy and local operation

**waid is a locally installed, locally running app. It processes coding-agent logs on your computer and does not upload your prompts, answers, source logs, or session data to the developer or any external server.** The current app has no telemetry, analytics, cloud sync, or remote AI API calls. The Windows release runs directly from a single EXE without an installer.

- The collector reads existing local logs and opens supported SQLite sources read-only; it does not modify source conversations or send prompts to agents. Orca SSH support reads Orca's existing local hook mirror, without connecting to the remote machine.
- Settings and saved session details stay in `%LOCALAPPDATA%\waid\settings.json` on Windows or `~/Library/Application Support/waid/settings.json` on macOS. `WAID_DATA_DIR` can change this location; choose a local folder if you do not want your own sync software to copy it elsewhere.
- The native desktop uses a local collector process. The optional CLI `--html --watch` dashboard serves data only on `127.0.0.1` (localhost), not on a LAN or public interface. Its bundled page uses only local resources. Custom HTML templates can contain scripts or external resources, so this statement applies to the bundled template.
- User-requested session return passes a session identifier to the installed app or switches an existing local window/tab. Those apps' own network activity, opening documentation/download links, and manually sharing exported files are separate from waid's local collection. Planned waidaway LAN features are not implemented in this version.

These concrete safeguards describe the current implementation; they are not a claim that software can be guaranteed free of vulnerabilities. See [release verification](#download-and-verify-a-release) and the [validation record](VALIDATION.md) for integrity checks and tested limits.

## Version roadmap

| Version | Milestone | Status |
|---|---|---|
| v0.2.0 | Windows support | Usable on Windows; stabilization continues. |
| v0.3.0 | Add macOS support, including MacBook and desktop Macs | Native AppKit app in development; real-Mac release validation pending |
| v0.4.0 | Add Android support and waidaway LAN access | Planned |
| v0.5.0 | Add iOS and iPadOS support for iPhone and iPad | Planned |
| v1.0.0 | Complete stabilization of the supported platforms and core workflows | Future stable release |

These are project milestones. macOS support does not include iPhone or iPad. Mobile support is intended to show coding-agent session activity collected on a PC or Mac; the connection design remains to be implemented. Platform additions do not by themselves mean stabilization is complete: v1.0.0 follows compatibility, reliability, and real-use validation across the supported platforms.

The v0.3.0 release goal is the core Windows session-management experience on Mac: collect real sessions, organize them, preserve settings across restarts, and return to supported sessions from a native app. Collector and shared desktop tests run on Apple Silicon and Intel in the [macOS checks workflow](.github/workflows/macos.yml); passing them alone does not validate the Mac UI, installation, signing, or real-agent workflows. The build deployment target is macOS 13.0; the oldest supported version still requires real-device validation.

The proposed v0.4.0 waidaway scope is session viewing and user-requested prompt delivery to supported sessions over the same LAN (wired or Wi-Fi), starting with a mobile browser client. The waid collector stays read-only; a separate delivery path must verify the destination session. Pairing, authentication, encryption, device revocation, and duplicate-send protection are required before shipping remote input. Remote access will default to off; internet relay service is a later scope. These capabilities are not implemented yet.

For macOS development and installation, see [the Mac guide](MACOS.md). The native AppKit app calls the same Rust collector, state tracking, search/filter, pin, exclusion/restore and template logic as Windows. Mac settings use `~/Library/Application Support/waid/settings.json`, and Orca hook discovery uses `~/Library/Application Support/orca/agent-hooks/last-status.json`. `WAID_DATA_DIR` and `ORCA_USER_DATA_PATH` override their respective folders. Native builds and automated smoke checks do not establish that installation or real-session return works on a user's Mac.

## Download and verify a release

Download `waid-v0.2.0-windows-x64.exe` from the [v0.2.0 release](https://github.com/laekhole/what-am-i-doing/releases/tag/v0.2.0), verify it as described below, and double-click it. **One EXE is enough: no extraction, installer, Rust, Cargo, Node, or WebView2 is required.** The collector, fonts, and font license are built in. The other two release files are for verification; the app does not need them at runtime. v0.1.0 ZIP releases use the older two-executable layout.

On first launch, waid discovers supported local logs automatically. If a session is missing, open **Show all**, clear search/filters and follow [diagnostics](#troubleshooting-missing-sessions). Existing history may initially show Unknown until a new request is observed.

**Upgrading from v0.1.0:** quit the old app through its tray menu, back up `%LOCALAPPDATA%\waid\settings.json` (or your `WAID_DATA_DIR`), then run the verified v0.2.0 EXE. The settings location is unchanged; old session keys migrate only when their match is unambiguous. An old companion `waid.exe` is not required by the new desktop app. There is no automatic updater.

```text
waid-v0.2.0-windows-x64.exe
waid-v0.2.0-windows-x64.exe.sigstore.json
SHA256SUMS.txt
```

Run these PowerShell commands in the folder containing all three matching release files. Replace `v0.2.0` with the downloaded tag. You need Cosign 3.1.3 or later; see the [Cosign installation guide](https://docs.sigstore.dev/cosign/system_config/installation/).

```powershell
$tag = 'v0.2.0'
$exe = "waid-$tag-windows-x64.exe"
$expected = (Get-Content -LiteralPath SHA256SUMS.txt -Raw).Trim()
$actual = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expected -cne "$actual  $exe") { throw 'Checksum mismatch.' }
cosign verify-blob --bundle "$exe.sigstore.json" --certificate-identity "https://github.com/laekhole/what-am-i-doing/.github/workflows/release.yml@refs/tags/$tag" --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' $exe
if ($LASTEXITCODE -ne 0) { throw 'Release signature verification failed.' }
```

SHA-256 checks file integrity. Cosign checks that the file was **signed by this repository's release workflow for the specified tag**. Do not run it if signature verification fails. The detached Sigstore signature is not a Windows Authenticode signature and **does not guarantee that Smart App Control or SmartScreen will allow execution**. This release process does not change your PC's security settings.

Maintainers can push a tag such as `v0.2.0` or `v0.2.0-rc.1` on a reviewed commit. The [release workflow](.github/workflows/release.yml) tests and builds the core and app on Windows x64, verifies the standalone EXE, generates checksums, and performs keyless signing and verification using GitHub OIDC. It attaches all three files to a draft release before publishing it. Tags with suffixes such as `-rc.1` are published as prereleases. No signing private keys or certificate secrets are stored in the repository. Sigstore's public transparency log records the signing identity and artifact digest.

The tagged commit must include the workflow and a matching `releases/<tag>.md` file, which becomes the release body. Releases are not published if the release workflow's Windows checks fail; macOS checks run separately and do not publish a Mac release. Check for a remaining draft if uploading or publishing fails. Existing releases and assets are not automatically overwritten. Match the tag, release notes and both Cargo package versions when preparing a release; filenames alone do not enforce version consistency.

To create an unsigned standalone EXE and checksum locally, run `./desktop/build.ps1 -Portable -Tag v0.2.0`. Output goes to `desktop/target/release/bundle`; existing output is not overwritten. To prepare an already-built desktop EXE for release, use `./desktop/package.ps1 -Tag v0.2.0 -OutputDirectory <new-output-folder>`. Local packaging does not include GitHub OIDC signing.

If local Windows application control blocks Rust tests or build scripts, use the [Windows checks workflow](.github/workflows/check.yml). It runs on GitHub-hosted Windows runners for pushes to `main`, pull requests, and manual **Actions → Windows checks → Run workflow** runs. After the checks pass, download the `waid-windows-x64` artifact containing the EXE and checksum; these test packages are unsigned and retained for seven days. The workflow does not publish a release.

Building on GitHub avoids a local build restriction, but downloaded EXEs still face the PC's application control policy. Authenticode signing and timestamping remain future release work; see [Microsoft's Smart App Control signing guidance](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control).

## Build from source on Windows

Only developers building from source need Rust (including Cargo) and linker tools for their Windows target. From the repository root:

```powershell
cargo test --release --locked
cargo build --release --locked
cargo test --manifest-path desktop/Cargo.toml --release --locked
cargo build --manifest-path desktop/Cargo.toml --release --locked
.\desktop\target\release\waid-desktop.exe
```

For the GNU target, add `--target x86_64-pc-windows-gnu` to the core and desktop commands and include the target directory in executable paths.

`waid-desktop.exe` runs on its own. It starts a second instance of itself in internal collector mode and only terminates that child when exiting. No companion executable is extracted or required. The independent `waid.exe` CLI remains available for terminal use; building the desktop also compiles the shared core directly. View the embedded font license through **Font license (폰트 라이선스)** in the tray menu, which opens a text copy in the app data folder.

## Using the app

The controls below describe Windows. See [Mac controls and current limits](MACOS.md) for the AppKit interface.

The default window is **600×780 logical pixels**, with expandable session cards in one column and an optional two-column view. Each card shows the project, latest request, status badge, and model beneath the status. Bundled Pretendard SemiBold body text and Bold headings keep text clear. The waid logo appears at the top, and harness logos appear on cards. Drag the title area to move the window and its edges to resize it.

- **Always on top** keeps the window above other normal windows. It is separate from session **Pin**.
- **Transparency** opens a **0–60%** slider. Always-on-top and transparency settings persist across restarts.
- Click a card header to expand the latest request, answer, and first prompt. Use the prompt's copy button to copy it. Collapsed cards show minutes ago within an hour, hours ago within 24 hours, and `yy-mm-dd` afterward; expanded cards always show `yy-mm-dd`.
- Click a harness logo or **Open session** to return to its saved connection, or to the matching existing Orca session when no connection is saved. Use **Connect (연결)** in the expanded controls or **···** menu to select an existing supported window, associate a copied ChatGPT session link, or clear a connection. See [Session return](#session-return) for the supported targets.
- **Minimize** keeps the window on the taskbar for normal restoration. **Close** hides it in the system tray. Click the tray icon to restore it, or choose **Exit** from its menu to quit.
- **···** expands the details, search, pin, dismiss, and template controls. Double-click a two-column list item or press Enter to open its session.
- **Search** (`Ctrl+F`) opens the expanded view. Search tasks, projects, models, agents, and folders, combined with status and agent filters. Use **Compact** or Esc to return to the small window.
- **Do not show in waid (waid에서 보지 않기)** in an expanded card removes that session from waid's default list only. The toolbar calls this **보지 않기** (`Ctrl+D`). It never deletes the original conversation or stops its agent. Find excluded sessions in **Show all (전체 보기)** and use **Restore** to bring them back. A session returns automatically when you send a **new user request in that conversation**, including from JSON-file and SQLite sources. Merely opening the conversation does not restore it. Revival needs distinguishable request evidence; a repeated identical request cannot be detected when the source exposes no changed request ID, request timestamp, or user-message history. Idle status, waiting for a response, or a missing process alone never removes it automatically. Up to 1,024 excluded sessions can be saved; restore entries to free space.
- The default list shows activity from the **last 24 hours**, plus pinned sessions, sessions observed since this launch, and entries whose date is unknown. Older history remains available through **Show all**; aging out of this view does not dismiss a session.
- **Pin** (`Ctrl+P`) keeps a session at the top; **Hide** (`Ctrl+H`) hides it from the list. **Show all** includes older, dismissed, and hidden entries. Search and status/agent filters still apply.
- Auxiliary sessions are hidden by default, including in **Show all**. Only **Include auxiliary** reveals subagents, automated reviews, and Orca dispatched workers. Classification uses session metadata and Orca's injected worker preamble within the bounded log read range.
- Session cards display the latest logged **Prompt**, preserving line breaks in the expanded card (up to 4,000 characters). User task labels remain in CLI output. Orca sessions identified by the current hook mapping show the Orca logo; provider and model identity stay separate.
- Cards also show the latest logged **context usage percentage** when both token count and context limit are available. When **25% or less remains**, the context line shows the remaining percentage with emphasis; observed compaction also keeps this line emphasized, including when usage is unknown. Both facts appear together when applicable, directly in collapsed cards and the two-column list without opening details. Expand a card for tokens / limit, the measurement's timestamp (UTC), and observed compaction with its timestamp when available. These are measurements, with no quality score, cumulative token total, or account-plan quota. Unknown values are shown as unknown, including a missing context limit; model names are never used to guess one.
- The list refreshes every two seconds while preserving selection, search, and filters. If the core fails, the app keeps the last successful list and shows the error text until a later refresh succeeds. A snapshot larger than the 16 MiB limit is skipped and the list recovers at the next valid snapshot.
- Use `Tab / Shift+Tab` to navigate, arrow keys and Page Up/Down in the list, and `Ctrl+C` to copy selected detail text.

Window settings, search, session organization, connections, and the applied template are saved in `%LOCALAPPDATA%\waid\settings.json`. Set `WAID_DATA_DIR` to use another folder. Source logs are never modified. Multiple windows sharing a settings file use last-write-wins behavior, so normally use one window.

Change templates through **··· → Templates → Duplicate default example → Edit → Preview → Apply**. See the [template guide](TEMPLATES.md) for all fields.

## What statuses and tasks mean

| Status | Meaning |
|---|---|
| Idle | An explicit interruption was logged, including one logged before the app started. |
| Working | A new user request was observed after startup. This persists until a response ends, an interruption occurs, or an error is logged. Elapsed time alone never changes a long-running task to idle or waiting. |
| Waiting | The response to a request made after startup has ended. The next logged user request changes it back to working. |
| Error | An error in the log event itself, including one logged before the app started. An `error` string in tool output alone does not qualify. |
| Unknown | No request/response boundary has been observed since the app started, or the source does not provide readable boundaries. Conversations that were working or waiting before launch start here. Process activity or file updates alone are not used to infer working or waiting. |
| Dismissed | A session the user has organized away in waid. This does not terminate the original conversation or process and is distinct from a turn ending. |

The app **collects existing logs without a date limit**, then applies the recent-activity view described above. The CLI defaults to the last 24 hours; `--history` removes that limit. Neither guarantees a list of currently open windows or live sessions; JSON `alive: null` means unknown. The displayed task is the latest actual user request found within the read range, unless a `WAID_TASK`, project `.waid`, or command-line label takes precedence.

The app starts a new observation baseline each time it launches. Before-launch Working/Waiting records show **Unknown**, with a separate **Last logged status** line on the card. This preserves useful history without treating it as newly observed activity. Delayed log writes delay status changes. CLI/JSON statuses continue to use the latest log and a 20-second activity threshold. JSON `request_at` is the last user request's Unix timestamp in seconds, or null if unavailable; request identity separately preserves finer source timestamps and available user-message counts.

## Session return

Orca, PowerShell, ChatGPT App, and Claude Code desktop are the first stabilization targets. Other terminal/editor integrations follow after these are validated. Log collection and returning to a window are separate capabilities.

| Target | Current behavior | Limit |
|---|---|---|
| Orca | Automatically selects the unique existing local terminal matching the original provider session and current launch. | Missing, ambiguous, ended, or stale mappings show details and an explanation. |
| ChatGPT App | Associate **Copy chat deep link** (`Ctrl+Alt+L` in ChatGPT) using the clipboard option in Connect, then open the saved link. | Only the documented `codex://threads/<thread-id>` route matching this Codex row is accepted. Dispatching a link does not confirm that the app opened the conversation. |
| PowerShell | Select an existing classic PowerShell console window in Connect; return to that exact window. | Windows Terminal/Orca/VS Code pseudoconsole tabs and ambiguous nested shells are excluded. A window connection does not identify the conversation inside it. |
| ChatGPT / Claude Code desktop window | Select an existing app window in Connect; return to it and choose the conversation in the app. | Window return only; no automatic Claude conversation route is claimed. |

Window connections verify the window handle, process identity, executable, and process start time again before returning. Reconnect after the app or console restarts. An expired saved connection reports an error. ChatGPT link behavior follows the [official deep-link documentation](https://learn.chatgpt.com/docs/reference/commands#deep-links); Claude's documented desktop workflow is [sidebar session selection](https://code.claude.com/docs/en/desktop). A saved ChatGPT link was verified against actual app navigation; the full physical-click connection flow, foreground focus and all four targets' complete walkthroughs remain validation items. See [VALIDATION.md](VALIDATION.md) for the precise evidence.

Orca return requires its installed CLI on the app's PATH, or an executable path in `ORCA_CLI_COMMAND`, as well as a current hook mapping. Collecting a log by itself does not establish a window connection.

## Troubleshooting missing sessions

```powershell
.\target\release\waid.exe doctor
```

Default sources are `~/.claude/projects`, `~/.codex/sessions`, and `~/.copilot/session-state`, plus explicitly configured `CODEX_HOME/sessions` and `COPILOT_HOME/session-state`. Environment variables must reach the app process. For other paths, configure a [custom adapter](CLI.md#에이전트-추가하기).

Orca SSH conversations are also collected from its local `%APPDATA%/orca/agent-hooks/last-status.json` mirror (`ORCA_USER_DATA_PATH` overrides the Orca folder). Version 2 records must match the current pane's launch authority. Available prompts, models, answers, and hook events appear alongside local sessions; remote paths are never opened as local files. `waid doctor` reports this source separately. The mirror contains the latest session per pane, not full remote history: sessions observed during a running waid process are retained until it exits, but overwritten records cannot be recovered after restart. Identical repeated requests are distinguishable only when their submit event is observed. SSH session return is still unsupported.

On Windows, the app enumerates process names and PIDs and reads command-line arguments only for known executables and interpreters. It does not query working directories, so it cannot reliably associate processes with individual logs. Read permissions, unsupported formats, or very large logs may cause missing entries. JSONL reads are bounded to a 128 KiB head and 256 KiB tail, with cached incremental observations; individual JSON files are limited to 4 MiB. A date-unlimited collection is not a complete transcript archive. Source-specific warnings appear in `waid doctor`, in a bounded optional `warnings` array in JSON output, and in the app.

## Agent support

**Claude Code and Codex are the priority collection sources.** Other registered readers and process detectors remain experimental until their real-source cases pass the same checks. The table reports evidence already collected, not a guarantee of complete product support.

Validation levels: **real-log** means the output was compared against this machine's actual logs (see the [validation record](VALIDATION.md)); **fixture-only** means synthetic logs or process stubs in the test suite; **path registration** means the default location is known and the generic reader is fixture-tested, but no real data from that product has been read; **detection-only** means the process is recognised and no history is read.

| Agent | Process detection | Records read | Validation |
|---|---|---|---|
| Claude Code, Codex | Executable or Node package | JSONL — requests, models, paths, statuses, answers | Real-log |
| Copilot CLI | Executable or Node package | JSONL (`events.jsonl` in session-state) | Fixture-only; no real Copilot session has been read |
| Cursor (editor) | cursor-agent CLI only (process stub) | SQLite (`state.vscdb`) — sessions, requests, timestamps; status stays unknown | Real-log for records (one local database); detection fixture-only |
| Cline, Roo Code, VS Code Chat | None (run inside the editor) | JSON files — requests, models, paths | Path registration (experimental) |
| Gemini CLI, opencode, Continue | Executable or Node/Bun package | JSON files | Path registration (experimental); detection fixture-only |
| Aider | Executable, Python script, or python -m aider | None (Markdown history not supported) | Detection-only |
| Goose | goose executable | None | Detection-only |

Path-registration sources may show nothing if the product's file format differs from what was assumed; this is not treated as complete support. Process-detection checks use selected stub executables and interpreters plus the test process's own arguments. These checks do not establish coverage of every product's real launcher.

Non-JSONL records do not provide turn-end events, so their **status stays unknown**. If no timestamp is recorded, the file timestamp is shown as an estimate. Missing evidence is not invented. SQLite uses the operating system's existing engine (`winsqlite3.dll` on Windows) and opens databases **read-only**. If the engine is unavailable or a schema changes, only that source is left empty; `waid doctor` explains why.

Context collection currently reads Codex JSONL `token_count.info.last_token_usage.total_tokens` and `model_context_window`, and Claude Code JSONL assistant input tokens plus cache creation/read input tokens. Claude's input-only calculation follows its [documented context usage calculation](https://code.claude.com/docs/en/statusline#context-window-fields); its transcript does not establish the active context limit, so percentage remains unknown. Other sources, including the Orca SSH hook mirror, currently have unknown context usage. These values describe the last logged measurement, not a live recount of unsent text or tool results. Explicit Codex `compacted` / `context_compacted` and Claude `compact_boundary` records mark compaction and clear the old usage until the next measurement. The existing bounded reads and in-memory cache apply: missing compaction evidence means unknown, not “never compacted,” and an unread gap clears the old measurement. A restart can lose compaction evidence outside the read range.

When only a process is found, its status is **unknown**. Models are shown only when explicitly specified in command-line arguments. Cursor editor windows and VS Code extensions are read from saved records, not detected as processes. Process detection inside WSL is not supported.

## CLI and documentation

JSON output now uses **schema 2**: `id` is a full, opaque source identity; `legacy_id` carries the previous eight-character display hash for migration only. Consumers must not truncate `id` or use `legacy_id` as a unique key. The desktop accepts schemas 1 and 2 and migrates saved organization when an old key maps unambiguously; colliding old keys are preserved without guessing a target. See [CLI extensions](CLI.md) for marker compatibility.

Each JSON session also contains `context`: nullable `used_tokens`, `window_tokens`, `observed_at`, and `compacted_at`, plus boolean `compaction_observed`. Timestamps are Unix seconds from source events; missing timestamps are not replaced with file modification time. `compaction_observed: false` means no compaction evidence was read. Percentage is derived from available tokens / limit. Older snapshots and saved rows without this additive field remain readable.

```text
waid                  Print a table once
waid --json --watch   Stream JSON
waid --json --history Include sessions from earlier dates
waid --waiting        Show only sessions awaiting the user
waid --agent claude   Filter by agent
waid --html --watch   Local HTML dashboard
waid doctor           Diagnose paths, adapters, and processes
```

The HTML watch dashboard defaults to `http://127.0.0.1:7423`; `--port 0` chooses a free port. `--interval SEC` changes CLI polling (minimum one second). Use `--help` for all flags, `--eject` for the bundled HTML template and `--keys` for template fields. CLI themes and adapters live under `~/.config/waid/` (or `XDG_CONFIG_HOME/waid`); they are separate from native desktop JSON templates and settings. Task labels from process environment variables depend on `/proc` access and are not available from Windows process enumeration.

[CLI extensions](CLI.md) · [User templates](TEMPLATES.md) · [Validation record](VALIDATION.md) · [Product scope](PRODUCT.md) · [Design decisions](DECISIONS.md) · [Development history](HISTORY.md)

[Development history](HISTORY.md) records each project request, resulting changes, checks, and remaining work in Korean. The repository's [agent instructions](AGENTS.md) require an update for each project-related request; this is an agent-maintained work log, not an automatic capture of conversations.

## Contributors and AI assistance

Created and maintained by [laekhole](https://github.com/laekhole), with AI coding assistance from:

- **Claude Opus 5 (Claude Code)** — implementation and documentation, credited in earlier commits.
- **OpenAI GPT-6 (Codex)** — implementation, testing, and documentation, including the standalone Windows executable.

AI contributions are acknowledged in commit messages with `Co-authored-by` trailers.

MIT
