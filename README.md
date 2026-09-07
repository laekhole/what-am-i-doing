<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/waid-on-dark.png">
    <img src="assets/waid-horizontal.png" alt="waid — what am I doing?" width="440">
  </picture>
</p>

# waid — what am I doing?

여러 코딩 AI 세션의 **현재 작업, 프로젝트, 에이전트·모델, 마지막으로 확인한 상태**를 한 화면에서 보는 Windows 앱입니다. Claude Code·Codex·Copilot CLI 로그를 읽기만 하며 에이전트에 명령을 보내지 않습니다. 개발 빌드이며 확인한 범위와 남은 검증은 [검증 기록](VALIDATION.md)에 있습니다.

## 릴리스 다운로드와 검증

[GitHub Releases](https://github.com/laekhole/what-am-i-doing/releases)에 게시되는 포터블 릴리스는 아래 세 파일로 구성됩니다. ZIP을 검증한 뒤 압축을 풀고 `waid-desktop.exe`를 실행하세요. 같은 폴더의 `waid.exe`도 필요합니다.

```text
waid-v1.0.0-windows-x64.zip
waid-v1.0.0-windows-x64.zip.sigstore.json
SHA256SUMS.txt
```

파일명이 같은 세 파일을 내려받은 폴더에서 아래 PowerShell 명령을 실행합니다. `v1.0.0`은 실제 다운로드한 태그로 바꾸세요. [Cosign 설치 안내](https://docs.sigstore.dev/cosign/system_config/installation/)에 따라 Cosign 3.1.3 이상이 필요합니다.

```powershell
$tag = 'v1.0.0'
$zip = "waid-$tag-windows-x64.zip"
$expected = (Get-Content -LiteralPath SHA256SUMS.txt -Raw).Trim()
$actual = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expected -cne "$actual  $zip") { throw '체크섬이 일치하지 않습니다.' }
cosign verify-blob --bundle "$zip.sigstore.json" --certificate-identity "https://github.com/laekhole/what-am-i-doing/.github/workflows/release.yml@refs/tags/$tag" --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' $zip
if ($LASTEXITCODE -ne 0) { throw '릴리스 서명 검증에 실패했습니다.' }
```

SHA-256은 파일의 일치를 확인하며, Cosign은 **이 저장소의 해당 태그·릴리스 워크플로 신원으로 서명한 파일**인지 확인합니다. 서명 검증에 실패하면 실행하지 마세요. 별도 Sigstore 서명은 Windows Authenticode 서명이 아니므로 **Smart App Control·SmartScreen 실행 허용을 보장하지 않습니다.** 이 릴리스 절차는 PC의 보안 설정을 변경하지 않습니다.

유지관리자는 검토한 커밋에 `v1.0.0` 또는 `v1.0.0-rc.1` 같은 태그를 푸시하면 됩니다. [릴리스 워크플로](.github/workflows/release.yml)가 Windows x64에서 코어·앱 테스트와 빌드, ZIP 내용 검사, 체크섬 생성, GitHub OIDC 기반 keyless 서명·검증을 실행합니다. 세 파일을 Draft Release에 첨부한 뒤 공개하며, `-rc.1` 등의 접미사가 있는 태그는 사전 릴리스로 게시합니다. 서명 개인키나 인증서 비밀은 저장소에 등록하지 않습니다. Sigstore 공개 투명성 로그에는 서명 신원과 아티팩트 다이제스트가 기록됩니다.

워크플로 파일을 포함한 커밋에 태그를 붙여야 합니다. CI를 통과하지 못하면 릴리스를 공개하지 않으며, 업로드·공개 도중 실패하면 남은 초안을 확인하세요. 이미 존재하는 릴리스나 자산을 자동 덮어쓰지 않습니다. ZIP의 버전은 태그를 따릅니다. 코어·앱의 Cargo 버전은 독립적으로 관리합니다.

로컬에서는 `./desktop/build.ps1 -Portable -Tag v1.0.0`으로 서명 전 ZIP·체크섬을 만들 수 있습니다. 출력은 `desktop/target/release/bundle`이며 기존 결과가 있으면 덮어쓰지 않습니다. 이미 빌드한 두 EXE만 패키징하려면 `./desktop/package.ps1 -Tag v1.0.0 -OutputDirectory <새-출력-폴더>`를 사용하세요. 로컬 패키징에는 GitHub OIDC 서명이 포함되지 않습니다.

## Windows에서 실행

Rust와 Windows 타깃의 링크 도구가 필요합니다. 저장소 루트에서:

```powershell
cargo test --release --locked
cargo build --release --locked
$env:WAID_CORE_PATH = Join-Path $PWD 'target/release/waid.exe'
cargo test --manifest-path desktop/Cargo.toml --release --locked
cargo build --manifest-path desktop/Cargo.toml --release --locked
.\desktop\target\release\waid-desktop.exe
```

GNU 타깃을 쓰려면 양쪽 명령에 `--target x86_64-pc-windows-gnu`를 추가하고 실행 경로에도 해당 타깃 디렉터리를 넣습니다.

같은 폴더의 `waid.exe`가 수집 코어입니다. 두 파일을 함께 두세요. 브라우저·Node·WebView2는 필요 없습니다. 앱은 자신이 실행한 코어만 종료합니다.

## 화면 사용

기본 창은 **360×420 논리 픽셀**의 작은 창입니다. 각 세션은 **프로젝트명 / 태스크명 / 상태 배지 / 에이전트·모델**을 구분한 카드로 표시합니다. 상태 배지는 기호·색상·텍스트를 함께 사용합니다. 앱 상단에는 waid 로고, 카드에는 하네스 로고가 표시됩니다. 제목 표시줄에는 **항상 위**, **최소화**, **닫기**가 있고, 제목 부분을 끌어 이동하고 가장자리를 끌어 크기를 조절합니다.

- **항상 위**는 다른 일반 창보다 위에 유지합니다. 세션 **고정**과 별개입니다.
- **투명도** 버튼을 누르면 **0~60%** 슬라이더가 펼쳐집니다. 항상 위·투명도는 재실행해도 유지합니다.
- **···**를 누르면 상세·검색·고정·종결·템플릿 버튼이 펼쳐집니다. 긴 요청은 목록을 더블 클릭하거나 Enter로 상세 화면에서 확인합니다.
- **검색**(`Ctrl+F`)은 확장 화면을 엽니다. 작업·프로젝트·모델·에이전트·폴더 검색과 상태·에이전트 필터를 함께 쓰며, **간단히** 또는 Esc로 작은 창으로 돌아옵니다.
- **종결**(`Ctrl+D`)은 선택한 세션을 기본 목록에서 제외합니다. **전체 보기**에서 확인하고 **되살리기**로 복귀시킵니다. 종결한 세션에 **새 사용자 요청**이 기록되면 자동 복귀합니다. 유휴·응답 대기·프로세스 미발견만으로 자동 종결하지 않습니다.
- **고정**(`Ctrl+P`)은 목록 위에 유지하고 **숨기기**(`Ctrl+H`)는 목록에서 감춥니다. **전체 보기**에는 종결·숨김·보조 항목을 포함합니다.
- 보조 세션은 기본적으로 숨깁니다. `보조 포함`을 켜면 하위 에이전트·자동 검토도 표시합니다.
- 갱신은 2초 간격이며 선택·검색·필터를 유지합니다. 코어 오류 시 마지막 정상 목록과 오류 안내를 함께 표시합니다.
- `Tab / Shift+Tab`으로 이동, 목록에서 방향키·Page Up/Down, 상세 텍스트 선택 후 `Ctrl+C` 복사를 지원합니다.

창 설정과 검색·정리·적용 템플릿은 `%LOCALAPPDATA%\waid\settings.json`에 저장합니다. `WAID_DATA_DIR`로 폴더를 바꿀 수 있습니다. 원본 로그는 수정하지 않습니다. 같은 설정 파일을 쓰는 여러 창은 마지막 저장이 우선하므로 보통 한 창만 사용하세요.

템플릿은 **··· → 템플릿 → 기본 예제 복제 → 수정 → 미리보기 → 적용**으로 바꿉니다. [템플릿 안내](TEMPLATES.md)에 전체 필드 설명이 있습니다.

## 상태와 작업의 의미

| 화면 | 의미 |
|---|---|
| 유휴 | 앱 실행 후 새 사용자 요청이 관측되지 않았거나 명시적인 중단 기록이 있는 세션입니다. 기존 대화는 이 상태에서 시작합니다. |
| 작업 중 | 앱 실행 후 새 사용자 요청을 확인했습니다. 응답 종료·중단·오류 기록이 올 때까지 유지하며, 긴 작업도 시간만으로 유휴나 대기로 바꾸지 않습니다. |
| 대기 중 | 앱 실행 후 요청한 작업의 응답이 끝났습니다. 다음 사용자 요청이 기록되면 다시 작업 중이 됩니다. |
| 오류 | 로그 이벤트 자체의 오류입니다. 도구 출력의 error 문자열만으로 판정하지 않습니다. |
| 미확인 | 요청·응답 경계를 읽을 수 없는 소스입니다. 프로세스나 파일 갱신만으로 작업 중·대기 중을 추정하지 않습니다. |
| 종결 | 사용자가 waid에서 정리한 세션입니다. 원본 대화·프로세스는 종료하지 않습니다. 턴 종료와 구분합니다. |

앱은 **날짜 제한 없이 기존 로그도 수집**합니다. CLI는 기본 최근 24시간이며 `--history`로 제한을 해제합니다. 현재 열린 창 목록이나 세션 생존은 보증하지 않으며 JSON의 `alive: null`은 미확인입니다. 표시하는 작업은 읽기 범위에서 찾은 최근 실제 사용자 요청이고, `WAID_TASK`·프로젝트 `.waid`·실행 인자 라벨이 있으면 그것을 우선합니다.

앱의 관측 기준은 실행할 때마다 새로 시작합니다. 로그가 늦게 기록되면 상태 변경도 늦어집니다. CLI/JSON의 상태는 기존처럼 마지막 로그와 20초 활동 기준을 사용하며, JSON의 `request_at`은 마지막 사용자 요청의 Unix 초 시각(시각이 없으면 null)입니다.

## 발견되지 않을 때

```powershell
.\target\release\waid.exe doctor
```

기본 `~/.claude/projects`, `~/.codex/sessions`, `~/.copilot/session-state`와 명시된 `CODEX_HOME/sessions`, `COPILOT_HOME/session-state`를 읽습니다. 앱 실행 프로세스에 환경변수가 전달되어야 합니다. 다른 경로는 [사용자 어댑터](CLI.md#에이전트-추가하기)를 설정하세요.

Windows에서는 이름·PID를 열거하고 알려진 실행 파일과 인터프리터에 한해 읽기 전용으로 실행 인자를 조회합니다. 작업 디렉터리는 조회하지 않아 프로세스와 개별 로그를 확실히 연결하지 못하며, 읽기 권한·지원하지 않는 형식·아주 큰 로그 때문에 누락될 수 있습니다.

## 에이전트별 지원 범위

| 에이전트 | 실행 감지 | 읽는 기록 |
|---|---|---|
| Claude Code, Codex | 지원 | JSONL — 요청·모델·경로·상태 |
| Copilot CLI | 직접 실행·Node 패키지 | JSONL(session-state의 events.jsonl) |
| Cursor (편집기) | cursor-agent CLI만 | SQLite(state.vscdb) — 세션·요청·시각 |
| Cline, Roo Code, VS Code Chat | 없음(편집기 안에서 돕니다) | JSON 파일 — 요청·모델·경로 |
| Gemini CLI, opencode, Continue | 직접 실행·Node/Bun 패키지 | JSON 파일 |
| Aider | 직접 실행·Python 스크립트·python -m aider | 아직 미지원(마크다운 기록) |
| Goose | goose 실행 파일 | 아직 미지원 |

JSONL이 아닌 기록에는 턴 종료 이벤트가 없습니다. 그래서 **상태는 미확인**으로 두고 시각도 기록에 없으면 파일 시각 추정으로 표시합니다 — 없는 근거를 지어내지 않습니다. SQLite는 운영체제에 이미 있는 엔진을 빌려 쓰고(Windows `winsqlite3.dll`), **읽기 전용**으로만 엽니다. 엔진이 없거나 스키마가 바뀌면 그 소스만 조용히 비고 `waid doctor`가 이유를 보여줍니다.

프로세스만 발견하면 상태는 **미확인**입니다. 모델은 실행 인자에 명시된 경우만 표시합니다. Cursor 편집기 창과 VS Code 확장은 프로세스로 잡지 않고 저장된 기록으로만 읽습니다. WSL 내부 프로세스 감지는 지원하지 않습니다.

## CLI와 문서

```text
waid                  표 한 번 출력
waid --json --watch   JSON 스트림
waid --json --history 이전 날짜의 세션도 포함
waid --waiting        사용자 차례만
waid --agent claude   특정 에이전트
waid --html --watch   로컬 HTML 대시보드
waid doctor           경로·어댑터·프로세스 진단
```

[CLI 확장](CLI.md) · [사용자 템플릿](TEMPLATES.md) · [검증 기록](VALIDATION.md) · [제품 범위](PRODUCT.md) · [설계 결정](DECISIONS.md)

MIT
