# Native macOS development app (v0.3.0 work)

The Mac app uses AppKit windows, menus, session list, selectable details, clipboard and a JSON template editor. Rust continues to own collection, observed status, search, filtering, pinning, hiding, dismissal, revival, settings and template validation. There is no browser or network server in this desktop path. Windows keeps its existing native shell.

**Release acceptance is pending.** The build targets macOS 13.0 on Apple Silicon and Intel. Compilation and automated checks on hosted runners do not establish support on macOS 13, installation through Finder, VoiceOver usability or correct return to real agent sessions.

## Build and install a development copy

On a Mac with Rust and Xcode command line tools:

```sh
cargo test --release --locked
MACOSX_DEPLOYMENT_TARGET=13.0 cargo test --manifest-path desktop/Cargo.toml --release --locked
bash desktop/package-macos.sh
```

The script refuses to overwrite its output folder. It creates `desktop/target/macos-bundle/waid.app`, an architecture-specific ZIP and `SHA256SUMS.txt`. It applies only an ad-hoc signature for development, not a Developer ID signature or notarization. CI uploads the same development ZIP for each architecture; it does not publish a release.

Extract the ZIP if using an artifact, then copy `waid.app` to `~/Applications` or `/Applications` and open it in Finder. No companion collector, browser runtime or Rust installation is required on the destination Mac. Gatekeeper acceptance of a downloaded artifact is still unverified; this project does not disable quarantine or system security. Developer ID signing, notarization and a clean-machine installation check remain release work.

For terminal development, `desktop/target/release/waid-desktop` opens the native app directly. The same binary supports `--waid-core --json --history`, `--waid-core doctor`, and `--licenses`. The app starts its own collector process and stops only that child on Quit. Closing the window hides it; collection continues until Quit.

## Controls and persistence

- Use the search field or **Command+F**, status and agent menus, **Show all**, and **Include auxiliary**. Selection and list scroll position survive ordinary refreshes. Select a row to read and copy its prompt, last answer and first prompt in the details pane.
- **Pin** / **Command+P**, **Hide**, and **Do not show in waid** / **Command+D** use the same organization as Windows. **Show all** exposes hidden and excluded rows; **Restore** reverses exclusion. A distinguishable new request in the same conversation revives an excluded session automatically. Original logs and agent processes are never modified.
- Double-click or use **Open session** / **Command+Return** for session return. **Copy prompt** uses the macOS clipboard. Standard text editing shortcuts work in search, selectable details and the template editor. **Command+H** hides the app, **Command+M** minimizes, **Command+W** closes a window, and **Command+Q** quits.
- The `waid` menu-bar item and Dock reopen the window. **Always on top** and 40–100% **Opacity** persist. Window geometry uses AppKit frame autosaving; session preferences use the shared settings file.
- **Templates…** provides Daylight and Midnight examples, editable JSON, Import, Export, Preview and Apply. Preview does not save; closing the editor cancels preview. Apply uses the same version, field, size, color and contrast validation as Windows. Mac uses native system fonts and a single list/detail split; Windows' two-column cards and custom title bar are not reproduced.

Settings live at `~/Library/Application Support/waid/settings.json`; `WAID_DATA_DIR` selects another folder. Saves are atomic. A failed explicit settings write rolls the action back and displays an error. An unreadable or invalid existing settings file stops startup with an error rather than overwriting it. Multiple instances still use last-write-wins settings, so use one instance normally.

Finder launches do not inherit your interactive shell profile. Default log directories under the user's home work without shell setup. Custom `CODEX_HOME`, adapter settings and `ORCA_USER_DATA_PATH` must reach the app process. For diagnostics with custom paths, launch the executable from a terminal with the appropriate variables set.

## Session return boundaries

| Target | Implemented behavior | Still requires real-Mac verification |
|---|---|---|
| Local Orca Claude/Codex session | Reuses the Windows exact provider-session, launch-authority and unique live-terminal checks, then calls `orca terminal switch`. CLI work runs off the UI thread. | Installed CLI discovery, stale/ambiguous refusal, visible focus and correct conversation on both architectures. |
| ChatGPT copied link | **Connect copied link** accepts only `codex://threads/<this-row-session-id>`, with no query, fragment or new-chat route. Return dispatches the saved link using macOS `open`. | ChatGPT installation, registered route and actual conversation selection. Dispatch success is not proof of selection. |
| Windows saved window / standalone Terminal / iTerm / Claude desktop | No Mac window identity or conversation-return implementation. Windows window associations report a platform error; unconnected rows try only exact Orca mapping. | These targets are not claimed as supported Mac return paths. |
| SSH session | Collected from the local Orca mirror when available; no SSH return or prompt delivery. | Remote control is outside v0.3.0. |

`ORCA_CLI_COMMAND` can select an absolute Orca executable path, including a development CLI. Without it, the app uses `orca` on its inherited PATH, extended for that subprocess with `/usr/local/bin`, `/opt/homebrew/bin` and `~/.local/bin` for Finder launches. It does not retry a failed call against a different Orca build. No shell is used to interpret session identifiers or links. See the [documented ChatGPT route](https://learn.chatgpt.com/docs/reference/commands#deep-links). Missing, ended, stale and ambiguous Orca mappings show an explanation and preserve the session details.

## Required release walkthrough

Record architecture, OS, app commit/build, source app versions and results for each step on both Apple Silicon and Intel. Keep private prompts and full snapshots out of public evidence.

1. Install the packaged app via Finder on a clean user account, including downloaded ZIP quarantine and signature checks. Confirm icon/Dock/menu appearance, initial focus, resize, minimize, close/reopen and Quit. Confirm no collector child remains after Quit.
2. Collect actual Claude Code and Codex sessions, including a local Orca session. Compare project, request, model, answer and logged state to original logs. Submit a new request, wait for response, interrupt and exercise an error; check observed status without relying on elapsed time.
3. Search/filter, pin, hide, dismiss, Show all and restore. Dismiss again, send a new request in that same conversation, and verify automatic revival. Check repeated identical requests where the source exposes request identity.
4. Apply a custom template and change opacity/topmost. Quit/relaunch: verify organization, connections, search/filter and applied template persist, old active history starts Unknown, and reopening a dismissed conversation alone does not revive it. Check invalid JSON and unwritable settings errors.
5. Return to the exact live Orca session and a manually associated ChatGPT conversation. Verify stale launches, duplicate mappings, ended sessions, wrong links and missing apps are refused/explained. No prompt may be sent by any of these actions.
6. Check Command shortcuts, Unicode clipboard, long prompts, narrow/resized windows, light/dark templates, multi-monitor restoration and VoiceOver. Verify the oldest intended OS separately from macOS 15 CI.

The automated AppKit smoke uses isolated `WAID_DATA_DIR`, creates real native controls, changes search/topmost/opacity, opens the editor and exercises close/reopen twice. Rust tests cover persistence, template preview/apply, organization, invalid links, revival and collector-error retention. These are fixture/control checks, not the walkthrough above.
