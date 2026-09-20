# 프로젝트 개선 히스토리

어떤 요청으로 무엇이 바뀌었고, 어디까지 확인했는지 기록한다. 2026-09-09부터 프롬프트별 기록을 시작한다.
에이전트가 [작업 규칙](AGENTS.md)에 따라 갱신하는 문서이며, 대화 원문을 자동 수집하는 기능은 아니다.

## 기존 기록과의 관계

- 이 문서: 요청 → 변경 내용과 이유 → 검증 → 남은 일.
- [설계 결정](DECISIONS.md): 선택한 방식과 대안, 설계 이유.
- [검증 기록](VALIDATION.md): 테스트·실환경 확인 결과와 미검증 범위.
- Git 커밋: 실제 파일 차이. `git log --oneline`으로 목록을, `git show <커밋>`으로 변경 내용을 확인한다. 커밋 하나가 프롬프트 하나에 대응하지는 않는다.

## 작성 형식

프로젝트 관련 요청마다 아래 형식으로 `프롬프트별 작업 기록` 맨 위에 추가한다. 같은 요청을 이어서 작업하면 해당 항목을 갱신한다. 변경 없는 조사·답변은 변경 항목에 `변경 없음`과 결과를 적는다. 요청은 요약하며 민감정보나 대화 원문을 복사하지 않는다.

```markdown
### YYYY-MM-DD HH:mm +09:00 — 작업 제목

- 요청: 사용자가 원한 결과 요약.
- 상태: 완료 / 부분 완료 / 차단 중.
- 변경·이유: 이전 상태에서 무엇을 어떻게 바꿨으며 왜 필요한지.
- 관련 파일: 수정한 파일 링크. 기존 커밋이 있으면 참조 가능.
- 검증: 실제 실행한 검사와 결과. 미실행이면 이유, 미검증이면 범위를 명시.
- 남은 일: 후속 작업·제약. 없으면 없음.
```

## 프롬프트별 작업 기록
### 2026-09-20 15:24 +09:00 — Mac 테스트 인계 문서 커밋·푸시

- 요청: 앞서 작성한 Mac 실기 테스트 인계 문서를 커밋하고 원격에 푸시하기.
- 상태: 변경 검토 완료 — 이 기록과 문서 3개를 커밋한 뒤 origin/main에 푸시한다.
- 변경·이유: 협업자가 저장소에서 테스트 절차를 받을 수 있도록 인계 문서·Mac 안내 링크·작업 기록을 함께 반영한다. 제품 코드 변경 없음.
- 관련 파일: [MACOS_HANDOFF.md](MACOS_HANDOFF.md), [MACOS.md](MACOS.md), [HISTORY.md](HISTORY.md).
- 검증: 변경 범위 확인, 원격 fetch 및 `git diff --check` 통과. 커밋·푸시 후 원격 HEAD 일치와 깨끗한 작업 트리를 확인한다. 문서만 변경하여 빌드·Mac 실기 검사는 재실행하지 않는다.
- 남은 일: 커밋·푸시 결과는 Git 이력으로 확인. Mac 협업자의 실기 테스트와 결과 제출은 별도 진행.

### 2026-09-20 15:21 +09:00 — Mac 협업자 실기 테스트 인계 문서 작성

- 요청: 저장소 협업 권한을 받은 Mac 사용자가 바로 실행·테스트할 수 있는 인계 문서 마련.
- 상태: 완료 — 테스트 절차·결과 양식 작성. Mac 실기 실행은 담당자 진행 전.
- 변경·이유: 소스 받기, 도구 확인, 자동 검사·개발 패키징, 별도 설정으로 실행, 실제 에이전트 수집·한영 전환·저장·종료 확인과 결과 보고 양식을 한 문서로 정리했다. Finder 환경변수 차이, 다운로드 설치·서명 제한, 현재 미구현 기능을 구분해 잘못된 통과 판정을 방지했다. 기존 Mac 안내에서 연결했다.
- 관련 파일: [테스트 인계](MACOS_HANDOFF.md), [Mac 안내](MACOS.md), [HISTORY.md](HISTORY.md).
- 검증: README·최근 기록·Mac 안내·실제 패키징 스크립트·CI 명령과 절차를 대조했다. 문서 로컬 링크 및 `git diff --check` 확인. 문서만 변경했으며 Windows 환경에서 Mac 빌드·테스트·실기를 실행하지 않았다. 기존 검증 근거는 [VALIDATION.md](VALIDATION.md)를 참고한다.
- 남은 일: 문서 커밋·원격 반영 후 Mac 협업자가 실제 테스트 결과 제출. 이번 요청에서는 커밋·푸시·타인에게 메시지 전송을 수행하지 않았다.

### 2026-09-14 18:13 +09:00 — 누적 변경 커밋·푸시

- 요청: 현재 변경 사항을 커밋하고 원격 저장소에 푸시하기.
- 상태: 완료 — 제품 변경 커밋과 origin/main 푸시 완료.
- 변경·이유: 누적된 한국어·영어 전환, 트레이·알림, 서명 준비와 문서 등 35개 파일을 `1952ca2`로 저장하고 푸시했다. 이번 요청의 제품 코드 추가 변경 없음.
- 관련 파일: [언어 선택](src/i18n.rs), [트레이](desktop/src/native/tray.rs), [서명 준비](SIGNING.md), [라이선스](LICENSE), [설계](DECISIONS.md), [검증](VALIDATION.md), [HISTORY.md](HISTORY.md) 등.
- 검증: 원격 fetch와 스테이징 차이 검사 통과, `1952ca2` 푸시 성공 및 깨끗한 작업 트리 확인. 최초 기록 삽입은 줄바꿈 차이로 적용되지 않아 이번 문서 커밋으로 보완했다. 빌드·테스트는 재실행하지 않았으며 기존 Windows 정책 차단·Mac 미검증 제한은 유지한다.
- 남은 일: 제품 변경 반영에는 없음. 이 완료 기록도 커밋·푸시한 뒤 최종 동기화를 확인한다.

### 2026-09-14 17:53 +09:00 — 개인 오픈소스 배포의 무료 서명 신청 준비

