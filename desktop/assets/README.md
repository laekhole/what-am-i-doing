# App and harness marks

`waid.png` is the 256px version of the approved `assets/waid-app-icon.png` tile, used in the app header, window and taskbar. Root `assets/waid.ico` contains 16–256px versions of that same tile for the executable, tray and installer. Regenerate both with `powershell -File assets/build-icons.ps1` from the repository root.

The marks below identify the harness that produced a session; they are not waid branding or model indicators. They are embedded in the app and require no network access at runtime.

- OpenAI mark for Codex: [OpenAI's official GitHub organization](https://github.com/openai), original [organization avatar](https://avatars.githubusercontent.com/u/14957082?v=4), downloaded 2026-09-06 as openai.png. See [OpenAI brand guidelines](https://openai.com/brand/).
- Claude mark for Claude Code: original [Claude favicon](https://claude.ai/favicon.ico), downloaded 2026-09-06 as claude.ico. See [Anthropic's official media resources](https://www.anthropic.com/news).

- Orca app mark: original `resources/build/icon.png` from the installed Orca application's `app.asar.unpacked`, copied 2026-09-08 as `orca.png`. Shown when Orca's hook mapping identifies the session host.

The files retain their original contents. Product names and marks belong to their respective owners; waid's MIT license does not relicense these third-party marks. waid does not imply sponsorship or endorsement. Unknown harnesses use a generic terminal symbol and their supplied display name.
