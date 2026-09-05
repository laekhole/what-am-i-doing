# waid — what am I doing?

에이전트 다섯 개를 돌려놓고 "내가 지금 뭘 하고 있더라" 하는 그 순간을 위한 도구.

돌고 있는 코딩 에이전트를 자동으로 찾아내, 다섯 개 컬럼으로 보여준다. 그 외에는 아무것도 하지 않는다.

```
$ waid
STATUS     TITLE                  AGENT        LLM         TASK
◐ waiting  web/fix/a11y-checkout  Claude Code  sonnet-4.6  결제 폼 접근성 개선 — 라벨, 포커스 순서…
● working  api/feat/auth-refresh  Claude Code  opus-4.6    JWT 리프레시 토큰 만료 처리를 구현해줘.…
✗ error    infra                  Claude Code  sonnet-4.6  테라폼 모듈 정리하고 중복 variable 제거
○ idle     payments               Codex        —           —
✓ done     docs/docs/i18n         Claude Code  haiku-4.5   README 다국어화 — 한국어/영어 분리
```

**waid는 에이전트를 실행하지 않는다.** tmux 세션을 만들지 않고, worktree를 파지 않고, 프롬프트를 보내지 않는다. 지금까지 하던 대로 각자의 터미널에서 `claude`를 치면, waid는 옆에서 볼 뿐이다. 파일 시스템은 읽기 전용으로만 만진다.

자세한 설계 근거는 [MANIFESTO.md](MANIFESTO.md).

---

## 플랫폼

| 플랫폼 | 상태 |
|---|---|
| Windows (네이티브) | 코어 v0.5.0 + 네이티브 앱 v0.5.2 개발 빌드; 이전 v0.5.1 설치·실행 확인 |
| macOS | `ps` 경로 유지, 이번 작업에서 실기 빌드·실사용 미검증 |
| Android 뷰어 | 개발 예정 |
| Linux / WSL | 기존 구현 유지. WSL → Windows Codex 읽기는 사용자 실환경 검증 보고 |

실환경 검증은 Windows → macOS → Android → WSL 순서입니다. Windows는 cwd를 못 얻어도 트랜스크립트로 표를 채우며, 프로세스를 못 봤다는 이유로 `done`을 만들지 않습니다. `waiting`은 마지막으로 확인한 턴 종료 신호이지, 현재 세션의 생존을 보증하는 값은 아닙니다. 상세 검증 범위와 미확인 항목은 [DECISIONS.md](DECISIONS.md)의 마지막 절에 기록합니다.

---

## 설치

```sh
cargo build --release
cp target/release/waid ~/.local/bin/
```

코어는 의존성 크레이트 0개, 별도 런타임 없음, 자발적 네트워크 요청 없음입니다. 별도 `desktop` 패키지만 Windows API 바인딩과 JSON 판독 의존성을 사용합니다. Tauri·WebView2는 제거했습니다.

Windows에서 코어만 빌드·실행:

```powershell
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --release
& "$env:USERPROFILE\.cargo\bin\cargo.exe" build --release
.\target\release\waid.exe doctor
.\target\release\waid.exe --html --watch
# 브라우저에서 http://127.0.0.1:7423 열기. 종료는 Ctrl-C.
```

Windows x64 앱·인스톨러 빌드 (Rust MSVC 도구 체인과 NSIS 필요, Node.js 불필요):

```powershell
.\desktop\build.ps1 -NsisPath 'C:\Program Files (x86)\NSIS\makensis.exe'
```

스크립트는 코어와 앱의 release 테스트가 통과한 뒤 빌드합니다. NSIS가 PATH에 있으면 `-NsisPath`를 생략할 수 있습니다. 이전 Tauri 빌드가 받은 NSIS 컴파일러도 재사용할 수 있으며 Tauri 자체는 필요 없습니다.

생성물: `desktop/target/release/bundle/nsis/waid_0.5.2_x64-setup.exe`. 일반 Windows 세션에서 이 설치 파일을 직접 실행하세요. 이전 앱은 먼저 종료하세요 (v0.5.0은 트레이의 `waid 종료`).
현재 사용자 전용 설치이며, 기본 설치 위치는 `%LOCALAPPDATA%\waid`입니다. WebView2나 브라우저 다운로드는 없습니다. 코드 서명은 아직 하지 않았습니다. 설치 없이 `desktop/target/release/waid-desktop.exe`를 실행해도 됩니다. 같은 폴더의 `waid.exe`는 함께 두세요.

