# Windows 검증 기록 — 2026-09-06

이번 변경의 기록입니다. DECISIONS.md에 남아 있는 이전 설치본·UI 측정과 구분합니다. **전체 완료 기준은 아직 미충족**이며 아래 직접 화면 검증이 남았습니다.

## 단일 EXE 배포 — 2026-09-08

Windows MSVC release에서 코어 단위 92개, CLI 통합 5개, 앱 단위 28개, 단독 EXE 통합 1개로 총 126개 통과. 기존 실환경 연결 검사 3개는 ignored 상태를 유지했습니다. 기존 JSON 도우미의 dead_code 경고 2개가 남습니다.

`desktop/tests/standalone.rs`는 앱 EXE 하나만 한글·공백 경로에 복사하고 이름을 바꾼 뒤, 내부 수집 모드가 지정한 로그를 JSON으로 반환하는지와 내장 폰트 라이선스를 확인합니다. 실행 폴더에 코어나 기타 파일을 추출하지 않는 것도 검사합니다. 앱의 소유 코어 종료·다른 코어 생존 검사는 같은 EXE의 내부 모드로 통과했습니다.

```powershell
cargo test --release --locked --offline
cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/standalone --release --locked --offline
./desktop/package.ps1 -Tag v0.2.0 -BinaryDirectory desktop/target/standalone/release -OutputDirectory desktop/target/standalone/release/bundle
```

빌드 셸에는 Windows SDK의 `rc.exe`가 필요합니다. 실행 중인 기존 앱의 EXE가 잠겨 있어 위 별도 출력 경로를 사용했습니다. 단일 배포 EXE는 4,557,824 bytes이며 체크섬과 재패키징 덮어쓰기 거부를 확인했습니다. release/check 워크플로는 actionlint, 빌드·패키징 스크립트는 PowerShell 구문 검사를 통과했습니다. GitHub 게시·서명과 선택적 NSIS 설치본 실행, 새 폰트 라이선스 메뉴의 직접 클릭은 수행하지 않았습니다. 아래 과거 기록의 두 EXE·ZIP 안내는 이전 빌드에 해당합니다.

## 환경과 재실행

Windows x64, Rust 1.98.1 GNU 타깃. MSVC 빌드 도구가 없어 저장소의 무시된 `.tools`에 Rust·MinGW binutils를 준비했습니다. 시스템 설치·전역 환경을 바꾸지 않았습니다. 코어 외부 크레이트는 0개이며 앱은 serde_json과 windows-sys를 사용합니다.

이 PC에서 재검사:

```powershell
$env:CARGO_HOME = Join-Path $PWD '.tools/cargo'
$env:RUSTUP_HOME = Join-Path $PWD '.tools/rustup'
$env:PATH = "$(Join-Path $PWD '.tools/binutils/mingw64/bin');C:/Program Files/Git/mingw64/bin;$env:PATH"
& ./.tools/cargo/bin/cargo.exe test --release --locked --target x86_64-pc-windows-gnu
$env:WAID_CORE_PATH = Join-Path $PWD 'target/x86_64-pc-windows-gnu/release/waid.exe'
& ./.tools/cargo/bin/cargo.exe test --manifest-path desktop/Cargo.toml --release --locked --offline --target x86_64-pc-windows-gnu
& ./.tools/cargo/bin/cargo.exe build --manifest-path desktop/Cargo.toml --release --locked --offline --target x86_64-pc-windows-gnu
```

최종 release 검사는 코어 단위 **64개**, CLI 통합 **3개**, 앱 **9개**, 합계 **76개** 통과입니다. 사용하지 않는 기존 JSON 도우미의 dead_code 경고가 남습니다. MSVC·Linux·macOS CI는 이번 환경에서 실행하지 않았습니다.

메모형 변경 당시 코어 498,688 bytes, 앱 488,960 bytes였다. 아래 알림 카드 변경으로 앱을 다시 빌드했다. 독립 코어와 앱 옆 코어의 SHA-256이 일치합니다. 앱의 PE import는 Windows 시스템 DLL만 포함하며 별도 GNU 런타임 DLL은 요구하지 않습니다. ZIP·설치 파일은 갱신하지 않았습니다.

## 실제 데이터 대조

읽기만 수행했으며 사용자 요청·로그·에이전트 프로세스를 변경하지 않았습니다. 원문 프롬프트나 전체 스냅샷을 문서에 복사하지 않았습니다.

2026-09-06 06:46 UTC 스냅샷에서 고유 세션 30개를 원본과 연결했습니다.

| 루트 | 최근 파일 갱신 후보 | 스냅샷과 연결된 로그 |
|---|---:|---:|
| 기본 .claude/projects | 11 | 4 |
| 기본 .codex/sessions | 18 | 18 |
| CODEX_HOME=.codex-pro 의 sessions | 8 | 8 |

30개 모두 모델·프로젝트 폴더·보조 세션 여부가 원본 메타데이터와 일치했습니다. 읽기 범위에서 최근 요청을 독립 추출할 수 있고 task 출처가 최근 요청인 12개는 요청 본문도 일치했습니다. timestamp가 있는 마지막 이벤트를 찾은 29개는 상태·시각이 일치했습니다. 나머지 항목을 검증한 것으로 확대 해석하지 않습니다.

처음 대조에서 Claude sidechain이 부모 sessionId를 공유하여 부모를 덮어쓰는 결함을 발견했습니다. 보조 로그는 경로로 식별하도록 수정했고 재대조에서 중복·보조 여부 불일치가 사라졌습니다. 최근에 부가정보만 갱신된 과거 Claude 로그는 마지막 이벤트 시각에 따라 제외했습니다.

