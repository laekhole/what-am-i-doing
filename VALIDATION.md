# Windows 검증 기록 — 2026-09-06

## 전수 점검과 공통 경로 정리 — 2026-09-23

### 조사 범위

기존 미커밋 문서·라이선스 변경을 보존한 작업 트리를 기준으로 조사했다. README.md와 최신 HISTORY.md를 읽었고 README.ko.md는 읽거나 수정하지 않았다. 새로운 Rust/JavaScript 의존성은 추가하지 않았다.

| 범위 | 조사한 경로 | 결과 |
|---|---|---|
| 로그·세션 수집 | `adapters`, `json`, `transcript`, `session`, `sqlite`, `time`, `matchers` | 파서 경계, 모델 원본, Cursor 폴백, worktree 브랜치, SQLite 사본 수정 |
| 프로세스·SSH 관찰 | `proc`, `proc/windows`, `orca`, `terminal`, `terminal.ps1`, `diag` | Orca 이벤트 순서 수정; 기존 읽기 전용 프로세스·화면 관찰 경계 유지 |
| CLI·HTML | `lib`, `main`, `render`, `serve`, `html`, `default.html`, `theme`, `tmpl`, `i18n` | 중복 필터·키 목록 정리, 연결 복구·갱신 누락·출력 오류·HTTP 요청 제한 보완 |
| 공통 데스크톱 | `desktop/src/main.rs`, `ui.rs`, `activate.rs` | 설정·수집 subprocess·요청 관측·복귀 검증 확인, 고정/숨김·빈 목록 안내 공통화 |
| Windows | `native`, `native/accordion`, `native/tray`, `activate/windows`, `visual` | 오류 안내 가림·한도 초과 무응답 수정, 기존 네이티브 검사와 샘플 화면 확인 |
| macOS | `macos.rs`, `macos/App.swift` | 빈 목록 안내·미리보기 롤백·전체 라이선스 메뉴 수정; 실제 Mac 빌드는 미실행 |
| 빌드·배포·문서 | Cargo manifests/lockfiles, build/package scripts, NSIS, GitHub workflows, 현재·구버전 안내 | 패키지 버전 검증·공통 고지·Linux CI 보완, 과거 문서 표시 |

### 수정한 문제

- 과도하게 깊은 JSON이 스택을 소진할 수 있던 경로에 128단계 제한을 적용했다. 잘못된 숫자, 비유한 수, 이스케이프 없는 제어문자와 깨진 surrogate를 거부한다.
- JSON과 SQLite가 모델 ID를 일찍 축약하거나 도구 결과에서 모델을 가져오던 차이를 공통 추출기로 해결했다. 표시명은 표시 단계에서만 정규화하고 JSON 이벤트 트리를 재복제하지 않는다.
- Linux/macOS Cursor의 구형 DB 질의 폴백, `.git` 파일을 사용하는 worktree/submodule의 브랜치 인식, 반복 홈 디렉터리 탐색을 보완했다.
- SQLite 폴백은 프로세스·조회별 임시 디렉터리를 사용하고 정상·실패 경로 모두 정리한다. 읽지 못한 WAL을 무시하지 않고 DB/WAL/SHM 합계 복사를 256 MiB로 제한한다.
- Orca 원격 이벤트의 밀리초를 캐시에 보존해 같은 초의 오래된 이벤트나 동일 관측 시각의 재입력이 최신 상태·요청 마커를 덮지 않게 했다.
- CLI와 HTML 서버가 같은 수집·필터 함수를 사용한다. `--keys`의 수동 목록과 로그 조회를 없애 `status.unknown`을 포함한 실제 렌더러 키를 출력한다. Windows doctor의 인자 수집 설명도 현행화했다.
- 잘못된 UTF-8 색상에서 CLI가 종료되는 문제, 제어문자로 표/터미널이 변하는 문제, 너비 0과 긴 헤더 초과, Windows/macOS TTY 판별을 수정했다. 작은따옴표 및 이스케이프된 큰따옴표 안의 `#`을 테마 주석으로 잘라내지 않는다. 무시되던 `status_priority`를 CLI 표의 안정 정렬에 적용하며 JSON/HTML 수집 순서는 유지한다.
- HTML은 HTTP 오류·실패·잘못된 문서에서 연결 중 표시를 내리고 재시도한다. SSE 끊김과 진행 중 fetch의 경쟁, 동시 갱신, branch/cwd/pid 등 표시 필드 변경 누락과 좁은 화면 카드 폭을 보완했다.
- 로컬 HTTP 요청의 Host·GET 메서드·헤더 크기·읽기/쓰기 대기 시간을 검사한다. 루프백 바인딩만으로 임의 도메인의 요청을 허용하던 경로를 닫는다.
- Windows 저장/복귀 안내를 수집 진단과 별도 줄로 표시하고 고정/숨김 1,024개 한도 초과를 알린다. Windows/Mac 초기 로딩·빈 목록·검색 불일치·초기 수집 실패 문구를 공통화하고, Mac 템플릿 저장 실패 시 미리보기를 보존한다.
- 전체 라이선스 본문을 CLI·Windows·Mac에서 공유한다. Windows 패키지 태그와 EXE 버전 불일치를 출력 생성 전에 거부하고 체크섬·덮어쓰기 검사를 빌드 경로에 연결했다. Mac 번들 버전은 Cargo에서 읽는다. Linux core 및 대시보드 JavaScript 검사를 CI에 추가했다.

### 검사 결과

- 변경 전 기준: 코어 단위 95개 + CLI 통합 7개, Windows 데스크톱 35개 + 단일 EXE 통합 1개 통과. 환경 의존 검사 4개 ignored. Windows SDK의 기존 `rc.exe` 디렉터리를 해당 명령의 PATH에 추가해 과거 SDK 탐색 실패를 해결했으며 전역 환경은 변경하지 않았다.
- 최종 `cargo test --release --locked`: 코어 단위 102개 + CLI 통합 7개 통과. `cargo test --manifest-path desktop/Cargo.toml --release --locked`: Windows 단위 37개 + 단일 EXE 통합 1개 통과, 환경 의존 4개 ignored. 총 147개 통과, 실패 0개이며 테스트에서 다시 호출한 언어 전환 자식 검사는 중복 집계하지 않았다.
- 최종 `cargo build --manifest-path desktop/Cargo.toml --release --locked`, `desktop/test-package.ps1`, `node tests/dashboard.cjs`, `bash -n desktop/package-macos.sh` 통과. Windows 패키지 검사는 버전 불일치 선제 거절·정상 복사와 SHA-256·기존 산출물 덮어쓰기 거절을 확인한다. Mac은 셸 문법과 Cargo 버전 추출만 확인했다.
- HTTP 검사는 원시 요청 9가지를 메모리 reader에서 검사하고 완결된 HTTP 요청 6가지를 실제 루프백 소켓으로 검사했다. 로컬 소켓 경로에서 중복 Host가 한 줄로 관측된 현상 때문에 malformed 헤더를 실제 소켓에서 모두 재현했다고 주장하지 않는다. 5초 I/O 제한은 API 적용 및 컴파일을 확인했으며 실제 5초 지연 재현은 미실행이다.
- 양쪽 crate의 `cargo clippy --all-targets --locked`는 종료 코드 0이다. 기존 타입 복잡도·FFI 타입 표기·테스트 스타일 경고는 남아 있으며 lint 경고 0 상태를 주장하지 않는다. 새 dead_code 경고를 포함한 미사용 함수 3개는 제거했다.
- `git diff --check` 통과. 기존 로컬 문서 검사기를 README.ko.md 읽기 제외 조건으로 실행해 Markdown 6개, 로컬/태그 링크 338개, 자산·UTF-8·코드펜스·Cargo/lock 버전·릴리스 노트 연결 검사를 통과했다.
- 격리한 `WAID_DATA_DIR`와 샘플 모드로 Windows 125% 배율, 750×975 화면의 카드·펼친 본문·하단 여백을 [직접 렌더 캡처](target/audit-20260923-demo/windows-preview.png)로 확인했다. 다른 창이 찍힌 Computer Use 캡처는 배치 증거에서 제외했다. 직접 실행한 샘플만 종료했다. 오류 2줄의 동시 실제 표시와 모든 DPI·테마 조합의 육안 검사는 미실행이다.

