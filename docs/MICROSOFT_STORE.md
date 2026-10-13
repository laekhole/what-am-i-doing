# Microsoft Store 배포 (MSIX)

상태 (2026-10-14): **Partner Center ID 반영 완료, 제출 전**. Store 배포본은 Microsoft가 서명하므로 SmartScreen·Smart App Control 차단을 피할 수 있다. GitHub의 단일 EXE 배포는 그대로 유지한다. MSIX 파일을 GitHub에서 직접 받게 하면 자체 인증서 서명이 다시 필요하므로, MSIX는 Store 경로로만 배포한다.

## 구성

| 파일 | 역할 |
|---|---|
| [desktop/msix/AppxManifest.xml](../desktop/msix/AppxManifest.xml) | 패키지 매니페스트 템플릿. 기존 EXE를 그대로 담는 full trust 데스크톱 앱 |
| [desktop/msix/identity.json](../desktop/msix/identity.json) | Partner Center가 발급한 패키지 ID (2026-10-14 반영) |
| [desktop/msix/Assets](../desktop/msix/Assets) | EXE·트레이와 같은 캐릭터 타일(`assets/waid-app-icon.png`, 382px)에서 만든 배율(100–200%)·작업 표시줄 크기별 이미지. 원본보다 큰 400% 배율은 만들지 않는다 |
| [desktop/package-msix.ps1](../desktop/package-msix.ps1) | 빌드된 EXE로 `.msix` 생성. `-TestSign`은 일회용 자체 서명 인증서로 서명하고 공개 `.cer`만 남김 |
| [msix.yml](../.github/workflows/msix.yml) | `main`·`msix/**` 브랜치 푸시나 수동 실행 시 Store 업로드용(미서명)과 테스트 서명 MSIX를 artifact로 생성 |

패키지 버전은 `Cargo.toml`의 버전에 `.0`을 붙인다(`0.2.1` → `0.2.1.0`). Store는 마지막 자리가 0이어야 하므로 `-rc` 같은 사전 배포 태그는 MSIX로 만들지 않는다.

## Partner Center 값 반영

앱 이름 예약 후 **제품 관리 → 제품 ID**(영어 화면: Product management → Product Identity)의 값을 `identity.json`에 옮긴다.

| identity.json | Partner Center |
|---|---|
| `name` | Package/Identity/Name |
| `publisher` | Package/Identity/Publisher (`CN=...`) |
| `publisherDisplayName` | Package/Properties/PublisherDisplayName |
| `displayName` | 예약한 앱 이름 |

값이 틀리면 업로드 단계에서 거부된다. 이 값들은 비밀이 아니며 저장소에 커밋해도 된다.

## 로컬 설치 테스트

1. `msix.yml` 실행의 `waid-msix-test-signed` artifact를 받는다.
2. 관리자 PowerShell에서 테스트 인증서를 신뢰 목록에 넣고 설치한다.
   ```powershell
   Import-Certificate -FilePath .\waid-v0.2.1-windows-x64.cer -CertStoreLocation Cert:\LocalMachine\TrustedPeople
   Add-AppxPackage .\waid-v0.2.1-windows-x64.msix
   ```
3. 시작 메뉴의 waid로 실행해 확인한다: 세션 목록 갱신(내부 수집기), 카드 펼침, 설정 저장 후 재실행 시 유지, 트레이 숨김·종료, Orca 복귀, 화면 관찰.
4. 테스트 후 제거한다.
   ```powershell
   Get-AppxPackage *waid* | Remove-AppxPackage
   Get-ChildItem Cert:\LocalMachine\TrustedPeople | Where-Object Subject -eq 'CN=21AB63A8-1BBC-4459-90AE-40236688B5A4' | Remove-Item
   ```

### MSIX에서 달라지는 점

