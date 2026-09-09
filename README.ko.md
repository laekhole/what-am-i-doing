<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/waid-on-dark.png">
    <img src="assets/waid-horizontal.png" alt="waid — what am I doing?" width="440">
  </picture>
</p>

# waid — what am I doing?

English: [README.md](README.md).

여러 코딩 AI 세션의 **현재 작업, 프로젝트, 에이전트·모델, 마지막으로 확인한 상태**를 한 화면에서 보는 네이티브 데스크톱 앱입니다. 기존 코딩 에이전트 로그를 읽어 표시하며 에이전트에 명령을 보내지 않습니다.

**v0.2.0은 Windows 사용 가능 단계이며 안정화 진행 중입니다.** 코어와 데스크톱 Rust 패키지는 모두 `0.2.0`입니다. macOS 네이티브 앱은 v0.3.0을 위한 개발 단계이고 Android·iOS/iPadOS·waidaway LAN 접근은 계획입니다. v0.1.0 이후 변경과 업그레이드 방법은 [v0.2.0 릴리스 노트](releases/v0.2.0.md#한국어)를 참고하세요.

[다운로드·검증](#릴리스-다운로드와-검증) · [화면 사용](#화면-사용) · [에이전트 지원](#에이전트별-지원-범위) · [문제 해결](#발견되지-않을-때) · [Mac 개발](MACOS.md)

## 핵심 기능과 편의사항

| 기능 | 할 수 있는 일 |
|---|---|
| 세션 한눈에 보기 | 프로젝트·최근 요청·에이전트·모델·관측 상태를 2초마다 갱신합니다. Claude Code와 Codex가 우선 수집 대상입니다. |
| 대화 미리보기 | 카드를 펼쳐 최근 요청·마지막 기록 답변·첫 요청을 읽고, 앱을 전환하지 않고 요청을 복사합니다. |
| 컨텍스트 확인 | 마지막 기록 사용량과 압축 근거를 표시합니다. Windows 카드·목록은 25% 이하가 남으면 남은 비율을 강조하며, 확인된 압축도 강조합니다. |
| 세션 정리 | 검색·필터·고정·숨김·제외·복원을 지원합니다. 제외한 대화에 구별 가능한 새 사용자 요청이 기록되면 자동 복귀합니다. |
| 작업으로 돌아가기 | 기존 로컬 Orca 세션, 연결한 ChatGPT 링크 또는 직접 지정한 Windows 창으로 이동합니다. [대상별 한계](#세션-돌아가기)가 있습니다. |
| Windows 창 편의 | 펼치는 1열 카드·2열 목록, 항상 위, 투명도 0~60%, 작업 표시줄 최소화, 트레이 숨김·복원, 키보드 조작을 제공합니다. |
| 화면 꾸미기 | Daylight·Midnight JSON 템플릿을 수정·미리보기·적용·가져오기·내보내기 하며 설정을 재시작 후에도 유지합니다. |
| 로컬 도구 | CLI 표, JSON 스트림, localhost HTML 대시보드, 사용자 로그 어댑터와 `waid doctor` 진단을 제공합니다. |

Windows 조작부·상태 표시는 현재 주로 한국어입니다. Mac은 AppKit 기본 조작부와 일부 공용 한국어 상태·상세 텍스트를 사용하며 언어 선택 기능은 없습니다.

## 사용 전 알아둘 점

- **현재 열린 세션 목록을 보증하는 도구가 아닌 로그 뷰어입니다.** 앱을 시작할 때 이전 작업 중·내 차례 기록은 **미확인**으로 시작합니다. 읽을 수 있는 요청·응답 이벤트가 있어야 바뀌며 로그 기록이 늦으면 표시도 늦습니다.
- **수집 지원과 세션 돌아가기 지원은 다릅니다.** 일부 에이전트는 합성 데이터 테스트나 프로세스 감지만 확인했습니다. JSON·SQLite 소스는 확실한 턴 종료 이벤트가 없어 미확인으로 둡니다.
- **컨텍스트는 마지막 기록 측정값입니다.** 실시간 토큰 수나 계정 사용 한도가 아닙니다. Claude 로그에는 컨텍스트 한도 근거가 없어 비율이 미확인이며, 압축 근거가 없다고 압축이 없었던 것은 아닙니다.
- **목록 제외는 waid 안에서만 적용됩니다.** 원본 대화 삭제·에이전트 중단과 무관하고 원래 앱을 여는 것만으로 자동 복귀하지 않습니다. 전체 보기에도 검색·필터가 적용되고 보조 세션은 보조 포함을 켜야 표시됩니다.
- **로컬 설정에 저장된 요청·답변이 포함될 수 있습니다.** 설정 파일당 앱 창 하나를 사용하고 업그레이드 전 설정을 백업하세요. JSON·HTML 내보내기나 화면 캡처를 공유하기 전 내용을 확인하세요.
- **릴리스 서명은 출처와 무결성을 확인합니다.** Sigstore는 Windows Authenticode가 아니므로 Smart App Control·SmartScreen이 EXE를 차단할 수 있습니다. macOS 설치·서명 승인, 접근성·혼합 DPI·장시간 실사용은 검증이 남아 있습니다.

최신 코드 `47f4317`의 2026-09-09 검사에서 [Windows CI](https://github.com/laekhole/what-am-i-doing/actions/runs/34308905362) **130개**, [Mac CI](https://github.com/laekhole/what-am-i-doing/actions/runs/34308905356) **아키텍처별 Rust 검사 121개**와 Apple Silicon·Intel AppKit 스모크·재시작 검사가 통과했습니다. 환경 의존 검사는 Windows 3개, Mac 아키텍처별 1개가 제외(ignored) 상태입니다. 실제 기기에서 남은 확인은 [검증 기록](VALIDATION.md)과 [Mac 안내](MACOS.md)를 참고하세요.

## 개인정보 보호와 로컬 실행

**waid는 사용자 컴퓨터에 내려받아 로컬에서 실행하는 프로그램입니다. 코딩 에이전트 로그를 내 컴퓨터에서 처리하며, 프롬프트·답변·원본 로그·세션 데이터를 개발자나 외부 서버 어디에도 업로드하지 않습니다.** 현재 앱에는 사용 통계 수집(텔레메트리), 분석 서비스, 클라우드 동기화, 외부 AI API 호출이 없습니다. Windows 버전은 설치 마법사 없이 EXE 하나로 실행합니다.

- 수집기는 기존 로컬 로그와 지원하는 SQLite 데이터베이스를 **읽기 전용**으로 읽습니다. 원본 대화를 수정하거나 에이전트에 프롬프트를 보내지 않습니다. Orca SSH 세션도 Orca가 이미 만든 로컬 훅 사본만 읽으며 원격 컴퓨터에 접속하지 않습니다.
- 설정과 저장된 세션 상세 정보는 Windows의 `%LOCALAPPDATA%\waid\settings.json`, macOS의 `~/Library/Application Support/waid/settings.json`에 보관합니다. `WAID_DATA_DIR`로 바꿀 수 있으며, 별도 동기화 프로그램을 통한 복사도 원하지 않으면 로컬 폴더를 지정하세요.
- 네이티브 앱은 로컬 수집기 프로세스를 사용합니다. 선택형 CLI `--html --watch` 대시보드는 **127.0.0.1(localhost)**에서만 데이터를 제공하며 LAN이나 인터넷 인터페이스에 열지 않습니다. 기본 HTML은 로컬 리소스만 사용합니다. 사용자가 넣는 HTML 템플릿에는 스크립트·외부 리소스가 포함될 수 있으므로 이 설명은 기본 템플릿에 해당합니다.
- 사용자가 세션 돌아가기를 누르면 설치된 앱에 세션 식별자를 전달하거나 기존 로컬 창·탭으로 이동합니다. 해당 AI 앱 자체의 통신, 문서·다운로드 링크 열기, 사용자가 내보낸 파일을 공유하는 행위는 waid의 로컬 수집과 별개입니다. 계획 중인 waidaway LAN 기능은 현재 버전에 구현되어 있지 않습니다.

이러한 로컬 처리와 읽기 전용 접근이 현재 구현의 보안 특성입니다. 소프트웨어에 취약점이 전혀 없다는 보장은 아닙니다. 배포 파일의 무결성 확인은 아래 릴리스 검증 절차를, 실제 확인한 범위는 [검증 기록](VALIDATION.md)을 참고하세요.

## 버전 로드맵

| 버전 | 목표 | 현재 상태 |
|---|---|---|
| v0.2.0 | Windows 지원 | 사용 가능, 안정화 진행 중 |
| v0.3.0 | MacBook·데스크톱 Mac을 포함한 macOS 지원 | 네이티브 AppKit 앱 개발 중, 실제 Mac 릴리스 검증 남음 |
| v0.4.0 | Android와 waidaway LAN 접근 | 계획 |
| v0.5.0 | iPhone·iPad용 iOS·iPadOS 지원 | 계획 |
| v1.0.0 | 지원 플랫폼과 핵심 흐름의 안정화 완료 | 향후 안정 버전 |

플랫폼 추가만으로 안정화가 완료되는 것은 아닙니다. 모바일은 PC·Mac에서 수집한 세션을 보는 방향이며 연결 설계는 구현 전입니다. v0.4.0의 제안 범위는 같은 LAN에서 세션 보기와 사용자 요청에 따른 프롬프트 전달입니다. 수집기는 읽기 전용을 유지하고 전달 경로를 분리하며, 원격 입력 출시 전 페어링·인증·암호화·기기 접근 철회·중복 전송 방지가 필요합니다. 원격 접근은 기본 꺼짐으로 계획하고 인터넷 중계는 후속 범위입니다.

Mac 개발·설치는 [MACOS.md](MACOS.md)를 참고하세요. Windows와 같은 Rust 수집기 및 상태·검색·필터·고정·제외/복원·템플릿 로직을 사용합니다. 빌드 대상은 macOS 13.0 이상이며 최소 버전, 설치·서명·실제 세션 복귀는 실기기 검증이 남아 있습니다. [macOS 검사](.github/workflows/macos.yml)는 Apple Silicon·Intel에서 자동 검사를 수행하지만 UI와 실제 에이전트 동작의 검증을 대신하지 않습니다.

## 릴리스 다운로드와 검증

[v0.2.0 릴리스](https://github.com/laekhole/what-am-i-doing/releases/tag/v0.2.0)에서 `waid-v0.2.0-windows-x64.exe`를 내려받아 검증하고 실행하세요. **EXE 하나면 됩니다. 압축 해제, 설치 프로그램, Rust·Cargo·Node·WebView2가 필요 없습니다.** 수집기·글꼴·글꼴 라이선스가 내장되어 있습니다. 나머지 두 파일은 검증용이며 실행 시 필요하지 않습니다. v0.1.0 ZIP 릴리스는 과거 두 EXE 구성을 사용합니다.

처음 실행하면 지원하는 로컬 로그를 자동으로 찾습니다. 세션이 없으면 **전체 보기**를 켜고 검색·필터를 비운 뒤 [진단 안내](#발견되지-않을-때)를 확인하세요. 기존 이력은 새 요청을 관측하기 전까지 미확인으로 보일 수 있습니다.

**v0.1.0에서 업그레이드:** 트레이 메뉴로 기존 앱을 종료하고 `%LOCALAPPDATA%\waid\settings.json`(또는 지정한 `WAID_DATA_DIR`)을 백업한 뒤 검증한 v0.2.0 EXE를 실행하세요. 설정 경로는 유지되며 이전 세션 키는 대응 대상이 유일할 때만 이관합니다. 새 데스크톱 앱에는 과거의 동반 `waid.exe`가 필요 없고 자동 업데이트 기능은 없습니다.

```text
waid-v0.2.0-windows-x64.exe
waid-v0.2.0-windows-x64.exe.sigstore.json
SHA256SUMS.txt
```

같은 릴리스의 세 파일을 내려받은 폴더에서 아래 PowerShell 명령을 실행합니다. `v0.2.0`은 실제 태그로 바꾸세요. [Cosign 설치 안내](https://docs.sigstore.dev/cosign/system_config/installation/)에 따라 Cosign 3.1.3 이상이 필요합니다.

```powershell
$tag = 'v0.2.0'
$exe = "waid-$tag-windows-x64.exe"
$expected = (Get-Content -LiteralPath SHA256SUMS.txt -Raw).Trim()
$actual = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expected -cne "$actual  $exe") { throw '체크섬이 일치하지 않습니다.' }
cosign verify-blob --bundle "$exe.sigstore.json" --certificate-identity "https://github.com/laekhole/what-am-i-doing/.github/workflows/release.yml@refs/tags/$tag" --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' $exe
if ($LASTEXITCODE -ne 0) { throw '릴리스 서명 검증에 실패했습니다.' }
```

SHA-256은 파일의 일치를, Cosign은 **이 저장소의 해당 태그·릴리스 워크플로 신원으로 서명된 파일**인지를 확인합니다. 서명 검증에 실패하면 실행하지 마세요. Sigstore 서명은 Windows Authenticode 서명이 아니므로 **Smart App Control·SmartScreen 실행 허용을 보장하지 않습니다.** 프로젝트의 릴리스 절차는 PC 보안 설정을 변경하지 않습니다.

검토한 커밋에 `v0.2.0` 또는 `v0.2.0-rc.1` 같은 태그를 푸시하면 [릴리스 워크플로](.github/workflows/release.yml)가 테스트·단일 EXE 빌드·검증·체크섬·GitHub OIDC 기반 서명을 수행합니다. 세 파일을 초안 릴리스에 첨부한 뒤 공개하고, 접미사가 있는 태그는 사전 릴리스로 게시합니다. 서명 개인키는 저장소에 보관하지 않으며 Sigstore 공개 로그에는 서명 신원과 아티팩트 다이제스트가 기록됩니다.

태그의 커밋에는 워크플로와 릴리스 본문으로 사용할 `releases/<태그>.md`가 있어야 합니다. 릴리스 워크플로의 Windows 검사가 실패하면 게시하지 않습니다. Mac 검사는 별도로 실행하며 Mac 릴리스를 게시하지 않습니다. 업로드·게시 실패 시 남은 초안을 확인하세요. 기존 릴리스·자산은 자동 덮어쓰지 않습니다. 릴리스 준비 시 태그·노트·두 Cargo 패키지 버전을 맞춰야 하며 파일명만으로 버전 일치를 검사하지는 않습니다.

로컬 서명 전 패키지는 `./desktop/build.ps1 -Portable -Tag v0.2.0`으로 만듭니다. 출력은 `desktop/target/release/bundle`이며 기존 결과를 덮어쓰지 않습니다. 이미 빌드한 EXE는 `./desktop/package.ps1 -Tag v0.2.0 -OutputDirectory <새-출력-폴더>`로 패키징합니다. 로컬 패키징에는 GitHub OIDC 서명이 없습니다.

로컬 애플리케이션 제어 정책이 빌드·테스트를 막으면 [Windows checks](.github/workflows/check.yml)를 이용할 수 있습니다. `main` 푸시·PR·수동 실행 시 검사하고, 성공하면 서명 없는 EXE·체크섬 아티팩트를 7일간 제공합니다. 릴리스를 게시하지는 않습니다. 다운로드한 EXE에도 PC의 실행 정책은 적용됩니다. Authenticode 서명·타임스탬프는 후속 배포 작업이며 [Microsoft 서명 안내](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control)를 참고하세요.

## Windows 소스 빌드

직접 빌드하는 개발자만 Rust·Cargo와 Windows 타깃의 링크 도구가 필요합니다. 저장소 루트에서:

```powershell
cargo test --release --locked
cargo build --release --locked
cargo test --manifest-path desktop/Cargo.toml --release --locked
cargo build --manifest-path desktop/Cargo.toml --release --locked
.\desktop\target\release\waid-desktop.exe
```

GNU 타깃은 양쪽 명령에 `--target x86_64-pc-windows-gnu`를 추가하고 실행 경로에도 해당 타깃 디렉터리를 넣습니다. 데스크톱 EXE는 자신을 내부 수집기 모드로 한 번 더 실행하며 종료할 때 자신이 만든 자식 프로세스만 종료합니다. 별도 `waid.exe`는 CLI용으로 계속 제공됩니다. 트레이 메뉴의 **글꼴 라이선스**에서 내장 라이선스를 열 수 있습니다.

## 화면 사용

아래는 Windows 기준이며 Mac 조작은 [MACOS.md](MACOS.md)를 참고하세요. 기본 창은 **600×780 논리 픽셀**이고 펼칠 수 있는 1열 카드와 선택형 2열 보기를 제공합니다. 카드에는 프로젝트·최근 요청·상태·모델, 하네스 로고가 표시됩니다. Pretendard 본문·제목 글꼴이 내장되어 있습니다. 제목 영역을 끌어 이동하고 가장자리로 크기를 조절합니다.

- **항상 위**와 **투명도(0~60%)**는 재시작해도 유지됩니다. 항상 위는 세션 **고정**과 별개입니다.
- 카드 헤더를 누르면 최근 Prompt·답변·첫 요청을 펼칩니다. Prompt는 줄바꿈을 보존하며 최대 4,000자까지 표시하고 복사 버튼을 제공합니다. 접힌 카드는 한 시간 이내 분, 하루 이내 시간, 이후 `yy-mm-dd`를 표시하며 펼친 카드는 날짜를 표시합니다.
- 하네스 로고나 **세션 열기**로 연결된 앱·창·Orca 세션으로 돌아갑니다. **연결**에서 기존 창이나 복사한 ChatGPT 세션 링크를 지정·해제합니다.
- **최소화**는 작업 표시줄에 남고 **닫기**는 트레이로 숨깁니다. 트레이 아이콘으로 복원하고 **종료**로 앱을 끝냅니다.
- **···**는 상세·검색·정리·템플릿 도구를 펼칩니다. 2열 목록에서는 더블 클릭·Enter로 세션을 엽니다. **검색**(`Ctrl+F`)은 작업·프로젝트·모델·에이전트·폴더와 상태·에이전트 필터를 함께 사용합니다. **간단히**나 Esc로 작은 창으로 돌아옵니다.
- **waid에서 보지 않기**(`Ctrl+D`)는 waid의 기본 목록에서만 제외합니다. 원본 대화를 삭제하거나 에이전트를 중단하지 않습니다. **전체 보기 → 되살리기**로 복원하거나 해당 대화에 **새 사용자 요청**이 기록되면 자동 복귀합니다. 대화를 여는 것만으로는 복귀하지 않습니다. 반복 요청은 소스에 바뀐 요청 ID·시각·메시지 이력이 있어야 구분할 수 있습니다. 제외 기록은 최대 1,024개이며 복원하면 자리가 비워집니다.
- 기본 목록에는 최근 **24시간** 활동, 고정 세션, 이번 실행 중 관측한 세션, 날짜 미확인 세션을 표시합니다. 오래된 이력은 **전체 보기**에서 확인하며 시간이 지났다고 세션을 제외 처리하지 않습니다.
- **고정**(`Ctrl+P`)은 목록 위에 유지하고 **숨기기**(`Ctrl+H`)는 목록에서 감춥니다. **전체 보기**는 과거·제외·숨김 항목을 포함하지만 검색·필터는 계속 적용됩니다. 하위 에이전트·자동 검토·Orca 작업자는 **보조 포함**을 켜야 표시합니다.
- **컨텍스트 사용률**은 로그에 토큰 수와 한도가 모두 있을 때 표시합니다. **25% 이하가 남았거나 압축 기록이 있으면 강조**하고, 접힌 카드와 2열 목록에서도 남은 비율·`압축됨`을 함께 확인할 수 있습니다. Windows 상세에는 토큰 수/한도·관측 시각(UTC)·압축 시각을 표시합니다. 품질 점수·누적 토큰·계정 요금제 잔량이 아니며 없는 값은 미확인으로 둡니다.
- 2초마다 갱신하면서 선택·검색·필터를 유지합니다. 수집기 오류가 나면 마지막 정상 목록과 오류를 유지하고 다음 정상 결과에서 회복합니다. 16 MiB를 넘는 스냅샷은 건너뜁니다.
- `Tab / Shift+Tab`, 방향키·Page Up/Down, 상세 텍스트 선택 후 `Ctrl+C`를 지원합니다.

설정 파일을 여러 창이 공유하면 마지막 저장이 우선하므로 보통 한 창을 사용하세요. 템플릿은 **··· → 템플릿 → 기본 예제 복제 → 수정 → 미리보기 → 적용**으로 바꿉니다. 전체 필드는 [템플릿 안내](TEMPLATES.md)에 있습니다.

## 상태와 작업의 의미

| 상태 | 의미 |
|---|---|
| 유휴 | 명시적 중단 기록이 있습니다. 실행 전 중단도 포함합니다. |
| 작업 중 | 앱 실행 후 새 사용자 요청을 관측했습니다. 응답 종료·중단·오류까지 유지하며 경과 시간만으로 바꾸지 않습니다. |
| 내 차례 | 앱 실행 후 요청한 작업의 응답이 끝났습니다. 다음 요청에서 작업 중으로 바뀝니다. |
| 오류 | 로그 이벤트 자체의 오류입니다. 실행 전 기록도 포함하며 도구 출력의 error 문자열만으로 판정하지 않습니다. |
| 미확인 | 실행 후 요청·응답 경계를 아직 관측하지 못했거나 소스에서 제공하지 않습니다. 실행 전 작업 중·대기 중인 대화도 여기서 시작합니다. |
| 목록 제외 | 사용자가 waid 목록에서 정리한 상태이며 원본 대화·프로세스 종료나 응답 종료와 다릅니다. |

앱은 날짜 제한 없이 기존 로그를 수집한 뒤 최근 활동 보기를 적용합니다. CLI 기본은 최근 24시간이며 `--history`로 제한을 해제합니다. 현재 열린 창이나 생존 세션 목록을 보증하지 않으며 JSON의 `alive: null`은 미확인입니다. 표시 작업은 읽기 범위에서 찾은 최근 실제 사용자 요청이고 `WAID_TASK`·프로젝트 `.waid`·실행 인자 라벨이 우선할 수 있습니다.

관측 기준은 실행할 때마다 새로 시작합니다. 실행 전 작업 중·대기 기록은 별도의 **마지막 기록**으로 표시합니다. 프로세스 활동·파일 갱신만으로 작업 중을 추정하지 않으며 로그 기록이 늦으면 표시도 늦습니다. CLI/JSON은 마지막 로그와 20초 활동 기준을 사용합니다. `request_at`은 사용자 요청의 Unix 초 시각 또는 null이며 요청 식별에는 더 정밀한 소스 시각과 메시지 수도 사용합니다.

## 세션 돌아가기

로그 수집과 기존 창으로 돌아가기는 별개 기능입니다.

| 대상 | 현재 동작 | 한계 |
|---|---|---|
| Orca | 원본 세션·현재 실행과 일치하는 유일한 기존 로컬 터미널로 이동 | 매핑 없음·중복·종료·오래된 매핑은 설명과 상세 표시 |
| ChatGPT App | ChatGPT의 Copy chat deep link(`Ctrl+Alt+L`)를 연결한 뒤 열기 | 해당 Codex 행의 `codex://threads/<thread-id>`만 허용. 링크 전달만으로 실제 이동 성공을 단정하지 않음 |
| PowerShell | 연결에서 선택한 기존 클래식 콘솔 창으로 이동 | Windows Terminal·Orca·VS Code 가상 콘솔 탭과 모호한 중첩 셸은 제외. 창 안의 대화 식별은 별개 |
| ChatGPT / Claude Code 데스크톱 창 | 사용자가 연결한 기존 앱 창으로 이동 | 대화는 앱 안에서 선택. Claude 대화 자동 이동은 지원한다고 주장하지 않음 |

창 핸들·프로세스·실행 파일·시작 시각을 다시 확인하므로 앱·콘솔 재시작 후에는 재연결이 필요합니다. ChatGPT 저장 링크의 실제 이동을 확인한 범위와 전체 연결 UI 흐름의 남은 검증은 [검증 기록](VALIDATION.md)에 있습니다.

Orca 이동에는 현재 훅 매핑과 앱 PATH의 Orca CLI(또는 `ORCA_CLI_COMMAND`로 지정한 실행 파일)가 필요합니다. 로그가 수집되었다고 창 연결까지 확인된 것은 아닙니다. ChatGPT의 허용 경로는 [공식 딥링크 문서](https://learn.chatgpt.com/docs/reference/commands#deep-links)를 따릅니다.

## 발견되지 않을 때

```powershell
.\target\release\waid.exe doctor
```

기본 `~/.claude/projects`, `~/.codex/sessions`, `~/.copilot/session-state`와 명시한 `CODEX_HOME/sessions`, `COPILOT_HOME/session-state`를 읽습니다. 환경변수가 앱 프로세스에 전달되어야 합니다. 다른 경로는 [사용자 어댑터](CLI.md#에이전트-추가하기)를 설정하세요.

Orca SSH는 로컬 `%APPDATA%/orca/agent-hooks/last-status.json` 사본을 읽습니다. Mac은 `~/Library/Application Support/orca/agent-hooks/last-status.json`이며 `ORCA_USER_DATA_PATH`로 Orca 폴더를 바꿉니다. 버전 2 기록은 현재 패널의 실행 정보와 일치해야 합니다. 원격 경로를 로컬 파일로 열지 않습니다. 사본은 패널별 최신 세션만 담아 재시작 후 덮어쓴 이력을 복구할 수 없고 SSH 세션 돌아가기는 미지원입니다.

Windows에서는 프로세스 이름·PID를 열거하고 알려진 실행 파일·인터프리터만 실행 인자를 조회합니다. 작업 디렉터리는 조회하지 않아 개별 로그와 확실히 연결하지 못합니다. 권한·미지원 형식·큰 로그는 누락을 일으킬 수 있습니다. JSONL은 앞 128 KiB·뒤 256 KiB와 캐시한 추가 관측을 사용하며 개별 JSON 파일은 4 MiB까지 읽습니다. 날짜 제한 없는 수집도 전체 대화 보관을 뜻하지 않습니다. `waid doctor`, JSON의 제한된 `warnings`, 앱에서 소스별 경고를 확인합니다.

## 에이전트별 지원 범위

**Claude Code와 Codex가 우선 검증 대상**입니다. 실로그 검증은 이 컴퓨터에서 실제 로그와 비교했다는 뜻이며 제품 전체 지원을 보장하지 않습니다. 픽스처 검증은 합성 입력을 이용한 테스트입니다.

| 에이전트 | 읽는 기록 / 감지 | 검증 수준 |
|---|---|---|
| Claude Code, Codex | JSONL 요청·모델·경로·상태·답변, 실행 감지 | 실로그 |
| Copilot CLI | session-state의 events.jsonl, 직접 실행·Node 패키지 감지 | 픽스처만, 실제 Copilot 세션 미검증 |
| Cursor 편집기 | SQLite state.vscdb의 세션·요청·시각, 상태 미확인 | 로컬 DB 1개 실로그, cursor-agent 감지는 프로세스 스텁 |
| Cline, Roo Code, VS Code Chat | JSON 요청·모델·경로, 프로세스 감지 없음 | 기본 경로 등록·범용 리더 픽스처, 실데이터 미검증 |
| Gemini CLI, opencode, Continue | JSON, 실행 파일·Node/Bun 패키지 감지 | 기본 경로 등록, 감지는 픽스처만 |
| Aider | 실행 파일·Python 감지, Markdown 이력 미지원 | 감지만 |
| Goose | 실행 파일 감지, 이력 미지원 | 감지만 |

JSONL이 아닌 기록은 턴 종료 이벤트가 없어 **미확인**으로 둡니다. 기록 시각이 없으면 파일 시각 추정으로 표시합니다. SQLite는 OS 엔진(Windows `winsqlite3.dll`)을 읽기 전용으로 사용하며 엔진 부재·스키마 변경 시 해당 소스만 비우고 doctor에 이유를 표시합니다. 경로 등록만 된 소스는 실제 형식 차이로 아무것도 표시하지 못할 수 있습니다.

컨텍스트는 Codex의 `token_count.info.last_token_usage.total_tokens`와 `model_context_window`, Claude의 assistant 입력 토큰과 캐시 생성·읽기 토큰을 읽습니다. Claude 로그에 한도가 없으면 모델명으로 추정하지 않아 비율은 미확인입니다. 다른 소스와 Orca SSH 사본도 현재 미확인입니다. 로그의 마지막 측정값이며 아직 보내지 않은 입력의 실시간 재계산이 아닙니다. Codex `compacted`/`context_compacted`, Claude `compact_boundary`를 관측하면 이전 사용량을 비우고 다음 측정을 기다립니다. 읽지 못한 구간이 있어도 사용량을 비웁니다. 압축 기록 미확인은 압축이 없었다는 뜻이 아니며 재시작 후 읽기 범위 밖의 기록은 사라질 수 있습니다.

프로세스만 발견한 경우 상태는 미확인이며 모델은 실행 인자에 명시된 경우만 표시합니다. Cursor 창·VS Code 확장은 프로세스가 아닌 저장 기록으로 읽고 WSL 내부 프로세스 감지는 지원하지 않습니다.

## CLI와 문서

JSON **schema 2**의 `id`는 전체 소스 식별자입니다. 이전 8자리 `legacy_id`는 마이그레이션용이므로 고유 키로 사용하거나 `id`를 자르지 마세요. 데스크톱은 schema 1·2를 읽고 이전 키가 유일하게 연결될 때만 설정을 이관합니다.

각 세션의 `context`에는 nullable `used_tokens`, `window_tokens`, `observed_at`, `compacted_at`과 boolean `compaction_observed`가 있습니다. 시각은 소스 이벤트의 Unix 초이며 파일 수정 시각으로 대체하지 않습니다. `false`는 읽은 압축 근거가 없다는 뜻입니다. 이 필드가 없는 과거 스냅샷·설정도 읽습니다.

```text
waid                  표 한 번 출력
waid --json --watch   JSON 스트림
waid --json --history 이전 날짜의 세션도 포함
waid --waiting        사용자 차례만
waid --agent claude   특정 에이전트
waid --html --watch   로컬 HTML 대시보드
waid doctor           경로·어댑터·프로세스 진단
```

HTML watch 대시보드의 기본 주소는 `http://127.0.0.1:7423`이며 `--port 0`은 빈 포트를 고릅니다. `--interval SEC`으로 CLI 갱신 간격을 바꿉니다(최소 1초). 전체 옵션은 `--help`, 기본 HTML 추출은 `--eject`, 템플릿 필드는 `--keys`입니다. CLI 테마·어댑터는 `~/.config/waid/`(또는 `XDG_CONFIG_HOME/waid`)에 두며 네이티브 JSON 템플릿·설정과 별개입니다. 프로세스 환경변수 작업 라벨은 `/proc` 접근이 필요하므로 Windows 프로세스 열거에서는 사용할 수 없습니다.

[CLI 확장](CLI.md) · [사용자 템플릿](TEMPLATES.md) · [검증 기록](VALIDATION.md) · [제품 범위](PRODUCT.md) · [설계 결정](DECISIONS.md) · [개선 히스토리](HISTORY.md)

[개선 히스토리](HISTORY.md)는 프로젝트 요청마다 변경·검증·남은 일을 한국어로 기록합니다. [작업 규칙](AGENTS.md)에 따라 에이전트가 갱신하는 문서이며 대화 원문 자동 수집 기능은 아닙니다.

## 기여와 AI 지원

[laekhole](https://github.com/laekhole)이 만들고 유지관리하며, Claude Opus 5(Claude Code)와 OpenAI GPT-6(Codex)의 구현·테스트·문서 작성 지원을 받았습니다. AI 기여는 커밋의 `Co-authored-by`에 표시합니다.

MIT