실제 로그의 첫 요청과 이후 요청이 다른 사례를 확인해 대표 요청과 현재 요청을 분리했습니다. 모델·폴더는 도구 출력 안의 중첩 문자열 대신 세션 이벤트의 메타데이터에서 읽습니다.

이 대조는 원본의 **마지막 기록**과의 일치 검사입니다. 실제 사용자의 승인 대기·중단·오류·재개를 모두 직접 유발하거나 창에서 관찰한 검증은 아닙니다.

## 재현 가능한 자동 검사

`src/tests.rs`, `tests/cli.rs`, `desktop/src/main.rs`, `desktop/src/native.rs`, `desktop/src/ui.rs`에 있습니다. 임시 파일·한글/공백 경로·자식 프로세스 환경을 사용하며 원본 로그는 수정하지 않습니다.

- CODEX_HOME 발견, 모델 변경, waiting → working → waiting → idle → error → working, 불완전한 줄 추가 후 완성, HTTP/SSE 갱신.
- 긴 주입 문맥 이후 실제 요청, 최신 요청/대표 요청 구분, 중첩 도구 출력의 가짜 모델·경로 무시, sidechain ID 충돌.
- timestamp 시차·윤년, metadata-only 갱신, 날짜 없는 이벤트 뒤 부가정보 추가, 파일 잘림·더 큰 파일로 교체, 캐시 무효화.
- 프로세스 미발견은 완료가 아님, 턴 종료는 waiting, 오래된 working은 unknown, 주입된 환경 문맥만으로 새 작업을 만들지 않음.
- 실제 Windows 컨트롤 생성, 검색·고정·숨김·선택·스크롤 유지, 수집 오류에서 마지막 목록 유지와 복구.
- 템플릿 가져오기/내보내기 처리, 임시 미리보기와 저장 적용의 분리, 적용 후 재읽기, 기본값 복구, 잘못된 JSON·낮은 대비 거부, 손상된 설정으로도 시작.
- 소유 코어 종료를 OS 프로세스 핸들로 확인하고 별도로 시작한 코어가 살아 있음을 확인.

템플릿 버튼의 처리 함수와 실제 파일 저장은 검사했지만 Windows 파일 선택 대화상자를 사람이 클릭한 검증은 아닙니다.

## 자원과 DPI 관련 회귀

실제 30개 세션을 1초 간격으로 120.5초 수집했습니다. 117개 스냅샷, 중복 0회, stderr 0 bytes. working/waiting/idle/unknown 상태가 관측됐습니다.

| 코어 측정 | 결과 |
|---|---|
| Working set | 처음 12.04 MiB, 마지막 12.00 MiB, 최대 12.28 MiB |
| Private bytes | 처음 3.67 MiB, 마지막 3.77 MiB, 최대 4.46 MiB |
| 핸들 | 처음·마지막·최대 151 |
| 누적 CPU | 마지막 3.171875초 / 약 120초 경과 |

이는 **2분 표본**이며 몇 시간 이상의 안정성·앱 전체 메모리·CPU 예산 통과를 보장하지 않습니다. 로컬 집계만 `.tools/collector-soak.json`에 남겼습니다. 실제 데이터 대조 스크립트·집계는 `.tools/session-audit.cjs`, `.tools/session-audit.json`에 있으며 Git에는 포함하지 않습니다.

기존 ListBox의 255픽셀 제한 때문에 306픽셀 행 설정이 LB_ERR로 실패하는 것을 재현했습니다. ListView와 소유 이미지 목록을 통한 행 측정으로 바꾸고 153·306·600픽셀 이상 행이 실제 컨트롤에 설정됨을 검사했습니다. 큰 글자·200% 배율을 위한 구조 검사이며 실제 모니터 배율 검증은 아닙니다.

네이티브 템플릿 400회 교체와 메시지 처리 후 GDI 핸들 수는 100회마다 55 / 61 / 61 / 61이었습니다. 초기 캐시 이후 증가가 멈췄습니다. 교체할 글꼴은 컨트롤에 새 글꼴을 연결한 뒤 반환하고 이미지 목록은 앱이 소유·해제합니다.

## 후속 직접 화면 확인 — 2026-09-06

초기 `orca` 미발견은 설치 부재로 확정할 문제가 아니었습니다. 제한된 셸에서는 명령을 찾지 못하고 설치 폴더도 접근 거부였지만, 승인된 `require_escalated` 실행에서 **같은 `orca` 명령**을 찾았습니다. 실행 중인 설치본은 `%LOCALAPPDATA%\Programs\orca`, 앱 버전은 1.4.197입니다. 다른 Orca 빌드로 전환하지 않았습니다.

`orca skills get computer-use`, `orca status --json`, `orca computer capabilities --json`으로 공식 가이드와 연결을 확인한 뒤 실제 창을 캡처·조작했습니다. 캡처와 클릭은 가능하지만 provider의 windows.focus는 false입니다. permissions 조회는 Windows의 accessibility/screenshots를 unsupported로 반환했고, 추가 설정을 여는 안내는 없었습니다.

### 직접 확인한 것