- **설정 파일 위치**: 앱의 `%LOCALAPPDATA%\waid\settings.json` 쓰기는 Windows가 `%LOCALAPPDATA%\Packages\<패키지 이름>\LocalCache\Local\waid\`로 옮긴다. 앱 동작은 같지만 기존 EXE 사용자의 고정·숨김 설정은 넘어오지 않는다. 앱을 제거하면 이 폴더도 지워진다. `WAID_DATA_DIR`을 지정하면 그 경로를 그대로 쓴다.
- **로그 읽기**: `~/.claude`, `~/.codex`, `%APPDATA%\orca` 같은 다른 프로그램의 파일은 가상화 없이 실제 위치에서 읽는다.
- **설치 폴더**: `C:\Program Files\WindowsApps` 아래에 읽기 전용으로 설치된다. 앱은 실행 파일 옆에 쓰지 않는다.
- **자식 프로세스**: 내부 수집기(`--waid-core`), 콘솔 창 조사(`--waid-console-probe`), PowerShell 화면 관찰, Orca CLI는 패키지 안에서도 같은 권한으로 실행된다.

## Store 제출 항목

### 기본 정보

- 가격: 무료. 시장: 전체.
- 범주: 개발자 도구 (Developer tools).
- 개인정보 처리방침 URL: `https://github.com/laekhole/what-am-i-doing/blob/main/docs/USER_GUIDE.md#privacy-and-local-operation`
- 웹 사이트: `https://github.com/laekhole/what-am-i-doing`
- 지원 연락처: `https://github.com/laekhole/what-am-i-doing/issues`
- 연령 등급(IARC) 설문: 사용자 간 상호작용 없음, 사용자 생성 콘텐츠 공유 없음, 위치·개인정보 수집 없음, 구매 없음. 실제 질문 문구에 맞춰 답한다.
- 개인정보 관련 질문("개인 정보에 접근·수집·전송하나요?"): **예**. 수집·전송은 하지 않지만, 개인 정보가 들어 있을 수 있는 로컬 대화 로그를 읽어 화면에 표시하므로 접근에 해당한다. [Store 정책 10.5.1](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)은 접근만으로도 처리방침을 요구하고, Win32·Desktop Bridge 앱은 항상 처리방침이 있어야 한다고 정한다. 로컬 전용이라 외부 전송 동의(10.5.2·10.5.3)는 해당하지 않는다. 처리방침 문서에는 Store 버전의 설정 파일 위치와 제거 시 삭제도 적어 두었다.

### 설명 (한국어)

**짧은 설명**

Orca·Claude Code·Codex 세션의 작업과 마지막 답변을 한 창에서 확인하는 Windows 앱.

**설명**

여러 터미널에서 코딩 에이전트를 돌리다가 "내가 뭐하고 있었지?" 싶을 때, waid가 세션마다 프로젝트·최근 요청·에이전트·모델·상태를 모아 보여줍니다.

- 내 차례 / 작업 중 / 유휴 / 오류 상태로 어느 세션부터 볼지 고릅니다.
- 카드를 펼쳐 마지막 요청과 기록된 답변을 읽습니다.
- 에이전트가 기록한 경우 남은 컨텍스트와 압축 여부를 표시합니다.
- 검색·필터·고정·숨기기로 세션을 정리하고, 지원되는 경우 원래 Orca 세션으로 돌아갑니다.

waid는 PC에 있는 에이전트 로그를 읽기만 합니다. 에이전트를 실행하거나 요청을 보내거나 대화를 수정하지 않으며, 텔레메트리·클라우드 동기화·계정이 없습니다. 오픈소스(MIT)입니다.

### 설명 (English)

**Short description**

See the latest task and answer of every Orca, Claude Code and Codex session in one window.

**Description**

When you have coding agents running in several terminals and wonder "what was I doing?", waid shows each session's project, latest request, agent, model and status.

- Waiting / Working / Idle / Error badges help you choose which session to check next.
- Expand a card to read the last request and the logged answer.
- Remaining context and compaction are shown when the agent logs them.
- Search, filter, pin and hide sessions, and return to the original Orca session where supported.

waid only reads agent logs on your PC. It does not launch agents, send prompts or edit conversations, and has no telemetry, cloud sync or account. Open source (MIT).

### 스크린샷

Store 데스크톱 스크린샷은 1366×768 이상이어야 한다. 기존 750×975 샘플 화면 두 장씩을 1920×1080 배경에 원본 크기로 나란히 배치했다.

- [세션 목록(한국어) + 펼친 카드(영어)](../assets/store/store-1-sessions-expanded.png)
- [세션 목록(영어) + 2열 보기](../assets/store/store-2-sessions-columns.png)