### 남은 검증과 설계 한계

- macOS Swift 컴파일·전용 Rust 동작 검사·실제 AppKit 및 패키지 설치는 이 Windows 환경에서 검증하지 않았다. Linux/macOS CI 설정을 로컬에서 실제 실행한 것으로 간주하지 않는다.
- 실제 에이전트 전체 조합, 편집기가 쓰는 중인 SQLite DB 사본의 일관성, SSH 숨은 pane 수집, 실제 세션 복귀 및 알림 배너는 이번 fixture 검사로 보장하지 않는다. 기존 환경 의존 4개 검사는 그대로 둔다.
- `transcript.rs`와 Windows UI는 여전히 큰 모듈이다. 이벤트별 매핑 분리와 복합 grapheme 표시 폭, 전체 TOML 문법, 장기 실행의 캐시 한도는 추가 요구·측정에 따라 분리할 후속 범위다. 무조건적인 파일 분할이나 파서 교체는 하지 않았다.
- 외부 배포·서명·설치 프로그램 실행은 하지 않았다. 구버전 문서의 내용은 보존하고 현재 안내로 연결했다. 설계 이유는 [D40](DECISIONS.md#d40--전수-점검의-공통화-범위-2026-09-23)을 참조한다.

## 라이선스 고지와 한국어 README 현행화 — 2026-09-14

- [THIRD_PARTY_NOTICES.txt](THIRD_PARTY_NOTICES.txt)에 Windows 앱 의존성 7개(Mac은 그중 5개), Rust 1.98.1 라이브러리·Unicode·compiler_builtins 0.1.160·libm 0.2.16 고지를 수집했다. Cargo registry와 동일 버전의 공식 rust-src/설치 문서를 대조해 라이선스 전문 24개와 파일별 저작권·허가 주석 18개를 보존했다. 원문 재대조·UTF-8·공백 검사 통과. 정확한 확인을 위해 공식 rust-src 구성요소를 설치했으며 프로젝트 의존성은 추가하지 않았다. 대상·선택 기능·Rust/Swift 도구체인 변경 시 고지 재검토가 필요하다.
- Windows의 기존 폰트 메뉴를 `라이선스 및 고지 / Licenses and notices`로 바꾸고 프로젝트 MIT·제삼자 고지·Pretendard OFL을 동일 상수로 내장했다. 메뉴는 기존 원자적 저장 함수로 앱 데이터 폴더의 LICENSES.txt를 만들며 desktop `--licenses`도 같은 전문을 출력한다. Mac 메뉴는 기존 폰트 고지를 유지한다.
- SDK를 명령 범위 PATH에 추가한 `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline`: desktop **35개 + 단일 EXE 1개 통과**, 환경 의존 **4개 ignored**. 고지 머리말의 Windows 범위를 명확히 한 최종 텍스트로 `--test standalone` 재실행 **1개 통과**, 같은 인자의 `cargo build` **종료 코드 0**. 동반 파일 없는 별도 폴더의 EXE가 내장 수집기와 `--licenses`로 정상 종료하고 고지 세 원문을 완전히 출력함을 검사했다. 기존 core dead_code 경고 2개가 남는다. 트레이에서 실제 텍스트 앱을 여는 물리 클릭은 별도 검증하지 않았다.
- [전달 EXE](target/release/waid-desktop-licenses.exe)는 최종 빌드와 SHA-256 `bf4d4d1c74cf4893e7ee1a21eedcbe36bc7878b86039f4b66918d54dd0f54bd9` 일치. 앞선 Windows 테스트 실행 차단과 달리 이번 검사·수집기/고지 실행은 통과했지만 공개 서명·다운로드 평판 검증을 뜻하지 않는다. 실제 Authenticode 서명은 적용하지 않았다. 기존 사용자 앱과 이전 전달 EXE는 보존했다.
- 타사 로고는 visual.rs의 OpenAI·Claude·Orca `include_bytes!`, Row.logo_id의 에이전트/호스트 선택, 카드·목록 렌더링 호출로 다시 확인했다. 기존 데모 캡처에서도 Claude·OpenAI가 보이지만 이를 이번 최종 EXE 실화면 검증으로 사용하지 않는다. 타사 자산은 변경하지 않았고 권리 고지가 OSS 허가나 SignPath 승인을 대신한다고 주장하지 않는다.
- 사용자가 명시적으로 요청한 [README.ko.md](README.ko.md)를 현재 영문 README와 동기화했다. 언어 선택·트레이 알림·SSH 화면 관찰·개인정보/제거 안내·라이선스·서명 준비와 공개 v0.2.0의 차이를 반영했다. 로컬 링크·이미지 38개, 앵커·UTF-8·코드 펜스 검사와 `git diff --check` 통과. 이전 공개 CI 수치를 최신 소스 전체의 성공으로 표현하지 않는다. macOS 최신 빌드·GUI와 실제 도구체인별 배포 검증은 미실행이다.

## SignPath 기본 여섯 조건 대조 — 2026-09-14

- 유지보수·Released·Documented: GitHub API에서 최신 공개 커밋일 2026-09-11, v0.2.0 공개일 2026-09-09와 단일 Windows EXE 자산·기능/설치/제약을 설명하는 릴리스 본문을 확인했다. 공개 main 루트에는 README가 있고 이번에 만든 LICENSE·SIGNING.md는 아직 없다. 최초 jq 인용 오류는 PowerShell JSON 파싱으로 바꿔 정상 조회했다.
- OSS 코드: `cargo metadata --manifest-path desktop/Cargo.toml --locked --offline --filter-platform x86_64-pc-windows-msvc --format-version 1` 성공. 실제 Windows 그래프의 외부 crate 7개는 itoa 1.0.18, memchr 2.8.3, serde_core 1.0.229, serde_json 1.0.151, windows-link 0.2.1, windows-sys 0.61.2, zmij 1.0.23이며 로컬 라이선스 파일을 확인했다. 모두 MIT를 선택할 수 있다. MIT OR Apache-2.0 같은 두 OSS 라이선스 선택을 상용 이중 라이선스로 판정하지 않는다. waid 두 패키지도 MIT이며 Pretendard OFL 1.1 전문은 EXE에 내장한다. [MIT](https://opensource.org/license/mit)와 [OFL 1.1](https://opensource.org/license/OFL-1.1)의 OSI 승인을 대조했다.
- 배포 고지 보완: `desktop/src/main.rs`의 `--licenses`와 트레이 메뉴는 FONT_LICENSE만 제공한다. `desktop/package.ps1`·release.yml은 EXE·체크섬·Sigstore만 배포하므로 waid 및 전이 crate 저작권/라이선스 고지를 포함하거나 동봉하는 경로가 없다. Rust 표준 라이브러리·도구체인 전체 고지와 과거 공개 바이너리 구성의 전수 검증은 수행하지 않았다.
- 타사 자산 미확정: [visual.rs](desktop/src/visual.rs)가 OpenAI PNG·Claude ICO·Orca PNG를 EXE에 내장한다. [출처 기록](desktop/assets/README.md)은 공식 avatar/favicon 및 설치된 Orca 리소스에서 가져왔다고 밝히지만 OSI 라이선스나 SignPath에서 인정할 근거는 없다. 특히 waid MIT가 이 자산을 재라이선스하지 않는다고 명시한다. 상표 사용 조건과 OSI 라이선스는 같은 것이 아니며, 이 확인만으로 위법 또는 지원 불가를 단정하지 않는다. waid 자체 그림도 [기존 시안 기록](assets/README.txt)만으로 권리·라이선스 범위를 전부 확인할 수 없다. [신청 준비 목록](SIGNING.md)에 구체 항목을 추가했다.
- No malware·No proprietary code: 확인한 기능과 호출 경로는 로컬 로그 읽기, 선택형 localhost 서버, 앱 데이터 저장, 기존 앱으로의 사용자 요청 복귀다. Windows DLL/API·설치된 Orca CLI 이용은 그 앱의 비공개 코드를 번들하는 것과 구분한다. 확인한 코드 의존성에서는 독점 라이브러리가 발견되지 않았지만, 이 정적 조사는 악성코드 검사나 모든 소스·바이너리의 안전 인증이 아니다. 임의 업로드·안티바이러스 검사·EXE 실행·시스템 변경을 하지 않았다.
- 판정: 유지보수·기존 형식의 릴리스·문서화는 근거가 있다. 모든 구성요소의 OSS 조건은 타사 로고와 배포 고지가 남아 있어 완전 충족으로 체크하지 않는다. [SignPath 조건](https://signpath.org/terms.html)의 여섯 기본 구독 조건과 Foundation 인증서의 추가 평판·MFA·수동 승인 조건을 구분한다. `git diff --check` 통과; 문서 점검이므로 빌드·회귀 테스트는 재실행하지 않았다.

## 오픈소스 서명 신청과 EXE 메타데이터 준비 — 2026-09-14

- GitHub API로 저장소 공개 상태와 기존 v0.2.0 단일 EXE 릴리스를 확인했다. SignPath 공식 신청 페이지와 공개된 임베드 폼 정의를 읽어 개인 유지관리자 선택, 회사명 선택사항, 필수 프로젝트·평판·연락처 정보와 동의·reCAPTCHA를 확인했다. 제출·계정 접근·MFA 상태 검사는 수행하지 않았다. 초기 프로젝트의 평판 심사 통과 여부는 미확정이다.
- 두 Cargo 패키지와 README의 기존 MIT 선언을 대조해 누락된 [LICENSE](LICENSE) 전문을 추가했다. [신청 초안](SIGNING.md)의 로컬 문서 링크가 존재함을 검사했다. 모든 외부 의존성·폰트·로고의 지원 자격과 배포 고지를 포괄적으로 검증한 것은 아니다.
- Windows SDK를 명령 범위 PATH에 추가한 `cargo build --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline` 성공. 기존 core dead_code 경고 2개가 남는다. 빌드에서 원래 아이콘·manifest 리소스 뒤에 Cargo 버전 기반 VERSIONINFO를 생성하며 서명자 명의를 넣지 않는다.
- `./desktop/test-version-info.ps1 -Executable ./desktop/target/context-label/release/waid-desktop.exe` 통과: ProductName=`waid`, 문자열 FileVersion/ProductVersion=`0.2.0`, 숫자 버전=`0.2.0.0`, 설명·원본 파일명 일치. 이전 메타데이터 없는 EXE는 검사기가 거부했다. 루트 재검사의 첫 인자명 오타는 수정 후 정상 통과로 구분했다. EXE는 실행하지 않았고 `Get-AuthenticodeSignature`는 여전히 `NotSigned`/`None`이다.
- `git diff --check` 통과. MSVC 리소스 빌드만 검증했으며 로컬 GNU target/windres가 없어 GNU 빌드는 미검증이다. 앱 동작 변경이 없는 리소스 준비여서 전체 Cargo 테스트를 재실행하지 않았다. 기존 전달용 bilingual EXE는 덮어쓰지 않았다. SignPath 승인·Authenticode 실제 서명·Smart App Control 실행 확인과 앞선 앱 영문화의 최종 실행 검증은 남아 있다.

## 한국어·영어 전환과 소개글 준비 — 2026-09-14

- `cargo test --release --locked --offline`: core **95개**, CLI 통합 **7개** 통과. 한국어·영어 help/진단/오류, `--lang` 검증과 `WAID_LANG` 우선순위, 영문 HTML의 한국어 대화 원문 보존·escaping·JSON 상태 불변·URL 언어 선택·SSE 갱신을 검사했다. 이후 언어 폼을 갱신 영역 밖에 두고 localhost 별칭을 허용한 최종 HTML의 template 검사 2개와 HTTP/SSE 통합 검사 1개도 통과했다. 번들 JavaScript 구문·loopback 호스트 분기와 폼 위치 확인을 수행했다.
- 배지 문구 축약 전 `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline`: desktop **35개**, 단일 EXE **1개** 통과, 환경 의존 **4개 ignored**. core와 합쳐 **138개 통과**다. 언어 설정 왕복·기존 설정 한국어 유지·대화 원문 보존과, 별도 프로세스의 Win32 컨트롤/상태/접근성/트레이 언어 전환을 검사했다. 설정 파일 잠금으로 저장 실패를 주입해 기존 언어·화면·저장 파일·미저장 표시를 보존함도 확인했다. 기존 native 검사 그룹은 한국어와 영어 환경 각각 7개 통과/1개 ignored였으며 GDI 개수는 반복 시 62로 유지됐다.
- 별도 `WAID_DATA_DIR=target/english-qa`와 `--demo`로 실제 사용자 대화를 수집하지 않는 중간 EXE를 실행했다. Computer Use의 접근성 트리로 영문 메뉴·샘플 카드·컨텍스트 상세를 확인했다. 도구의 화면 캡처는 다른 창에 가려 시각 근거에서 제외했다. PID·실행 경로·창 소유자를 검증하는 로컬 Win32 helper로 샘플 창만 렌더링한 [영문 카드](target/english-qa/cards-en.png), [영문 목록](target/english-qa/native-en.png), [영문 필터](target/english-qa/filters-en.png), [한국어 전환](target/english-qa/filters-ko.png)을 확인했다. 이는 PrintWindow 렌더링과 네이티브 메시지 호출이며 물리적 마우스 조작·혼합 DPI 검증은 아니다. 영문 필터/버튼/본문의 읽기와 즉시 한국어 전환을 확인했다.
- 위 카드에서 긴 영문 컨텍스트가 잘리는 점을 보고 최종 코드의 Windows 배지를 `25.0% left · Compacted` / `Context ? · Compacted`로 줄였다. 카드 너비 계산과 카드·목록의 표시가 같은 함수를 사용하며 한국어·상세·접근성 문구는 유지한다. 해당 배지 assertions를 기존 격리 검사에 추가했다. **이 마지막 수정 후 desktop 검사 바이너리는 컴파일됐으나 Windows 앱 제어 정책(os error 4551)으로 실행 전 차단**됐다. 앞선 138개 통과를 이 최종 검사의 통과로 대체하지 않는다.
- 최종 `cargo build --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline` **종료 코드 0**. [전달용 EXE](target/release/waid-desktop-bilingual.exe)는 빌드 산출물과 SHA-256 `dfd05c535fbbb102657a8a79525084895c267a1d2448cf6d5de62a707086a8ec`가 일치한다. 이 최종 EXE의 샘플 실행 역시 Windows 앱 제어에 차단돼 최종 배지 실화면·재실행 검증은 미완료다. 정책 변경·우회·다른 이름으로 테스트 차단 우회는 하지 않았다. 기존 core dead_code 경고 2개는 남는다.
- Mac Rust/Swift에 선택·메뉴·접근성·편집기 갱신을 반영하고 기존 action/스모크 검사를 보강했다. action 검사는 별도 프로세스로 격리해 전역 언어 변경이 병렬 검사에 간섭하지 않는다. Rust 구문 검토와 차이 검사는 통과했지만 **이 Windows 환경에서 Swift 컴파일·Mac 스모크·실기는 미실행**이다. 기존 Mac CI 성공을 이번 변경의 결과로 주장하지 않는다.
- `git diff --check` 통과. 기존 실행 중인 waid·사용자 설정·대화를 보존했고 검증용 샘플 프로세스만 종료했다. [소개글](LAUNCH.md)은 로컬 초안이며 커밋·푸시·새 릴리스·Reddit/GeekNews 게시를 하지 않았다. 공개된 v0.2.0 EXE는 이번 영어 전환을 포함하지 않는다. 수집 진단은 시작 언어를 유지하고 운영체제 문구·사용자 템플릿은 번역 대상이 아닌 제약을 README와 [D37](DECISIONS.md#d37--한국어영어-전환과-공개-소개-준비-2026-09-14)에 기록했다.

## 트레이 세션 요약과 답변 알림 — 2026-09-14

- Windows SDK 10.0.26100.0의 `rc.exe`를 해당 명령의 PATH에 추가하고 `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/tray --release --locked --offline` 실행: 제품 코드의 최종 동작 변경까지 반영한 **desktop 33개·단일 EXE 1개 통과**, 기존 환경 의존 3개 ignored. 새 요청별 중복 제거, 실행 전 답변 제외, Waiting→Waiting의 새 요청, 숨김/재등장, 신뢰 없는 화면 관찰, 묶기, 확인 표시와 Waiting 분리, 필터 독립성, UTF-16 길이 제한을 검사했다. 네이티브 검사에서는 숨긴 창의 갱신·메뉴 항목·콜백을 통한 세션 선택/상세 복귀·알림 설정 저장·수집 오류 표시·Explorer 아이콘 재등록과 배지 100회 갱신의 GDI 증가 상한(+2)을 확인했다.
- 별도 설정/어댑터 경로에서 **샘플 로그 하나만 읽는 실제 EXE**를 실행했다. 기본 어댑터 12개를 샘플 경로/감지하지 않는 실행명으로 대체하고 Orca hook 경로와 SSH 화면 관찰을 분리한 뒤 `doctor`에서 reader 1개·샘플 1개·실행 감지/Orca/SSH 0개를 확인했다. 실행 전 답변의 미확인 표시와 첫 닫기 안내, 저장된 안내 확인값, 창을 숨긴 상태에서 새 요청/답변을 추가했을 때 `작업 중 1`→`새 답변 1 · 내 차례 1` 갱신을 확인했다. [실제 메뉴 캡처](target/tray-qa/menu.png)는 최종 제품 동작 빌드의 샘플 세션이며 실제 사용자 대화가 아니다.
- Orca Computer Use의 창 목록·접근성 조회를 사용했다. 숨긴 앱/일시적인 메뉴는 도구에서 찾지 못하는 경우가 있고 일부 창 캡처는 다른 앱/경계 밖 픽셀이 섞여 시각 근거에서 제외했다. 샘플 PID·실행 경로를 검증하는 로컬 Win32 보조 스크립트로 샘플 창을 배치하고 트레이 콜백을 호출해 실제 네이티브 메뉴 영역만 캡처했다. 이는 실제 마우스 우클릭·모든 모니터/DPI 조합의 종단 검증을 뜻하지 않는다.
- **Windows 자체 알림 표시 확인:** 명시적으로 실행한 환경 의존 검사에서 `SHQueryUserNotificationState = 5`, 네이티브 알림 요청 성공과 **`NIN_BALLOONSHOW` 수신**을 확인했다. 명령은 위 Cargo 옵션에 `--bin waid-desktop windows_notification_reports_display_through_shell_callback -- --ignored --nocapture`를 추가했다. UI Automation으로 알림 제목을 찾는 별도 시도는 제목을 찾지 못했으므로 배너 픽셀/배치 확인으로 기록하지 않는다.
- 이후 같은 환경 의존 검사를 직접 알림 함수 호출에서 **숨긴 창의 일반 수집 갱신→알림 전송 경로**까지 검사하도록 보강했다. 이 최종 테스트 바이너리는 컴파일됐지만 **Windows 애플리케이션 제어 정책(os error 4551)이 실행 전에 차단**했다. 앞선 직접 알림 확인을 이 보강 검사의 통과로 대체하지 않는다. 최종 코드에는 해당 검사가 ignored로 남으며 명시적으로 실행해야 한다. 초기 콜백 상수 누락 경고와 테스트 변수 이름 충돌도 수정 전 결과이며, 남은 빌드 경고는 기존 core dead_code 2개다. 정책 변경·우회는 하지 않았다.
- 기존 사용자 waid와 기본 설정·원본 대화를 보존했고 샘플 앱과 그 수집기만 종료했다. 실제 에이전트 앱 네 종류로의 복귀, 알림 배너 직접 클릭, 방해 금지/잠금/장시간 실행·1시간 경과, 모든 DPI에서 배지 가독성 및 Mac 동작은 이번에 실기 검증하지 않았다. 설계와 OS 알림 보관 한계는 [D36](DECISIONS.md#d36--창을-숨긴-동안의-트레이와-답변-알림-2026-09-14)을 따른다.
- **최종 실행 파일:** `cargo build --manifest-path desktop/Cargo.toml --target-dir desktop/target/tray --release --locked --offline` 종료 코드 0. 최종 EXE의 `--licenses`를 stdout/stderr를 끝까지 읽고 종료를 기다리는 프로세스 호출로 확인해 종료 코드 0과 내장 SIL 라이선스를 확인했다. 최초 PowerShell 직접 대입 호출은 종료 대기를 올바르게 처리하지 못해 파이프 종료(os error 109)를 발생시켰으며, 이를 제품 성공/OS 정책 차단으로 기록하지 않는다. [전달용 EXE](target/release/waid-desktop-tray.exe)와 빌드 원본의 SHA-256은 `D452663E5F64202CC0532784FD76C85DA827D70D17EC70B1230876A543B5C548`로 일치한다. 새 경로에 준비했으며 기존 실행 파일을 덮어쓰지 않았다. `git diff --check` 통과.

## PowerShell SSH 화면 감지 — 2026-09-10

- `cargo test --release --locked --offline`: 최종 코드에서 core **94개**, CLI **6개** 통과. 화면 판별 회귀 검사에는 Codex 배너·하단 표시, Claude 배너, 한글 미리보기, 혼합 에이전트·단순 언급·셸 복귀 제외, 입력 길이 제한, 대화/요청/경로/상태 정보를 지어내지 않는 조건을 포함했다. 최초 배너만 지원하던 코드의 검사는 Windows 정책(os error 4551)으로 실행되지 않았다. 이후 실제 화면에서 배너가 사라지는 경우를 확인해 수정한 최종 코드의 결과와 구분한다. 정책 변경·우회는 하지 않았다.
- `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline`: desktop **31개**, 단일 EXE **1개** 통과, 환경 의존 **3개 ignored**. 이번 변경과 기존 미커밋 UI 수정이 함께 있는 상태의 검사다.
- 실제 Windows Terminal의 기존 SSH 창을 입력·탭 전환 없이 읽었다. UIA helper는 문서 1개·약 11,000 UTF-16 단위, 읽기 오류 없음이었다. 최종 CLI `--json` 수집 1회는 **2,458ms**에 전체 스냅샷을 반환했고, `terminal_screen` Codex **1개**, 화면과 일치하는 모델, 추정 작업명, Unknown 상태, 비어 있는 context/session_id/cwd/pid를 확인했다. 이 시간은 로컬 로그 수집까지 포함하며 UIA 전용 지연이 아니다. 원문·작업명은 검증 문서에 복사하지 않았다.
- [새 EXE](target/release/waid-desktop-ssh.exe)는 desktop release 산출물과 SHA-256이 일치한다. 별도 `WAID_DATA_DIR`에서 새 앱을 실행해 접근성 트리의 `화면 관찰` 카드·Codex 모델·추정 미리보기·미확인 상태와 실제 사용 중 바뀐 입력의 반영을 확인했다. 화면 캡처는 기존 항상 위 창/다른 앱에 가려 육안 근거에서 제외했다. 새 EXE의 `--waid-core --json`, `WAID_TERMINAL_SCAN=0`에서 관찰 0개, `--licenses` 실행을 확인했다. 검증용 앱과 자식 수집기만 종료했으며 기존 waid와 기본 설정은 보존했다. 최종 `git diff --check` 통과.
- 별도 SSH 원격 조회는 `BatchMode=yes`, `StrictHostKeyChecking=yes`, `UpdateHostKeys=no`, `PermitLocalCommand=no`, `ClearAllForwardings=yes`와 연결 제한을 적용해 한 번 시도했으나 인증 단계에서 거부됐다. 원격 명령 실행·tmux 조회·원격 설정 변경 성공으로 기록하지 않는다. 제품은 SSH 연결을 시도하지 않는다.
- 실제 Claude SSH 화면, 기본 콘솔, 긴 화면/다중 pane/복사 모드, 장시간 성능과 UIA 장애 주입은 미검증이다. 숨은 탭·tmux pane 전체와 정확한 요청/상태/대화 식별은 구현 범위 밖이다. 세부 설계는 [D35](DECISIONS.md#d35--일반-ssh의-로컬-화면-관찰-지원-2026-09-10)에 기록했다.

## SSH 터미널 텍스트 비용·탭 구조 조사 — 2026-09-10

- 10:43 +09:00, 실제 Windows Terminal **1.24.2607.10001**의 기존 창을 읽기 전용으로 조회했다. Orca Computer Use에서 탭 5개와 터미널 본문 1개를 확인했으며, 직접 UI Automation 조회에서도 서로 다른 runtime ID의 TabItem 5개·선택된 탭 1개·`TermControl`의 TextPattern 1개였다. 탭 제목 네 개가 같았다. 탭 선택·포커스·스크롤·원격 명령·설정은 변경하지 않았다. 탭 헤더 TextBlock도 TextPattern을 제공하므로 본문과 구분해야 한다.
- [측정 스크립트](.tools/ssh-observation/measure-uia.ps1)를 `powershell.exe -NoProfile -Mta -File .tools/ssh-observation/measure-uia.ps1 -WindowHandle <현재 창 핸들> -Repeats 10`으로 실행했다. 조회 대상이 WindowsTerminal인지 확인하고, 직접 API 호출만 Stopwatch로 측정한다. 스크립트·[최종 결과](.tools/ssh-observation/uia-measurements.json)는 Git에서 무시된 로컬 산출물이다. 터미널 본문은 메모리에서 길이만 세며 출력·저장하지 않는다. 창 핸들과 runtime ID는 영구 세션 식별자가 아니다.

| 조회 | 표본 | 중앙값 | 최소–최대 |
|---|---:|---:|---:|
| 화면 범위 최대 4,000 UTF-16 코드 단위 | 10회 | 0.436 ms | 0.332–13.505 ms |
| 문서 끝부분 최대 4,000 UTF-16 코드 단위 | 10회 | 1.1765 ms | 0.860–13.764 ms |
| 전체 문서 11,062 UTF-16 코드 단위 | 10회 | 0.3685 ms | 0.338–0.465 ms |

- 별도 1회 하위 UI 요소 탐색은 32개·34.145 ms였다. 최대값에는 첫 호출/JIT 등 초기 비용이 섞일 수 있으나 원인을 분리 측정하지 않았다. 전체 문서가 작은 이번 사례에서는 호출 수가 적은 전체 읽기가 끝부분 범위 이동보다 빨랐다. 이 결과를 긴 스크롤백에도 일반화하지 않는다. Orca CLI의 별도 1회 전체 상태 조회 2,341 ms는 CLI·통신·트리 생성 비용을 포함하므로 직접 텍스트 읽기 시간으로 사용하지 않는다.
- 최초 스크립트는 TextPatternRangeEndpoint 네임스페이스 오류로 실패했고, 설치된 .NET 형식 확인 후 수정했다. 중간 탐색에서 탭 제목 TextBlock도 측정 대상에 포함된 것을 확인해 `TermControl`로 한정한 뒤 위 최종 결과를 얻었다. 길이 상한 검사는 최종 실행에서 통과했다.
- **한계:** 현재 본문 하나의 짧은 호출 지연 측정이다. 지속적인 CPU·메모리 사용, 입력 지연, 고속 출력, 큰 스크롤백, 수십 개 탭/창, 탭 전환·최소화, 다른 터미널과 UIA 이벤트 폭주는 측정하지 않았다. 실제 원격 tmux의 버전·소켓·인증·숨은 pane 조회 및 에이전트 상태 판독도 미검증이다. 기존 소스의 상태 규칙과 Microsoft/tmux 공식 문서를 대조했으며 설계 판단은 [D34](DECISIONS.md#d34--ssh-터미널-관찰과-tmux-수집-검토-2026-09-10)에 기록했다. 제품 코드 변경이 없어 Cargo 빌드·테스트는 실행하지 않았다.

## 카드 정렬·높이 개선 — 2026-09-10

- `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline`: 최종 변경에서 데스크톱 **31개**, 단일 EXE **1개** 통과, 기존 실환경 연결 3개 ignored. 실제 네이티브 카드 갱신·접기·스크롤·행 제거·목록 제외/복원, 과거 상태·컨텍스트·압축 기록의 접근성 문구와 상세 보존을 포함한다. 기존 코어 dead_code 경고 2개는 유지한다.
- 기본 14px 글꼴에서 접힌 높이는 104px(과거 상태가 하나라도 있으면 125px)에서 94px로 변경했다. 마지막 기록을 컨텍스트 오른쪽에 배치해 별도 전역 높이 상태를 제거했다. 펼친 본문은 526→426px이며 긴 텍스트는 기존 읽기 전용 편집창의 스크롤로 확인할 수 있다.
- 별도 `WAID_DATA_DIR`와 `--demo`로 125% 배율의 실제 Windows 화면을 확인했다. [변경 전](target/card-layout-preview/before.png), 밝은 테마 [접힘](target/card-layout-preview/daylight-collapsed.png)·[펼침](target/card-layout-preview/daylight-expanded.png), [최소 480 논리 픽셀 폭](target/card-layout-preview/daylight-narrow.png), 어두운 테마 [접힘](target/card-layout-preview/midnight-collapsed.png)·[펼침](target/card-layout-preview/midnight-expanded.png)을 직접 확인했다. 프로젝트·시간·상태 정렬, 프로젝트와 컨텍스트 글자 왼쪽 정렬, 25%·압축·미확인 표시, 클릭 후 접힘 전환과 한 줄 버튼 배치를 검증했다. 다른 창에 가렸거나 초기화가 끝나지 않은 캡처는 증거에서 제외했다.
- 최종 release EXE를 [target/release/waid-desktop.exe](target/release/waid-desktop.exe)에 복사했고 빌드 산출물과 SHA-256 일치 및 `--licenses` 종료 코드 0·내장 폰트 라이선스를 확인했다. 직접 띄운 샘플만 종료했으며 사용자 기본 설정은 변경하지 않았다. 공개 배포·실제 세션 복귀·다른 DPI·최대 24px 사용자 글꼴의 육안 검증은 수행하지 않았다.

## 기존 아이콘 복원·상단 렌더링 — 2026-09-10

- 사용자가 assets에 복원한 앱 타일·마스코트·가로 로고와 재생성한 ICO·desktop/assets/waid.png가 Git HEAD의 기존 자산과 바이트 단위로 일치합니다. ICO의 9개 PNG 프레임 크기·RGBA·투명도와 디코딩을 확인했습니다. question_mark 보관 폴더는 변경하지 않았고 활성 빌드에서 참조하지 않습니다.
- 상단 마스코트는 96px HICON 변환 후 DrawIconEx 재축소를 제거하고, 기존 GDI+로 원본 PNG를 현재 창의 DPI에 맞춘 32 논리 픽셀 크기에 직접 그립니다. [Microsoft의 보간 품질 설명](https://learn.microsoft.com/en-us/windows/win32/gdiplus/-gdiplus-using-interpolation-mode-to-control-image-quality-during-scaling-use)에 따라 HighQualityBicubic을 지정했습니다. 메모리 스트림은 이미지 해제 뒤 Release하며 새로운 패키지 없이 windows-sys의 Com 기능만 활성화했습니다.
- 변환 대상 PNG가 이미지 미리보기에 매핑되어 기존 직접 덮어쓰기는 실패했습니다. 임시 파일 완성 후 File.Replace로 교체하도록 수정한 스크립트가 실제 매핑 상태에서도 성공했고, 결과 PNG·ICO의 원본 일치를 확인했습니다.
- `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline --bin waid-desktop`: 컴파일 성공, 테스트 실행 파일은 Windows 애플리케이션 제어 정책(os error 4551)으로 **실행되지 않았습니다**. 추가한 32·40·48·64·96px 및 밝고 어두운 배경의 네이티브 렌더링 회귀 검사도 미실행입니다. Com 기능 누락으로 생긴 초기 컴파일 오류는 수정했으며 정책 변경·우회는 하지 않았습니다.
- 같은 캐시의 `cargo build --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline`는 종료 코드 0입니다. 최종 파일을 요청한 [target/release/waid-desktop.exe](target/release/waid-desktop.exe)에 복사했습니다. 4,566,016 bytes, SHA-256 `7927f4e9cf3ca500b88b39a459dab0b397ea48546c8354279025de7e9ab42a69`. 이전 question-logo 실행 파일의 실제 출력 위치는 desktop/target/context-label/release였으며 루트 target/release에 있었던 것은 아닙니다.
- 최종 EXE의 `--licenses` 종료 코드 0, PE 아이콘 리소스 9개와 복원한 ICO의 바이트 일치, 기존 마스코트 PNG 포함·물음표 PNG 미포함을 확인했습니다. 별도 WAID_DATA_DIR·샘플 모드로 최종 EXE를 직접 실행하고 750×975 물리 픽셀(기본 600×780, 125%)에서 [밝은 테마](target/logo-rollback-preview/daylight.png)와 [어두운 테마](target/logo-rollback-preview/midnight.png)의 상단 윤곽·눈·투명 배경을 확인했습니다. 이 PNG들은 무시된 로컬 검증 산출물입니다. 직접 띄운 샘플만 종료했고 사용자 기본 설정은 수정하지 않았습니다.
- `git diff --check` 통과. 기존 코어 dead_code 경고 2개는 유지합니다. 전체 자동 테스트·다른 배율의 실화면·Mac 실행·설치 프로그램 재설치·공개 배포는 미실행입니다.

## 물음표 마스코트 로고 — 2026-09-10

- 1254px 원본 두 개를 유지하고 기존 PowerShell 변환 코드로 Windows 앱 타일 256px, 상단 마스코트 96px, ICO 9개 크기(16·20·24·32·40·48·64·128·256px)를 생성했습니다. 모든 ICO 엔트리의 크기·오프셋·RGBA·투명도와 PNG 디코딩, 256px 엔트리와 앱 PNG의 바이트 일치를 확인했습니다.
- `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline --bin waid-desktop`: 30개 통과, 실환경 연결 3개 ignored. 상단 마스코트 참조를 96px 파생본으로 바꾼 뒤 `native::tests::window_controls_persist_and_maximize_is_unavailable`를 다시 실행해 통과했습니다. 새 PNG 두 개의 Windows 디코딩과 창·설정 경로를 확인하며, 기존 코어 dead_code 경고 2개는 유지합니다.
- Windows SDK `rc.exe` 경로를 해당 빌드 셸에만 추가하고 같은 캐시에서 `cargo rustc ... -- -C extra-filename=-question-logo` release 빌드를 완료했습니다. 최종 [새 EXE](desktop/target/context-label/release/waid-desktop-question-logo.exe)의 PE 아이콘 리소스 9개가 새 ICO와 바이트 단위로 일치하며 `--licenses`가 종료 코드 0으로 내장 라이선스를 반환했습니다.
- 별도 `WAID_DATA_DIR`와 `--demo`로 화면 확인했습니다. 첫 캡처는 다른 창에 가려져 제외했고, 샘플 전용 항상 위 설정으로 상단의 물음표와 32 논리 픽셀 배치를 확인했습니다. 큰 원본을 직접 로딩한 외곽선이 거칠어 96px bicubic 파생본으로 보완했습니다. 파생 PNG 육안 확인과 최종 EXE 네이티브 디코딩은 통과했으나 최종 화면 캡처는 이미지 보기 창에 가려져 개선 후 화면 증거로 인정하지 않았습니다. 직접 띄운 샘플 프로세스만 종료했습니다.
- 설치 스크립트의 Windows 앱 설정·시작 메뉴·설치/제거 아이콘 연결과 Mac 패키징의 새 원본 참조를 소스로 확인했습니다. 실제 재설치·설치된 앱 아이콘 캐시·Mac 실행/패키징·공개 배포는 미실행입니다. README.ko.md는 읽거나 수정하지 않았습니다.

## v0.2.0 기능·문서 검토 — 2026-09-09

- 현재 코어·데스크톱 패키지 버전은 모두 `0.2.0`입니다. README 두 언어와 `releases/v0.2.0.md`를 수집·상태·정리·컨텍스트·세션 복귀·템플릿·CLI·배포 구현에 맞춰 검토했습니다. 공개 Windows 릴리스와 v0.3.0 Mac 개발 단계를 구분합니다.
- 코드 `47f4317`의 [Windows CI](https://github.com/laekhole/what-am-i-doing/actions/runs/34308905362)는 **130개**(코어 93, CLI 6, 데스크톱 30, 단일 EXE 1) 통과, 환경 의존 3개 ignored입니다. 이전 로컬 실행 정책 때문에 미확인으로 남았던 컨텍스트 배지 회귀 검사도 이 hosted 실행에서 통과했습니다.
- 같은 코드의 [macOS CI](https://github.com/laekhole/what-am-i-doing/actions/runs/34308905356)는 Apple Silicon·Intel 각각 **121개**(코어 90, CLI 6, 데스크톱 24, 단일 실행 파일 1) 통과, 실제 Orca 환경 검사 각 1개 ignored입니다. 양쪽 AppKit 스모크·재시작 검사와 개발 패키징도 통과했습니다.
- 이번 검토에서는 새 실세션 조작·수 시간 사용·스크린리더·혼합 DPI·깨끗한 Mac 설치 검사를 수행하지 않았습니다. 기존 ChatGPT 저장 링크 이동 확인은 전체 연결 UI·포커스 검증 완료로 확대하지 않습니다. 아래 날짜별 기록과 [Mac 출시 확인 목록](MACOS.md)의 남은 항목은 유지합니다.
- Mac 컨텍스트 표시를 코드로 추적한 결과 상세 요약은 포함되지만 목록 `lines`에 컨텍스트 문자열이 없어 Swift의 강조 처리가 표시할 범위를 찾지 못합니다. Windows의 토큰·한도·측정 시각 상세 블록도 Mac에는 연결되지 않았습니다. 이 한계는 [MACOS.md](MACOS.md)에 기록했으며 이번 Windows 릴리스 문서 작업에서 앱 코드는 수정하지 않았습니다.
- 로컬 문서 검증: Markdown 7개·로컬/태그 링크·이미지 경로·UTF-8·코드 블록·두 패키지와 lock 버전 일치, README/릴리스 PowerShell 9개 블록 구문과 실제 노트 경로 계산, actionlint 및 `git diff --check` 통과. 릴리스 워크플로는 자동 커밋 목록 대신 버전별 한·영 노트 파일을 사용합니다.
- 릴리스 커밋 `94bf72f`와 `v0.2.0` 태그를 함께 푸시했습니다. 해당 커밋의 [Windows 검사](https://github.com/laekhole/what-am-i-doing/actions/runs/34310231306), [Mac 양쪽 아키텍처 검사](https://github.com/laekhole/what-am-i-doing/actions/runs/34310231349), [Windows 릴리스](https://github.com/laekhole/what-am-i-doing/actions/runs/34310231143)가 모두 성공했습니다. 태그 릴리스 로그에서 Windows 130개 통과·환경 의존 3개 ignored와 Sigstore 서명 검증 성공을 재확인했습니다.
- [게시된 v0.2.0](https://github.com/laekhole/what-am-i-doing/releases/tag/v0.2.0)은 EXE·체크섬·Sigstore 번들 세 파일을 포함합니다. 본문은 `releases/v0.2.0.md`와 줄바꿈 정규화 후 일치합니다. 세 파일을 다시 내려받아 체크섬과 GitHub 자산 다이제스트를 비교하고 Cosign 3.1.3으로 이 저장소의 `release.yml@refs/tags/v0.2.0` 서명 신원을 검증했습니다(`Verified OK`). 내장 수집기의 `--waid-core --version`도 `waid 0.2.0`을 반환했습니다.
- 게시 EXE SHA-256: `8f03aa4d20862dc903325dc1915362a6abd4b4a27c0259048c9d51c1e9d9b4af`. 이는 배포 파일의 무결성·출처 확인이며 Authenticode 또는 남은 수동 사용성 검증을 대신하지 않습니다.

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

## ChatGPT existing-session return check (2026-09-08)

- Checked the running ChatGPT desktop package `OpenAI.Codex_26.901.6511.0_x64` and the current standalone waid EXE (SHA-256 `24eb3ed86e2305f4de288c4a2321854272275add756ff4e924dd4d84c9efb4bb`). Local session-index identifiers matched collected waid rows.
- Windows dispatch of an existing `codex://threads/<thread-id>` link changed ChatGPT's active conversation header to the intended existing conversation. Then an isolated waid instance loaded a saved `chatgpt_link` association. With ChatGPT displaying a different existing conversation, sending `BM_CLICK` to that waid row's native logo button ran the production activation path and returned ChatGPT to the associated conversation. The active header was independently read through Windows accessibility before and after; sidebar presence alone was not counted as success.
- Passed the existing `activate::tests::chatgpt_links_cannot_create_chats_or_cross_session_boundaries` and `native::tests::displayed_feed_populates_refreshes_and_removes_focused_cards` tests using the standalone release target directory. These cover mismatched/invalid links, logo row selection, and association save/clear behavior.
- This validates an existing saved association and actual app navigation, not the complete physical-click setup flow. Computer Use keyboard input returned `window_not_focused` even with restoration requested, and its synthetic mouse clicks did not open the requested controls. Therefore `Ctrl+Alt+L`, the live Connect popup selection, foreground focus, and physical mouse hit-testing were not marked passed. Captured pixels were clipped/occluded; the navigation evidence is accessibility state, not visual-layout QA.
- Validation settings and a native-click helper are under ignored `.tools/chatgpt-return`; conversation titles and IDs are not included in this record. Only validation-owned waid processes were cleaned up. User settings and agent conversations were not modified, and no prompt was submitted. Claude live return remains a separate follow-up.

## Session context measurements (2026-09-09)

- Added latest logged context tokens, explicit context limit, source measurement timestamp, and observed compaction to the collector JSON and Windows cards/details. No cumulative totals, account quotas, quality scores, thresholds, or usage-dependent colors are collected or displayed. Claude transcript input/cache counts are available without an inferred model limit; unsupported sources remain unknown.
- `cargo test --release --locked`: 93 core tests and 6 CLI integration tests passed. Desktop tests using `--manifest-path desktop/Cargo.toml --target-dir desktop/target/standalone --release --locked`: 29 passed, 3 unrelated environment-specific checks ignored; the standalone EXE integration test also passed. Regression checks cover latest vs cumulative usage, cached-input accounting, invalid/missing values, compaction reset, bounded-read gaps, log replacement, old snapshots/settings, and native card refresh.
- A real-history snapshot contained 143 Codex sessions with context tokens and limits and 20 Claude sessions with input/cache tokens; two Codex rows had compaction evidence within the bounded read range. No original prompts or logs were changed. Missing compaction evidence is not proof that compaction never happened.
- Computer Use visually checked collapsed sample cards (62.0%, unknown limit, and unknown usage) and expanded token/limit, timestamp, and compaction text at the default font and 125% Windows scale. Captures are in ignored `.tools/context-preview/collapsed.png` and `expanded.png`; the lower window edge was covered by the taskbar. The accessibility tree independently contained the new values. The sample process used isolated settings and was stopped afterward. Other font sizes and mixed-DPI layouts were not visually revalidated.
- The default desktop target was locked by the running app. A fresh target encountered Windows application-control error 4551; the existing standalone cache and Windows SDK resource compiler path allowed the final desktop checks to pass, including child-collector launch. Windows security policy was not changed. The build is in `desktop/target/standalone/release/waid-desktop.exe`; the user's running app was not replaced.

## Context badges without opening details (2026-09-09)

- Per the follow-up request, 25% or less remaining and observed compaction now emphasize the context line. The label shows the remaining percentage at that boundary and `압축됨` for observed compaction; both appear together when applicable. Unknown limits do not trigger the percentage condition. The Windows card header and two-column list share the renderer; the Mac presentation receives the same condition and label.
- The release desktop build and standalone EXE integration test passed using `desktop/target/standalone`. Added checks for the exact 25% boundary, just above it, exhausted context, missing/zero limits, compaction without a measurement, retained compaction after a smaller measurement, and native refresh while collapsed. The desktop unit-test executable was blocked twice before execution by Windows application-control error 4551; these new automated assertions were compiled but are not marked passed. Mac compilation and visual checks were not available on this Windows machine.
- Computer Use screenshots at the default font and 125% Windows scale verified the highlighted labels in collapsed cards and the two-column list, including simultaneous remaining percentage and compaction, with unknown usage unhighlighted. Captures are in ignored `.tools/context-badges/collapsed.png` and `two-columns.png`. The sample used isolated settings; only its processes were stopped after checking. `git diff --check` passed. The built EXE SHA-256 is `02715257c20dff7a521e17245182a75ad13adca3e12f27dcbf2fdc7c2bbd6642`.


## Documentation and pre-push checks (2026-09-09)

- Updated English and Korean README privacy statements from the current implementation: local log collection, read-only SQLite access, local settings, no external upload/telemetry/cloud sync/AI API calls, and the optional dashboard's `127.0.0.1` listener. The statement distinguishes custom HTML, user-configured synced folders, and other apps' network activity; it is not a vulnerability-free guarantee or a packet-capture audit.
- Refreshed the Korean guide for standalone EXE releases, the platform roadmap, session organization/return, source validation limits, and context measurements/badges.
- Re-ran `cargo test --release --locked`: 93 core and 6 CLI integration tests passed. The desktop release test command with `--target-dir desktop/target/standalone` reached the compiled unit-test executable, but Windows application control blocked execution with error 4551. No security policy was changed; the new desktop assertions remain unverified locally. Mac checks were not run on this Windows host.

## Native macOS app implementation and automated checks (2026-09-09)

**v0.3.0 release acceptance is still pending.** The native AppKit implementation is now present; passing the checks below does not validate installation and real-session return on a user's Mac. See [MACOS.md](MACOS.md) for installation, controls, supported return targets and the required device walkthrough.

- macOS code commit `5951a17`: [Apple Silicon and Intel CI](https://github.com/laekhole/what-am-i-doing/actions/runs/34302645779) passed on `macos-15` and `macos-15-intel`. Each passed **118 Rust checks**: collector 89, CLI 6, desktop 22 and standalone binary 1. The read-only real-Orca test remains ignored (one per architecture). The Swift AppKit library compiled and linked into the Rust executable on both architectures.
- [Windows CI](https://github.com/laekhole/what-am-i-doing/actions/runs/34302256132) passed **127 checks**: collector 92, CLI 6, desktop 28 and standalone binary 1, with three existing real-environment tests ignored. It built and packaged the Windows EXE. That run used `b40bdaf`; subsequent code changes were confined to the macOS module and its workflow. Local Windows core and desktop release checks also passed using an isolated desktop target directory after adding the installed Windows SDK resource compiler to that command's PATH.
- Both Mac jobs packaged an architecture-specific `.app` ZIP, generated its icon and checksum, and passed `plutil` plus ad-hoc `codesign --verify --strict`. Downloaded ZIP checksums and executable/icon entries were independently checked locally. No Developer ID signing or notarization was performed.
- Each Mac job ran the packaged AppKit app twice with isolated settings and a synthetic Codex transcript. The actual collector child supplied the row. Native checks covered selection, multiline Unicode prompt copying, pin/dismiss/restore, search, topmost, opacity, template editor, close/reopen, and exiting. The first process saved a pin, 75% opacity and Midnight; the second process verified their restoration. These calls exercise native controls programmatically, not physical mouse/keyboard input or real coding-agent conversations.
- Rust Mac action checks cover settings read-back, search, template preview versus apply, invalid/mismatched links, request-based revival, retention after collector errors and rollback after a settings-save failure. Shared exact-Orca matching tests now also run on macOS. No validation test dispatched a ChatGPT link, switched an actual Orca session, or sent a prompt.
- Reviewed all four Daylight/Midnight content-view PNGs produced by both architectures: readable Unicode text, distinct selected rows, visible status/model, list/detail separation, native controls and toolbar placement. These are content-view captures; Dock/menu interaction, physical hit testing, VoiceOver, multiple monitors and long real-session lists remain manual checks. Local evidence is under ignored `.tools/macos-qa/5951a17-arm64` and `.tools/macos-qa/5951a17-intel`; CI artifacts also contain the PNGs and development ZIPs.
- `actionlint`, Bash packaging syntax and committed-diff whitespace checks passed. Existing and concurrently edited context-measurement/badge work was retained as uncommitted changes, outside this Mac implementation's final committed diff. No source logs or user settings were modified by Mac validation. No release tag or GitHub Release was created.

Still required before calling macOS support complete: Finder installation from a downloaded/quarantined package on clean Apple Silicon and Intel accounts, actual Claude/Codex collection and organization across restart, exact return to a live local Orca session and a copied ChatGPT conversation, rejection of stale/ambiguous targets, Developer ID/notarization distribution checks, accessibility/keyboard/manual window behavior, and validation of the oldest intended OS (the build target is 13.0, CI ran on 15). Standalone Terminal/iTerm/Claude desktop window return and SSH return are not implemented Mac targets.
