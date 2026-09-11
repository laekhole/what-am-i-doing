# App and harness marks

`waid.png` is the 256px version of the original `assets/waid-app-icon.png` tile, used in the Windows window and taskbar. The header renders the original transparent `assets/waid-mascot.png` directly with GDI+ bicubic interpolation at 32 logical pixels, adjusted for the window DPI. It does not stretch an intermediate HICON. The 382px tile and 408px mascot sources retain their original dimensions and have no question mark.

Root `assets/waid.ico` contains 16, 20, 24, 32, 40, 48, 64, 128 and 256px versions of the tile for the executable, tray and installer; installed shortcuts and Windows Settings use that executable icon. Regenerate the PNG and ICO with `powershell -File assets/build-icons.ps1`, then rebuild the app. The script replaces complete files so an open image preview does not require overwriting its mapped image data. `assets/question_mark/` is an archived concept and is not consumed by the app or builds.

The Mac packaging script generates its Dock/Finder ICNS from the same `assets/waid-app-icon.png` source at 16–1024px. `desktop/icons/` and `desktop/icon.svg` are legacy assets that the current native builds do not consume.

The marks below identify the harness that produced a session; they are not waid branding or model indicators. They are embedded in the app and require no network access at runtime.

- OpenAI mark for Codex: [OpenAI's official GitHub organization](https://github.com/openai), original [organization avatar](https://avatars.githubusercontent.com/u/14957082?v=4), downloaded 2026-09-06 as openai.png. See [OpenAI brand guidelines](https://openai.com/brand/).
- Claude mark for Claude Code: original [Claude favicon](https://claude.ai/favicon.ico), downloaded 2026-09-06 as claude.ico. See [Anthropic's official media resources](https://www.anthropic.com/news).

- Orca app mark: original `resources/build/icon.png` from the installed Orca application's `app.asar.unpacked`, copied 2026-09-08 as `orca.png`. Shown when Orca's hook mapping identifies the session host.

The files retain their original contents. Product names and marks belong to their respective owners; waid's MIT license does not relicense these third-party marks. waid does not imply sponsorship or endorsement. Unknown harnesses use a generic terminal symbol and their supplied display name.