- 별도 `.tools/ui-review-data` 설정으로 샘플 앱 실행. 1140×820 실제 창에서 작업·프로젝트·상태·모델·상세 영역 표시를 확인했습니다.
- 템플릿 편집 → 어두운 기본 복제 → 미리보기 버튼을 클릭하고 배경·색상·아이콘·필드 순서가 바뀐 실제 화면을 확인했습니다.
- 적용 버튼 뒤 테스트 설정의 template.name이 Midnight로 저장됨을 확인했습니다. 기본값 복구 뒤 Daylight로 복원됨도 확인했습니다.
- 샘플 창 닫기 후 프로세스가 사라졌습니다. 이 샘플 모드는 수집 코어를 띄우지 않습니다.
- 새 빌드를 `.tools/ui-review-live` 설정으로 실제 수집 모드에서 열어, 원본에서 수집한 세션 목록·모델·현재/대표 요청·마지막 기록 근거가 화면에 표시됨을 확인했습니다. 이 시점에는 28개 중 보조를 제외한 16개가 표시됐습니다.

도구의 클릭 결과는 synthetic_input/unverified입니다. 성공 플래그만 믿지 않고 변경된 트리, 실제 스크린샷 및 저장 파일을 대조한 위 항목만 확인한 것으로 기록합니다. 실제 사용자 데이터는 수정하지 않았습니다.

### 화면에서 발견한 수정

기존 Windows EDIT에 LF만 전달하여 템플릿 JSON이 한 문단처럼 줄바꿈돼 보였습니다. 표시 문자열을 CRLF로 정규화하고, 실제 컨트롤의 두 번째 줄 시작 위치를 확인하는 회귀 검사를 추가했습니다. 새 빌드의 실제 편집 화면에서 줄바꿈·들여쓰기가 구분됨을 확인했습니다. 기본 글꼴 지정은 Malgun Gothic으로 바꿨습니다.

변경 후 앱 테스트 6개 재통과, release 재빌드 성공. 앱 467,456 bytes. 코어는 변경하지 않았습니다.

### 아직 남은 직접 검사

키보드 입력은 `window_not_focused`입니다. 안내에 따라 `--restore-window`로 한 번 재시도했지만 다음 오류가 남았습니다.

> keyboard input requires the target window to be focused; restoreWindow was requested but the target window is still not focused; bring it forward manually or check desktop permissions

[computer-use SKILL.md](C:/Users/PC/.agents/skills/computer-use/SKILL.md)를 통해 읽은 `orca skills get computer-use` 가이드는 restore 요청 후에도 실패하면 재시도를 멈추고 직접 포커스를 주거나 권한을 확인하도록 지시합니다. 캡처·버튼 클릭과 키보드 입력 가능 여부를 구분합니다. 검증용 실제 수집 창은 일반 목록 상태로 열어 두었습니다. 사용자가 검색 입력란을 직접 클릭해 포커스를 준 뒤 키보드 검사를 이어갈 수 있습니다.

미완료 항목:

1. 직접 키보드 입력으로 검색·수정·단축키·탭 이동 검증.
2. 실제 파일 선택 대화상자를 통한 가져오기·내보내기, 수정본 적용·재실행 확인.
3. 100/150/200% 및 혼합 DPI 모니터 이동, 작은 창의 실제 가독성·스크린 리더 확인.
4. 실제 사용자 승인 대기·중단·오류·재개를 자연스러운 사용 중 원본 창과 대조.
5. 몇 시간 이상 다중 세션 사용, 실제 닫기 버튼으로 소유 코어만 종료되는지 직접 확인. 프로세스 소유 정리 자동 검사는 이미 통과했습니다.

개별 로그 읽기 오류 표시, 로그 범위 밖 정보, 동일 앞부분·메타데이터를 보존한 비정상 중간 덮어쓰기, Windows 프로세스와 로그의 정확한 연결은 알려진 제한입니다. 전체 완료 기준은 아직 충족하지 않았습니다.


## 메모형 UI와 기존 세션 수집 — 최신 변경

기본 360×420, 최소 280×220 논리 픽셀의 메모형 UI로 변경했다. 상세·검색·템플릿은 필요할 때만 800×640으로 펼친다. 기본·어두운 템플릿 모두 세 줄 표시를 지원한다. 전체 보기와 수동 종결·되살리기, 새 요청에 따른 자동 복귀를 추가했다.

### 실제 Windows 창에서 확인한 흐름

별도 .tools/sticky-review의 어댑터·로그·설정만 사용했다. 테스트용 Codex 형식 JSONL을 실제 코어가 읽도록 실행했으며, 아래는 실제 사용자 대화의 상태 전환을 유발한 검증이 아니다.

1. 360×420 실제 창에서 테스트 3세션의 작업 / 프로젝트·상태 / 에이전트·모델을 확인했다.
2. 추가 기능 → 종결 클릭 후 3개에서 2개로 감소하고 설정에 종결 사본 1개가 저장됐다.
3. 전체 보기에서 3개로 돌아오며 종결한 세션은 목록 아래에 종결 표시로 나타났다. 기본 보기로 돌아가면 다시 2개였다.
4. 테스트 로그에 모델 메타데이터만 추가해도 종결 사본은 1개를 유지했다. 앱을 재실행한 실제 화면도 2개였다.
5. 같은 세션에 같은 문장이지만 새로운 시각의 사용자 요청을 추가하자 종결 사본이 0개로 줄고 실제 화면이 3개로 복귀했다. 목록 복귀 안내도 확인했다.
6. --history 적용 후 2020년 timestamp와 파일 수정 시각을 가진 별도 로그도 실제 창에 4번째 세션으로 표시됐다. 상태는 중단 / 유휴이며 자동 종결되지 않았다.
7. 이 검증 창의 닫기 버튼을 클릭한 뒤 앱 PID 22684와 미리 기록한 자식 코어 PID 9468이 모두 사라졌다. 클릭 도구는 후속 app_not_found를 반환했으므로 프로세스 소멸을 별도로 확인했다. 다른 코어를 유지하는 소유권 검증은 자동 테스트 결과와 구분한다.