앱은 작은 Windows 기본 목록 창입니다. 세션마다 제목·상태 / 작업 / 에이전트·모델의 세 줄이며, 긴 문장은 말줄임합니다. 카드·배지·애니메이션·브라우저·HTTP 서버를 쓰지 않습니다.

v0.5.2는 UI 정돈 단계입니다. 새 UI의 직접 시각 검사·설치는 아직 하지 않았고, 기존 설치본은 그대로입니다. **waid를 켠 뒤 관찰한 세션만 남기는 필터는 아직 미구현입니다.** Windows tasklist만으로는 열려 있는 유휴 세션과 과거 트랜스크립트를 정확히 구별할 수 없어 처리 기준을 확인 중입니다 (D20).

- 상단에는 최소화와 닫기만 있습니다. 최대화는 없습니다. 제목 부분을 끌어 이동하고 테두리로 크기를 조절합니다.
- 목록을 우클릭하면 `항상 위`와 `불투명도`(30~100%)를 조절할 수 있습니다. 키보드는 목록에 포커스를 두고 Shift+F10입니다.
- 모르는 모델·작업은 `—`, 추론한 작업은 `≈`로 표시합니다. 목록은 읽기 전용입니다.
- 닫기는 앱과 자신이 만든 waid를 종료합니다. 트레이·토스트는 없으며 에이전트는 건드리지 않습니다.

창 설정은 재실행하면 기본값으로 돌아옵니다. 상태 수집은 동봉 코어의 `--json --watch`, 기본 2초 간격입니다. 앱은 내용이 바뀔 때만 목록을 갱신하며 선택·스크롤 위치를 세션 id로 유지합니다. **120ms는 갱신 간격이 아니라 코어 첫 출력 목표입니다.**

Windows 단발 출력은 느린 tasklist를 기다리지 않으므로 트랜스크립트 없는 프로세스가 빠질 수 있습니다. `--watch`의 다음 갱신이나 `doctor`로 확인하세요. 실행 파일 이름만 같은 데스크톱 앱 보조 프로세스는 개별 세션과 구별할 수 없습니다.

`doctor`에서 Claude가 0개라면 설치 여부보다 실제 JSONL 존재 여부를 확인하세요. 이 PC에서는 `.claude`는 있지만 `projects`가 없고 `sessions`가 비어 있었습니다. 세션을 만든 적이 없다고 단정하거나 다른 경로를 추측으로 추가하지 않았습니다.

---

## 사용법

```
waid                    표 한 번 출력 (기본)
waid --watch            2초마다 갱신
waid --waiting          내 입력을 기다리는 세션만
waid --agent claude     특정 에이전트만
waid --json             JSON 스냅샷
waid --html             HTML 대시보드
waid --html --watch     로컬 대시보드 서버 (http://127.0.0.1:7423)
waid doctor             왜 내 세션이 안 보이는지 진단
```

셸에 물려 쓰기:

```sh
# 기다리는 에이전트가 생기면 알림
watch -n5 'waid --waiting --json | grep -q waiting && notify-send "에이전트가 기다립니다"'

# 프롬프트에 대기 개수 표시
waid --waiting --json | grep -c '"id"'
```

### TUI는 없습니다

`waid --watch`는 2초마다 표를 다시 그릴 뿐, `htop` 같은 전체화면 UI로 뜨지 않습니다. 의도된 것입니다.

TUI가 더 주는 건 키보드 상호작용뿐인데, 화면에 뜨는 세션은 보통 다섯에서 열 줄입니다. 그 정도 목록에 스크롤은 필요 없고, 정렬은 `waiting`이 위로 오는 것 하나면 충분하고, 항목을 골라서 할 일은 애초에 없습니다 — waid는 개입하지 않으니까요. 대신 ratatui와 crossterm이 딸려와 의존성 0개와 바이너리 5MB 예산이 동시에 깨집니다.

상호작용이 필요하면 바깥에서 붙이면 됩니다:

```sh
waid --json | fzf
waid --html --watch
```

---

## 커스터마이징

세 겹으로 나뉜다. 위로 갈수록 쉽고, 아래로 갈수록 자유롭다.

테마·HTML 템플릿은 CLI 출력용입니다. 네이티브 앱은 레이어 3의 별도 표시기이므로 HTML/CSS를 사용하지 않습니다.

### 레이어 1 — 테마 파일

`~/.config/waid/theme.toml`. 표의 컬럼 구성과 색을 바꾼다.

