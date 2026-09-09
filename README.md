<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/waid-on-dark.png">
    <img src="assets/waid-horizontal.png" alt="waid — what am I doing?" width="440">
  </picture>
</p>

# waid — what am I doing?

Korean version: [README.ko.md](README.ko.md).

A Windows app that shows the **current task, project, agent/model, and last observed status** of multiple coding AI sessions in one place. It only reads coding-agent logs (see [Agent support](#agent-support) for which sources are validated); it does not send commands to agents. This is a development build. See the [validation record](VALIDATION.md) for verified behavior and outstanding checks.

## Version roadmap

| Version | Milestone | Status |
|---|---|---|
| v0.2.0 | Windows support | Usable on Windows; stabilization continues. |
| v0.3.0 | Add macOS support, including MacBook and desktop Macs | In development; native Mac UI not yet implemented |
| v0.4.0 | Add Android support and waidaway LAN access | Planned |
| v0.5.0 | Add iOS and iPadOS support for iPhone and iPad | Planned |
| v1.0.0 | Complete stabilization of the supported platforms and core workflows | Future stable release |

These are project milestones. macOS support does not include iPhone or iPad. Mobile support is intended to show coding-agent session activity collected on a PC or Mac; the connection design remains to be implemented. Platform additions do not by themselves mean stabilization is complete: v1.0.0 follows compatibility, reliability, and real-use validation across the supported platforms.

The v0.3.0 release goal is the core Windows session-management experience on Mac: collect real sessions, organize them, preserve settings across restarts, and return to supported sessions from a native app. Collector and shared desktop tests run on Apple Silicon and Intel in the [macOS checks workflow](.github/workflows/macos.yml); passing them alone does not validate the Mac UI, installation, signing, or real-agent workflows. The minimum supported macOS version remains to be established through release validation.

The proposed v0.4.0 waidaway scope is session viewing and user-requested prompt delivery to supported sessions over the same LAN (wired or Wi-Fi), starting with a mobile browser client. The waid collector stays read-only; a separate delivery path must verify the destination session. Pairing, authentication, encryption, device revocation, and duplicate-send protection are required before shipping remote input. Remote access will default to off; internet relay service is a later scope. These capabilities are not implemented yet.

For macOS development, run `cargo test --release --locked` for the collector and `cargo build --manifest-path desktop/Cargo.toml --release --locked` followed by `cargo test --manifest-path desktop/Cargo.toml --release --locked` for shared desktop logic. The CLI can be run with `cargo run --release --locked -- --json --history`; the desktop executable currently has no Mac UI. Mac settings use `~/Library/Application Support/waid/settings.json`, and Orca hook discovery uses `~/Library/Application Support/orca/agent-hooks/last-status.json`. `WAID_DATA_DIR` and `ORCA_USER_DATA_PATH` override their respective folders.

## Download and verify a release

Download `waid-<tag>-windows-x64.exe` from [GitHub Releases](https://github.com/laekhole/what-am-i-doing/releases), verify it as described below, and double-click it. **One EXE is enough: no extraction, installer, Rust, Cargo, Node, or WebView2 is required.** The collector, fonts, and font license are built in. The other two release files are for verification; the app does not need them at runtime. Older ZIP releases still use the two-executable layout.

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

The tagged commit must include the workflow file. Releases are not published if CI fails; check for a remaining draft if uploading or publishing fails. Existing releases and assets are not automatically overwritten. The download filename follows the tag; the core and app Cargo versions are managed independently.

To create an unsigned standalone EXE and checksum locally, run `./desktop/build.ps1 -Portable -Tag v0.2.0`. Output goes to `desktop/target/release/bundle`; existing output is not overwritten. To prepare an already-built desktop EXE for release, use `./desktop/package.ps1 -Tag v0.2.0 -OutputDirectory <new-output-folder>`. Local packaging does not include GitHub OIDC signing.

If local Windows application control blocks Rust tests or build scripts, use the [Windows checks workflow](.github/workflows/check.yml). It runs on GitHub-hosted Windows runners for pushes to `main`, pull requests, and manual **Actions → Windows checks → Run workflow** runs. After the checks pass, download the `waid-windows-x64` artifact containing the EXE and checksum; these test packages are unsigned and retained for seven days. The workflow does not publish a release.

Building on GitHub avoids the local build restriction, but downloaded EXEs still face the PC's application control policy. For Windows execution trust, the next signing step is **RSA Authenticode signing and timestamping of the standalone desktop EXE**, followed by signature verification, checksums, and the existing Sigstore signing. This requires a trusted signing provider; the current workflow does not perform Authenticode signing. See [Microsoft's Smart App Control signing guidance](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control).

[SignPath Foundation](https://signpath.org/terms.html) offers an application-based option for eligible open-source projects. [Azure Artifact Signing](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart) supports public trust for South Korean organizations; individual developers are currently limited to the US and Canada. Signing does not override an organization's [explicit deny policy](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/create-appcontrol-deny-policy) or guarantee [SmartScreen reputation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).

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

The default window is **600×780 logical pixels**, with expandable session cards in one column and an optional two-column view. Each card shows the project, latest request, status badge, and model beneath the status. Bundled Pretendard SemiBold body text and Bold headings keep text clear. The waid logo appears at the top, and harness logos appear on cards. Drag the title area to move the window and its edges to resize it.

- **Always on top** keeps the window above other normal windows. It is separate from session **Pin**.
- **Transparency** opens a **0–60%** slider. Always-on-top and transparency settings persist across restarts.
- Click a card header to expand the latest request, answer, and first prompt. Use the prompt's copy button to copy it. Collapsed cards show minutes ago within an hour, hours ago within 24 hours, and `yy-mm-dd` afterward; expanded cards always show `yy-mm-dd`.
- Click a harness logo or **Open session** to return to its saved connection, or to the matching existing Orca session when no connection is saved. Use **Connect (연결)** in the expanded controls or **···** menu to select an existing supported window, associate a copied ChatGPT session link, or clear a connection. See [Session return](#session-return) for the supported targets.
- **Minimize** keeps the window on the taskbar for normal restoration. **Close** hides it in the system tray. Click the tray icon to restore it, or choose **Exit** from its menu to quit.
- **···** expands the details, search, pin, dismiss, and template controls. Double-click a two-column list item or press Enter to open its session.
- **Search** (`Ctrl+F`) opens the expanded view. Search tasks, projects, models, agents, and folders, combined with status and agent filters. Use **Compact** or Esc to return to the small window.
- **Do not show in waid (waid에서 보지 않기)** in an expanded card removes that session from waid's default list only. The toolbar calls this **보지 않기** (`Ctrl+D`). It never deletes the original conversation or stops its agent. Find excluded sessions in **Show all (전체 보기)** and use **Restore** to bring them back. A session returns automatically when you send a **new user request in that conversation**, including from JSON-file and SQLite sources. Merely opening the conversation does not restore it. Revival needs distinguishable request evidence; a repeated identical request cannot be detected when the source exposes no changed request ID, request timestamp, or user-message history. Idle status, waiting for a response, or a missing process alone never removes it automatically.
- The default list shows activity from the **last 24 hours**, plus pinned sessions, sessions observed since this launch, and entries whose date is unknown. Older history remains available through **Show all**; aging out of this view does not dismiss a session.
- **Pin** (`Ctrl+P`) keeps a session at the top; **Hide** (`Ctrl+H`) hides it from the list. **Show all** includes older, dismissed, and hidden entries. Search and status/agent filters still apply.
- Auxiliary sessions are hidden by default, including in **Show all**. Only **Include auxiliary** reveals subagents, automated reviews, and Orca dispatched workers. Classification uses session metadata and Orca's injected worker preamble within the bounded log read range.
- Session cards display the latest logged **Prompt**, preserving line breaks in the expanded card (up to 4,000 characters). User task labels remain in CLI output. Orca sessions identified by the current hook mapping show the Orca logo; provider and model identity stay separate.
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

Window connections verify the window handle, process identity, executable, and process start time again before returning. Reconnect after the app or console restarts. An expired saved connection reports an error. ChatGPT link behavior follows the [official deep-link documentation](https://learn.chatgpt.com/docs/reference/commands#deep-links); Claude's documented desktop workflow is [sidebar session selection](https://code.claude.com/docs/en/desktop). Actual link dispatch and all four targets' live return flows remain release validation items.

## Troubleshooting missing sessions

```powershell
.\target\release\waid.exe doctor
```

Default sources are `~/.claude/projects`, `~/.codex/sessions`, and `~/.copilot/session-state`, plus explicitly configured `CODEX_HOME/sessions` and `COPILOT_HOME/session-state`. Environment variables must reach the app process. For other paths, configure a [custom adapter](CLI.md#에이전트-추가하기).

Orca SSH conversations are also collected from its local `%APPDATA%/orca/agent-hooks/last-status.json` mirror (`ORCA_USER_DATA_PATH` overrides the Orca folder). Version 2 records must match the current pane's launch authority. Available prompts, models, answers, and hook events appear alongside local sessions; remote paths are never opened as local files. `waid doctor` reports this source separately. The mirror contains the latest session per pane, not full remote history: sessions observed during a running waid process are retained until it exits, but overwritten records cannot be recovered after restart. Identical repeated requests are distinguishable only when their submit event is observed. SSH session return is still unsupported.

On Windows, the app enumerates process names and PIDs and reads command-line arguments only for known executables and interpreters. It does not query working directories, so it cannot reliably associate processes with individual logs. Read permissions, unsupported formats, or very large logs may cause missing entries. Source-specific warnings appear in `waid doctor`, in a bounded optional `warnings` array in JSON output, and in the app.

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

When only a process is found, its status is **unknown**. Models are shown only when explicitly specified in command-line arguments. Cursor editor windows and VS Code extensions are read from saved records, not detected as processes. Process detection inside WSL is not supported.

## CLI and documentation

JSON output now uses **schema 2**: `id` is a full, opaque source identity; `legacy_id` carries the previous eight-character display hash for migration only. Consumers must not truncate `id` or use `legacy_id` as a unique key. The desktop accepts schemas 1 and 2 and migrates saved organization when an old key maps unambiguously; colliding old keys are preserved without guessing a target. See [CLI extensions](CLI.md) for marker compatibility.

```text
waid                  Print a table once
waid --json --watch   Stream JSON
waid --json --history Include sessions from earlier dates
waid --waiting        Show only sessions awaiting the user
waid --agent claude   Filter by agent
waid --html --watch   Local HTML dashboard
waid doctor           Diagnose paths, adapters, and processes
```

[CLI extensions](CLI.md) · [User templates](TEMPLATES.md) · [Validation record](VALIDATION.md) · [Product scope](PRODUCT.md) · [Design decisions](DECISIONS.md)

## Contributors and AI assistance

Created and maintained by [laekhole](https://github.com/laekhole), with AI coding assistance from:

- **Claude Opus 5 (Claude Code)** — implementation and documentation, credited in earlier commits.
- **OpenAI GPT-6 (Codex)** — implementation, testing, and documentation, including the standalone Windows executable.

AI contributions are acknowledged in commit messages with `Co-authored-by` trailers.

MIT
