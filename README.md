<img src="assets/waid_logo.png" alt="waid 로고" width="120" align="right">

# waid — what am I doing?

여러 코딩 AI 세션의 **현재 작업, 프로젝트, 에이전트·모델, 마지막으로 확인한 상태**를 한 화면에서 보는 Windows 앱입니다. Claude Code·Codex·Copilot CLI 로그를 읽기만 하며 에이전트에 명령을 보내지 않습니다. 개발 빌드이며 확인한 범위와 남은 검증은 [검증 기록](VALIDATION.md)에 있습니다.

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

기본 창은 **360×420 논리 픽셀**의 작은 창입니다. 각 세션을 휴대폰 알림처럼 둥근 카드로 표시하고 앞에 하네스 로고를 둡니다. 카드에는 작업 / 프로젝트·상태 / 에이전트·모델이 표시됩니다. 제목 표시줄에는 **항상 위**, **최소화**, **닫기**가 있고, 제목 부분을 끌어 이동하고 가장자리를 끌어 크기를 조절합니다.

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
| 내 차례 | 마지막 기록이 턴 종료입니다. 승인 요청 대기와 새 요청을 보낼 차례는 구분하지 못합니다. |
| 작업 중 | 최근 20초 안에 요청·작업 시작·도구 활동 등을 확인했습니다. |
| 중단 / 유휴 | 명시적인 중단 기록(turn_aborted, Copilot abort/shutdown 등)입니다. 프로세스 미발견으로 만들지 않습니다. |
| 오류 | 로그 이벤트 자체의 오류입니다. 도구 출력의 error 문자열만으로 판정하지 않습니다. |
| 미확인 | 해석할 상태가 없거나 작업 기록이 20초 넘게 갱신되지 않았습니다. 긴 도구 실행도 포함될 수 있습니다. |
| 종결 | 사용자가 waid에서 정리한 세션입니다. 원본 대화·프로세스는 종료하지 않습니다. 턴 종료와 구분합니다. |

앱은 **날짜 제한 없이 기존 로그도 수집**합니다. CLI는 기본 최근 24시간이며 `--history`로 제한을 해제합니다. 현재 열린 창 목록이나 세션 생존은 보증하지 않으며 JSON의 `alive: null`은 미확인입니다. 표시하는 작업은 읽기 범위에서 찾은 최근 실제 사용자 요청이고, `WAID_TASK`·프로젝트 `.waid`·실행 인자 라벨이 있으면 그것을 우선합니다.

## 발견되지 않을 때

```powershell
.\target\release\waid.exe doctor
```

기본 `~/.claude/projects`, `~/.codex/sessions`, `~/.copilot/session-state`와 명시된 `CODEX_HOME/sessions`, `COPILOT_HOME/session-state`를 읽습니다. 앱 실행 프로세스에 환경변수가 전달되어야 합니다. 다른 경로는 [사용자 어댑터](CLI.md#에이전트-추가하기)를 설정하세요.

Windows에서는 이름·PID를 열거하고 알려진 실행 파일과 인터프리터에 한해 읽기 전용으로 실행 인자를 조회합니다. 작업 디렉터리는 조회하지 않아 프로세스와 개별 로그를 확실히 연결하지 못하며, 읽기 권한·지원하지 않는 형식·아주 큰 로그 때문에 누락될 수 있습니다.

## 에이전트별 지원 범위

| 에이전트 | 실행 감지 | 저장된 작업·모델·상태 |
|---|---|---|
| Claude Code, Codex | 지원 | 지원 |
| Copilot CLI | 직접 실행·Node 패키지 | session-state의 events.jsonl 지원 |
| Gemini CLI, OpenCode | 직접 실행·Node/Bun 패키지 | 아직 미지원 |
| Aider | 직접 실행·Python 스크립트·python -m aider | 아직 미지원 |
| Cursor CLI, Goose | cursor-agent / goose 실행 파일 | 아직 미지원 |

프로세스만 발견하면 상태는 **미확인**입니다. 모델은 실행 인자에 명시된 경우만 표시합니다. Cursor 편집기·VS Code 확장 에이전트는 CLI와 별개이며, WSL 내부 프로세스 감지는 지원하지 않습니다.

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