- 요청: 개인 GitHub 오픈소스라는 배포 조건을 반영해 Windows 공개 서명 경로를 구체화하기.
- 상태: 부분 완료 — SignPath 신청 자료와 EXE 제품 메타데이터 준비·로컬 검증 완료. 신청 제출·심사·실제 서명·차단 해소는 미완료.
- 변경·이유: 기존 MIT 선언의 누락된 전문, README 제거 안내와 Code signing policy 연결, 영어 신청 초안과 승인 전/후 작업 구분을 추가했다. 개인 유지관리자 신청이 가능함을 확인하되 초기 프로젝트의 평판 심사 통과를 단정하지 않았다. 기존 Windows 리소스 빌드에 Cargo 버전 기반 제품명·파일/제품 버전을 추가하고 실행 없이 읽는 검사 하나를 남겼다. 새 의존성은 없다. 서명 서비스 승인 전에는 기존 릴리스 워크플로우를 변경하지 않는 이유와 후속 서명 순서는 [D38](DECISIONS.md#d38--오픈소스-windows-코드-서명-준비-2026-09-14)에 기록했다.
- 관련 파일: [라이선스](LICENSE), [신청 초안·정책](SIGNING.md), [README](README.md), [Windows 리소스 빌드](desktop/build.rs), [메타데이터 검사](desktop/test-version-info.ps1), [설계](DECISIONS.md), [검증](VALIDATION.md), [HISTORY.md](HISTORY.md).
- 검증: 공식 조건·실제 신청폼·공개 GitHub 릴리스 확인. MSVC release 빌드와 EXE 메타데이터 검사 통과, 이전 메타데이터 없는 EXE의 거부 확인. 문서 로컬 링크와 `git diff --check` 통과. EXE는 `NotSigned`이며 실행하지 않았다. GNU 빌드와 전체 앱 회귀 검사는 미실행. 정확한 명령·결과·제한은 [검증 기록](VALIDATION.md#오픈소스-서명-신청과-exe-메타데이터-준비--2026-09-14)을 참고한다.
- 남은 일: 검토한 준비 파일의 공개 저장소 반영, 유지관리자 계정·MFA와 구성요소 라이선스 확인, 신청자 연락처·약관 동의·reCAPTCHA를 포함한 공식 폼 제출과 평판 심사. 승인 후 서명 서비스 연결·수동 승인·서명/체크섬/Sigstore 검증 및 실제 Windows 다운로드·실행 확인이 필요하다. 사용자 연락처를 추정하거나 저장하지 않았고 기존 변경·실행 앱·설정을 보존했다. 이번 작업에서 신청·가입·결제·커밋·푸시·릴리스·OS 정책 변경은 수행하지 않았다.

### 2026-09-14 17:47 +09:00 — Windows 네이티브 앱 기준으로 점검 범위 정정

- 요청: 앞선 출시 체크리스트 점검이 Windows 네이티브 응용프로그램이라는 점을 반영했는지 확인.
- 상태: 완료 — Windows 출시 기준으로 답변 범위와 우선순위를 명확히 했다.
- 변경·이유: 제품 코드·설정 변경 없음. 앞선 답변에서 선택형 HTML과 개발 중인 Mac 문제를 함께 강조한 범위 혼선을 정정했다. Windows의 오류 안내, 데이터 저장·삭제 안내와 지원 경로, 라이선스 전문, 접근성·DPI·최종 실행 검증에 초점을 맞췄다. 웹 SEO·파비콘·모바일 CTA와 HTML/Mac 전용 문제를 Windows 네이티브 출시 필수 보완으로 취급하지 않는다. 개인정보·라이선스 안내도 별도 웹페이지 구축 요구가 아니다.
- 관련 파일: [HISTORY.md](HISTORY.md). 기존 점검 근거: [Windows UI](desktop/src/native.rs), [사용 안내](README.md), [검증 기록](VALIDATION.md).
- 검증: 최신 작업 기록과 직전 소스 점검 결과를 재확인했다. `git diff --check -- HISTORY.md` 통과. 범위 설명만 정정했으므로 빌드·자동 테스트·실행 중 앱 조작은 수행하지 않았다.
- 남은 일: 범위 확인 답변에는 없음. Windows 구현 보완과 기존 미검증 사항은 앞선 점검 결과대로 남는다.

### 2026-09-14 16:58 +09:00 — 웹 출시 체크리스트 20개를 앱 기준으로 점검

- 요청: 웹사이트 출시 요소 20개를 waid가 데스크톱 응용프로그램이라는 차이에 맞춰 점검하기.
- 상태: 완료 — 현재 미커밋 변경을 포함한 소스·문서·자산 조사와 HTML 오류 재현 완료. 보완 구현은 이번 요청 범위 밖이다.
- 변경·이유: 제품 코드·설정 변경 없음. SEO·가입 전환 요소는 네이티브 앱과 localhost 대시보드의 적용 여부를 구분했다. Windows 폼 오류가 수집 경고에 가려지는 표시 우선순위, HTML 갱신 실패 후 실시간 표시 유지, Mac 초기 로딩·빈 결과 안내 부족을 확인했다. 개인정보 안내는 README에 있으나 저장 데이터 삭제 안내·명시적 지원/비공개 보안 제보 경로·프로젝트 MIT 라이선스 전문은 찾지 못했다. 추적 없는 현재 제품에 쿠키 배너·분석 설치를 누락 기능으로 간주하지 않았다.
- 관련 파일: [HISTORY.md](HISTORY.md). 조사 근거: [README.md](README.md), [Windows 표시·폼](desktop/src/native.rs), [카드 접근성](desktop/src/native/accordion.rs), [Mac 상태](desktop/src/macos.rs), [Mac UI](desktop/src/macos/App.swift), [설정 저장](desktop/src/ui.rs), [HTML](src/default.html), [HTML 렌더러](src/html.rs), [로컬 서버](src/serve.rs), [템플릿 이스케이프](src/tmpl.rs), [이미지 사용](desktop/src/visual.rs), [릴리스 흐름](.github/workflows/release.yml), [기존 검증 기록](VALIDATION.md).
- 검증: README·최신 HISTORY·소스 호출 경로와 자산 크기를 읽기 전용으로 대조했다. 실제 HTML 스크립트를 Node v24.19.0 vm·가짜 DOM/응답에서 실행해 네트워크 실패 후 live 유지, 연결 끊김 뒤 HTTP 500에서 live 재표시의 오류 재현 2개를 assert로 확인했다(종료 코드 0; 정상 동작 통과를 뜻하지 않음). Windows 폼 오류 가림과 Mac 안내 부족은 정적 경로 확인이며 실기 재현은 미실행이다. 활성 ICO의 PNG 압축 프레임 9개와 1024px·353 KiB Orca 원본의 32 논리 px 표시를 확인했다. HTML 348px 미만 넘침은 CSS에 근거한 예상이며 실제 브라우저 미검증이다. `git diff --check -- HISTORY.md` 통과. 코드 변경이 없어 Cargo 빌드·회귀 검사는 재실행하지 않았고 실제 사용자 앱·설정·대화와 기존 변경을 보존했다.
- 남은 일: 발견한 오류 피드백·Mac 초기/빈 상태 보완, 개인정보 접근·삭제·지원/신고 안내와 라이선스 전문 정리, 접근성·좁은 창·혼합 DPI 실기 확인. 마지막 영문 배지 변경 이후 Windows 실행/테스트 정책 차단과 Mac 최신 빌드 미검증은 [VALIDATION.md](VALIDATION.md)의 기존 미완료 사항이며 이번 점검으로 해소하지 않았다. 체크리스트 검토는 전체 보안 검증을 의미하지 않는다.

### 2026-09-14 16:57 +09:00 — Windows 앱 제어 차단 원인과 공개 배포 서명 조사

- 요청: 웹 공개 전에 Windows 앱 제어에 차단되는 문제를 해결할 수 있는 배포 방법 확인.
- 상태: 부분 완료 — 로컬 차단 원인 진단·서명 방식 조사 완료, 실제 인증서 발급·서명·차단 해소 검증은 미완료.
- 변경·이유: 제품 코드·릴리스 구성·OS 설정 변경 없음. 현재 EXE의 미서명 상태와 Smart App Control 차단 근거를 확인하고, 기존 Sigstore 배포 검증에 Windows용 RSA Authenticode 서명과 타임스탬프를 추가하는 경로를 제안했다. [Microsoft 서명 요구사항](https://learn.microsoft.com/en-us/windows/apps/develop/smart-app-control/code-signing-for-smart-app-control)과 [SmartScreen 평판 안내](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation)에 따라 코드 서명과 신규 다운로드 평판을 구분했다. 서명만으로 모든 PC의 정책·경고 해소를 보장하지 않는다.
- 관련 파일: [HISTORY.md](HISTORY.md). 조사 대상: [릴리스](.github/workflows/release.yml), [빌드](desktop/build.ps1), [패키징](desktop/package.ps1), [배포 후보](target/release/waid-desktop-bilingual.exe).
- 검증: `Get-AuthenticodeSignature` 결과 `NotSigned`, 서명자·타임스탬프 없음. Smart App Control 레지스트리 값 `VerifiedAndReputablePolicyState=1`과 CodeIntegrity 로그의 해당 EXE·테스트 바이너리 차단 이벤트 3077/3033을 확인했다. 현재 사용자·로컬 컴퓨터 인증서 저장소의 코드 서명 인증서는 0개였다. `CiTool -lp -json`은 접근 거부되어 전체 정책 목록은 확인하지 못했다. 릴리스는 EXE 복사·SHA256SUMS 생성 후 Sigstore 서명만 수행함을 읽기 검토했다. Microsoft·인증기관·SignPath 공식 문서에서 한국 조직용 Artifact Signing, 개인용 IV 인증서, 무료 오픈소스 심사, Store MSIX 배포 조건을 대조했다. `git diff --check -- HISTORY.md` 통과. 조사·기록만 변경하여 빌드·자동 테스트는 재실행하지 않았다.
- 남은 일: 배포자 명의와 서명 서비스 확정, 외부 본인/조직 확인과 인증서 발급 또는 무료 지원 승인. 이후 최종 EXE에 Authenticode 서명·타임스탬프 → 서명 검증 → SHA256SUMS 재생성 → Sigstore 서명 순서를 릴리스에 연결하고 Smart App Control이 켜진 환경에서 실제 다운로드·실행을 확인해야 한다. 가입·결제·신청 제출·릴리스 게시·OS 정책 변경은 수행하지 않았다.

### 2026-09-14 16:28 +09:00 — 앱 한국어·영어 전환과 Reddit·GeekNews 소개 초안

- 요청: 외부 사용자 피드백을 받기 위해 기존 한국어 앱에 영어를 추가하고 Reddit·GeekNews 소개글 초안을 준비. 범위 확인 결과 실제 게시·웹사이트 공개는 제외.
- 상태: 구현·초안·최종 EXE 빌드 완료 — 중간 빌드 회귀·실화면 확인 완료, 마지막 배지 수정 이후 테스트·EXE 실행 검증은 Windows 정책에 차단.
- 변경·이유: 기존 설정에 언어 선택을 저장하고 Windows·Mac UI와 CLI·로컬 HTML의 제품 문구를 번역했다. 새 설치는 시스템 언어를 감지하고 기존 설정은 한국어를 유지하며 대화 원문은 보존한다. 새 의존성 없이 공용 언어 선택을 사용한다. 기존 트레이·알림의 미커밋 작업을 보존했다. 설계와 수집 진단의 재시작 제약은 [D37](DECISIONS.md#d37--한국어영어-전환과-공개-소개-준비-2026-09-14)을 참고한다.
- 관련 파일: [언어 선택](src/i18n.rs), [Windows UI](desktop/src/native.rs), [카드](desktop/src/native/accordion.rs), [트레이](desktop/src/native/tray.rs), [공용 표시](desktop/src/main.rs), [설정](desktop/src/ui.rs), [Mac UI](desktop/src/macos/App.swift), [Mac 연결](desktop/src/macos.rs), [세션 복귀](desktop/src/activate.rs), [Windows 복귀](desktop/src/activate/windows.rs), [CLI](src/lib.rs), [HTML](src/default.html), [렌더러](src/html.rs), [로컬 서버](src/serve.rs), [검사](tests/cli.rs), [소개 초안](LAUNCH.md), [사용 안내](README.md), [CLI 안내](CLI.md), [Mac 안내](MACOS.md), [검증](VALIDATION.md), [설계](DECISIONS.md).
- 검증: core 95개·CLI 7개·desktop 35개·단일 EXE 1개로 총 138개 통과, 환경 의존 4개 ignored. 언어 저장·이전 설정 호환·원문 유지·저장 실패 복원과 Windows 한국어/영어 컨트롤·필터·샘플 카드 렌더링을 확인했다. 마지막 HTML 보완의 template 2개·HTTP/SSE 1개도 통과했다. 배지 문구를 축약한 최종 desktop 테스트 바이너리와 EXE는 Windows 앱 제어 정책(os error 4551)으로 실행 전 차단됐다. 최종 release 빌드·전달본 해시 일치·`git diff --check`는 통과. [최종 EXE](target/release/waid-desktop-bilingual.exe), 진단·렌더러·검사 변경과 자세한 결과는 [검증 기록](VALIDATION.md#한국어영어-전환과-소개글-준비--2026-09-14)을 참고한다. Mac Swift 빌드·스모크·실기는 미실행.
- 남은 일: 정책이 허용하는 환경에서 마지막 배지 수정 이후 테스트·EXE 실행 및 Mac 검증. 기존 사용자 앱·설정과 이전 미커밋 변경을 보존했으며 샘플만 종료했다. 실제 공개에는 새 영문 포함 릴리스와 소개글 게시가 필요하고 이번 요청에서는 커밋·푸시·배포·게시·정책 변경을 하지 않았다.

### 2026-09-14 11:01 +09:00 — 트레이 세션 요약·바로가기와 답변 도착 알림 구현

- 요청: 앞선 제안 중 가장 적절한 방향으로 창을 상시 띄우지 않는 waid 사용 경험 개선.
- 상태: 구현·실행 파일 준비 완료 — 기본 회귀 검사와 Windows 알림 표시 확인 완료. 보강한 추가 통합 검사 1건은 OS 정책 차단으로 미검증.
- 변경·이유: 기존 트레이에 상태 요약·새 답변/오류 배지·내 차례 세션 바로가기·무음 묶음 알림·알림 설정과 1시간 쉬기를 추가했다. 요청 단위로 중복을 막고 사용자 차례와 새 답변 표시를 분리했다. 검색 필터 독립성, 숨김/제외/보조 세션 배제, 수집 오류, 첫 닫기 안내 및 복귀 실패 시 마지막 답변 상세를 보완했다. 기존 수집·복귀를 재사용해 서비스·의존성을 추가하지 않았다. 설계는 [D36](DECISIONS.md#d36--창을-숨긴-동안의-트레이와-답변-알림-2026-09-14)에 기록했다.
- 관련 파일: [트레이 상태·알림·검사](desktop/src/native/tray.rs), [Windows 창](desktop/src/native.rs), [카드 확인](desktop/src/native/accordion.rs), [설정](desktop/src/ui.rs), [사용 안내](README.md), [제품 범위](PRODUCT.md), [설계](DECISIONS.md), [검증](VALIDATION.md), [새 EXE](target/release/waid-desktop-tray.exe), [HISTORY.md](HISTORY.md).
- 검증: 제품 동작 변경을 반영한 desktop 33개·단일 EXE 1개 통과, 기존 환경 의존 3개 ignored. 별도 샘플 EXE의 숨김 중 갱신·실제 트레이 메뉴를 확인했고, 명시적으로 실행한 네이티브 알림 검사는 Windows의 NIN_BALLOONSHOW 수신을 확인했다. 이후 같은 검사를 일반 수집 갱신 경로까지 보강한 최종 테스트 바이너리는 컴파일 후 Windows 애플리케이션 제어 정책(os error 4551)에 차단되어 미실행이다. 최종 release 빌드와 EXE 라이선스 실행·전달본 해시 일치·`git diff --check` 통과. 초기 경고/테스트 컴파일 오류와 PowerShell 출력 파이프 오류는 수정·호출 보완 전 결과로 구분했다. 자세한 근거는 [검증 기록](VALIDATION.md#트레이-세션-요약과-답변-알림--2026-09-14)에 있다.
- 남은 일: 기존 waid를 트레이에서 종료하고 새 EXE 실행으로 적용. 정책이 허용하는 환경에서 보강한 알림 통합 검사 재확인. 실제 에이전트 대상별 복귀·배너 클릭·방해 금지·장시간/혼합 DPI 검증은 남아 있다. 기존 앱·설정·대화를 보존했고 검증용 앱과 수집기만 종료했으며 정책 변경·우회·커밋·배포는 하지 않았다.

### 2026-09-14 10:11 +09:00 — 창을 상시 표시하지 않는 트레이·알림 사용 방식 제안

- 요청: waid를 최소화하거나 닫아 둔 상태에서도 세션의 답변 종료와 사용자 차례를 확인할 수 있는 방향 및 보완 대안 제안.
- 상태: 완료 — 현재 구현 조사와 기능 방향 제안. 구현은 요청 범위에 포함되지 않음.
- 변경·이유: 제품 코드·설정 변경 없음. 기존 최소화·트레이 숨김·백그라운드 수집과 정적인 트레이 메뉴를 확인했다. 상태 요약·세션 바로가기·새 답변 알림을 우선 제안하고, 사용자 차례와 새 알림 확인 여부를 구분했다. 전역 단축키·일시적인 요약 패널은 후속 대안으로 제시했다. 기존 설계 이력은 [설계 결정](DECISIONS.md)을 참고한다.
- 관련 파일: [HISTORY.md](HISTORY.md). 조사 근거: [README.md](README.md), [제품 범위](PRODUCT.md), [Windows 창·트레이](desktop/src/native.rs), [실행 이후 상태 관찰](desktop/src/main.rs), [목록 정책](desktop/src/ui.rs).
- 검증: README·최신 기록·트레이 등록/메뉴/숨김·수집 갱신·상태 관찰·필터 코드를 읽고 Microsoft의 트레이 표시 및 방해 금지 공식 안내와 대조했다. `git diff --check -- HISTORY.md` 통과, 변경 파일은 HISTORY.md만 확인했다. 문서 변경만 있어 빌드·자동 테스트·실행 중 앱 조작은 수행하지 않았다.
- 남은 일: 제안 답변에는 없음. 기능 구현·알림 API의 포터블 EXE 적용 검증·실사용 검증은 후속 작업이다. 과거 답변 재알림 방지, 신뢰할 수 있는 답변 종료 근거, 알림 중복 방지 및 수집 오류 표시는 구현 시 반영해야 한다.

### 2026-09-11 15:01 +09:00 — 미커밋 변경 확인 및 커밋·푸시

- 요청: 작업 트리에 커밋할 변경이 있는지 확인하고, 있으면 커밋 후 원격에 푸시하기.
- 상태: 완료.
- 변경·이유: 기존 작업 트리의 코드·문서·자산 변경과 이 기록을 함께 커밋하고 원격 `main`에 푸시한다.
- 관련 파일: [변경 목록](DECISIONS.md), [작업 기록](HISTORY.md), [검증 기록](VALIDATION.md), [코드·자산 변경](src/session.rs).
- 검증: 원격 동기화 상태 확인, 스테이징된 `git diff --check` 통과, `0ae3ef6` 커밋 생성 및 `origin/main` 푸시 완료. 최종 `git status`는 깨끗하고 로컬·원격 HEAD가 일치한다.
- 남은 일: 없음.

### 2026-09-10 14:06 +09:00 — PowerShell SSH 에이전트 화면 감지 구현

- 요청: 기존 로컬 앱과 PowerShell에서 인식하는 에이전트에 더해 일반 SSH 안의 코딩 에이전트도 waid에 표시하기.
- 상태: 부분 완료 — 로컬 화면 관찰 구현·회귀 검사·실제 카드 접근성 확인·새 실행 파일 준비 완료. 숨은 tmux 전체 수집은 추가 SSH 인증 불가로 미구현.
- 변경·이유: 로컬 프로세스는 ssh.exe까지만 보인다는 원인을 확인했다. 원격 조회를 한 번 시도했으나 비대화형 인증이 거부돼, 원격 설치 없는 UI Automation 보조 경로를 추가했다. 노출된 Codex·Claude Code 화면에서 에이전트와 추정 입력/작업명·보이는 Codex 모델을 읽고 `화면 관찰` 항목으로 표시한다. 탭·문서 구분, 읽기/시간 상한, 캐시, 진단과 비활성화 환경변수를 추가했다. 상태·컨텍스트·원격 경로·대화 ID는 추정하지 않는다. 설계·제약은 [D35](DECISIONS.md#d35--일반-ssh의-로컬-화면-관찰-지원-2026-09-10)에 기록했다.
- 관련 파일: [화면 판별·검사](src/terminal.rs), [Windows UIA 조회](src/terminal.ps1), [수집 통합](src/session.rs), [진단](src/lib.rs), [사용 안내](README.md), [설계 결정](DECISIONS.md), [검증 기록](VALIDATION.md), [새 실행 파일](target/release/waid-desktop-ssh.exe), [HISTORY.md).
- 검증: 최종 core release 94개+CLI 6개, desktop 31개+단일 EXE 1개 통과, 환경 의존 3개 ignored. 초기 수정본의 테스트는 Windows 정책으로 실행 차단됐으며, 화면 하단 인식 등을 보완한 최종 코드의 검사는 정상 실행됐다. 실제 SSH Codex 화면을 최종 CLI가 1개로 감지하고 모델·추정 작업·Unknown 상태와 비어 있는 대화 ID/원격 경로를 확인했다. 별도 설정으로 새 앱을 실행해 카드의 접근성 텍스트·갱신을 확인했고 캡처는 다른 창에 가려 육안 근거에서 제외했다. 새 EXE 해시 일치·내장 수집·비활성화 스위치·라이선스·`git diff --check` 통과. 자세한 결과는 [검증 기록](VALIDATION.md#powershell-ssh-화면-감지--2026-09-10)에 기록했다.
- 남은 일: 기존 waid를 종료한 뒤 새 EXE 실행으로 적용. 별도 인증 연결과 숨은 tmux pane·비활성 탭·정확한 상태 수집은 지원하지 않는다. 사용자의 기존 앱·기본 설정·다른 요청의 미커밋 수정사항을 보존했고 검증용 앱과 그 수집기만 종료했다.

### 2026-09-10 13:54 +09:00 — 남은 컨텍스트를 프로젝트와 시간 사이로 이동

- 요청: 남은 컨텍스트 n%를 프로젝트명과 n시간 전 사이에 표시하기.
- 상태: 부분 완료 — 소스·안내 수정과 release EXE 생성 완료. Windows 애플리케이션 제어 정책으로 자동 테스트 및 최종 EXE 실행 검증이 차단됨.
- 변경·이유: 컨텍스트를 첫 줄의 프로젝트명 오른쪽·시간 왼쪽으로 옮기고 세로 정렬을 공유했다. 글자 너비를 측정해 배지 공간을 확보하고 프로젝트에 최소 표시 폭을 남겼다. 좁은 창·큰 글꼴에서 예약 폭이 가용 폭을 넘지 않도록 제한했다. 과거 상태는 기존 하단에 유지하며 직전 요청에서 줄인 카드·본문 높이는 변경하지 않았다. 실행 중인 기존 EXE와 사용자 설정은 보존하고 수정본은 별도 이름으로 준비했다.
- 관련 파일: [카드 배치](desktop/src/native/accordion.rs), [사용 안내](README.md), [새 EXE](target/release/waid-desktop-context.exe), [HISTORY.md](HISTORY.md).
- 검증: `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline`는 컴파일 후 테스트 프로세스가 정책(os error 4551)으로 실행되지 않았다. 초기 수정 EXE의 125% 밝은 테마 [펼침](target/context-inline-preview/expanded.png)·[접힘](target/context-inline-preview/collapsed.png)에서 프로젝트→컨텍스트→시간→상태 순서와 세로 정렬·클릭 전환을 직접 확인했다. 이후 좁은 폭 예약 상한 한 줄을 보완한 최종 `cargo build`는 종료 코드 0이며 복사본과 빌드 산출물이 일치한다. 최종 EXE는 GUI와 `--licenses` 모두 Windows 정책으로 실행 차단되어 성공으로 기록하지 않는다. `git diff --check` 통과. 기존 [이전 검증 기록](VALIDATION.md#카드-정렬높이-개선--2026-09-10)의 통과 결과와 구분한다.
- 남은 일: Windows 정책이 허용하는 환경에서 최종 EXE 실행과 자동 검사 재확인. 정책 변경·우회는 하지 않았다. 원래 실행 중인 앱에는 재실행 전까지 이전 배치가 표시된다.

### 2026-09-10 10:51 +09:00 — 다중 탭·tmux 수집 방식 이해 확인

- 요청: PowerShell 다중 탭 감지의 한계와 tmux를 확인하려면 waid가 별도로 SSH에 접속해야 하는지 정리.
- 상태: 완료 — 기존 조사 결과의 적용 범위 설명.
- 변경·이유: 제품 코드·설정 변경 없음. 로컬 에이전트는 지원 로그로 탭과 무관하게 수집할 수 있고, SSH 화면 관찰은 비활성 탭·숨은 tmux pane에 한계가 있음을 구분했다. 원격 설정 없이 tmux 전체를 조회하는 후보는 waid의 별도 SSH 연결이며, pane 조회와 정확한 작업 상태 판단은 별개임을 재확인했다.
- 관련 파일: [HISTORY.md](HISTORY.md). 기존 근거: [설계 검토 D34](DECISIONS.md#d34--ssh-터미널-관찰과-tmux-수집-검토-2026-09-10), [상세 검증](VALIDATION.md#ssh-터미널-텍스트-비용탭-구조-조사--2026-09-10), [사용 안내](README.md).
- 검증: 최신 기록과 직전 조사·실측 결과를 바탕으로 설명했다. `git diff --check -- HISTORY.md` 통과. 추가 실측·원격 접속·빌드·테스트는 수행하지 않았다.
- 남은 일: 설명 요청에는 없음. 별도 SSH 수집은 미구현이며 인증·원격 조회 권한·상태 근거 검증이 필요하다.

### 2026-09-10 10:45 +09:00 — SSH 화면 감지 성능·다중 탭·상태·tmux 한계 검증

- 요청: 터미널 글자 읽기의 성능 비용, 여러 탭의 식별, 세션 상태 판단 방법을 확인하고 tmux 중심 사용 환경을 반영하기.
- 상태: 완료 — 현재 Windows Terminal의 UI 구조·텍스트 호출 지연 실측과 공식 코드/문서 검토 완료. 원격 tmux 실행·수집 구현은 미실행.
- 변경·이유: 제품 코드·설정 변경 없음. 실제 탭 5개에 본문 1개만 노출되는 점과 제목 중복을 확인했다. 본문 읽기 지연은 작았지만 숨은 tmux pane과 작업 상태를 충분히 수집할 수 없어 화면 관찰은 보조 경로로 한정하는 검토를 남겼다. 별도 SSH를 통한 기존 tmux 조회는 원격 설치 없이 가능한 후보이나 추가 인증·조회 권한과 정확한 상태 근거가 필요하다.
- 관련 파일: [설계 검토 D34](DECISIONS.md#d34--ssh-터미널-관찰과-tmux-수집-검토-2026-09-10), [상세 실측·한계](VALIDATION.md#ssh-터미널-텍스트-비용탭-구조-조사--2026-09-10), [HISTORY.md](HISTORY.md). Git 무시 로컬 산출물: [측정 스크립트](.tools/ssh-observation/measure-uia.ps1), [측정값](.tools/ssh-observation/uia-measurements.json).
- 검증: Orca Computer Use 읽기 전용 구조 조회 및 Windows UI Automation MTA 직접 호출 3종 각 10회 측정과 길이 상한 검사 통과. 초기 진단 스크립트 형식 오류·제목 TextPattern 포함 문제는 수정한 최종 결과와 구분해 상세 기록했다. Microsoft Terminal·UIA·OpenSSH·tmux 공식 자료 및 기존 상태 로직 대조. 문서·측정 JSON·스크립트 구조와 `git diff --check` 확인. 제품 코드 변경이 없어 Cargo 빌드·테스트 미실행. 터미널 본문 원문은 저장하지 않았다.
- 남은 일: 조사 요청에는 없음. 장시간 CPU·입력 지연·고속 출력·다른 탭/터미널, 실제 tmux 조회·추정 상태 정확도는 미검증. 기존 세션 입력·탭 전환·원격 접속·사용자 설정 변경은 하지 않았으며 다른 요청의 수정사항을 보존했다.

### 2026-09-10 10:08 +09:00 — 원격 설정 없는 SSH 에이전트 감지 가능성 조사

- 요청: 원격 서버에 설정을 추가하기 어려운 환경에서 waid가 PC 쪽 정보만으로 SSH 내부 에이전트를 알아챌 수 있는지 확인.
- 상태: 완료 — 코드·공식 문서 기반 가능 경로와 한계 설명. 구현·실환경 검증은 하지 않음.
- 변경·이유: 제품 코드·설정 변경 없음. 로컬 SSH 실행·명령줄만으로 접속 후 실행한 에이전트를 확정할 수 없지만, Windows Terminal·기본 콘솔의 UI Automation 텍스트를 관찰하는 경로는 있음을 확인했다. 이를 이용한 화면 기반 감지를 제안하며 세션 식별·작업 완료·컨텍스트 정확성을 보장하지 않는다. waid가 SSH를 새로 실행해 출력을 받는 ConPTY 방식과 이미 허용된 원격 파일 접근을 사용하는 대안도 조사했으나 채택하지 않았다.
- 관련 파일: [HISTORY.md](HISTORY.md). 조사 근거: [프로세스 열거](src/proc/windows.rs), [식별 조건](src/matchers.rs), [세션 수집](src/session.rs), [기존 창 연결](desktop/src/activate/windows.rs), [사용 안내](README.md).
- 검증: 최신 기록·수집/창 연결 코드와 [Microsoft Terminal 접근성 문서](https://github.com/microsoft/terminal/blob/main/doc/terminal-a11y-2023.md), [ConPTY 문서](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session), [OpenSSH](https://man.openbsd.org/ssh.1)·[SFTP](https://man.openbsd.org/sftp.1) 매뉴얼 대조. `git diff --check -- HISTORY.md` 통과. 코드 변경이 없어 빌드·테스트 미실행. 실제 터미널 텍스트·비활성 탭·tmux·권한별 감지율은 미검증.
- 남은 일: 조사 답변에는 없음. 구현 시 지원할 터미널에서 텍스트 읽기와 SSH 창/탭 연결을 먼저 검증하고, 화면에서 확인할 수 없는 값은 알 수 없음으로 유지해야 한다. 기존 다른 요청의 수정사항을 보존했다.

### 2026-09-10 10:06 +09:00 — 카드 정렬과 높이 개선

- 요청: 프로젝트 위치를 기준으로 시간·상태·남은 컨텍스트를 정렬하고 긴 카드 목록의 UI/UX를 개선하기.
- 상태: 완료 — 배치·여백 수정, 자동 검사와 실제 화면 확인, target/release 실행 파일 갱신 완료. 2026-09-10 10:12 +09:00 갱신.
- 변경·이유: 샘플 실화면에서 컨텍스트가 프로젝트명을 밀어내고 시간·상태가 아래로 처지는 문제를 확인했다. 프로젝트·시간·상태에 같은 세로 영역을 사용하고 컨텍스트는 요청 아래에서 프로젝트 글자와 왼쪽을 맞췄다. 시간 너비를 실측하고 상태 열을 제한해 제목 공간을 확보했다. 과거 상태는 하단 오른쪽에 두어 한 세션 때문에 전체 카드가 한 줄씩 늘어나는 공용 상태를 제거했다. 기본 14px 글꼴에서 접힌 카드 높이는 104/125→94 논리 픽셀, 펼친 본문은 526→426으로 줄였다. 미리보기는 스크롤을 유지하고 세션 열기·목록 제외 버튼은 한 줄로 배치했다. 기존 로고와 다른 미커밋 작업을 보존했다.
- 관련 파일: [카드 표시·기존 검사](desktop/src/native/accordion.rs), [사용 안내](README.md), [검증 기록](VALIDATION.md), [새 EXE](target/release/waid-desktop.exe), [HISTORY.md](HISTORY.md). 화면 산출물은 무시된 target/card-layout-preview에 보관한다.
- 검증: 최종 release 검사에서 데스크톱 31개·단일 EXE 1개 통과, 환경 의존 3개 ignored. 실제 125% 배율에서 밝은/어두운 테마의 접힘·펼침과 최소 480 논리 픽셀 폭을 확인했다. 다른 창에 가렸거나 초기화 중인 캡처는 증거에서 제외했다. 최종 EXE와 빌드 산출물 해시 일치·내장 라이선스 실행과 `git diff --check` 통과. 구체적인 명령·화면·제한은 [검증 기록](VALIDATION.md)에 남겼다.
- 남은 일: 이번 카드 개선 요청은 없음. 새 EXE 재실행으로 반영된다. 다른 DPI·최대 사용자 글꼴·Mac 실화면·공개 배포는 미실행이며 사용자 기본 설정은 보존했다.

### 2026-09-10 09:58 +09:00 — PowerShell SSH 에이전트 표시 지원 범위 확인

- 요청: PowerShell에서 SSH로 접속해 실행한 코딩 에이전트가 waid에 표시되는지 확인.
- 상태: 완료 — 현재 문서·수집 코드 기준 지원 범위 확인.
- 변경·이유: 제품 코드·설정 변경 없음. 일반 PowerShell SSH는 원격 프로세스·로그를 자동 수집하지 않으므로 기본적으로 표시되지 않는다. Orca SSH는 로컬에 미러링된 유효한 v2 훅을 통해 Codex·Claude 세션을 수집하는 별도 경로임을 확인했다.
- 관련 파일: [HISTORY.md](HISTORY.md). 조사 근거: [사용 안내](README.md#troubleshooting-missing-sessions), [세션 수집](src/session.rs), [Windows 프로세스 수집](src/proc/windows.rs), [Orca 훅 수집](src/orca.rs), [로그 경로](src/adapters.rs).
- 검증: README·최신 기록과 로컬 로그/프로세스 및 Orca 훅 수집 경로를 대조했다. `git diff --check -- HISTORY.md` 통과. 코드 변경이 없어 빌드·테스트는 실행하지 않았으며 사용자의 실제 PowerShell SSH 세션은 조사하지 않았다.
- 남은 일: 지원 범위 답변에는 없음. 일반 SSH 세션 표시에는 별도 원격 로그 전달·수집 연동이 필요하며 이번 요청에서 구현하지 않았다.

### 2026-09-10 09:17 +09:00 — 모바일·PC 외부 연결 원안 저장과 Astra 검토

- 요청: 모바일·Desktop 외부 연결 원안을 지정한 Markdown 파일로 저장하고 Astra에게 치명적 문제·과도한 복잡성, 특히 WebRTC·TURN·Cloudflare 동시 도입 필요성을 검토시키기.
- 상태: 완료 — 원안 보존, GPT-6 Astra 독립 검토와 저장소 대조, 공식 문서 기반 검토 기록 작성 완료. 설계 채택·구현은 하지 않음.
- 변경·이유: 제공된 아키텍처 본문 30개 절을 새 파일로 저장했다. 원안을 고치지 않고 [설계 검토](DECISIONS.md#d33--모바일pc-외부-연결-아키텍처-astra-검토-2026-09-10)에 QR 신뢰 결합, E2E 종단, 브라우저 전제, 권한 해제·중복 명령, 익명 중계 남용, OS·NAT 한계를 기록했다. WebRTC와 TURN은 같은 연결 경로이며 사용자별 Tunnel/custom relay를 함께 넣는 부분이 중복이라는 판단, 최소 구성·운영비 산식·질문 12개 답을 정리했다. 기존 localhost 서버·SSE와 현재 모바일 브라우저 우선 계획의 제약도 대조했다. 제품 소스·의존성·README 로드맵 변경 없음.
- 관련 파일: [원안](mobile-pc-connect-architecture.md), [검토·설계 근거](DECISIONS.md#d33--모바일pc-외부-연결-아키텍처-astra-검토-2026-09-10), [HISTORY.md](HISTORY.md).
- 검증: UTF-8·원안 번호 1~30·코드 블록 43개 닫힘 확인. 원안과 검토 문서의 구조·추가 로컬 링크 및 `git diff --check` 확인. RFC 8827/8831/8445/8656, Cloudflare Tunnel·TURN, W3C, Chrome, Apple·Android 공식 문서 조회. 코드 변경이 없어 빌드·테스트는 미실행. 실기·공격 재현·NAT 성공률·비용 측정은 하지 않았으며 [기존 검증 기록](VALIDATION.md)과 구분한다.
- 남은 일: 이번 저장·검토 요청은 없음. 구현 시 클라이언트 형태, pairing 프로토콜·라이브러리, 운영 주체·예산, background·원격 입력 범위 결정과 실환경 검증 필요. 검토는 미채택 제안이며 기존 수정사항을 보존했다.

### 2026-09-10 08:58 +09:00 — 기존 아이콘 복원과 상단 마스코트 렌더링 수정

- 요청: assets에 복원한 기존 로고로 롤백하고 물음표 시안은 question_mark에 보관하기. 좌측 상단 아이콘의 깨진 모양을 수정하고 실행 파일을 target/release 경로에 준비하기.
- 상태: 완료 — 롤백·렌더링 수정·요청 경로의 EXE 생성과 실제 밝은/어두운 화면 확인 완료. 2026-09-10 09:05 +09:00 갱신. 자동 테스트는 Windows 정책으로 실행 차단됨.
- 변경·이유: 복원된 382px 앱 타일·408px 마스코트를 사용하며 사용자 question_mark 보관본은 유지했다. 고정 96px HICON을 화면 배율에 맞춰 다시 축소하던 상단 경로를, 원본 투명 PNG에서 실제 크기로 한 번 보간하는 GDI+ 렌더링으로 바꿨다. 이미지보다 메모리 스트림을 오래 유지하고 함께 해제하며 중간 96px PNG는 제거했다. 변환 대상 PNG가 미리보기에서 열려 덮어쓰기에 실패해 임시 파일 완성 후 교체하도록 수정했다. 이전 EXE는 desktop/target/context-label/release에 있었고 요청한 루트 target/release에는 없었음을 확인했다.
- 관련 파일: [이미지 렌더링·검사](desktop/src/visual.rs), [상단 표시](desktop/src/native.rs), [Windows API 기능](desktop/Cargo.toml), [아이콘 변환](assets/build-icons.ps1), [자산 안내](assets/README.txt), [내장 자산 안내](desktop/assets/README.md), [내장 아이콘](desktop/assets/waid.png), [새 EXE](target/release/waid-desktop.exe), [검증 기록](VALIDATION.md), [HISTORY.md](HISTORY.md).
- 검증: 원본·재생성 PNG/ICO가 Git의 기존 자산과 일치, 최종 EXE 내장 아이콘 9개 일치·기존 마스코트 포함·물음표 PNG 미포함 확인. release 빌드와 --licenses 실행 성공. 실제 125% 배율의 밝은/어두운 테마에서 상단 마스코트를 직접 확인했다. 테스트는 컴파일 성공 후 Windows 정책(os error 4551)으로 실행 차단되어 통과로 기록하지 않는다. `git diff --check` 통과. 구체적 명령·화면·검증 한계는 [검증 기록](VALIDATION.md)에 남겼다.
- 남은 일: 요청한 롤백·파일 준비는 없음. 전체 자동 테스트와 다른 DPI·Mac 실화면 검증은 미실행. 정책 변경·우회와 공개 배포는 하지 않았다.

### 2026-09-10 08:54 +09:00 — waidaway 사용자 호스팅과 외부 접속 대안 검토

- 요청: 개발자 운영 중계서버 없이 사용자가 웹서버를 실행하는 초기 waidaway에서 외부 접속에 공인 IP가 필요한지와 대안 확인.
- 상태: 완료 — 문서 기반 조사·설명. 연결 방식 확정과 구현은 하지 않음.
- 변경·이유: 제품 코드·설정·로드맵 변경 없음. 직접 IPv4 접속의 공인 주소·포트포워딩 조건, 고정 IP와 공인 IP의 차이, DDNS의 CGNAT 한계, IPv6 및 사용자 계정의 Tailscale·Cloudflare Tunnel 대안을 비교했다. 개발자 서버 운영을 없애는 목표에는 Tailscale을 초기 후보로 제안하되 외부 연결 조정·중계 서비스 의존은 구분한다.
- 관련 파일: [HISTORY.md](HISTORY.md). 현재 범위 확인: [README.md](README.md).
- 검증: README와 최신 작업 기록 확인. [Tailscale 연결 방식](https://tailscale.com/docs/reference/connection-types)·[Serve](https://tailscale.com/docs/features/tailscale-serve), [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/), ASUS DDNS·AVM IPv6 공식 문서를 조회했다. 문서 기록의 `git diff --check -- HISTORY.md` 통과. 코드 변경이 없어 빌드·테스트는 미실행이며 실제 회선·휴대폰 외부 접속은 미검증.
- 남은 일: 사용자 설치 부담과 브라우저 단독 접속 요구에 따라 방식 선택. 구현 시 인증·기기 해제와 실제 외부망 접속 검증 필요.

### 2026-09-10 08:40 +09:00 — 물음표 마스코트 로고와 앱 아이콘 적용

- 요청: 사용자가 수정한 물음표 로고의 크기를 확인하고 필요한 변환 후 앱 로고·아이콘·설정 등 사용 위치에 적용하기.
- 상태: 완료 — 필요한 크기 변환·참조 적용·Windows 테스트·새 EXE 빌드 완료. 2026-09-10 08:47 +09:00 검증 기록 갱신.
- 변경·이유: 새 앱 타일과 마스코트는 1254px 정사각형으로 원본을 보존했다. 기존 변환 스크립트로 이전 그림이 남아 있던 256px 앱 PNG와 누락된 9개 크기의 ICO를 갱신했다. 화면 확인에서 큰 마스코트를 Windows 아이콘으로 바로 로딩할 때 거친 외곽선이 보여, 기존 bicubic 축소 코드를 재사용해 상단용 96px 투명 PNG를 만들고 참조를 바꿨다. 앱의 32 논리 픽셀 표시 크기는 유지했다. README·Mac 패키징은 이미 교체된 원본 경로를 참조한다. 자산 안내의 이전 치수와 사용 위치를 바로잡았다. 이전 요청의 소스 수정과 사용자 old/ 보관본은 유지했다.
- 관련 파일: [아이콘 변환](assets/build-icons.ps1), [Windows 이미지 로딩](desktop/src/visual.rs), [상단 마스코트](desktop/assets/waid-mascot.png), [Windows 앱 이미지](desktop/assets/waid.png), [Windows 아이콘](assets/waid.ico), [원본 크기·적용 안내](assets/README.txt), [내장 자산 안내](desktop/assets/README.md), [검증 기록](VALIDATION.md), [HISTORY.md](HISTORY.md). 사용자 제공 원본은 assets/waid-*.png. 새 실행 파일은 [waid-desktop-question-logo.exe](desktop/target/context-label/release/waid-desktop-question-logo.exe).
- 검증: Windows 데스크톱 30개 통과·환경 의존 3개 ignored, 96px 파생본 적용 후 네이티브 디코딩·창/설정 검사 재통과. PNG/ICO의 크기·투명도·구조와 최종 EXE의 아이콘 리소스 9개 일치, release 빌드·내장 라이선스 실행·`git diff --check` 통과. 초기 샘플 상단 물음표는 직접 확인했지만 최종 캡처는 다른 이미지 창에 가려진 한계가 있다. 상세 명령·범위는 [검증 기록](VALIDATION.md)에 남겼다.
- 남은 일: 요청한 파일 적용은 없음. 실제 사용은 새 EXE 실행이 필요하다. 설치된 앱 갱신·아이콘 캐시, Mac 실행/패키징·공개 배포와 최종 화면 재확인은 미실행.

### 2026-09-09 17:04 +09:00 — 시간 표시 원위치 복구와 컨텍스트 인접 배치 정정

- 요청: ‘n분 전’ 위치를 바꾸지 말고 원래 자리 옆에 ‘남은 컨텍스트 n%’를 표시하도록 이전 수정을 바로잡기.
- 상태: 완료 — 위치 수정, 데스크톱 테스트와 새 실행 파일 빌드 완료.
- 변경·이유: 앞선 요청을 잘못 해석해 시간까지 하단으로 옮긴 배치를 수정했다. 시간/날짜의 좌우·상하 좌표와 오른쪽 정렬을 변경 전과 동일하게 복구하고, 실제 글자 너비를 측정해 남은 컨텍스트를 바로 왼쪽에 배치했다. 제목 영역은 컨텍스트와 겹치지 않도록 줄이고 영문 안내를 정정했다. 잔여율 계산과 강조 조건은 앞선 수정 그대로 사용한다.
- 관련 파일: [카드 배치](desktop/src/native/accordion.rs), [README.md](README.md), [촬영용 미리보기 실행 파일](C:/waid-demo/launch-assets/desktop-setup/launch-waid-preview.cmd), [HISTORY.md](HISTORY.md).
- 검증: `git diff --check` 통과. 기존 빌드 경로에서 `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline --bin waid-desktop` 실행: 데스크톱 30개 통과, 실환경 연결 3개 ignored. 시간 표시 영역과 정렬을 Git의 변경 전 코드와 대조했다. 새 출력 경로의 최초 전체 테스트는 Windows 애플리케이션 제어 정책이 새 serde_core 빌드 스크립트를 차단해 실패했다. 정책을 변경하지 않고 이미 빌드된 의존성 캐시로 앱 검사를 완료했다. 별도 샘플 창의 Computer Use 캡처는 다른 창에 가려져 육안 위치 검증으로 인정하지 않았고, 직접 띄운 샘플 프로세스만 종료했다.
- 추가 검증: 같은 의존성 캐시에서 `cargo rustc`의 최종 바이너리 이름만 `-C extra-filename=-position`으로 지정해 release 빌드 종료 코드 0을 확인했다. 생성물을 `desktop/target/context-label/release/waid-desktop-position.exe`로 복사했다. PowerShell의 직접 출력 캡처는 파이프 종료로 실패했지만 Python `subprocess.run`으로 종료를 기다리며 캡처한 `--licenses`는 종료 코드 0·내장 라이선스 확인을 통과했다. 촬영용 미리보기 실행 파일도 이 새 경로로 갱신했다.
- 남은 일: 요청한 코드 수정은 없음. 실제 화면 육안 검증·사용 중인 앱 재시작·공개 배포는 미실행.

### 2026-09-09 16:55 +09:00 — 컨텍스트 위치 수정본으로 촬영용 waid 재시작 준비

- 요청: 촬영용 waid의 컨텍스트 위치가 이전 배치로 보이는 상황에서 재시작 방법 안내.
- 상태: 완료 — 수정 빌드용 실행 파일 준비와 수집 확인, 실제 GUI 재시작은 사용자 실행 단계로 안내했다.
- 변경·이유: 촬영용은 기존 공개 v0.2.0 EXE를 사용하고 있어 같은 실행 파일 재시작만으로는 배치가 바뀌지 않는다. 기존 촬영용 환경변수 다섯 개를 그대로 쓰고 컨텍스트 위치 수정 빌드를 실행하는 `launch-waid-preview.cmd`를 추가했다. 로컬 미리보기 빌드임을 명시했으며 공개 배포본과 기존 실행 파일은 변경 없음.
- 관련 파일: [미리보기 실행 파일](C:/waid-demo/launch-assets/desktop-setup/launch-waid-preview.cmd), [HISTORY.md](HISTORY.md).
- 검증: 실행 중인 촬영용 EXE 경로와 수정 빌드의 존재·수정 시각·서로 다른 SHA-256을 확인했다. 두 실행 파일의 환경변수 설정 다섯 개가 일치하며, 수정 빌드의 내부 JSON 수집기를 촬영용 환경으로 실행해 종료 코드 0·데모 프로젝트 두 개·SSH 0개를 확인했다. 기존 트레이 종료 처리의 설정 저장과 실제 종료 경로를 소스로 확인했다. `git diff --check` 통과. 앱 코드 변경이 없어 빌드·단위 테스트는 반복하지 않았고 새 GUI 위치의 육안 확인은 미실행이다.
- 남은 일: 사용자가 촬영용 waid를 트레이 메뉴에서 종료한 뒤 새 미리보기 실행 파일을 열어 화면 확인. 실제 재시작·공개 릴리스 교체는 하지 않았다.

### 2026-09-09 16:38 +09:00 — Orca SSH 신규 세션 미표시 원인 확인

- 요청: Orca에서 새로 연 SSH 세션 두 개가 waid에 보이지 않는 이유와 지원 여부 확인.
- 상태: 완료 — SSH 수집 정상과 현재 보이는 촬영용 창의 수집 제한을 확인했다.
- 변경·이유: 제품 코드·실행 설정 변경 없음. Orca SSH 훅 수집은 이미 지원한다. 일반용과 촬영용 waid가 동시에 실행 중이며, 현재 노출된 촬영용 창은 전용 Codex 로그와 빈 Orca 훅 경로만 사용해 SSH 세션을 수집하지 않는 구성임을 확인했다.
- 관련 파일: [HISTORY.md](HISTORY.md). 조사 근거: [Orca 수집기](src/orca.rs), [사용 안내](README.md), [촬영용 실행 설정](C:/waid-demo/launch-assets/desktop-setup/launch-waid.cmd).
- 검증: Orca CLI에서 연결된 원격 Codex 터미널 두 개를 확인하고 로컬 v2 훅의 현재 실행 매핑·세션 식별자와 대조했다. 일반용 실행 파일의 `--waid-core --json --history`에서 두 세션 모두 `orca_hook`·`working`·보조 세션 아님으로 수집됐고 저장된 숨김·제외·검색·상태 필터에도 걸리지 않았다. 촬영용 실행 환경을 별도 셸에서 재현하면 데모 프로젝트 두 개와 SSH 0개가 나온다. UI 접근성 조회에서도 현재 보이는 창은 촬영용이며 데모 프로젝트 두 개만 표시했다. 일반용 프로세스는 존재하지만 Computer Use 창 목록에는 나타나지 않아 해당 GUI의 실제 표시는 미검증이다. 문서만 변경하여 빌드·단위 테스트는 실행하지 않았고 `git diff --check`를 확인했다.
- 남은 일: 원인 조사에는 없음. 일반용 창 복원·전환과 촬영용 설정 변경은 하지 않았다.

### 2026-09-09 16:21 +09:00 — 남은 컨텍스트 문구와 시간 옆 배치

- 요청: 컨텍스트를 ‘남은 컨텍스트’로 명시하고 ‘n분 전’ 옆에 표시하기.
- 상태: 완료 — 소스·기존 회귀 검사·안내 수정, 별도 실행 파일 빌드와 데스크톱 테스트 통과.
- 변경·이유: 사용률과 잔여율을 조건에 따라 바꾸던 요약을 항상 잔여율로 통일했다. 0% 미만은 0%로 표시하며 미확인·25% 이하 강조·압축 기록은 유지했다. Windows 카드의 요청 미리보기 아래에 시간/날짜와 남은 컨텍스트를 같은 줄에 붙여 제목 공간을 확보했다. 공용 요약을 쓰는 목록·상세·접근성 문구와 기존 테스트 기대값, 영문 사용 안내도 갱신했다.
- 관련 파일: [공용 요약](desktop/src/main.rs), [카드 배치](desktop/src/native/accordion.rs), [회귀 검사](desktop/src/ui.rs), [README.md](README.md), [HISTORY.md](HISTORY.md).
- 검증: `git diff --check` 통과. 최초 데스크톱 release 테스트는 셸 PATH의 rc.exe 누락으로 실패했고, 설치된 Windows SDK 경로를 해당 명령에만 추가했다. 이후 실행 중인 기본 출력 EXE 교체가 접근 거부로 실패하여 별도 경로를 사용했다. `cargo test --manifest-path desktop/Cargo.toml --target-dir desktop/target/context-label --release --locked --offline` 종료 코드 0: 데스크톱 30개·단일 EXE 1개 통과, 기존 실환경 연결 3개 ignored. 잔여율 100%·0%·25% 경계·한도 초과·미확인·압축 후 잔여율과 실제 네이티브 카드 갱신·접기 검사를 포함한다. 기존 코어 dead_code 경고 2개가 남는다.
- 남은 일: 요청한 코드 수정은 없음. 새 실행 파일은 `desktop/target/context-label/release/waid-desktop.exe`에 생성했다. 실행 중인 앱 교체·재시작, 화면 육안 확인, Mac 실행 검증·배포는 하지 않음.

### 2026-09-09 15:51 +09:00 — 기존 Orca 창의 촬영 데스크톱 이동 안내

- 요청: 데스크톱 1에서 실행 중인 Orca가 촬영용 데스크톱 2에서 열리지 않는 상황 해결.
- 상태: 창 이동 확인 완료 — 2026-09-09 16:17 +09:00 계속 작업. 사용자가 이동을 알린 뒤 Windows API로 Orca와 waid가 현재 같은 가상 데스크톱에 있음을 확인했다. 전체 데모·촬영 준비는 계속 진행 중.
- 변경·이유: 제품 변경 없음. 재실행 대신 Windows 작업 보기에서 기존 Orca 창을 데스크톱 2로 이동하도록 촬영 안내를 보완했다. 기존 앱·세션을 종료하지 않았다.
- 관련 파일: [가상 데스크톱 안내](C:/waid-demo/launch-assets/virtual-desktop-runbook.md), [HISTORY.md](HISTORY.md).
- 검증: Orca 창 목록에서 실행 중인 창 하나와 도구 focus·moveResize 미지원 확인. 사용자 이동 후 IsWindowOnCurrentVirtualDesktop/GetWindowDesktopId 조회 S_OK로 현재 데스크톱 소속을 확인하고 데모 터미널로 전환했다. Orca의 도구 좌표에는 배율 오류가 있어 캡처가 잘렸고, 읽기 전용 DWM 조회로 실제 오른쪽 작업영역 1920×1032를 확인했다. 이동 자체는 사용자가 수행했다.
- 남은 일: 새 PowerShell·Claude·ChatGPT 창은 다른 가상 데스크톱에 있어 한 번에 이동을 요청했다. 기존 실제 세션·촬영 준비 계속.

### 2026-09-09 15:20 +09:00 — 녹화 담당과 네 앱을 사용하는 데모 구성 확대

- 요청: 로그인 완료 후 에이전트가 녹화를 맡고, PowerShell·Orca·Claude 앱·Codex 앱을 함께 사용해 waid의 기능을 보여 주기.
- 상태: 부분 완료 — 2026-09-09 16:20 +09:00 계속 작업. Orca·클래식 PowerShell에서 실제 Codex 세션 두 개의 요청 수집과 작업 중·내 차례 표시를 대조했다. Claude 앱·Codex 앱 새 세션과 리허설·본 녹화는 미실행.
- 변경·이유: 촬영 담당을 에이전트로 바꾸고 네 실행 환경의 실제 세션을 확인하는 동선으로 갱신했다. Orca에 데모 todo-app 폴더와 전용 Codex 홈으로 실행하는 터미널 하나를 만들었다. 클래식 PowerShell용 실행 스크립트를 준비했으나 아직 실행하지 않았다. 제품 소스·릴리스 변경과 인증·업무 로그 복사는 없음.
- 관련 파일: [촬영 실행표](C:/waid-demo/launch-assets/recording-plan.md), [가상 데스크톱 안내](C:/waid-demo/launch-assets/virtual-desktop-runbook.md), [검증 요약](C:/waid-demo/launch-assets/verification-summary.md), [시작 안내](C:/waid-demo/launch-assets/START-HERE.md), [PowerShell 실행 준비](C:/waid-demo/launch-assets/desktop-setup/launch-notes-powershell.ps1), [HISTORY.md](HISTORY.md).
- 검증: 설치된 FFmpeg의 gdigrab 사각형 캡처·프레임률 옵션, libx264 합성 입력 인코딩 및 tty 세션 q 정상 종료를 확인했다. 화면 녹화·MP4 저장 검사로 확대하지 않는다. DPI-aware 조회에서 주 화면 물리 1920×1080, 작업영역 1920×1020, 125% 배율 확인. 사용자 로그인 완료 보고와 별도 Codex 홈의 인증 파일 존재만 확인했으며 내용은 읽지 않았다. Orca 생성 결과와 터미널의 새 폴더 신뢰 확인 대기 상태를 읽었다. 입력은 agent_prompt_blocked로 거부되어 우회하지 않았다. 데모 터미널 대상 화면 캡처에 작업 전환 화면과 업무 창들이 포함되는 것을 발견하여 촬영 자료로 사용하지 않았다. v0.2.0은 transcript 디렉터리 복수 지정은 지원하지만 파일별·Orca pane별 읽기 제한은 없음을 소스로 확인했다.
- 추가 확인: 설치 Orca의 hook producer는 앱 전역 프로필에 연결 정보를 기록하고 매번 파일을 교체한다. 터미널별 hook 경로는 확인되지 않아 업무 세션을 읽지 않는 조건에서 Orca 자동 복귀 검증은 남긴다. 새 데모 JSONL의 원본 하드링크는 파일 내용을 바꾸지 않는 후보지만 실제 생성·연동은 미검증이다. PowerShell 준비 스크립트 구문 검사와 `git diff --check` 통과. 저장소 변경은 HISTORY.md만 확인했다.
- 실제 세션 진행: waid를 재시작하지 않은 상태에서 16:09:39 Orca todo-app 요청을 전송했다. 16:09:52 작업 중 카드를 직접 확인했고 원본 task_complete는 16:10:37.157, 이후 내 차례와 대조했다. PowerShell 자동 한글 입력·클립보드 실패는 제출하지 않고 해당 준비 프로세스만 종료한 뒤 UTF-8 요청을 공식 CLI 시작 인자로 전달하여 해결했다. 16:15:14 새 클래식 콘솔 실행, 16:15:32 notes-lookup 작업 중과 todo-app 내 차례가 함께 보이는 실제 PNG를 확인했다. notes 원본 task_complete는 16:15:57.017이며 이후 내 차례 표시 확인. notes의 SelfCheck 실행은 16:15:42 PSSecurityException/UnauthorizedAccess로 차단되어 우회하지 않았고 검사 성공으로 기록하지 않는다. codex app 명령으로 notes 프로젝트 열기 요청 exit 0과 ChatGPT 창 존재만 확인했으며 앱 화면·세션 수집 성공은 아님. 상세 결과는 별도 [관찰 기록](C:/waid-demo/launch-assets/evidence/desktop-observations.md)에 반영했다.
- 작업 범위 재확인: 16:23 저장소 상태에는 README와 desktop 소스의 다른 변경도 보인다. 이번 촬영 준비에서는 이 파일들을 수정하지 않았고, 내려받아 검증한 공개 v0.2.0 EXE를 계속 사용한다. 새 launcher 구문 검사와 `git diff --check`는 통과했다.
- 남은 일: 세 앱 창을 촬영 데스크톱으로 옮긴 뒤 Claude·Codex 앱의 실제 세션 준비와 전용 로그 경로 확인, 카드 상세·복귀·항상 위 대조. 안전한 영역 리허설·영상 검사 후 사용자 시작 신호를 받아 본 녹화. 기존 공유 로그를 단순히 켜거나 필터링한 가짜 훅으로 검증하지 않는다.

### 2026-09-09 14:43 +09:00 — VM 대신 Windows 가상 데스크톱 촬영으로 전환

- 요청: 별도 VM 대신 Win+Ctrl+D로 생성하는 Windows 가상 데스크톱에서 기존 홍보 데모 준비를 이어가기.
- 상태: 부분 완료 — 2026-09-09 15:08 +09:00 계속 진행. 사용자가 가상 데스크톱으로 전환했다고 알려준 뒤 공개 EXE GUI의 빈 목록을 직접 확인했다. 촬영용 Codex 터미널을 열었으며 사용자 초기 설정·로그인 완료를 기다리는 단계다. 실제 세션 수집·촬영은 미실행.
- 변경·이유: 가상 데스크톱은 설치 환경·사용자·로그를 공유하므로 깨끗한 Windows 첫 실행 검증과 분리했다. 공개 v0.2.0 사용자 어댑터 대체 기능으로 12개 내장 정의를 덮어쓰고 Codex 데모 절대 경로 하나만 reader로 지정했다. waid 설정·테마·Orca 훅과 Codex 홈을 별도 경로로 지정하는 실행 파일 4개를 만들었다. 데모 3개를 별도 작업 폴더에 복사하고 상위 폴더 탐색 경계를 위해 빈 Git 저장소를 초기화했다. 제품 소스·README·릴리스 변경, 인증·기존 업무 로그 복사는 없음.
- 관련 파일: [가상 데스크톱 실행 안내](C:/waid-demo/launch-assets/virtual-desktop-runbook.md), [시작 안내](C:/waid-demo/launch-assets/START-HERE.md), [검증 요약](C:/waid-demo/launch-assets/verification-summary.md), [관찰 기록](C:/waid-demo/launch-assets/evidence/desktop-observations.md), [실행 설정](C:/waid-demo/launch-assets/desktop-setup/launch-waid.cmd), [촬영표](C:/waid-demo/launch-assets/recording-plan.md), [Show GN 초안](C:/waid-demo/launch-assets/show-gn-draft.md), [HISTORY.md](HISTORY.md). 이전 VM 안내와 ZIP은 과거 자료로 표시해 보존했다.
- 검증: 릴리스 EXE 해시 재대조 일치, 설치된 Codex 버전 명령 0.153.4 확인. 관련 수집·경로 소스가 v0.2.0 태그와 일치. 공개 EXE의 내부 `--waid-core doctor` 종료 코드 0, 사용자 어댑터 12개·Codex 데모 reader 1개·세션 0개·다른 reader/매칭 프로세스 없음·별도 Orca 훅 0개 확인. 이후 15:03:55 GUI 실행, 15:04 직접 화면에서 ‘세션 0개·내 차례 0개’와 빈 목록 안내 확인 및 증거 캡처 저장. 작업표시줄이 창 하단에 겹치는 초기 배치 제약 발견. Codex는 최초 cmd 방식에서 창 핸들이 없어 해당 준비 프로세스를 종료하고, 별도 Windows Terminal 창으로 열어 창 제목을 확인했다. 로그인 화면은 캡처하지 않았다. 자료 내부 링크·실행 파일·어댑터 점검 및 `git diff --check` 통과. 실제 세션 수집 성공으로 확대하지 않음. 상세 관찰은 별도 자료에 남겼으며 과거 [VALIDATION.md](VALIDATION.md)는 수정하지 않았다.
- 남은 일: 촬영용 Codex 로그인·초기 설정을 사용자가 직접 완료한 뒤 실제 세션 3개·상태/상세/항상 위 확인, 창 크기·위치 조정, 리허설·녹화 시작 신호 대기. 공개 버전의 OS 프로세스 검색과 Python/Aider 예외 감지는 끌 수 없으므로 설정을 보안 격리로 설명하지 않는다. 깨끗한 Windows의 첫 실행 검증은 미확인으로 남는다.

### 2026-09-09 13:52 +09:00 — 첫 Show GN 홍보용 VM 데모·촬영 자료 준비

- 요청: 별도 Windows VM에서 공개 배포본의 첫 실행과 실제 AI 세션을 확인하고, 20~25초 녹화 동선·자막·Show GN 초안을 준비하기. 제품 수정·릴리스 변경·공개 게시 없이 진행하기.
- 상태: 부분 완료 — 공개 파일 다운로드·무결성·서명 검증과 촬영 자료·데모 seed 준비 완료. 접근 가능한 촬영 VM을 발견하지 못해 VM 실행·실세션 대조·리허설·녹화는 미실행.
- 변경·이유: 제품 소스·공개 README·릴리스는 변경 없음. 별도 `C:\waid-demo\launch-assets`에 검증 요약, VM 단계별 절차·관찰표, 25초 실행표, 짧은 한국어 자막, 미게시 Show GN 초안, 배포 파일·출처 증거와 최소 예제 3개를 마련했다. notes-api는 추가 런타임·서버가 필요 없는 PowerShell notes-lookup으로 단순화했다. 문서 확인·호스트 파일 검사·향후 VM 검증을 구분했다.
- 관련 파일: [준비 자료](C:/waid-demo/launch-assets/START-HERE.md), [검증 요약](C:/waid-demo/launch-assets/verification-summary.md), [VM 실행 절차](C:/waid-demo/launch-assets/vm-runbook.md), [촬영 실행표](C:/waid-demo/launch-assets/recording-plan.md), [Show GN 초안](C:/waid-demo/launch-assets/show-gn-draft.md), [HISTORY.md](HISTORY.md). 과거 제품 검증은 [VALIDATION.md](VALIDATION.md)와 별개로 유지.
- 검증: 익명 실시간 API로 v0.2.0 정식 공개·세 자산 확인, 실제 다운로드한 세 파일의 GitHub 다이제스트와 EXE SHA256SUMS 일치. Cosign 3.1.3 직접 검증은 지정 태그 워크플로 신원으로 `Verified OK`, 종료 코드 0. Authenticode 조회는 NotSigned이며 실제 차단 관찰은 아님. 공개 main/태그 README 양언어와 릴리스 본문 비교 일치. 호스트의 별도 Edge 프로필 DOM 검사·PowerShell smoke에서 seed 정상 경로와 미완성 과제 재현. 자료 Markdown 5개 내부 링크 및 PowerShell 블록 7개 구문 검사 통과. 전달 ZIP 32개 파일의 필수 항목·검사 프로필 제외·압축 안 EXE 해시 일치와 `git diff --check` 통과. 저장소 변경은 HISTORY.md만 있다. 상세 근거는 별도 준비 자료에 보관.
- 남은 일: 사용자 VM 창/접속 확인, VM에서 재검증·첫 실행, 사용자 직접 로그인 후 실제 세션 3개 관찰, 카드 가독성·항상 위 대조, 리허설과 녹화 시작 신호, 영상 검토·최종 자막 확정. 자동 승인 검토가 숨김 검증 명령과 임시 브라우저 자료 정리를 정책상 거부했다. 검증은 직접 읽기 전용 명령으로 완료했고 임시 자료는 보존했다. README의 source-build doctor 경로는 배포본 사용자에게 그대로 실행 가능한 안내가 아니라는 제약을 기록했으며 수정하지 않았다.

### 2026-09-09 13:14 +09:00 — 기능 검토와 v0.2.0 문서·릴리스 정리

- 요청: 현재 프로젝트 기능을 전체 검토하고 핵심 기능·편의사항·주의사항을 두 README와 버전에 맞는 릴리스에 반영한 뒤 커밋·푸시하기.
- 상태: 완료 — 문서 작성·커밋·main/태그 푸시·v0.2.0 게시·다운로드 검증 완료.
- 변경·이유: 수집·상태·정리·컨텍스트·세션 복귀·템플릿·CLI·배포 흐름을 코드와 대조했다. 두 README 앞부분에 기능표와 주의사항을 추가하고 업그레이드·지원 범위·실제 검증 근거를 정리했다. 현재 두 패키지 버전 0.2.0에 맞춘 한·영 릴리스 노트를 추가하고 태그 워크플로가 해당 파일을 본문으로 사용하게 했다. 과거 ZIP·최소화 설명을 바로잡고 Mac 컨텍스트 목록·상세 연결의 남은 한계를 기록했다. 함께 도입된 기존 개발 이력과 작업 규칙은 보존했다.
- 관련 파일: [README.md](README.md), [README.ko.md](README.ko.md), [releases/v0.2.0.md](releases/v0.2.0.md), [릴리스 워크플로](.github/workflows/release.yml), [PRODUCT.md](PRODUCT.md), [MACOS.md](MACOS.md), [VALIDATION.md](VALIDATION.md), [HISTORY.md](HISTORY.md). 릴리스 커밋: `94bf72f`, 태그: `v0.2.0`.
- 검증: Markdown 7개·로컬/태그 링크·이미지 경로·UTF-8·코드 블록·패키지/lock 버전 일치 검사, PowerShell 9개 블록 구문·릴리스 노트 경로 확인, actionlint와 `git diff --check` 통과. 릴리스 커밋의 Windows·Mac CI 및 Windows 태그 배포가 모두 성공했다. 태그 배포는 Windows 130개 검사 후 EXE·체크섬·Sigstore 번들을 게시했다. 게시 본문과 커밋한 한·영 노트가 줄바꿈 정규화 후 일치하고, 내려받은 EXE의 SHA-256·GitHub 자산 다이제스트·태그 워크플로 Sigstore 신원·내장 코어 버전 0.2.0을 직접 확인했다. 앱 소스는 변경하지 않았으며 실행 링크와 상세 근거는 [검증 기록](VALIDATION.md)에 있다.
- 남은 일: 이번 문서·릴리스 요청은 없음. Mac 실기기·컨텍스트 표시 연결·접근성·혼합 DPI·장시간 실사용과 일부 연결 UI 검증은 후속 제품 작업으로 남긴다.

### 2026-09-09 13:04 +09:00 — 요청별 개선 히스토리 도입

- 요청: 프로젝트가 어떻게 개선되고 프롬프팅할 때마다 무엇이 변경되는지 기록이 있는지 확인하고, 없다면 지금부터 남기기.
- 상태: 완료.
- 변경·이유: 기존 Git·설계 결정·검증 기록은 있었지만 요청과 변경을 묶는 통일된 기록 및 갱신 규칙은 없었다. 이 문서에 작성 형식과 과거 주요 변경 요약을 추가하고, 에이전트가 프로젝트 관련 요청마다 기록하도록 작업 규칙을 추가했다. README에서 기록으로 바로 이동할 수 있다.
- 관련 파일: [HISTORY.md](HISTORY.md), [AGENTS.md](AGENTS.md), [README.md](README.md). 작업 시작 기준 커밋: `47f4317`.
- 검증: 문서 링크와 과거 요약의 커밋 참조 확인, `git diff --check` 통과. 문서만 변경하여 앱 빌드·테스트는 실행하지 않음.
- 남은 일: 없음. 이후 요청부터 같은 형식으로 누적한다.

## 기록 도입 전 주요 변경 요약

아래는 기존 커밋 메시지와 문서에서 확인한 주요 변경을 정리한 것이다. 과거 프롬프트를 복원한 기록이나 전체 변경 목록은 아니다. 날짜는 커밋 작성일 기준이며, 당시 검증의 범위와 한계는 [검증 기록](VALIDATION.md)을 참고한다.

| 날짜 | 주요 변경 | 근거 커밋 |
|---|---|---|
| 2026-09-05 | 초기 프로젝트 등록, 가벼운 Windows 네이티브 데스크톱 셸 추가. | `6bb1938`, `262f63f` |
| 2026-09-06 | Windows 세션 카드, 창 표시 설정 저장, 에이전트 실행 환경 감지와 Copilot 세션 판독 추가. | `c2d7399`, `599f67a` |
| 2026-09-07 | JSON·SQLite 세션 소스 지원, 브랜드 자산과 앱 아이콘, Windows 릴리스 서명 자동화, 영문 README 도입. | `b38121e`, `2f9f57d`, `43a2e29`, `6da1062` |
| 2026-09-08 | 대화 미리보기와 카드 개선, 단일 EXE 배포, 세션 식별·진단·Orca 훅 수집·창 복귀·트레이 복원 개선. | `ae255c8`, `b6cd939` |
| 2026-09-09 | v1.0.0까지 플랫폼 로드맵 문서화, 세션 제외 기능 노출과 macOS 지원 착수. | `e231518`, `5213f0b` |
| 2026-09-09 | AppKit 세션 관리 앱과 Mac 개발용 번들 추가, 레이아웃·설정 재시작 검증 보강. 실제 Mac 설치·세션 복귀 등 출시 수용 검증은 남아 있음. | `7191b5a`, `de31190`, `5951a17`, `a34f486` |
| 2026-09-09 | 컨텍스트 사용량·압축 관측과 배지 표시, 로컬 처리 및 외부 업로드 경계 문서화. | `47f4317` |