```toml
[columns]
order = ["status", "title", "task", "agent", "llm"]

[columns.title]
width = 24
truncate = "middle"      # start | middle | end

[sort]
status_priority = ["waiting", "working", "error", "idle", "done"]

[symbols]
working = "●"
waiting = "◐"
idle    = "○"
done    = "✓"
error   = "✗"

[colors]
waiting = "#f5a623"      # 가장 눈에 띄어야 한다
working = "#4ade80"
error   = "#ef4444"
```

### 레이어 2 — HTML 템플릿

기본 대시보드를 꺼내서 자기 것으로 만든다.

```sh
waid --html --eject > mine.html
waid --html --template mine.html --watch
```

CSS는 전적으로 당신 소유다. waid는 스타일시트를 강제하지 않는다. 기본 템플릿의 `:root` 변수만 바꿔도 인상이 완전히 달라지고, 마크업을 통째로 갈아엎어도 된다.

템플릿 문법은 넷뿐이다:

```
{{key}}                        값 치환 (항상 HTML 이스케이프)
{{#each sessions}} … {{/each}}  반복
{{#if key}} … {{else}} … {{/if}}
{{#unless key}} … {{/unless}}
```

쓸 수 있는 키 전체 목록:

```sh
waid --html --keys
```

주요 키:

| 키 | 설명 |
|---|---|
| `{{count}}` `{{count.waiting}}` | 전체/상태별 세션 수 |
| `{{any}}` `{{any.waiting}}` | 존재 여부 (조건 분기용) |
| `{{title}}` | `디렉터리/브랜치` |
| `{{agent.display}}` | `Claude Code`, `Codex` … |
| `{{llm.display}}` `{{llm.present}}` | 모델명, 알아냈는지 여부 |
| `{{task.text}}` `{{task.inferred}}` | 지시 내용, 추론값인지 여부 |
| `{{status.state}}` `{{status.label}}` `{{status.since}}` | `waiting`/`working`/…, 표시명, 경과 |
| `{{status.waiting}}` … | 상태별 불리언 |
| `{{branch}}` `{{cwd}}` `{{pid}}` `{{alive}}` | 부가 정보 |

`--watch`로 띄우면 SSE로 갱신을 밀어준다. 서버가 하는 일은 셋뿐이다:

```
GET /                 렌더된 템플릿
GET /snapshot.json    현재 스냅샷
GET /events           변경 신호 (SSE)
```

템플릿에 `[data-waid-root]` 요소와 기본 템플릿 끝의 `<script>` 조각을 남겨두면 자동 갱신이 따라온다. 렌더링 로직은 서버 한 곳에만 있으므로, 마크업을 어떻게 바꾸든 갱신은 계속 동작한다. 127.0.0.1에만 바인딩하며 인증·업로드·프록시는 없다.

### 레이어 3 — 임의 렌더러

```sh
waid --json --watch | your-own-thing
```

여기서부터는 당신의 세계다. 메뉴바 앱이든, i3blocks 위젯이든, e-ink 패널이든.

> **설계 원칙:** 커스터마이징 요청이 들어오면 먼저 "레이어 3으로 충분한가"를 묻는다. 충분하다면 코어에 아무것도 추가하지 않는다.

---

## `task` 컬럼에 대하여

수동 감지 방식에서는 사용자가 무엇을 시켰는지 알 방법이 원칙적으로 없다. waid는 다음 순서로 시도한다:

1. `WAID_TASK` 환경변수 또는 리포지토리 루트의 `.waid` 파일 → **확실**
2. 커맨드라인 인자 (`claude -p "…"`) → **확실**
3. 트랜스크립트의 첫 사용자 프롬프트를 정규화해서 발췌 → **추론**
4. 없음 → `—`

추론한 값은 표에서 흐리게, HTML에서는 `≈` 표시와 함께 나온다. **추론한 값을 확실한 값처럼 보이게 만들지 않는다.**

선명한 화면을 원하면 세션 시작 전에 라벨을 붙여라:

```sh
WAID_TASK="결제 폼 접근성" claude
```

---

## 지원 에이전트

프로세스 감지: Claude Code, Codex, Gemini CLI, opencode, Aider, Cursor CLI, Copilot CLI, Goose

트랜스크립트 판독(= `llm`/`task`/정확한 `status`): Claude Code, Codex

나머지는 프로세스만 잡히므로 `idle`과 `—`로 표시된다. 알 수 없는 것은 알 수 없다고 쓴다.

```sh
waid doctor   # 어느 경로가 잡히고 안 잡히는지
```

### 에이전트 추가하기