샘플 캡처는 무시된 .tools/sticky-review/revived.png와 history.png에 있다. 실제 사용자 정보는 샘플에 넣지 않았다. 최신 앱을 일반 설정·실제 수집 모드로 실행했다. 해당 실제 데이터 화면 캡처는 자동 승인 검토가 프로젝트명·작업 등 세션 메타데이터를 computer-use 서비스에 전달할 권한이 없다는 이유로 거절했다. 이 캡처는 수행하지 않았으며 테스트 창 캡처로 대체 완료했다고 간주하지 않는다. 사용자 승인 없이 다른 캡처 경로로 우회하지 않는다.

### 자동 검사와 수집 범위 확인

새 사용자 요청 식별자는 영속 FNV-1a r1 형식으로 만들고, 같은 문장의 새 시각 / 모델만 바뀐 기록을 구별하는 회귀 테스트를 추가했다. 종결 저장·재읽기·전체 보기·빠른 턴의 복귀·식별자 부재도 검사한다. 네이티브 컨트롤에서 작은 기본 창, 숨겨진 상세 영역, 종결·전체 보기·복귀의 항목 수를 검사했다.

CLI 통합 테스트는 파일 시각과 이벤트 시각이 모두 오래된 유휴 로그, 파일만 새로 갱신된 오래된 턴 종료 로그가 --history에서 수집되고 기본 CLI에서는 제외됨을 검증한다. 두 세션 모두 완료로 바뀌지 않는다.

실제 원본 로그를 읽은 동일 실행의 수집 표본: 기존 최근 24시간 방식 24세션 / 179ms / 44,817 bytes, 날짜 제한 해제 461세션 / 1,443ms / 1,022,489 bytes. 437개가 24시간보다 오래됐고 상태는 waiting 368, working 1, idle 6, unknown 86이었다. 이 숫자는 당시 표본이며 전체 원본의 모든 필드를 다시 대조했다는 의미가 아니다. 읽기 요약 캐시는 최대 4,096개, 앱 스냅샷 상한은 16 MiB다. 대량 과거 로그는 첫 수집이 더 오래 걸리며 개별 파일의 앞/뒤 읽기 제한은 유지한다.

날짜 제한 해제 상태로 추가 30초 관찰한 결과 27개 스냅샷에서 각 461세션, 중복 0개, stderr 0 bytes였다. 한 줄 JSON 최대 크기는 910,937 bytes였다. 이 짧은 표본은 장시간 자원 안정성 검증을 대신하지 않는다.

직접 키보드·파일 대화상자·혼합 DPI·몇 시간 실사용 및 실제 사용자 대화의 전체 상태 전환 대조는 앞서 기록한 대로 남아 있다. 전체 제품 목표는 완료 처리하지 않았다.


## 알림 카드와 창 표시 설정 — 2026-09-06

이번 사용자 요청의 변경: 둥근 세션 카드·하네스 로고, 최대화 없는 제목 표시줄, 항상 위, 투명도 0~60% 및 설정 저장이다. 이전 독립 앱 구현과 함께 로컬 커밋 대상으로 검토했다. 배포 스크립트·설치 파일·CI의 기존 작업 중 변경은 이 커밋에 포함하지 않는다.

### 자동 검사

코어 64개 + CLI 통합 3개 + 앱 9개가 통과했다. Windows 앱 검사는 일반 데스크톱 권한으로 실행했다. 제한된 환경에서는 SetWindowPos가 성공을 반환해도 TOPMOST 속성이 반영되지 않는 경우가 있어, 해당 결과를 정상 동작의 증거로 쓰지 않았다.

- 공식 로고 두 개를 실제 Windows 아이콘 핸들로 디코딩한다. 모델이나 표시명을 바꿔도 agent.name을 하네스 ID로 유지한다.
- 최대화 스타일·기본 제목 표시줄·시스템 메뉴 최대화가 없고, 최대화 명령을 보내도 최대화되지 않는다.
- 항상 위를 켜고 끌 때 실제 WS_EX_TOPMOST를 대조한다. 창 생성 시 저장된 TOPMOST와 LAYERED 스타일을 반영하고, 이후 확장 스타일 변경이 TOPMOST를 덮어쓰지 않게 했다.
- 투명도 60%에서 Windows alpha가 102/255인지 확인한다. 설정 저장 후 새 UI 스레드에서 재시작을 3회 검사하며 항상 위·alpha 복원과 최소화·복귀를 확인한다. 범위를 벗어난 저장 값은 불투명도 100%로 복구한다.
- 슬라이더 PREPAINT의 빈 rc를 재현하고 실제 컨트롤 크기로 그린 메모리 DC의 픽셀을 확인한다. 이 수정 전 실제 화면에서는 손잡이가 잘리고 채널이 그려지지 않았다.
- 기존 종결·전체 보기·자동 복귀, 검색·고정·숨김·스크롤·선택, 템플릿 적용·복구와 반복 GDI 자원 검사도 재통과했다.

### 실제 샘플 창 확인

