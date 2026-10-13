# waid 홍보 계획 · GeekNews · Reddit 템플릿

## 2026-10-13 GeekNews 게시 완료 확인

- 상태: **사용자가 Show GN 게시 완료**, 공개 URL은 [waid 소개 글](https://news.hada.io/topic?id=35282)이다. 페이지가 제공한 게시 시각은 2026-10-13 15:15:11 +09:00이며, 15:18 +09:00에 비로그인 HTTP 조회로 본문을 확인했다. 아래의 미게시·사람 확인 대기 기록은 게시 전 상황이다.
- 공개 내용: 제목, `내가 뭐하고 있었지?` 도입, Orca·Codex·Claude Code 지원, 샘플 사용 사례, 실행부터 세션 정리까지의 사용법, 클론·맞춤화·추가 도구 요청 안내가 포함되어 있다. 스크린샷 두 장과 마스코트는 텍스트 링크로 표시된다.
- 확인: 공개 글 HTTP 200. README, 스크린샷 2개, 마스코트, v0.2.0 릴리스, 사용 가이드의 링크 6개가 모두 HTTP 200으로 열렸다. 사용자 브라우저 제어·로그인·게시글 수정·댓글 작성은 하지 않았으며, 화면의 실제 시각 배치는 확인하지 않았다.
- 이미지 안내: GeekNews는 [공식 이용법](https://news.hada.io/guidelines)에 따라 본문 이미지를 지원하지 않는다. 본문 첫머리에 `[📷 스크린샷과 사용법 보기](https://github.com/laekhole/what-am-i-doing#readme)`를 추가하면 이미지가 있는 README로 안내할 수 있다. 이 문구는 제안이며 공개 글에는 반영하지 않았다.
- 남은 일: 게시 완료 확인에는 없음. 홈·목록의 노출 순위는 확인하지 않았고, 자동 모니터링이나 댓글 응대는 예약하지 않았다.

## 2026-10-13 GitHub README 공개 반영

- 상태: **GitHub 반영 완료**, 2026-10-13 13:50 +09:00 확인. [공개 README](https://github.com/laekhole/what-am-i-doing#readme)에 대표 이미지, 샘플 화면 2장, 한국어 사례, 상세 사용법과 클론·도구 요청 접기 영역을 넣었다. 기존 영문 설치·빌드·참고 문서는 보존했다.
- 최종 대표 문구: **내가 뭐하고 있었지?** 사용자의 후속 지시에 따라 표지 이미지, HTML 제목·본문·메타 설명, 게시용 텍스트, README 대체 텍스트와 소개 문단을 맞췄다. 원본 로고와 마스코트는 보존했다.
- 공개 파일: [대표 이미지](https://github.com/laekhole/what-am-i-doing/blob/main/assets/promo/geeknews-cover.png), [HTML 원본](https://github.com/laekhole/what-am-i-doing/blob/main/docs/geeknews-preview.html), [게시용 원고](https://github.com/laekhole/what-am-i-doing/blob/main/docs/geeknews-post.txt). GitHub의 HTML 링크는 소스 보기이며, 디자인과 CSS 애니메이션은 저장소를 클론한 뒤 로컬에서 열어 볼 수 있다. 별도 사이트 호스팅은 설정하지 않았다.
- 변경 커밋: `55a9ff0` (`docs: bring waid visual walkthrough into README`). GitHub API에서 README·커버·HTML·텍스트의 blob SHA가 로컬과 일치하고 공개 README의 새 문구·커버·접기 영역 2개가 포함됨을 확인했다.
- GeekNews: 원고와 이미지가 공개 저장소에서 접근 가능해졌으나 **게시 자체는 여전히 미완료**다. 중복 확인과 Turnstile 사람 확인·최종 등록이 남아 있다.

## 2026-10-13 그림이 있는 소개자료와 상세 사용법

- 현재 원고: [이미지 포함 미리보기](docs/geeknews-preview.html), [GeekNews 붙여넣기용 원고](docs/geeknews-post.txt). 제목은 `waid - Orca·Codex·Claude Code의 작업과 답변을 한눈에 보는 Windows 앱`으로 갱신했다. 아래 이전 준비 기록보다 이 원고를 우선한다.
- 대표 이미지: [내가 뭐하고 있었지?](assets/promo/geeknews-cover.png). 기존 파란 마스코트를 참조해 built-in image_gen으로 만든 홍보 일러스트다. 후속 요청에 따라 waid의 뜻을 살린 문구로 교체했다. 원본 로고는 보존했다. [생성 프롬프트·자산 설명](assets/promo/README.txt)에 생성 경로와 실제 프롬프트를 기록했다.
- 미리보기 구성: 대표 이미지, 움직이는 기존 마스코트, 실제 앱 샘플 화면 2장, 결제 화면·로그인 API·설치 문서 작업 사례, 다운로드부터 답변 펼치기·세션 정리까지의 사용법, 클론·맞춤화·새 도구 요청 안내. 마스코트의 움직임은 소개자료의 장식이며 앱 기능이 아니다. 영어 화면은 최신 소스 빌드, 두 화면의 내용은 샘플 데이터라고 명시했다.
- 지원·기여 문구: 사용자가 지정한 현재 지원 대상을 **Orca, Codex, Claude Code 세 가지**로 소개한다. Orca 안에서 실행하는 Codex·Claude Code도 설명하며, 임의 터미널 탭이나 SSH 세션으로 모두 복귀한다고 약속하지 않는다. MIT 저장소를 클론해 자기 작업 방식에 맞게 고칠 수 있고, 원하는 도구를 댓글·이슈로 요청하면 로그 형식을 확인해 연동을 추가한다는 문구를 넣었다.
- 사용 사례의 범위: 샘플 화면의 세 작업을 따라가는 구체적 예시다. 실제 사용자의 후기나 시간 절약 성과를 새로 만들어 넣지 않았다. 1열 카드 헤더 펼침, 처음의 미확인 상태, 새 요청·답변 관측, 필터 해제와 전체 보기, Orca 복귀 조건을 사용자 가이드 및 v0.2.0 태그 소스와 대조했다.
- GeekNews 적용: [공식 이용법](https://news.hada.io/guidelines)은 본문 이미지 삽입을 지원하지 않는다. 따라서 붙여넣기용 원고에는 공개된 원본 앱 화면·마스코트 링크를 넣었다. 새 커버와 HTML도 후속 README 반영 요청에서 공개 저장소에 올렸으며 결과는 상단 기록을 따른다. 미리보기는 외부 리소스 요청 없이 저장소 안의 이미지·글꼴을 사용한다.
- 게시 상태: 여전히 **미게시**. 앞서 확인한 Turnstile 사람 확인 단계는 남아 있다. 이번 보강에서는 사용자 브라우저를 제어하거나 GeekNews 등록을 다시 시도하지 않았다.

## 2026-10-13 백그라운드 게시 준비와 퇴고 결과

- 상태: **미게시 — 글 등록에 Cloudflare Turnstile 사람 확인 필요**. 사용자가 지정한 계정정보로 HTTPS 로그인에 성공했고 `/write`에서 정상 작성 폼을 확인했다. 이전 가입 후 일주일 제한은 더 이상 표시되지 않는다. 작성 폼은 사람 확인 위젯과 비활성화된 등록 버튼을 제공하므로 제출하지 않았다. 사용 중인 브라우저를 열거나 제어하지 않았다.
- 당시 준비 원고: [제목·유형·URL·본문](docs/geeknews-post.txt). 이 파일은 후속 요청으로 보강했으며 현재 내용은 상단 기록을 따른다. 당시 제목은 `waid - 여러 Claude Code·Codex 세션을 한눈에 보는 Windows 앱`, 유형은 **Show**, URL은 원본 GitHub 저장소였다. 아래 9월 원고는 이전 기록으로 보존한다.
- 퇴고: 요청한 [writing-deslop](https://github.com/laekhole/writing-deslop/blob/main/SKILL.md)를 기존 GitHub 인증으로 읽고 적용했다. 반복되는 기능 설명과 포괄적인 의견 요청을 줄였고, 실제 로그에 기록된 답변과 사용자가 선택하는 숨기기를 분명히 했다. 기능·버전·서명 제약·링크를 별도로 대조하고 다른 에이전트가 원문과 수정안을 검토했다. 짧은 원고이므로 지표 스크립트는 실행하지 않았으며 문체 점수는 산출하지 않았다.
- 사실 확인: GitHub API의 최신 릴리스는 [v0.2.0](https://github.com/laekhole/what-am-i-doing/releases/tag/v0.2.0), 발행 시각은 2026-09-09 04:17:38 UTC다. Windows EXE 4,564,992바이트와 SHA-256·Sigstore 파일 목록을 확인했다. 한국어 배포본과 소스 전용 영어 화면·알림 기능을 구분했다. 공개 EXE를 이번에 다운로드하거나 실행하지 않았다.
- 이미지: 기존 [한국어 샘플 화면](assets/screenshots/sessions-ko.png)을 확인했고 샘플 프로젝트임이 표시되어 있다. [GeekNews 이용법](https://news.hada.io/guidelines)에 따라 본문에는 이미지 삽입 대신 링크를 넣었다. 새 이미지나 영상은 만들지 않았다.
- 중복 확인 범위: 웹 검색에서 `waid`, `what-am-i-doing`의 GeekNews 결과는 나오지 않았다. 사이트 내부 검색의 초기 HTML에는 검색 결과가 없어 중복 없음으로 확정하지 않았다. 실제 게시 전 내부 검색 결과 또는 계정의 기존 글을 다시 확인해야 한다.
- 계정정보: 지정한 파일을 메모리에서 읽어 GeekNews 로그인에만 사용했다. 비밀번호와 세션 쿠키를 출력하거나 별도 파일로 저장하지 않았고 원본 계정정보 파일은 Git 제외 목록에 추가했다. 임시 HTTP 점검 스크립트는 기존 Git 제외 경로 `.tools/geeknews/`에 있다.
- 게시 URL·시각: 없음. 등록 POST를 보내지 않았으며 검토 대기 상태도 아니다.
- 남은 일: 사용자가 가능한 때 [글 등록](https://news.hada.io/write)에서 Show를 선택하고 위 원고를 입력한 뒤 사람 확인과 등록을 완료해야 한다. 게시 이후 URL·시각·실제 노출 상태를 기록한다. 자동 재시도나 댓글 응대는 예약하지 않았다.

## 2026-10-01 Show GN 게시 진행 결과

- 확인 시각: 2026-10-01 14:34 +09:00.
- 상태: **미게시 — 가입 후 일주일 제한으로 차단**. computer use로 로그인된 계정의 프로필과 [글 등록 화면](https://news.hada.io/write)을 확인했다. 가입일은 2026-10-01이며, 등록 화면에 “가입 후 일주일이 지나야 작성할 수 있습니다.”가 표시된다. 본문 입력·등록 제출은 수행하지 않았다.
- 게시 URL·게시 시각: 없음. 글이 생성되지 않았으므로 공개 완료나 검토 대기로 기록하지 않는다.
- 원고 확인: 아래 짧은 Show GN 원고를 README와 대조했고, [GitHub 공개 API](https://api.github.com/repos/laekhole/what-am-i-doing/releases/latest)에서 최신 릴리스가 여전히 v0.2.0임을 확인했다. 원고 변경 없음. 공개 EXE 실행은 이번에 재검증하지 않았으며, 최근 로컬 실행 차단 기록은 [HISTORY.md](HISTORY.md)에 남아 있다.
- 채널 확인: [GeekNews 이용법](https://news.hada.io/guidelines)의 가입 후 일주일 제한과 자작 프로젝트 Show 등록 규칙을 확인했다. 기존 게시물의 중복 여부는 이번에 확인하지 않았다.
- 다음 실행: **2026-10-08 가입 완료 시각으로부터 일주일이 지난 뒤** 등록 가능 여부를 재확인한다. 정확한 해제 시각은 미확인이다. 최신 릴리스·중복 글을 다시 확인하고, 아래 원고를 Show 유형으로 입력·제출한 뒤 게시 URL·시각·공개 상태를 기록한다. 자동 재시도는 예약하지 않았다.

## 2026-09-29 실행 계획과 다음 세션 인계

이번 요청은 홍보 계획 수립이며, 실제 게시는 사용자가 이후 세션에서 computer use로 실행을 요청할 때 진행한다. 아래 최신 실행 순서를 먼저 따른다. 기존 상세 계획과 한·영 원고는 뒤에 보존했다. 현재 게시 실적은 확인되지 않았고 이번 세션에서는 외부에 게시하지 않았다.

### 어디에, 어떤 순서로

| 순서 | 채널 | 실행 내용 | 진행 조건 |
|---|---|---|---|
| 1 | GeekNews Show GN | 아래 짧은 한국어 원고와 GitHub 링크로 1회 소개. 첫날 댓글 응대 시간 확보 | 공개 EXE 첫 실행과 소개 기능 확인, 계정 등록 자격 및 기존 중복 글 확인 |
| 2 | 이미 사용하는 X 또는 Threads 중 한 곳 | Show GN 게시 후 2~3일 안에 아래 짧은 글과 샘플 화면 공유 | 사용자가 지정한 기존 계정. 화면이 공개 배포본과 다르면 빌드 차이 표시 |
| 3 | r/ClaudeCode | 영어로 Claude Code 다중 세션 문제와 사용법 소개 | 영어 배포본 실행 확인 후. 게시 당일 규칙·고정글에서 자작 도구 허용 형식 확인. Showcase 전용이면 해당 스레드 댓글로 게시 |
| 4 | Hacker News Show HN | 첫 사용자 피드백을 반영한 뒤 검토 | 영어로 바로 사용 가능하고 제작자가 직접 쓴 원고가 있을 때. AI 원고는 사용하지 않음 |

여러 커뮤니티에 같은 글을 한꺼번에 뿌리지 않는다. 국내 첫 반응을 보고 설명과 설치 문제를 고친 뒤 영어권으로 확장한다. Product Hunt·유료 광고·새 랜딩페이지는 첫 10명의 실제 사용을 확인한 뒤 판단한다. 채널 간 간격은 응대를 위한 제안이며 플랫폼 규칙이 아니다.

### 첫 2주 일정

- D0~D2: 공개 다운로드 파일 실행, 자신의 Claude Code/Codex 세션 표시, 아래 원고와 버전 일치 확인. 샘플 화면 1장 준비. 가능하면 Windows 사용자 3명에게 첫 실행 확인을 받되, 모집·확인 실적을 만들어 쓰지 않는다.
- D3: 준비가 되면 Show GN 1회 게시. 게시 직후 원문 URL과 공개 노출 상태 기록. 당일 1~2시간은 피드백에 대응할 수 있는 시간대로 선택한다.
- D4~D7: 기존 SNS 한 곳 공유. 댓글·이슈에서 설치 차단, 세션 미표시, 상태 혼동을 모아 가장 빈번한 문제부터 수정한다.
- D8~D14: 영어 배포본과 커뮤니티 규칙이 준비되면 Reddit 한 곳 게시. 아직 한국어 배포본뿐이면 영어권 일정만 미룬다.
- 4주 목표: 실제 사용 확인 10명, 첫 사용 7일 뒤 재사용 확인 5명. 운영 목표이며 예상 성과가 아니다. 다운로드·star는 보조 지표로 별도 기록한다.

### 지금 사용할 짧은 Show GN 원고

제목: **waid - 여러 Claude Code·Codex 세션을 한눈에 보는 Windows 앱**

등록 유형: **Show**. 화면에서 접두사를 자동으로 붙이면 제목에 `Show GN:`을 중복 입력하지 않는다.

링크: https://github.com/laekhole/what-am-i-doing

본문:

Claude Code나 Codex를 여러 창에서 쓰다 보면, 어느 세션에 무슨 일을 맡겼는지 확인하려고 창을 하나씩 열어보게 됩니다. 이때 작업과 마지막 답변을 한곳에서 볼 수 있도록 **waid(what am I doing?)**라는 Windows 앱을 만들었습니다.

Claude Code와 Codex가 남긴 로컬 로그를 읽어 세션별 정보를 카드로 모아 보여줍니다.

- **작업 확인:** 프로젝트, 최근 요청, 에이전트·모델과 로그에서 관찰한 상태를 한눈에 볼 수 있습니다.
- **마지막 대화 확인:** 카드를 펼쳐 최근 요청과 마지막 답변을 읽을 수 있습니다.
- **세션 정리:** 검색·필터로 세션을 찾고, 자주 보는 세션은 고정하거나 끝난 작업은 숨길 수 있습니다.

로그는 PC 안에서만 처리하며 프롬프트와 답변을 외부로 전송하지 않습니다. 원본 대화를 수정하거나 에이전트에 명령을 보내지 않고, 사용 통계도 수집하지 않습니다.

Windows 10/11 x64에서 별도 설치 과정 없이 EXE 하나로 실행하는 앱이며, 소스는 MIT 라이선스로 공개했습니다. 현재 공개된 v0.2.0은 한국어 UI입니다. 저장소의 영어 화면은 최신 소스 빌드 기준이고, macOS 앱은 개발 중입니다.

아직 초기 버전이라 알려둘 한계도 있습니다. 상태는 로그를 기준으로 판단하므로 기록이 늦으면 표시도 늦어질 수 있습니다. EXE에 Windows 코드 서명(Authenticode)이 없어 SmartScreen이나 Smart App Control에서 실행이 차단될 수도 있습니다. 다운로드 검증 방법은 저장소에 안내했습니다.

Windows에서 Claude Code·Codex 세션을 여러 개 쓰는 분들께 도움이 되는지 궁금합니다. **다시 볼 세션을 찾는 데 도움이 됐는지, 한눈에 보고 싶은 정보가 더 있는지** 의견 부탁드립니다. 세션이 목록에 나타나지 않는다면 사용한 에이전트와 실행 환경을 댓글이나 GitHub 이슈로 알려주세요.

### 기존 SNS용 짧은 원고

Claude Code·Codex 세션을 여러 개 쓰면 어디에 무슨 일을 맡겼는지 헷갈리죠. 프로젝트·최근 요청·마지막 답변을 한 창에서 보는 Windows 앱 waid를 만들었습니다. 로컬 로그만 읽는 MIT 오픈소스입니다. 공개 EXE는 한국어 UI이며 미서명입니다. 사용해 보고 의견 주세요.

https://github.com/laekhole/what-am-i-doing

이미지는 [한국어 샘플 화면](assets/screenshots/sessions-ko.png)을 후보로 사용한다. 게시 전에 화면을 열어 공개할 개인정보가 없는지, 설명과 버전이 맞는지 확인한다. GeekNews 본문은 이미지를 지원하지 않으므로 저장소 README의 화면으로 안내한다. 30초 영상은 있으면 좋지만, 첫 게시를 위해 반드시 새로 만들 필요는 없다.

### 다음 세션의 computer use 작업 순서

1. 이 문서, README.md, 최신 HISTORY.md를 읽고 computer-use 스킬을 적용한다. 공개 릴리스가 바뀌었으면 제목·본문·화면의 버전 사실부터 갱신한다.
2. 사용자가 지정한 채널과 로그인된 계정을 확인한다. 로그인·2FA·CAPTCHA가 필요하면 그 단계만 사용자에게 요청한다. 계정 생성이나 다른 채널 추가를 임의로 진행하지 않는다.
3. 채널 규칙과 계정 게시 자격, 사용자의 기존 게시물에서 중복 여부를 확인한다. GeekNews는 가입 후 일주일이 지나야 링크 등록이 가능하다고 안내한다. Reddit 자작 도구 허용 여부는 아직 미확인이다.
4. 지정된 원고를 작성 화면에 입력하고 제목·링크·줄바꿈·공개 범위를 확인한다. 사용자가 해당 채널 게시를 요청한 세션에서는 그 권한으로 제출까지 진행한다. 이 문서 자체가 즉시 게시 지시는 아니다.
5. 제출 후 실제 게시 페이지를 열어 원문 URL·게시 시각·노출 상태를 아래에 기록한다. 검토 대기와 공개 완료를 구분한다. 제출 결과가 모호하면 기존 글부터 확인하고 중복 제출하지 않는다.
6. 댓글 답변이나 추가 채널 게시 요청이 있으면 그 범위에서 진행한다. 종료된 세션에서 자동으로 모니터링하거나 후속 게시한다고 약속하지 않는다.

다음 세션에 붙여 넣을 요청:

> C:\whatamidoing의 LAUNCH.md 최신 실행 계획과 짧은 Show GN 원고를 읽고, computer use로 로그인된 내 GeekNews 계정에 Show 유형으로 게시해 줘. 최신 공개 버전과 게시 규칙·중복 글을 확인하고, 원고의 사실관계를 맞춘 뒤 제출까지 진행해. 완료하면 게시 URL과 시각을 LAUNCH.md 및 HISTORY.md에 기록해 줘. 로그인이나 CAPTCHA가 필요하면 그 단계만 알려줘.

| 채널 | 상태 | 게시 시각 | 게시 URL / 후속 결과 |
|---|---|---|---|
| Show GN | 미게시·사람 확인 필요 | — | 2026-10-13 HTTPS 로그인·정상 작성 폼 확인. Turnstile 필수로 미제출. 최신 원고는 [별도 파일](docs/geeknews-post.txt) |
| 기존 SNS | 미게시·계정 미지정 | — | 사용자가 채널 지정 후 진행 |
| Reddit | 보류 | — | 영어 공개 배포본·게시 규칙 확인 필요 |
| Show HN | 후순위 | — | 제작자 직접 작성 원고 필요 |

### 이번 확인 근거

- 2026-09-29 GitHub 공개 API에서 최신 릴리스 `v0.2.0` 확인: https://api.github.com/repos/laekhole/what-am-i-doing/releases/latest . 웹 검색 도구의 `/releases/latest` 결과는 v0.1.0으로 연결되어 최신 판단에는 API를 사용했다. 다운로드 파일 실행은 이번에 재검증하지 않았다.
- [GeekNews 이용법](https://news.hada.io/guidelines): Show 유형, 가입 후 일주일 제한, 본문 이미지 미지원, 실제 사용 가능한 작업물 조건 확인.
- [Reddit 스팸 정책](https://support.reddithelp.com/hc/en-us/articles/360043504051-Spam): 커뮤니티별 규칙을 따르는 방식으로 진행. r/ClaudeCode 규칙 조회는 실패했으므로 게시 당일 브라우저에서 확인한다.
- [HN 일반 지침](https://news.ycombinator.com/newsguidelines.html), [Show HN 지침](https://news.ycombinator.com/showhn.html): 후속 검토 채널이며 아래 기존 AI 원고 사용 금지 안내를 유지한다.

---

2026-09-14 기준으로 작성한 미게시 원고다. 각 플랫폼의 제목·링크·본문을 나눠 복사하면 된다. 공개 v0.2.0 EXE와 최신 소스의 차이를 본문에도 적었다. 새 릴리스를 배포하면 해당 문단과 다운로드 링크를 갱신한다.

[humanizer 3.0.0](https://github.com/blader/humanizer/blob/main/SKILL.md)의 문체 지침을 아래 두 소개 원고에 적용했다. 과장과 반복을 줄이고, 확인되지 않은 개인 경험이나 사용 성과는 넣지 않았다.

## 목표와 대상

첫 4주는 **Windows에서 Claude Code·Codex 세션을 여러 개 사용하는 개발자 10명이 실제로 써 보고, 그중 5명이 일주일 뒤에도 다시 쓰는지 확인하는 기간**으로 잡는다. 수치는 시장 통계나 성과 예측이 아닌 초기 운영 목표다. 개인 유지관리자 1명, 유료 홍보비 0원, 주 4~6시간의 홍보·응대 시간을 가정한다. 제품 수정·서명 준비 시간과 비용은 별도다.

우선 대상은 하루에 코딩 AI 세션을 3개 이상 오가며 프로젝트·요청·자기 차례를 놓치는 Windows 개발자다. Claude Code·Codex의 여러 세션을 쓰거나 두 도구를 함께 쓰는 사람이 적합하다. 한 세션만 쓰는 사람과 Mac·모바일 사용자는 첫 모집의 핵심 대상에서 제외하고, 후자는 해당 플랫폼 출시 때 다시 소개한다.

핵심 문구:

> **여러 코딩 AI 세션, 어디까지 했는지 한눈에.**
> waid는 Claude Code·Codex의 작업과 마지막으로 관찰한 상태를 모아 보여주는 Windows 앱입니다. 기존 로그를 로컬에서 읽고, 프롬프트를 외부로 업로드하지 않습니다.

콘텐츠는 ‘어느 창에서 무슨 일을 시켰는지 잊었다’는 실제 상황으로 시작하고, 프로젝트·최근 요청·내 차례 확인으로 해결되는 장면을 보여준다. 단일 EXE·로컬 처리·오픈소스는 사용을 결정할 근거로 뒤에 배치한다. Rust 구현과 전체 기능 목록은 README에서 설명한다.

## 공개 전에 맞출 것

현재 배포와 구현 상태는 [README](README.md), 실행 검증은 [VALIDATION](VALIDATION.md), 서명 진행은 [SIGNING](SIGNING.md)을 기준으로 한다. 공개 GitHub API에서도 2026-09-14의 최신 릴리스가 [v0.2.0](https://github.com/laekhole/what-am-i-doing/releases/tag/v0.2.0)임을 확인했다.

| 준비 항목 | 완료 기준 |
|---|---|
| 소개 내용과 배포 파일 일치 | 새 언어 전환·트레이 기능을 소개하려면 그 기능이 포함된 실제 다운로드 파일로 확인한다. 영상에도 버전을 표시한다. |
| 첫 실행 | 개발 환경 밖의 Windows PC 3대에서 다운로드→실행→자신의 Claude Code 또는 Codex 세션 확인을 시도하고 성공/차단을 기록한다. 보안 설정 해제를 정상 설치 절차로 삼지 않는다. |
| 배포 신뢰 | 현재 Authenticode 미서명과 실행 차단을 숨기지 않고 다운로드 안내에 명시한다. 서명 준비와 기본 보안 설정에서의 실행 검증을 진행한다. 초기 3대 결과만으로 모든 Windows 환경을 보장하지 않는다. |
| 30초 데모 | 샘플 프로젝트 3개로 작업 목록→내 차례→답변 펼치기를 보여준다. 상태 변화·알림은 촬영한 빌드에서 실제 관찰되는 경우에만 담는다. |
| 첫 화면 안내 | README 상단에 한 문장 설명, 대표 화면 또는 GIF, Windows 다운로드, 세션 미표시 안내를 가까이 둔다. 별도 웹사이트는 초기 필수 작업으로 두지 않는다. |
| 피드백 경로 | GitHub Issues에서 버전·Windows 버전·에이전트/실행 환경·막힌 단계·기대/실제 동작을 받는다. 로그 원문 제출을 기본 요구로 삼지 않는다. |

초기 3~5명에게는 한계를 공개한 테스트 모집으로 접근한다. 실제 다운로드·실행 문제를 정리하기 전에는 대규모 채널 노출을 늘리지 않는다. 코드 서명 자체가 모든 PC의 경고 해소를 보장한다는 문구도 사용하지 않는다.

## 채널과 순서

| 우선순위 | 채널 | 올릴 내용과 행동 요청 |
|---|---|---|
| 1 | 기존에 교류하는 Windows 개발자·참여 중인 소규모 개발 커뮤니티 | 자발적 테스트 참가자 3~5명 모집. ‘세션 3개를 쓰는 날 10분 써 보고, 안 잡힌 세션과 가장 유용한 정보를 알려 주세요.’ |
| 2 | GeekNews Show GN | 아래 한국어 초안을 바탕으로 만든 이유·30초 데모가 있는 GitHub·Windows 사용 조건·확인한 한계를 소개한다. 첫 공개 게시물은 여기 한 곳에 집중한다. |
| 3 | Reddit의 Claude Code 사용자 커뮤니티 한 곳 | 영어 빌드 검증 후 Windows와 Claude Code 사용 경험 중심으로 소개한다. r/ClaudeCode의 당시 Weekly Showcase를 우선 확인하고, 자작 도구 게시 허용 여부에 맞춰 게시 형식을 정한다. |
| 보조 | 이미 사용하는 X·개인 블로그 | 동일한 짧은 데모와 제작 이유를 재사용한다. 기존 계정이 없다면 이를 위해 여러 계정을 운영하는 작업은 미룬다. |
| 후속 | Hacker News Show HN | 초기 실행 문제를 해결하고 사용 사례가 모인 뒤 검토한다. 제작자가 직접 설명·질의응답할 시간을 확보할 때 진행한다. |

GeekNews는 자작 앱을 **Show 유형**으로 등록하도록 요구하며, 직접 사용 가능한 작업물을 대상으로 한다. GitHub를 등록 링크로 쓰고 영상은 저장소에서 보여준다. 동일 저장소를 작은 수정마다 다시 올리지 않는다. [GeekNews 운영방침](https://news.hada.io/guidelines)

Reddit은 반복·대량 홍보를 금지하고 커뮤니티마다 추가 규칙을 적용한다. r/ClaudeCode의 공개 목록에서 Weekly Showcase를 확인했지만, 별도 규칙 페이지의 본문은 조회되지 않아 일반 게시물 허용을 확정하지 않았다. 게시 당일 규칙·고정글·flair를 확인하고 개발자 본인임을 밝힌다. [Reddit 스팸 정책](https://support.reddithelp.com/hc/en-us/articles/360043504051-Spam), [r/ClaudeCode](https://old.reddit.com/r/ClaudeCode/)

Show HN은 사용자가 직접 써 볼 수 있어야 하며 추천·댓글 동원을 금지한다. HN 일반 지침은 AI 생성·AI 편집 텍스트 게시도 금지하므로, 이 문서의 문구를 복사하지 않고 제작자가 자신의 경험으로 직접 작성한다. [Show HN 지침](https://news.ycombinator.com/showhn.html), [HN 일반 지침](https://news.ycombinator.com/newsguidelines.html)

## 4주 실행표

1주차의 배포·실행 준비가 지연되면 이후 공개 일정도 순연한다. 아래는 실행 제안이며 릴리스·촬영·모집·게시를 완료했다는 기록이 아니다.

| 기간 | 할 일 | 확인할 결과 |
|---|---|---|
| 1주차 | 새 배포 후보와 소개 문구 대조, Windows PC 3대 첫 실행 확인, 기존 데모 준비 자료를 활용해 영상 1개·대표 화면 1개 준비, 3~5명 테스트 | 실행 성공/차단 원인과 세션 미표시 문제 파악. 문제가 남으면 먼저 수정하고 공개 규모를 유지한다. |
| 2주차 | Show GN 한 곳 게시, 기존 SNS가 있으면 데모 공유, 첫날 답변 시간 2시간 확보·이후 매일 15분 확인 | 누적 실제 사용 확인 10명 목표. 댓글·이슈에서 반복되는 불편 3개 추출. |
| 3주차 | 가장 빈번한 사용 방해 문제 1~2개 개선, 영어 다운로드·실행 재확인 후 Reddit 한 곳 소개 | 영어 사용자도 첫 세션을 찾는지, 한국어 사용자와 같은 문제를 겪는지 확인. |
| 4주차 | 재연락에 동의한 참가자에게 첫 사용 7일 뒤 재사용 여부 확인, 기존 글에 해결된 문제·새 릴리스 안내, 후속 채널 판단 | 최초 사용 확인 10명 중 재사용 확인 5명 목표. 유지 이유·중단 이유를 다음 제품 개선에 반영. |

서로 다른 채널의 큰 게시물은 2~3일 이상 간격을 두어 답변 시간을 확보한다. 이 간격은 운영상 제안이며 플랫폼이 정한 게시 허용 주기가 아니다. 정확한 ‘최적 게시 시각’보다 제작자가 댓글에 응답할 수 있는 시간을 선택한다.

## 데모 구성과 소개글 방향

30초 영상 하나를 README·커뮤니티·기존 SNS에서 재사용한다. 기존 촬영 준비는 HISTORY에 기록되어 있지만 최종 영상 완성은 확인되지 않았으므로 새로 촬영 여부와 버전을 확인한다.

| 시간 | 장면 | 전달할 내용 |
|---|---|---|
| 0~5초 | 샘플 프로젝트를 쓰는 코딩 AI 창 3개 | ‘어디에 무슨 일을 맡겼더라?’ |
| 5~15초 | waid에서 프로젝트·최근 요청·관찰 상태 확인 | ‘작업을 한곳에서 다시 파악’ |
| 15~25초 | 내 차례 필터와 마지막 답변 펼치기 | ‘지금 확인할 세션 찾기’ |
| 25~30초 | 앱 화면과 짧은 자막 | ‘Windows · 로컬 처리 · GitHub에서 사용해 보기’ |

샘플 화면에는 데모임을 표시하고 실제 업무 대화·경로·개인정보를 노출하지 않는다. 정적인 샘플로 실제 상태 변화나 알림이 검증된 것처럼 연출하지 않는다. 세션 복귀는 별도의 지원 조건이 있어 첫 데모의 필수 장면으로 두지 않는다.

한국어 제목 제안: **Show GN: waid — 여러 Claude Code·Codex 세션의 작업을 한눈에 보는 Windows 앱**

Reddit 제목 제안: **I built a Windows app to keep track of my Claude Code and Codex sessions**

아래 기존 초안의 기능 목록은 채널에 맞게 3개 정도로 줄여도 된다. 첫 행동 요청은 ‘사용해 보고 세션을 찾는 데 도움이 됐는지 알려 달라’로 통일한다. ‘모든 에이전트 완벽 지원’, ‘정확한 실시간 상태’, ‘모든 창/탭 즉시 복귀’, ‘Mac·모바일 지원’, 측정하지 않은 시간 절감 수치는 홍보 문구로 사용하지 않는다.

## 성과 기록과 다음 판단

앱에 분석 SDK를 추가하지 않고, 참가자가 자발적으로 알려준 실행·사용 결과와 GitHub 릴리스 자산의 다운로드 수를 구분해 기록한다. 다운로드 수에는 재다운로드와 제작자 검증이 포함될 수 있어 사용자 수로 환산하지 않는다. 유입 채널은 참가자의 선택적 답변으로 확인하며 계정별 행동 추적은 하지 않는다.

| 지표 | 기록 방법 | 다음 행동 |
|---|---|---|
| 첫 실행 | 테스트를 시도한 PC 수 / 성공 수 / 차단 수 | 차단·빈 목록이 반복되면 신규 채널 확대보다 배포·수집 문제부터 해결한다. |
| 실제 사용 | 자신의 세션을 확인하고 일할 때 사용했다고 응답한 사람 수, 목표 10명 | 다운로드만 있고 실제 사용 확인이 적으면 시작 안내와 첫 세션 발견 과정을 점검한다. |
| 7일 뒤 재사용 | 최초 사용 확인자 중 7일 경과 대상 수 / 응답 수 / 재사용 확인 수, 첫 10명 중 5명 목표 | 미응답은 별도 기록한다. 충분한 응답에서도 재사용이 적으면 홍보량보다 반복해서 쓸 이유를 개선한다. |
| 문제와 효용 | 반복 불편 상위 3개, 도움이 된 구체적 상황 | 사용을 막는 문제 1~2개부터 해결한다. 인용·사용 후기 공개는 해당 사용자의 동의를 받은 경우에만 한다. |

현재의 추천은 **실행 확인 → 소규모 테스트 → Show GN → 개선 → Reddit → 재사용 확인** 순서다. 유료 광고·Product Hunt 집중 출시·장편 영상·별도 랜딩페이지는 초기 사용과 재사용이 확인된 뒤 다시 판단한다. Mac 출시는 실제 Mac 배포가 준비됐을 때 새 홍보 기회로 활용한다.
## GeekNews

### 제목

Show GN: waid - Claude Code·Codex 세션을 모아 보는 Windows 앱

### 링크

[GitHub 저장소](https://github.com/laekhole/what-am-i-doing)

### 본문

여러 코딩 AI 세션에 맡긴 작업과 마지막 답변을 한곳에서 확인할 수 있는 waid(what am I doing?)를 만들고 있습니다. Claude Code와 Codex 세션을 여러 개 열어두고 쓰는 분들께 소개합니다.

기존 로컬 로그를 읽어서 세션마다 프로젝트, 최근 요청, 에이전트와 모델, 마지막으로 확인한 상태를 카드로 보여주는 Windows 네이티브 앱입니다. 창을 하나씩 열어보기 전에 어디까지 진행됐는지 살펴볼 수 있습니다.

- 카드를 펼치면 최근 요청과 마지막으로 기록된 답변을 읽을 수 있습니다.
- 검색과 필터로 세션을 찾고, 자주 보는 세션은 고정하거나 끝난 작업은 목록에서 숨기고 제외할 수 있습니다.
- 로그에 남은 컨텍스트 사용량과 압축 기록을 보여줍니다. 남은 비율은 사용량과 한도를 모두 알 수 있을 때만 표시합니다.
- 지원하는 연결을 설정하면 기존 세션이나 창으로 돌아갈 수 있습니다. 복귀 범위는 사용하는 앱에 따라 다릅니다.

로그는 PC 안에서 처리합니다. waid가 프롬프트나 답변을 외부 서버로 보내지 않고, 사용 통계 수집과 클라우드 동기화도 없습니다. 원본 대화를 수정하거나 에이전트에 명령을 보내는 기능은 없습니다.

상태 표시는 로그가 기록되는 시점에 영향을 받습니다. 로그가 늦으면 표시도 늦고, 과거 대화가 목록에 있다고 지금 실행 중이라는 뜻은 아닙니다. Claude Code와 Codex를 우선 다듬고 있으며 다른 연동은 검증 수준에 차이가 있습니다.

소스는 MIT 라이선스로 공개했습니다. [Windows x64용 v0.2.0](https://github.com/laekhole/what-am-i-doing/releases/tag/v0.2.0)은 설치 없이 EXE 하나로 실행합니다. 이 배포본의 UI는 한국어이며, 한국어·영어 전환과 트레이 상태 요약·새 답변 무음 알림은 최신 소스에 들어 있습니다. 이 기능을 쓰려면 현재는 소스 빌드가 필요합니다. Mac 버전은 개발 중입니다.

현재 EXE에는 Windows Authenticode 서명이 없어 Smart App Control이나 SmartScreen이 실행을 차단할 수 있습니다. 다운로드 페이지에 검증 방법과 알려진 제한을 적어두었습니다.

여러 세션 중 다음에 돌아갈 곳을 고를 때, 최근 요청과 마지막 답변 외에 어떤 정보가 필요한지 궁금합니다. 세션이 목록에 잡히지 않는 경우에는 사용한 에이전트와 실행 환경을 댓글이나 GitHub 이슈에 남겨주세요.

## Reddit

### Title

I built waid, a Windows app for viewing multiple Claude Code and Codex sessions

### Body

I'm the developer of waid ("what am I doing?"), a native Windows app for keeping track of multiple coding AI sessions. It reads existing local logs and puts each session's project, latest request, agent, model, and last observed status in one window.

If you have several Claude Code or Codex sessions open, you can check what you asked each one to do and read its last logged answer before switching back.

- Expand a card to read the latest prompt and answer.
- Search and filter the list, pin sessions, or hide and dismiss finished work.
- Check logged context usage and compaction records. Remaining percentages need both a usage measurement and a known context limit.
- Return to a connected session or window where the integration supports it.

waid processes logs on your computer. It doesn't upload prompts or answers, collect telemetry, or sync to a cloud service. It doesn't edit the original conversations or send instructions to agents.

The status comes from logs, so delayed logs mean delayed updates. An old conversation in the list doesn't tell you whether it's still open. Claude Code and Codex are the main integrations I'm working on; the others have less testing.

The source is MIT licensed. The [v0.2.0 download](https://github.com/laekhole/what-am-i-doing/releases/tag/v0.2.0) is a single Windows x64 EXE with a Korean UI. English/Korean switching, tray status summaries, and quiet reply notifications are in the current source and require a source build for now. A native Mac version is in development.

The EXE doesn't yet have a Windows Authenticode signature, so Smart App Control or SmartScreen may block it. The release page covers verification and known limitations.

[Source and build instructions](https://github.com/laekhole/what-am-i-doing)

When you're choosing which session to return to, what do you need beyond the latest prompt and answer? If a session is missing from waid, a comment or GitHub issue with the agent and how you run it would help me investigate.

## 작성자용 메모

이 절은 게시 본문에 포함하지 않는다.

GeekNews 원고는 [Show GN 공식 안내](https://hada.io/blog/geeknews-show/)에 맞춰 실제 기능과 실행 링크, 받고 싶은 피드백을 담았다. Reddit 원고는 특정 커뮤니티를 전제로 하지 않는다. 게시할 곳의 자작 프로젝트 허용 여부와 flair를 확인한다. 커뮤니티마다 추가 규칙이 있다는 점은 [Reddit 공식 안내](https://support.reddithelp.com/hc/en-us/articles/360043504051-Spam)에서도 설명한다.

화면을 첨부한다면 `--demo`의 샘플 세션으로 촬영하고 해당 빌드의 언어를 선택한다. 영문 화면을 첨부할 때는 소스 빌드 화면임을 명시한다. 제품의 실환경 검증과 Mac 미검증 범위는 [VALIDATION.md](VALIDATION.md)를 참고한다.
