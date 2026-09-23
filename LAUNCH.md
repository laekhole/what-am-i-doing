# waid 홍보 계획 · GeekNews · Reddit 템플릿

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