실제 대화를 포함하지 않은 --demo와 별도 .tools/rounded-review, .tools/rounded-final 설정을 사용했다. 360×420 및 사용자가 늘린 497×420 창에서 둥근 카드와 Claude·Codex 마크, 한글 작업·상태·모델을 확인했다. 기본 제목 표시줄이 중복으로 보이던 초기 구현을 고쳐 최소화·닫기만 남겼다. 사용자 클릭 후 도구가 샘플 창을 다시 조작할 수 있었다.

항상 위 버튼의 체크 표시 변경과 그 시점의 설정 저장을 확인했다. 투명도 버튼을 열어 수정된 슬라이더의 전체 손잡이·채널이 표시됨을 확인했고, 실제 드래그 뒤 표시 값이 0%에서 31%로 바뀌었다. 반투명 창 뒤의 실제 작업 내용을 캡처하지 않도록 해당 조작에는 --no-screenshot을 사용했다. 투명도 저장·재시작·OS alpha는 위 자동 검사의 범위와 구분한다.

일부 캡처는 도구가 다른 창에 가려진 영역을 반환했다. 가려진 이미지와 상태가 변하지 않은 합성 클릭은 성공 증거에 포함하지 않는다. 샘플 카드 캡처는 .tools/rounded-final/cards.png에 있으며 원본 대화나 실제 세션 캡처는 커밋에 포함하지 않는다.

이번 변경 범위의 회귀 검사를 통과했으며, 혼합 DPI 모니터·스크린 리더·몇 시간 실사용 등 앞서 기록한 제품 전체 검증 한계는 남아 있다.

## 2026-09-06 — 추가 코딩 에이전트 감지

- 코어 69개, CLI 통합 4개, 새 코어를 포함한 데스크톱 9개 통과(총 82개). Windows 자기 프로세스의 실제 실행 인자를 std::env::args와 대조했다. 한글·공백·따옴표, 없는 PID, Node/Python 진입점, 패키지 문자열 오탐을 검사했다.
- Copilot의 기본/설정 경로, 부가 JSONL 제외, 프로젝트·모델·대표/최근 요청, 모델 변경, 자식 이벤트 제외, 중단·오류·재개, 불완전한 append와 파일 교체를 합성 로그로 검사했다.
- 실제 Windows Node/Python으로 만든 대기 스크립트와 직접 실행 stub 8개를 실행했다. Gemini/Copilot/OpenCode의 Node 진입점, Aider 모듈, Cursor/Goose/OpenCode 실행 이름 7개를 감지하고 일반 Node 스크립트의 패키지 언급 1개는 제외했다. 30회 스냅샷 모두 일치, 총 1,893 ms(프로세스 시작 비용 포함). 원본 에이전트는 실행하지 않았고 fixture 자식만 종료했다. 장시간 메모리 검증을 대신하는 수치는 아니다.
- 이 PC의 .gemini, OpenCode, Goose 기본 데이터 루트와 Copilot session-state가 없었다. 실제 Copilot 대화·다른 제품의 실제 작업 상태 전환은 미검증이며 fixture 검증과 구분한다. 기존 실제 Claude/Codex 로그 수집은 새 코어에서도 각각 170/294개가 나오는 것을 집계 확인했다. 이 숫자는 실행 시점의 결과이며 대화 내용은 기록하지 않았다.
- 이번 변경은 코어 감지에 한정하며 새 제품의 실제 화면 검증을 수행했다고 주장하지 않는다. 기존 네이티브 화면·템플릿 검증 기록은 위 항목을 따른다.

OS 감지 회귀를 재현하려면 Windows에 Rust GNU 도구 체인, Node, Python이 있어야 한다. 아래는 실제 에이전트 대신 tests의 대기 stub만 실행한다. 결과에는 fixture PID·제품·상태만 출력한다. 코어를 먼저 빌드하고 저장소 루트에서 실행한다:

```powershell
New-Item -ItemType Directory -Force .tools/agent-detection-check | Out-Null
rustc --edition 2021 --target x86_64-pc-windows-gnu tests/fixtures/process_stub.rs -o .tools/agent-detection-check/helper.exe
node tests/windows-processes.cjs
```


## 2026-09-07 — JSONL 이 아닌 소스 (JSON 파일, SQLite)

- 코어 75개, CLI 통합 4개 통과(총 79개). 새 검사: JSON 파일 한 개가 세션 하나가 되는지(대표/최근 요청, 모델, cwd, 밀리초 시각, 고정 파일명일 때 상위 폴더가 식별자), 사용자 요청이 없는 JSON 은 세션이 되지 않는지, `file://` URI 경로 해석, SQLite 행의 열 이름 매핑과 빈 대화 제외, 스키마가 다를 때 그 질의만 실패하는지.
- 이 PC 의 `winsqlite3.dll`(1.1 MB, Win11)로 실제 열기·질의·행 읽기를 확인했다. 테스트는 같은 엔진으로 임시 DB 를 만들어 돌리므로 픽스처 바이너리를 저장소에 넣지 않는다.
- **실제 데이터 대조는 Cursor 하나뿐이다.** 이 PC 의 `AppData\Roaming\Cursor\User\globalStorage\state.vscdb`(6.7 MB, composerHeaders 21행, cursorDiskKV 853행)에서 `waid --json --history` 로 7개 세션이 목록에 떴다. 요청 텍스트·갱신 시각·보조 여부를 확인했고 상태는 전부 미확인으로 표시됐다. 질의 자체는 17행 2.1 ms 였다.
- 첫 질의는 `h.isBestOfNSubcomposer` 열이 없어 실패했고 doctor 의 "소스 문제"에서 발견해 고쳤다. 실패한 소스가 앱을 멈추지 않는다는 것도 이때 확인했다.
- Cline·Roo Code·VS Code Chat·Continue·Gemini CLI·opencode 는 **경로만 등록했고 실제 데이터로 대조하지 않았다.** 이 PC 에는 해당 대화 기록이 없다(`~/.continue/sessions/sessions.json` 은 빈 배열, VS Code 의 `chat.ChatSessionStore.index` 도 비어 있음). 형식이 다르면 목록에 안 뜰 수 있으며, 이를 지원 완료로 간주하지 않는다.
- macOS·Linux 의 `libsqlite3` 경로와 WAL 로 잠긴 DB 의 사본 폴백은 이 환경에서 재현하지 못했다. 편집기 실행 중 읽기는 미검증이다.
- 이번 변경은 코어 수집에 한정한다. 새 에이전트 카드의 실제 화면 확인은 하지 않았다.