코드를 고칠 필요 없다. `~/.config/waid/adapters/mytool.toml`:

```toml
[adapter]
name    = "mytool"
display = "사내 에이전트"
exec    = ["mytool", "mytool-cli"]   # argv[0] 의 basename 후보
markers = ["@vendor/mytool"]         # 커맨드라인 표지 (선택)

[transcript]
dir  = "~/.mytool/sessions"          # 선택. 있으면 llm/task/status 가 열린다
dirs = ["/opt/agent/logs"]           # 루트가 여럿이면 목록으로
```

정규식이 아니라 실행 파일 이름 목록이다. 커맨드라인 전체를 정규식으로 훑으면 `vim claude.md`가 Claude Code로 잡힌다. 우리가 묻는 것은 "이 프로세스의 실행 파일이 에이전트인가"뿐이다.

`[transcript] dir` 을 주면 그 경로의 JSONL을 훑어 모델명·첫 프롬프트·cwd·에러 표지를 찾는다. 형식별 코드를 쓸 필요는 없다.

### 여러 실행 환경을 한 화면에

같은 에이전트를 WSL, PowerShell, Git Bash, 데스크톱 앱에서 번갈아 쓰면 트랜스크립트도 각각 다른 홈에 쌓입니다. waid는 이걸 한 표로 모읍니다.

프로세스 열거로는 안 됩니다. WSL 안의 `claude`는 별도 커널 네임스페이스의 리눅스 프로세스라 Windows의 `tasklist`로는 `wsl.exe`밖에 안 보이고, 반대도 마찬가지입니다. **대신 파일시스템은 경계를 넘습니다.** WSL에서 Windows가 `/mnt/c/...`로 보이고, Windows에서 WSL이 `\\wsl$\<배포판>\home\...`으로 보입니다.

그래서 `~/.claude/projects` 같은 경로는 **홈 후보 전체**로 펼쳐집니다. 어댑터에 한 줄만 적으면 되고, 경계 처리는 waid가 합니다. 어느 루트가 잡히고 안 잡히는지는 `waid doctor`가 보여줍니다:

```
트랜스크립트 (최근 24시간)
  claude       3 개
      [O] /home/minki/.claude/projects
      [O] /mnt/c/Users/minki/.claude/projects
      [· 없음] /mnt/c/Users/Public/.claude/projects
```

절대 경로는 펼쳐지지 않고 그대로 하나입니다.

**내장 정의 덮어쓰기**는 같은 `name` 을 쓰면 된다:

```toml
[adapter]
name    = "aider"
display = "Aider (사내 빌드)"
exec    = ["aider", "aider-dev"]     # 사내 래퍼 이름 추가

[transcript]
dir = "~/.aider/sessions"
```

어댑터에 문제가 있으면 조용히 무시되지만 `waid doctor` 가 이유를 알려준다:

```
어댑터          ~/.config/waid/adapters (2개 로드됨)

어댑터 문제
  ! broken.toml: adapter.name 이 없습니다

에이전트
  claude       트랜스크립트 O    실행 중 2   내장
  aider        트랜스크립트 O    실행 중 0   ← aider.toml
  mytool       트랜스크립트 X    실행 중 1   ← mytool.toml
```

---

## 성능 예산

주장이 아니라 숫자로 못 박는다.

| 항목 | 예산 | 현재 |
|---|---|---|
| 코어 바이너리 크기 | < 5 MB | Windows x64 407,552 bytes |
| 네이티브 앱 실행 파일 | — | 이전 v0.5.1: 240,128 bytes |
| NSIS 설치 파일 | < 15 MB | 이전 v0.5.1: 297,625 bytes |
| 외부 크레이트 | 0개 | 0개 |
| 네트워크 요청 | 0회 | 0회 |
| 첫 출력 | < 120 ms | Windows 최근 5회 82.5~140.1ms, 초과 사례 있음 |

첫 출력 측정은 반복 실행 결과이며 진정한 콜드 스타트 보증이 아닙니다. 최종 네이티브 설치본의 관측 3회에서 앱·코어·콘솔 호스트 working set 합은 약 34.7 MB였습니다. 수명이 짧은 tasklist가 표본 사이에 실행될 수 있어 최대 사용량을 보증하는 수치는 아닙니다. 고정 20세션·유휴 CPU·120ms 예산 충족은 아직 완료하지 않았고 예산도 완화하지 않았습니다. `cargo test`는 코어 58개 단위 + 1개 HTTP/SSE 통합 테스트, `desktop`은 4개 테스트를 실행합니다.

MIT
