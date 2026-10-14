# waid Privacy Policy

Effective date: 2026-10-14 · Applies to waid for Windows (GitHub EXE and Microsoft Store), the macOS development build and the `waid` CLI. [한국어](#한국어)

## Summary

waid runs only on your computer. It reads coding-agent logs that already exist on your computer and shows them in one window. **waid does not send any of this information to the developer or to any other server.** It has no account, telemetry, analytics, advertising, cloud sync or remote AI API calls.

## What waid accesses

To show your sessions, waid reads, read-only, local files written by supported coding agents (for example Claude Code, Codex and Orca) under your user profile. These files can contain:

- the prompts you sent and the answers the agents logged, which may include personal information you typed;
- project folder names and paths, Git branch names, agent and model names, and timestamps;
- session identifiers and token or context usage recorded by the agent.

To match sessions to running agents and windows, waid also reads the list of running processes (names, command lines and working folders) and the titles of open windows. They are used only for matching; only a window or link you choose to connect to a session is remembered in the settings file.

On Windows, while an SSH client is running, waid also reads the text visible in terminal windows to recognize agents running over SSH. It never types into, scrolls or changes those windows.

waid does not access your contacts, location, camera, microphone or files unrelated to supported agents.

## How the information is used

The information is used only to display session status, the latest request and answer, and context usage in waid, and to return to a session you choose. It is not used for advertising, profiling or any other purpose.

## Storage

waid stores your settings and saved session details (pins, hidden sessions, previews) in a local file:

- Windows (EXE): `%LOCALAPPDATA%\waid\settings.json`
- Windows (Microsoft Store): `%LOCALAPPDATA%\Packages\laekhole.waid_…\LocalCache\Local\waid\settings.json`
- macOS: `~/Library/Application Support/waid/settings.json`
- or the folder set in `WAID_DATA_DIR`

This file can contain prompt and answer text and is stored unencrypted, like the agent logs it comes from. It stays on your computer unless you or your own backup or sync software copy it.

## Sharing

waid does not sell, share or transmit any information to the developer or third parties. When you ask waid to return to a session, it passes a session identifier to the app already installed on your computer (for example Orca). The optional CLI dashboard serves data only to `127.0.0.1` on your own computer.

## Retention and deletion

You control all data. Delete the settings file above to remove waid's saved data. Uninstalling the Microsoft Store version removes its settings automatically. waid never deletes or edits the agents' own logs.

## Children

waid is a developer tool and is not directed to children.

## Changes and contact

Changes to this policy are published in this file in the [public repository](https://github.com/laekhole/what-am-i-doing), with a new effective date. Questions: [GitHub issues](https://github.com/laekhole/what-am-i-doing/issues).

---

## 한국어

# waid 개인정보 처리방침

시행일: 2026-10-14 · Windows용 waid(GitHub EXE, Microsoft Store), macOS 개발 빌드, `waid` CLI에 적용됩니다.

### 요약

waid는 사용자의 컴퓨터에서만 실행됩니다. 컴퓨터에 이미 있는 코딩 에이전트 로그를 읽어 한 창에 보여 줍니다. **waid는 이 정보를 개발자나 다른 어떤 서버에도 보내지 않습니다.** 계정, 텔레메트리, 분석, 광고, 클라우드 동기화, 외부 AI API 호출이 없습니다.

### 접근하는 정보

세션을 보여 주기 위해 waid는 지원하는 코딩 에이전트(예: Claude Code, Codex, Orca)가 사용자 프로필 아래에 기록한 로컬 파일을 읽기 전용으로 읽습니다. 이 파일에는 다음이 들어 있을 수 있습니다.

- 사용자가 보낸 요청과 에이전트가 기록한 답변. 사용자가 입력한 개인정보가 포함될 수 있습니다.
- 프로젝트 폴더 이름과 경로, Git 브랜치, 에이전트·모델 이름, 시각
- 세션 식별자, 에이전트가 기록한 토큰·컨텍스트 사용량

세션과 실행 중인 에이전트·창을 맞추기 위해 실행 중인 프로세스 목록(이름, 명령줄, 작업 폴더)과 열린 창의 제목도 읽습니다. 이 정보는 맞추는 데에만 쓰며, 사용자가 세션에 직접 연결한 창이나 링크만 설정 파일에 기억합니다.

Windows에서는 SSH 클라이언트가 실행 중일 때, SSH로 실행 중인 에이전트를 알아보기 위해 터미널 창에 보이는 글자도 읽습니다. 그 창에 입력하거나, 스크롤하거나, 창을 바꾸지 않습니다.

연락처, 위치, 카메라, 마이크, 지원 에이전트와 관계없는 파일에는 접근하지 않습니다.

### 이용 목적

세션 상태, 최근 요청과 답변, 컨텍스트 사용량을 waid 화면에 표시하고, 사용자가 고른 세션으로 돌아가는 데에만 씁니다. 광고, 프로파일링 등 다른 목적으로 쓰지 않습니다.

### 저장

설정과 저장된 세션 정보(고정, 숨김, 미리보기)는 로컬 파일에 저장합니다.

- Windows(EXE): `%LOCALAPPDATA%\waid\settings.json`
- Windows(Microsoft Store): `%LOCALAPPDATA%\Packages\laekhole.waid_…\LocalCache\Local\waid\settings.json`
- macOS: `~/Library/Application Support/waid/settings.json`
- 또는 `WAID_DATA_DIR`로 지정한 폴더

이 파일에는 요청·답변 내용이 들어갈 수 있으며, 원본 에이전트 로그와 마찬가지로 암호화하지 않고 저장합니다. 사용자나 사용자의 백업·동기화 프로그램이 복사하지 않는 한 컴퓨터 밖으로 나가지 않습니다.

### 제공

waid는 어떤 정보도 판매하거나 개발자·제3자에게 제공·전송하지 않습니다. 세션으로 돌아가기를 누르면, 컴퓨터에 이미 설치된 앱(예: Orca)에 세션 식별자를 넘깁니다. 선택 기능인 CLI 대시보드는 사용자 컴퓨터의 `127.0.0.1`에서만 데이터를 제공합니다.

### 보관과 삭제

모든 데이터는 사용자가 관리합니다. 위 설정 파일을 지우면 waid가 저장한 정보가 삭제됩니다. Microsoft Store 버전은 앱을 제거하면 설정도 함께 삭제됩니다. waid는 에이전트의 원본 로그를 지우거나 고치지 않습니다.

### 아동

waid는 개발자용 도구이며 아동을 대상으로 하지 않습니다.

### 변경과 문의

이 방침을 바꾸면 [공개 저장소](https://github.com/laekhole/what-am-i-doing)의 이 파일에 새 시행일과 함께 게시합니다. 문의: [GitHub 이슈](https://github.com/laekhole/what-am-i-doing/issues).