## 2026-09-08 — 카드에서 기존 Orca 세션 탭으로 이동

- 코어 75개·CLI 통합 4개·데스크톱 11개 테스트 통과. 원본 세션 ID의 JSON/UI 전달, 같은 폴더의 다른 세션, 탭 재사용, 종료·원격·중복 연결 거부를 검사했다. 카드 클릭 알림은 상세 보기로 폴백하고 빈 영역 클릭은 무시하며, 비동기 연결 실패 시 선택이 유지되는 것도 네이티브 테스트로 확인했다.
- 로컬 Orca 1.4.197에서 현재 Codex 세션 ID와 `ORCA_TERMINAL_HANDLE`을 대조하고 공개 CLI의 기존 탭 전환 성공 응답을 확인했다. 재현: Orca의 Codex 터미널에서 `cargo test --manifest-path desktop/Cargo.toml --release --locked current_orca_session_switches -- --ignored`. 기본 테스트에서는 실제 탭을 전환하지 않는다.
- 릴리스 실행 파일을 빌드했다. Orca 훅 기록 v2를 읽기 전용으로 사용하며 현재 실행 권한 기록·pane·세션을 대조한다. Orca 기록 형식 변경은 상세 보기로 폴백한다. PowerShell·Windows Terminal 개별 탭, ChatGPT·Claude 일반 채팅 수집·대화 이동은 미지원이다. 이번 실환경 전환 검증은 Codex에 한정하며 Claude Code 실환경 전환은 미검증이다.

## 2026-09-08 — 1열 대화 카드 시안 반영

- 기본 1열 목록을 네이티브 버튼·읽기 전용 텍스트 컨트롤로 구성했다. 카드 안에서 요청·마지막 답변·첫 요청 요약을 펼치고 접으며, 첫 요청 복사와 기존 세션 열기를 제공한다. 기본 글자 14px, 프로젝트 17px, 날짜·상태 배지, 스크롤을 적용했다. 기존 2열·검색·종결·템플릿·창 설정은 유지한다.
- 코어 기존 75개 + 새 답변 회귀 1개, CLI 4개, 데스크톱 11개 검사 통과. 새 검사는 Claude/Codex/Copilot의 텍스트 답변, 사고·도구 블록 제외, 새 요청 후 이전 답변 제거를 확인한다. 네이티브 검사에는 카드 교체·높이·본문 갱신·스크롤 범위·접기를 추가했다. 실제 Orca 탭 전환 검사는 기존처럼 1개 ignored다.
- JSON 스냅샷에 선택적 last_answer를 추가했다. 기존 읽기 범위 안에서 최대 4,000자를 수집하고 종결 저장은 1,000자로 제한한다. SQLite 답변 수집, 메시지별 시각, 직접 전송·첨부는 이번 변경에 포함하지 않는다.
- 별도 설정의 --demo 창에서 카드 펼침을 실제 화면과 접근성 트리로 확인했다. 사용자 대화는 시각 검증에 사용하지 않았다. 가려진 캡처와 상태가 변하지 않은 합성 클릭은 성공 증거에서 제외했다. 혼합 DPI·스크린리더·장시간 사용은 미검증이다.

## 2026-09-08 — 글자 굵기와 대비 개선

- 본문은 내장 Pretendard SemiBold(600), 제목은 실제 Bold(700) 폰트를 사용한다. 요청 미리보기는 1열·2열 모두 본문 색으로 표시하고 Daylight 보조 색상을 #3F4F6D로 진하게 조정했다.
- Windows GetTextMetricsW로 실제 선택된 굵기 600/700과 Pretendard 서체를 확인했다. 앱 검사 11개 통과, 실제 탭 전환 검사 1개는 기존대로 ignored다. 기존 크기·ClearType·사용자 투명도 설정은 유지한다.

## 2026-09-08 — 짧은 날짜와 상태 아래 모델

- 카드 날짜·활동 요약을 yy-mm-dd로 줄이고 1열·compact 2열의 상태 배지 아래에 모델을 표시했다. 날짜 미확인은 그대로 유지하며 상세 화면의 원본 활동 시각은 보존한다.
- 데스크톱 12개 테스트 통과, 실제 탭 전환 1개 ignored. ISO 시각·날짜만 있는 값·윤일·빈 값·한글 미확인 값의 표시를 검사했다. 기존 카드·선택·로고·정리·템플릿·폰트 검사도 통과했다.

### 날짜 표시 후속 조정

