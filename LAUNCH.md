# Reddit · GeekNews 소개글 초안

아직 게시하지 않은 초안이다. 현재 공개된 v0.2.0 EXE에는 이번 언어 전환이 포함되어 있지 않으므로, 아래 글은 변경 사항을 공개 저장소와 새 Windows 릴리스에 반영한 뒤 사용한다. Mac은 개발 중으로 소개하며 다운로드 가능한 정식 지원으로 표현하지 않는다.

## Reddit

**Title**

I built waid to keep track of what my coding agents are doing

**Body**

I kept losing track of which coding-agent session was working on which task, and which one was waiting for me. Switching between windows just to check was getting distracting, so I started building **waid — “what am I doing?”**

It's a small native desktop app that reads existing local coding-agent logs and puts the project, latest request, agent/model, and last observed status in one place. Claude Code and Codex are the main integrations I'm focusing on.

You can:

- Expand a session to read its latest prompt and last logged answer.
- Search, pin, hide, and dismiss sessions to keep the list manageable.
- See remaining context when the source logs provide enough information.
- Return to a connected session or window, within the supported integrations.
- Keep it in the Windows tray and get quiet notifications for newly observed replies.
- Switch between English and Korean.

It runs locally, with no telemetry, cloud sync, or prompt uploads by waid. It observes logs; it doesn't send instructions to agents. Windows runs from a single EXE. A native Mac version is in development.

One limitation: these are **observed log states**, so an old conversation isn't proof that a session is still open. Missing or delayed logs mean unknown or delayed status. Some integrations are much less tested than Claude Code and Codex.

I'm the developer, and I'd like to get feedback beyond my own workflow. If you work with several coding-agent sessions, what information helps you decide which one to return to next? Reports of missing sessions would also help.

Source, downloads, and support details: [github.com/laekhole/what-am-i-doing](https://github.com/laekhole/what-am-i-doing)

## GeekNews

**제목**

Show GN: waid — 여러 코딩 AI 세션이 무슨 일을 하는지 한눈에 보는 데스크톱 앱

**링크**

[GitHub 저장소](https://github.com/laekhole/what-am-i-doing)

**본문**

Claude Code와 Codex를 여러 개 켜 놓고 일하다 보면 어느 세션에 무슨 작업을 맡겼는지, 어디에서 제 답변을 기다리는지 자꾸 놓치게 됐습니다. 창을 하나씩 돌아다니며 확인하는 일을 줄이려고 **waid(what am I doing?)**를 만들고 있습니다.

기존 로컬 코딩 에이전트 로그를 읽어 프로젝트, 최근 요청, 에이전트·모델, 마지막으로 관찰한 상태를 한곳에 보여주는 네이티브 데스크톱 앱입니다.

- 카드를 펼쳐 최근 요청과 마지막 답변 확인
- 검색, 고정, 숨김, 목록 제외로 세션 정리
- 로그에 근거가 있는 경우 남은 컨텍스트와 압축 기록 표시
- 지원하는 연결을 통해 기존 세션이나 창으로 복귀
- Windows 트레이에서 상태 확인 및 새 답변 무음 알림
- 한국어·영어 전환

waid는 로컬에서 동작하고 프롬프트·답변을 외부 서버로 올리지 않습니다. 텔레메트리와 클라우드 동기화도 없으며, 에이전트에 명령을 보내는 기능은 없습니다. Windows는 설치 없이 단일 EXE로 실행하고, 네이티브 Mac 버전은 개발 중입니다.

상태는 로그에서 관찰한 결과입니다. 과거 대화가 현재 열린 세션이라는 뜻은 아니고, 로그가 없거나 늦게 기록되면 상태도 미확인이거나 늦게 갱신됩니다. Claude Code·Codex를 우선 다듬고 있으며 다른 연동은 검증 수준에 차이가 있습니다.

혼자 쓰면서 만드는 데서 벗어나 실제 사용 의견을 받아보고 싶어 공유합니다. 여러 코딩 AI 세션을 동시에 쓸 때 어떤 정보가 가장 필요한지, 세션이 안 잡히는 환경은 무엇인지 알려주시면 개선에 도움이 될 것 같습니다.

## 게시 준비

- 영어·한국어 전환이 포함된 Windows 빌드를 새 릴리스로 올리고 다운로드·언어 전환을 확인한다. 기존 v0.2.0 파일을 영문판이라고 소개하지 않는다.
- 스크린샷은 `--demo`의 샘플 세션을 사용한다. 이번 작업의 검증 결과와 Mac 미검증 범위는 [VALIDATION.md](VALIDATION.md)를 참고한다.
- Reddit은 게시할 커뮤니티를 정한 뒤 해당 커뮤니티의 현재 자작 프로젝트 게시 규칙과 flair를 확인한다. 특정 subreddit이나 계정으로 게시하는 작업은 수행하지 않았다.