모두 샘플 데이터이며 화면 안에 샘플 표시가 있다. 화면에 타사 에이전트 로고가 보인다(아래 [타사 로고](#타사-로고) 참고). 영어 화면과 2열 보기는 Store에 올리는 0.2.1 빌드에 들어 있다. 스크린샷은 언어별로 1장 이상 필요하며, 같은 이미지라도 언어마다 따로 올린다.

Store 로고(1:1 App tile icon, 300×300)는 [Microsoft 문서](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images) 본문에서는 선택(없으면 패키지 아이콘 사용)이지만 같은 문서 FAQ는 필수라고 해서, 패키지의 300px 타일을 그대로 [store-logo-300.png](../assets/store/store-logo-300.png)로 준비했다. 올리는 편이 안전하다.

### 인증 참고 사항 (Notes for certification)

runFullTrust 사용 이유와 테스트 방법을 적는다.

> waid is a desktop dashboard that reads local log files written by coding-agent CLIs (Claude Code, Codex, Orca) under the user's profile and shows their status. It needs runFullTrust because it is a native Win32 app that reads these files outside its package, enumerates processes and windows to match sessions, and starts its own collector process. It also runs a read-only Windows PowerShell script to observe terminal tabs. It does not send data over the network.
>
> Without any coding agent installed, the app opens with an empty session list; this is expected. Close hides it to the system tray; Exit is in the tray menu.

## 타사 로고

앱의 에이전트 표시에 OpenAI·Claude·Orca 로고가 들어 있다([출처 기록](../desktop/assets/README.md)). 다른 제품과 호환됨을 알리려고 상표를 쓰는 것 자체는 흔하지만, Store 정책 10.1(지식재산권)에 따라 심사에서 문제가 될 수 있다. 현재 방침은 **그대로 제출하고 반려되면 텍스트·일반 아이콘으로 교체**하는 것이다. Store 목록의 이름·아이콘·스크린샷 설명에는 타사 상표를 제품명처럼 쓰지 않는다.

## Partner Center 가입 후 순서

1. **가입 완료**: 개인(Individual) 계정, 본인 확인(신분증) 승인까지 기다린다.
2. **앱 이름 예약**: 앱 및 게임 → 새 제품 → MSIX or PWA app → 이름 예약.
3. **ID 값 전달**: 제품 관리 → 제품 ID(Product management → Product Identity)의 세 값을 `identity.json`에 반영하고 푸시한다. `main`의 `msix.yml` 실행이 만든 `waid-msix-store-unsigned` artifact가 제출할 패키지다.
4. **제출물 작성**
   - 가격 및 가용성: 무료, 시장 전체, 공개. GitHub v0.2.1 릴리스와 같은 시점에 내기로 했으므로 비공개 단계를 두지 않는다.
   - 속성: 범주, 개인정보 처리방침 URL, 웹 사이트, 지원 연락처. 앱이 개인 정보에 접근하는지 묻는 질문은 로컬 대화 로그를 읽으므로 **예**로 답하고 처리방침 URL을 넣는 편이 안전하다.
   - 연령 등급: IARC 설문.
   - 패키지: 위 `.msix` 업로드. Identity가 예약 값과 다르면 여기서 거부된다.
   - Store 목록(한국어·영어): 위 설명과 [스크린샷](../assets/store).
   - 제출 옵션 → 인증 참고 사항: 위 runFullTrust 사용 이유와 테스트 방법.
5. **제출**: 제출과 같은 시점에 GitHub v0.2.1 릴리스(노트·README 링크·태그)를 낸다. 인증은 최대 3영업일이며, 반려 사유가 오면 그에 맞춰 고친다.
6. **게시 후**: Store 링크를 README에 넣고 `release.yml`을 연결한다.

**버전**: 첫 제출은 `0.2.1`(패키지 0.2.1.0)이다. `main`에는 GitHub v0.2.0 이후 변경(영어 UI, 트레이 집계, 답변 알림 등)이 있어 0.2.0으로 올리면 GitHub v0.2.0과 내용이 달라진다. 0.3.0은 macOS 실기 검증을 마친 릴리스에 쓰므로 Windows 변경만 담은 이번 배포는 0.2.1로 하고, GitHub v0.2.1 릴리스와 내용을 맞춘다.

## 업데이트 제출

- Partner Center에서 새 제출(Start update)을 만들고 새 `.msix`를 올려 다시 인증받는다. 인증은 [최대 3영업일](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-certification-process)이며, 게시 후 기존 사용자에게는 Store가 자동으로 업데이트한다.
- 패키지 버전은 이전 제출보다 높아야 하고 마지막 자리는 계속 0이다. 따라서 Store에 올릴 때마다 `Cargo.toml` 버전을 올린다(0.2.1 → 0.2.2).
- 설명·스크린샷 같은 목록 정보만 바꿀 때도 새 제출로 인증을 거친다.

## 이후 작업

- 첫 제출이 통과하면 `release.yml`에서 안정 태그마다 Store 업로드용 MSIX를 만들도록 연결.
- README 설치 안내에 Store 링크 추가 (게시 후).