접힌 1열 카드의 최근 기록은 1분 미만 방금 전, 1시간 미만 N분 전, 24시간 미만 N시간 전, 그 이후 yy-mm-dd로 표시한다. 펼친 카드는 항상 yy-mm-dd다. 코어의 기존 RFC 3339 파서를 재사용하며 새 로그가 없어도 시간 경계에서 표시를 갱신한다. 날짜·시간대·1분/1시간/24시간 경계·미확인·미래 시각·펼침 상태를 포함한 데스크톱 14개 테스트 통과, 실제 탭 전환 1개 ignored.

## Remote integration check (2026-09-08)

- Rebased the desktop card changes onto the remote activity tracking, tray behavior, and signed release packaging.
- Release tests: core 76 passed, CLI 4 passed, desktop 16 passed; the live Orca session-switch test remains explicitly ignored. Core and desktop release builds succeeded.
- The portable package check verified both executable hashes and the bundled font license inside the ZIP.

## Adversarial review fixes (2026-09-08)

- Orca orchestration run `run_7a91ce6da03f` used Claude Fable 5.1 (medium) for documentation, Opus5 (high) for collection diagnostics, gpt-6-astra (high) for desktop state and stream recovery, and gpt-5.6-sol (high) for request identity and file URI decoding. All four tasks completed and their worker resources were released.
- Prelaunch Working/Waiting sessions now show Unknown with before-launch evidence; explicit Idle/Error remain intact. Closed JSON/SQLite sessions can revive on new request evidence regardless of inferred status. Legacy request markers migrate conservatively and persist without a false revival.
- Collection warnings reach JSON output, doctor, and the desktop while healthy rows remain available. Warning storage and JSON reads are bounded; warnings clear after recovery. The desktop drains oversized snapshot frames and continues with the next frame instead of permanently stopping the reader.
- File URI decoding preserves raw and percent-encoded UTF-8, including Korean paths, and rejects malformed escapes and control characters. README and PRODUCT distinguish actual-source validation, synthetic fixtures, and experimental source registrations.
- Final Windows release checks: `cargo test --release --locked` passed 85 core tests and 5 CLI integration tests; `cargo test --manifest-path desktop/Cargo.toml --release --locked` passed 17 desktop tests with 1 live Orca session-switch test ignored. Both release builds succeeded. The desktop build used the newly built core through `WAID_CORE_PATH` and the Windows SDK resource compiler on PATH.
- The CLI watch integration test verifies healthy rows survive malformed JSON and warnings disappear after repair. A JSONL regression test failed before the final fix and passed afterward: appended malformed lines below the cached head limit and lines crossing the sampling boundary are reported for both LF and CRLF.
- These checks used synthetic source files and native automated tests. No new live-source comparison, manual desktop visual check, multi-hour soak, mixed-DPI check, or cross-platform run was performed. Identical repeated SQLite requests remain indistinguishable when the source exposes no distinct request ID, request timestamp, or user-message history. No release archive was rebuilt or published in this pass.

## Accepted review decisions and remaining bug fixes (2026-09-08)

