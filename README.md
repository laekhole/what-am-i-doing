<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/waid-on-dark.png">
    <img src="assets/waid-horizontal.png" alt="waid — what am I doing?" width="440">
  </picture>
</p>

# waid — what am I doing?

Korean version: [README.ko.md](README.ko.md).

A Windows app that shows the **current task, project, agent/model, and last observed status** of multiple coding AI sessions in one place. It only reads Claude Code, Codex, and Copilot CLI logs; it does not send commands to agents. This is a development build. See the [validation record](VALIDATION.md) for verified behavior and outstanding checks.

## Download and verify a release

Portable releases on [GitHub Releases](https://github.com/laekhole/what-am-i-doing/releases) contain the following three files. Verify the ZIP, extract it, and run `waid-desktop.exe`. Keep `waid.exe` in the same folder.

```text
waid-v1.0.0-windows-x64.zip
waid-v1.0.0-windows-x64.zip.sigstore.json
SHA256SUMS.txt
```

Run these PowerShell commands in the folder containing all three matching release files. Replace `v1.0.0` with the downloaded tag. You need Cosign 3.1.3 or later; see the [Cosign installation guide](https://docs.sigstore.dev/cosign/system_config/installation/).

```powershell
$tag = 'v1.0.0'
$zip = "waid-$tag-windows-x64.zip"
$expected = (Get-Content -LiteralPath SHA256SUMS.txt -Raw).Trim()
$actual = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expected -cne "$actual  $zip") { throw 'Checksum mismatch.' }
cosign verify-blob --bundle "$zip.sigstore.json" --certificate-identity "https://github.com/laekhole/what-am-i-doing/.github/workflows/release.yml@refs/tags/$tag" --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' $zip
if ($LASTEXITCODE -ne 0) { throw 'Release signature verification failed.' }
```

SHA-256 checks file integrity. Cosign checks that the file was **signed by this repository's release workflow for the specified tag**. Do not run it if signature verification fails. The detached Sigstore signature is not a Windows Authenticode signature and **does not guarantee that Smart App Control or SmartScreen will allow execution**. This release process does not change your PC's security settings.

Maintainers can push a tag such as `v1.0.0` or `v1.0.0-rc.1` on a reviewed commit. The [release workflow](.github/workflows/release.yml) tests and builds the core and app on Windows x64, checks ZIP contents, generates checksums, and performs keyless signing and verification using GitHub OIDC. It attaches all three files to a draft release before publishing it. Tags with suffixes such as `-rc.1` are published as prereleases. No signing private keys or certificate secrets are stored in the repository. Sigstore's public transparency log records the signing identity and artifact digest.

The tagged commit must include the workflow file. Releases are not published if CI fails; check for a remaining draft if uploading or publishing fails. Existing releases and assets are not automatically overwritten. The ZIP version follows the tag; the core and app Cargo versions are managed independently.

To create an unsigned ZIP and checksum locally, run `./desktop/build.ps1 -Portable -Tag v1.0.0`. Output goes to `desktop/target/release/bundle`; existing output is not overwritten. To package two already-built EXEs, use `./desktop/package.ps1 -Tag v1.0.0 -OutputDirectory <new-output-folder>`. Local packaging does not include GitHub OIDC signing.

## Run on Windows

Install Rust and linker tools for your Windows target. From the repository root:

```powershell
cargo test --release --locked
cargo build --release --locked
$env:WAID_CORE_PATH = Join-Path $PWD 'target/release/waid.exe'
cargo test --manifest-path desktop/Cargo.toml --release --locked
cargo build --manifest-path desktop/Cargo.toml --release --locked
.\desktop\target\release\waid-desktop.exe
```

For the GNU target, add `--target x86_64-pc-windows-gnu` to the core and desktop commands and include the target directory in executable paths.

`waid.exe` is the collection core. Keep it alongside the desktop executable. No browser, Node, or WebView2 is required. The app only terminates the core process it started.

## Using the app

The default window is **600×780 logical pixels**, with expandable session cards in one column and an optional two-column view. Each card shows the project, latest request, status badge, and model beneath the status. Bundled Pretendard SemiBold body text and Bold headings keep text clear. The waid logo appears at the top, and harness logos appear on cards. Drag the title area to move the window and its edges to resize it.

- **Always on top** keeps the window above other normal windows. It is separate from session **Pin**.
- **Transparency** opens a **0–60%** slider. Always-on-top and transparency settings persist across restarts.
- Click a card header to expand the latest request, answer, and first prompt. Use the prompt's copy button to copy it. Collapsed cards show minutes ago within an hour, hours ago within 24 hours, and `yy-mm-dd` afterward; expanded cards always show `yy-mm-dd`.
- Click a harness logo or **Open session** to switch to the matching existing Orca session. This uses the original session ID; an unavailable session produces a message instead of opening a different conversation.
- **Minimize** and **Close** hide the window in the system tray. Click the tray icon to restore it, or choose **Exit** from its menu to quit.
- **···** expands the details, search, pin, dismiss, and template controls. Double-click a two-column list item or press Enter to open its session.
- **Search** (`Ctrl+F`) opens the expanded view. Search tasks, projects, models, agents, and folders, combined with status and agent filters. Use **Compact** or Esc to return to the small window.
- **Dismiss** (`Ctrl+D`) removes the selected session from the default list. Find it in **Show all** and use **Restore** to bring it back. A dismissed session returns automatically when a **new user request** is logged. Idle status, waiting for a response, or a missing process alone never dismisses it automatically.
- **Pin** (`Ctrl+P`) keeps a session at the top; **Hide** (`Ctrl+H`) hides it from the list. **Show all** includes dismissed, hidden, and auxiliary entries.
- Auxiliary sessions are hidden by default. Enable **Include auxiliary** to show subagents and automated reviews.
- The list refreshes every two seconds while preserving selection, search, and filters. If the core fails, the app shows the last successful list alongside an error message.
- Use `Tab / Shift+Tab` to navigate, arrow keys and Page Up/Down in the list, and `Ctrl+C` to copy selected detail text.

Window settings, search, session organization, and the applied template are saved in `%LOCALAPPDATA%\waid\settings.json`. Set `WAID_DATA_DIR` to use another folder. Source logs are never modified. Multiple windows sharing a settings file use last-write-wins behavior, so normally use one window.

Change templates through **··· → Templates → Duplicate default example → Edit → Preview → Apply**. See the [template guide](TEMPLATES.md) for all fields.

## What statuses and tasks mean

| Status | Meaning |
|---|---|
| Idle | No new user request has been observed since the app started, or an explicit interruption was logged. Existing conversations start here. |
| Working | A new user request was observed after startup. This persists until a response ends, an interruption occurs, or an error is logged. Elapsed time alone never changes a long-running task to idle or waiting. |
| Waiting | The response to a request made after startup has ended. The next logged user request changes it back to working. |
| Error | An error in the log event itself. An `error` string in tool output alone does not qualify. |
| Unknown | The source does not provide readable request/response boundaries. Process activity or file updates alone are not used to infer working or waiting. |
| Dismissed | A session the user has organized away in waid. This does not terminate the original conversation or process and is distinct from a turn ending. |

The app **collects existing logs without a date limit**. The CLI defaults to the last 24 hours; `--history` removes that limit. Neither guarantees a list of currently open windows or live sessions; JSON `alive: null` means unknown. The displayed task is the latest actual user request found within the read range, unless a `WAID_TASK`, project `.waid`, or command-line label takes precedence.

The app starts a new observation baseline each time it launches. Delayed log writes delay status changes. CLI/JSON statuses continue to use the latest log and a 20-second activity threshold. JSON `request_at` is the last user request's Unix timestamp in seconds, or null if unavailable.

## Troubleshooting missing sessions

```powershell
.\target\release\waid.exe doctor
```

Default sources are `~/.claude/projects`, `~/.codex/sessions`, and `~/.copilot/session-state`, plus explicitly configured `CODEX_HOME/sessions` and `COPILOT_HOME/session-state`. Environment variables must reach the app process. For other paths, configure a [custom adapter](CLI.md#에이전트-추가하기).

On Windows, the app enumerates process names and PIDs and reads command-line arguments only for known executables and interpreters. It does not query working directories, so it cannot reliably associate processes with individual logs. Read permissions, unsupported formats, or very large logs may cause missing entries.

## Agent support

| Agent | Process detection | Records read |
|---|---|---|
| Claude Code, Codex | Supported | JSONL — requests, models, paths, statuses |
| Copilot CLI | Direct executable or Node package | JSONL (`events.jsonl` in session-state) |
| Cursor (editor) | cursor-agent CLI only | SQLite (`state.vscdb`) — sessions, requests, timestamps |
| Cline, Roo Code, VS Code Chat | None (run inside the editor) | JSON files — requests, models, paths |
| Gemini CLI, opencode, Continue | Direct executable or Node/Bun package | JSON files |
| Aider | Direct executable, Python script, or python -m aider | Not yet supported (Markdown history) |
| Goose | goose executable | Not yet supported |

Non-JSONL records do not provide turn-end events, so their **status stays unknown**. If no timestamp is recorded, the file timestamp is shown as an estimate. Missing evidence is not invented. SQLite uses the operating system's existing engine (`winsqlite3.dll` on Windows) and opens databases **read-only**. If the engine is unavailable or a schema changes, only that source is left empty; `waid doctor` explains why.

When only a process is found, its status is **unknown**. Models are shown only when explicitly specified in command-line arguments. Cursor editor windows and VS Code extensions are read from saved records, not detected as processes. Process detection inside WSL is not supported.

## CLI and documentation

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

MIT
