# macOS 실기 테스트 인계

Mac 협업자는 이 문서 순서대로 빌드·실행한 뒤 아래 결과 양식을 GitHub Issue 또는 PR에 남겨 주세요. 목적은 실제 Mac에서 실행, 실제 Claude Code/Codex 수집, 설정 저장, 한/영 전환이 동작하는지 확인하는 것입니다. 이 문서 작성 시점에는 이번 실기 테스트를 수행하지 않았습니다.

현재 macOS 앱은 v0.3.0 개발 중이며 정식 Mac 릴리스가 아닙니다. 과거 CI 통과는 최신 소스의 실기 통과를 뜻하지 않습니다. 자세한 구현 범위는 [MACOS.md](MACOS.md), 기존 근거는 [VALIDATION.md](VALIDATION.md)를 참고하세요.

## 1. 준비와 소스 받기

- macOS 13 이상인 Apple Silicon 또는 Intel Mac. 우선 가진 기기에서 진행하고 다른 아키텍처·OS는 미실행으로 남깁니다.
- Git, Rust stable/Cargo, Xcode Command Line Tools. `xcode-select -p`, `rustc --version`, `cargo --version`, `xcrun swiftc --version`으로 확인합니다. Command Line Tools가 없으면 `xcode-select --install` 후 설치를 마칩니다. Rust가 없으면 [Rust 설치 안내](https://www.rust-lang.org/tools/install)를 따릅니다.
- 실제 수집 테스트에는 Claude Code 또는 Codex와 새 테스트 대화가 필요합니다. 둘 다 있으면 각각 확인하고, 없는 제품은 미실행으로 표시합니다. Orca는 세션 복귀 테스트에만 필요합니다.

새로 받는 경우:

```sh
git clone https://github.com/laekhole/what-am-i-doing.git
cd what-am-i-doing
git switch main
git rev-parse HEAD
```

기존 체크아웃이면 로컬 변경을 보존하고 `git status --short`를 확인한 뒤 깨끗한 `main`에서 `git pull --ff-only` 합니다. 인계 문서가 아직 원격에 없다면 유지관리자가 문서 변경을 반영한 뒤 받습니다. 테스트 결과에는 반드시 **실제로 실행한 커밋 SHA**를 적습니다.

## 2. 자동 검사와 패키징

저장소 루트의 같은 터미널에서 한 줄씩 실행합니다. 실패하면 해당 단계 출력과 종료 코드를 기록하고, 후속 단계를 통과로 처리하지 마세요.

```sh
sw_vers
uname -m
rustc --version
cargo --version
xcrun swiftc --version
git rev-parse HEAD
git status --short
cargo test --release --locked
MACOSX_DEPLOYMENT_TARGET=13.0 cargo test --manifest-path desktop/Cargo.toml --release --locked
bash desktop/package-macos.sh
```

실패 직후 `echo $?`로 종료 코드를 확인할 수 있습니다. 테스트의 passed/failed/ignored 수를 각각 남깁니다. `ignored`는 통과가 아닙니다.

기본 결과는 `desktop/target/macos-bundle/waid.app`, 아키텍처별 ZIP, `SHA256SUMS.txt`입니다. 기존 출력 폴더가 있으면 패키징이 거부됩니다. 기존 파일을 지우지 말고 `bash desktop/package-macos.sh desktop/target/macos-bundle-retest-01`처럼 **존재하지 않는 새 출력 경로**를 사용하고 이후 명령의 경로도 바꿉니다.

## 3. 실행과 우선 확인 (약 20–30분)

기존 waid가 있다면 Command+Q로 종료합니다. 개발 실행은 아래처럼 별도 설정 폴더를 사용합니다. 실제 에이전트 로그는 읽지만 기존 waid 설정과는 분리됩니다. 이후 재실행도 같은 터미널·같은 설정 폴더를 사용하세요.

```sh
export WAID_DATA_DIR="$(mktemp -d "$HOME/Library/Application Support/waid-mac-test.XXXXXX")"
printf '테스트 설정 폴더: %s\n' "$WAID_DATA_DIR"
desktop/target/macos-bundle/waid.app/Contents/MacOS/waid
```

앱을 종료하면 터미널로 돌아옵니다. 재실행할 때는 마지막 실행 명령만 반복합니다. 터미널을 새로 열었다면 출력해 둔 테스트 설정 폴더를 `WAID_DATA_DIR`로 다시 지정합니다. 이 폴더에는 대화 미리보기가 저장될 수 있으므로 원본을 보고서에 첨부하지 않습니다.

| 번호 | 할 일 | 기대 결과 |
|---|---|---|
| A | 첫 실행, 창 크기 변경, 목록 선택 | 충돌·멈춤 없이 창과 상세가 표시되고 갱신 중 선택이 유지됨. 로그가 없으면 빈 목록도 정상 |
| B | Claude Code/Codex에서 비밀 없는 새 대화로 요청하고 답변 기다리기 | 프로젝트·요청·답변·명시된 모델이 해당 대화와 일치. 약 2초 갱신 주기와 로그 지연을 감안해 실제 전환 시간을 기록 |
| C | 실행 중 새 요청 → 응답 완료 관찰 | 읽을 수 있는 이벤트에 따라 Working → Waiting을 확인. 시작 전 과거 활동이 Unknown인 것은 정상. 추측으로 상태를 채우지 않음 |
| D | English / 한국어 전환 | 컨트롤·메뉴·상세 문구가 바뀌고 대화 원문은 유지. 한글·긴 요청의 잘림과 복사도 확인 |
| E | 검색·에이전트/상태 필터, Pin, Hide, Show all | 조건에 맞는 목록, 고정·숨김·전체 보기 동작. 필터를 해제해 표시 여부를 재확인 |
| F | 목록 제외 → Show all → Restore, 다시 제외 후 같은 대화에 새 요청 | 수동 복원과 새 요청에 의한 자동 복원. 기존 대화를 열기만 하면 제외 유지 |
| G | Midnight 적용, 불투명도·항상 위·언어·고정 설정 후 Command+Q 및 재실행 | 같은 테스트 설정 폴더에서 설정 유지. 재실행 이전 활동은 Unknown부터 시작 |
| H | Command+F/P/Return, 복사, Command+M/W/H, Dock/메뉴 막대에서 복원 | 지원 단축키·한글/줄바꿈 복사·최소화·닫기·숨김·복원이 정상. 닫기는 종료가 아니며 Command+Q가 종료 |
| I | Command+Q 후 Activity Monitor 확인 | 해당 테스트 앱과 그 수집기 자식 프로세스가 남지 않음. 다른 에이전트 프로세스는 유지 |

누락되면 검색·필터를 비우고 Show all 및 필요 시 Include auxiliary를 켠 뒤, 앱 종료 후 같은 터미널에서 진단합니다:

```sh
desktop/target/macos-bundle/waid.app/Contents/MacOS/waid --waid-core doctor
```

기본 로그는 `~/.claude/projects`, `~/.codex/sessions` 등을 읽습니다. 사용자 지정 경로는 해당 환경변수가 앱에 전달되어야 합니다. 진단 결과를 공유할 때 사용자명·개인 경로를 가립니다. 전체 JSON/원본 대화 로그는 기본 제출물에 포함하지 않습니다.

## 4. 환경이 있으면 추가 확인

- **실제 세션 복귀:** Orca에서 살아 있는 로컬 Claude/Codex 대화를 선택해 Open session 또는 더블클릭. 정확한 대화가 화면에 선택되는지 확인합니다. 종료된 대화·불명확한 연결은 이유를 표시해야 합니다. 다른 대화가 열리면 우선순위 높은 실패로 기록합니다. ChatGPT 연결은 해당 행과 일치하는 `codex://threads/<session-id>` 복사 링크만 대상으로 하고, 호출 성공뿐 아니라 실제 대화 선택까지 확인합니다. 설치되지 않은 대상은 미실행입니다.
- **Finder 설치:** 개발 실행을 종료한 뒤 ZIP을 풀고 `waid.app`을 `~/Applications` 등으로 복사해 Finder에서 엽니다. 기존 앱을 덮어쓰지 마세요. Finder 실행은 위 터미널의 `WAID_DATA_DIR`를 상속하지 않으며 기본 waid 설정을 사용하므로, 기존 설정이 있으면 별도 macOS 테스트 계정에서 진행합니다. 기본 로그 수집·아이콘·Dock·메뉴를 확인합니다.
- **다운로드 설치:** GitHub Actions의 `macOS checks`에서 테스트할 커밋과 일치하는 성공한 실행의 `waid-macos-15`(Apple Silicon) 또는 `waid-macos-15-intel`(Intel) 아티팩트를 사용합니다. 아티팩트 보관 기간은 7일이므로 없으면 해당 커밋으로 워크플로우를 다시 실행합니다. 내려받은 아티팩트를 푼 뒤 내부 ZIP과 체크섬이 있는 폴더에서 `shasum -a 256 -c SHA256SUMS.txt`를 실행합니다. 앱은 개발용 ad-hoc 서명이며 Developer ID 서명·공증은 없습니다. 차단되면 문구와 실행 경로를 기록하고 설치 항목을 차단으로 남깁니다. 보안 설정 해제나 quarantine 제거는 테스트 절차에 포함하지 않습니다. 로컬 빌드 실행 성공과 다운로드 설치 성공을 구분합니다.
- **확장 확인:** 외부 모니터 연결/해제, 좁은 창·긴 대화, VoiceOver, 템플릿 Preview 취소·잘못된 JSON, 30분 이상 사용, 절전 복귀. 실행 시간과 재현 조건을 적습니다.

현재 Mac은 Windows와 다른 목록/상세 UI를 사용합니다. Windows 트레이 답변 알림·2열 카드·창 연결, Mac Terminal/iTerm/Claude 데스크톱의 정확한 대화 복귀, SSH 복귀는 이번 Mac 통과 기준이 아닙니다. 컨텍스트의 목록 배지와 상세 정보 일부도 아직 Windows와 같지 않습니다. Claude 컨텍스트 비율처럼 원본에 근거가 없는 값은 Unknown일 수 있습니다. 미구현 사항과 새 회귀를 구분하세요.

## 5. 결과 전달 양식

아래를 GitHub Issue에 복사하거나 문서 PR로 제출합니다. 코드 수정이 필요하면 `main`에 바로 올리지 말고 별도 브랜치와 PR로 변경·재검증 결과를 남깁니다. 에이전트를 사용한다면 [AGENTS.md](AGENTS.md)의 작업 기록 규칙도 따릅니다.

```text
제목: [macOS test] OS / Apple Silicon 또는 Intel / 커밋 앞 7자리
테스트 날짜·시간대:
macOS 버전 / 칩 / 메모리:
실제 git SHA / 로컬 변경 여부:
Rust / Cargo / Swift 버전:
Claude Code / Codex / Orca / ChatGPT 버전 (없는 것은 없음):
실행 경로: 로컬 빌드 / Finder / 다운로드 아티팩트 (CI 실행 번호)
설정: 별도 WAID_DATA_DIR / 새 macOS 계정 / 기존 기본 설정
자동 검사: core passed/failed/ignored, desktop passed/failed/ignored
패키징: 통과 / 실패 (종료 코드)
A~I: 각 번호별 통과 / 실패 / 차단 / 미실행 및 짧은 근거
세션 복귀 / Finder 설치 / 다운로드 설치 / 확장 확인:

문제별 기록:
- 재현 단계:
- 기대 결과 / 실제 결과:
- 재현 빈도 / 상태 전환에 걸린 시간:
- 오류 메시지·종료 코드 / 개인정보를 가린 화면:
- 실행을 막는 문제인지 / 사용은 가능한 문제인지:

최종 판단: 이 환경에서 기본 사용 가능 / 수정 후 재검증 필요
미실행·차단 항목과 다음 확인 사항:
```

자동 검사·빌드 실패, 충돌, 잘못된 대화로 복귀, 설정 손실은 먼저 보고합니다. 한 기기 통과만으로 다른 아키텍처나 macOS 13, 다운로드 설치까지 통과했다고 기록하지 않습니다.