- Orca orchestration run `run_89a7edf5ee6e` assigned the collection fixes to gpt-5.6-sol (high), native reentrancy/UI work to gpt-6-astra (high), and Windows session return to gpt-6-astra (high). All three workers completed and their terminals were released; the coordinator integrated settings migration, policy, documentation, and final checks.
- Fixed the reproduced first-populated-window abort: no mutable feed-handle borrow spans Win32 calls that can synchronously reenter drawing, focus, layout, or destruction callbacks. The regression displays a one-column window before its first populated snapshot, updates it, removes a focused card, and repopulates it. Tray icons are recreated on `TaskbarCreated` without restarting Explorer in the test.
- Fixed all five remaining collection findings: DB/WAL cache fingerprints retain separate precise modification times and sizes; full source IDs prevent eight-character hash collisions from deleting sessions; SQLite step failures discard partial results and report diagnostics; JSON model extraction selects the newest event metadata and excludes tool payloads; JSON/SQLite r3 request markers preserve fractional time and available user-message counts. The supplied `adee487a` collision keeps both sessions, with distinct display suffixes.
- Applied startup Unknown plus separate last logged status, a default recent 24-hour view retaining pins/current-launch observations/undated entries, and Show all history. Existing short-ID settings migrate only when the matching full identity is unambiguous. Marker-version migration does not revive dismissed sessions. Settings exceeding the 8 MiB read limit fail before replacing the saved file.
- Core release tests: 89 unit + 5 CLI integration passed. Desktop release suite: 26 passed; environment-dependent checks are opt-in. The six activation tests were rerun after the final PowerShell correction. Three opt-in read-only checks also passed: current Orca terminal mapping, installed ChatGPT/Claude window process identities, and a synthetic classic PowerShell console owner. Both release builds succeeded. These are 120 ordinary tests plus 3 environment-specific checks; no real agent session received input from tests.
- Native tests covered prior-status accessibility and height, recent-list aging without a changed core snapshot, association save/clear and notices, tray restoration, and 400 template previews with stable GDI samples `[54, 54, 54, 54]`. Native GUI tests were serialized. MSVC builds used the installed Windows SDK resource compiler on task-local PATH and `WAID_CORE_PATH` pointing at the freshly built core.
- Actual desktop visual checks used only synthetic logs and isolated settings under `.tools/adversarial-final`. At 125% scaling, the recent view showed two cards, Show all restored the 48-hour-old third card, and startup history, expanded request/answer/first-prompt text, and the Connect control were visible. Screenshot evidence is `recent.png`, `history.png`, and `expanded.png` in that ignored directory. Synthetic click responses were marked unverified by the provider; changes were independently checked in the returned accessibility tree and rendered image.
- A synthetic PowerShell console exposed a missed case in the first implementation: a classic console's HWND can be owned directly by PowerShell. Candidate discovery now accepts that ownership while retaining console membership and process-identity validation. The opt-in regression is reproducible with an existing classic console PID: set `WAID_TEST_CONSOLE_PID`, then run `cargo test --manifest-path desktop/Cargo.toml --release --locked classic_console_owner_is_discovered -- --ignored`.
- The final release executable's detached probe returned exactly one target for that synthetic console, including membership and creation-time checks. The actual Connect popup displayed ChatGPT, Claude, and the fixture PowerShell window. Subsequent computer-use selection returned `app_not_found`; the fixture app process and owned core remained running with no visible main window, so this was not counted as a successful window return or a reproduced process crash. No association was saved. All created fixture processes were cleaned up afterward.
- First-stage session return uses exact existing Orca mapping, explicitly associated official ChatGPT conversation links, and manually chosen ChatGPT/Claude/classic PowerShell windows. It does not claim Windows Terminal tab identity or an undocumented Claude conversation route. Foreground-window refusal and stale bindings report errors. The [ChatGPT route](https://learn.chatgpt.com/docs/reference/commands#deep-links), [Claude sidebar workflow](https://code.claude.com/docs/en/desktop), and [pseudoconsole limitation](https://learn.microsoft.com/en-us/windows/console/getconsolewindow) were checked against official sources.
- Release gates remain actual four-target return flows, keyboard/screen-reader checks, mixed-DPI behavior, and multi-hour real-use CPU/memory/GDI stability. Automatic checks and synthetic windows do not satisfy those gates. This remains a development build; no release archive or publication was made in this pass.

## Orca SSH collection fix (2026-09-08)

- Reproduced the missing-project report: the running desktop showed only recent `whatamidoing` conversations. Orca's terminal inventory placed the missing projects on an SSH execution host; their provider JSONL files were absent from the Windows collection roots.
- Added read-only collection of Orca's local version 2 hook mirror, validating pane, connection, worktree, and launch authority. Host-scoped IDs keep remote conversations separate. Hook prompts, models, answers, and explicit events feed the existing snapshot and startup observation policy. `SubagentStop` is not parent completion.
- The release core returned 13 remote conversations across `integrations`, `AUDPlatform`, `pre-fs`, and `renew-bid`. After rebuilding and restarting the user's desktop, its accessibility tree contained all four projects alongside `whatamidoing`. The captured pixels were occluded by Orca, so this is an accessibility-tree check, not visual-layout validation.
- `cargo test --release --locked`: 92 unit tests and 5 CLI integration tests passed. Desktop release tests: 28 passed, 3 environment-specific checks remained ignored. Regression coverage includes stale authority, pane replacement, repeated submit identity, startup status, and dismissal preservation after the hook cache restarts. The desktop release build succeeded; its bundled core hash matched the tested core.
- Remote records observed during a running core survive pane replacement in memory. Orca's mirror is not a transcript archive: overwritten conversations cannot be reconstructed after waid restarts. SSH session return remains unsupported; no agent received commands and no source logs were modified.

## v0.2.0 preparation and hosted Windows checks (2026-09-08)

- Aligned both Cargo packages and lockfiles to 0.2.0. CLI help now reads the package version at compile time; README release examples use v0.2.0.
- Added `Windows checks` for main pushes, pull requests, and manual runs. It reuses `desktop/build.ps1` on a GitHub-hosted Windows runner and uploads an unsigned ZIP and checksum after tests and packaging checks succeed. It needs no signing credentials and does not publish releases.
- Local checks passed: core `cargo check --release --locked --offline`, core release build, both packages' locked Cargo metadata at 0.2.0, actionlint for both workflows, and `git diff --check`.
- The preceding fresh-build test attempt was blocked before execution with Windows error 4551; the desktop build script was blocked too. Read-only inspection found Smart App Control enabled (`VerifiedAndReputablePolicyState = 1`) and matching Code Integrity event 3077 entries naming `VerifiedAndReputableDesktop`. Windows policy was not changed. Earlier test counts above are historical evidence, not a successful rerun of this version.
- The new workflow has not been pushed or run on GitHub. No v0.2.0 tag, release archive, or public release was created. Authenticode signing still requires a signing provider; the existing ZIP-level Sigstore signature does not supply Windows executable trust.

## Minimize visibility fix (2026-09-08)

- Reproduced ordinary Windows minimization removing the window's visible style: `WM_SIZE(SIZE_MINIMIZED)` called `ShowWindow(SW_HIDE)`. The added assertion failed before the fix. The custom minimize button and Windows minimization now retain a minimized window; Close still hides to the tray.
- Passed all 4 native window tests, including minimize/restore, close/tray restoration and Explorer tray-icon recreation, and the standalone EXE integration test. Commands used `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/standalone --release --locked --offline` with `--bin waid-desktop native::tests::` and `--test standalone`, respectively. `git diff --check` passed.
- Replaced the running desktop with the rebuilt EXE after checking its hash against the tested build. The restarted desktop and its owned collector were running, and Computer Use found a normal, non-minimized window with live session cards.
- The user's exact triggering action remains unconfirmed. A live Escape-key check was not performed because Computer Use refused keyboard input without foreground focus; this is not evidence of an Escape-triggered close.
