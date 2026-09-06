# CLI 사용과 확장

네이티브 앱은 TEMPLATES.md의 JSON 템플릿을 사용합니다.

## CLI 커스터마이징

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

## 지원 에이전트

프로세스 감지: Claude Code, Codex, Gemini CLI, opencode, Aider, Cursor CLI, Copilot CLI, Goose

트랜스크립트 판독(= `llm`/`task`/마지막 `status`): Claude Code, Codex, Copilot CLI

나머지는 프로세스만 감지하며 상태는 `unknown`이다. 명시적인 실행 옵션이 없으면 모델·작업도 미확인이다. Windows에서는 알려진 CLI와 Node/Bun/Deno/Python의 인자만 제한된 읽기 권한으로 조회한다. 조회 거부 시 실행 파일 이름으로 폴백한다. 별도 PowerShell 폴링·관리자 권한·외부 라이브러리를 요구하지 않는다.

패키지 표지는 실행 스크립트 경로의 구성 요소에서만 비교한다. `node server.js @github/copilot`, `rg @google/gemini-cli`, eval 코드 안의 언급은 세션이 아니다. npx/pnpm/uv/uvx 같은 설치·실행 관리자는 제외하고 실제 자식 CLI를 감지한다. 임의의 런처 옵션·Cursor IDE·VS Code 확장·WSL 내부 프로세스는 지원 범위에 포함하지 않는다.

Copilot은 [공식 저장 경로](https://docs.github.com/en/copilot/concepts/agents/copilot-cli/chronicle)의 `~/.copilot/session-state/<id>/events.jsonl`을 읽는다. [COPILOT_HOME](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference)을 설정하면 그 아래 session-state도 읽는다. 다른 --config-dir 경로는 사용자 어댑터로 설정한다. 인증 설정 파일이나 SQLite DB는 읽지 않는다.

[공식 이벤트 스키마](https://github.com/github/copilot-sdk/blob/main/nodejs/src/generated/session-events.ts)를 기준으로 session.start의 프로젝트·모델·세션 ID, user.message의 현재 요청, session.model_change의 최신 모델을 사용한다. agentId가 붙은 보조 이벤트는 부모 작업·상태를 덮어쓰지 않는다. assistant.turn_end는 모델 호출 종료이므로 작업 중 근거로만 쓰고, session.idle은 내 차례, abort/정상 shutdown은 중단·유휴, session.error/오류 shutdown은 오류다. 재요청은 같은 세션 ID를 유지하며 요청 식별자를 바꾼다. 비용·모델 변경만으로 재개하지 않는다. 버전에 따라 idle 이벤트가 디스크에 남지 않으면 기다리는 상태를 확정하지 않고 미확인으로 둔다.

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
markers = ["@vendor/mytool"]         # 실행 스크립트의 패키지 경로 (선택)

[transcript]
dir  = "~/.mytool/sessions"          # 선택. 있으면 llm/task/status 가 열린다
dirs = ["/opt/agent/logs"]           # 루트가 여럿이면 목록으로
```

정규식이 아니라 실행 파일 이름 목록이다. 커맨드라인 전체를 정규식으로 훑으면 `vim claude.md`가 Claude Code로 잡힌다. 우리가 묻는 것은 "이 프로세스의 실행 파일이 에이전트인가"뿐이다.

`[transcript] dir` 을 주면 그 경로의 JSONL을 훑어 지원하는 이벤트에서 모델·요청·cwd·상태를 읽는다. 새로운 로그 형식은 리더 수정이 필요할 수 있다.

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
