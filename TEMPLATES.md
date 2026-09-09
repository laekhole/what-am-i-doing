# Windows UI 템플릿

네이티브 앱의 JSON 템플릿입니다. CLI HTML/CSS와 별개이며 재빌드 없이 바꿀 수 있습니다. 코어가 로그 이벤트를 읽고 앱이 실행 이후의 요청을 추적합니다. 템플릿에는 실행 코드나 상태 규칙이 없습니다.

## 복제 · 수정 · 미리보기 · 적용

1. **··· → 템플릿**을 누르면 확장 화면으로 전환되고 현재 적용본의 복사본이 상세 영역에 열립니다.
2. **밝은 기본** 또는 **어두운 기본**으로 예제를 복제합니다. 아직 적용된 설정은 바뀌지 않습니다.
3. JSON을 수정합니다. `font_size`를 18로 바꾸거나 `fields` 순서를 바꿔보세요. **내보내기**로 별도 파일에 저장해 외부 편집기에서 수정해도 됩니다.
4. **미리보기**를 누르면 현재 세션 목록에 임시 적용됩니다. 저장된 템플릿은 바뀌지 않습니다. 목록이 비어 있다면 검색·필터를 해제하세요.
5. **적용**을 누르면 저장되어 재실행에도 사용합니다. 적용하지 않고 **편집 닫기**를 누르면 마지막 적용 상태로 돌아갑니다.
6. **기본값 복구**는 밝은 기본을 즉시 적용·저장합니다. 고정·숨김·종결 기록은 유지합니다.

**가져오기**는 UTF-8 JSON을 검증해 편집 영역에 넣습니다. 미리보기 후 적용하세요. **내보내기**는 편집 중인 유효한 JSON을 저장하며 세션 데이터를 넣지 않습니다. 기존 파일을 고르면 Windows 덮어쓰기 확인을 거칩니다.

[Daylight](desktop/templates/daylight.json)는 옅은 회색 배경·흰 둥근 카드, [Midnight](desktop/templates/midnight.json)는 어두운 배경을 사용합니다. 1열은 대화를 펼치는 카드이며, 2열의 기본 compact 배치는 왼쪽 로고, 위쪽 프로젝트·에이전트, 아래 작업 내용입니다. 두 목록 모두 오른쪽 상태 배지 아래에 모델을 표시하고 긴 이름은 말줄임 처리합니다.

컨텍스트 사용률과 확인된 압축 기록은 두 목록에 기본 표시하며 `fields` 설정과 별개입니다. 평소에는 `colors.muted`로 표시하고, 남은 컨텍스트가 25% 이하이거나 압축 기록이 있으면 `colors.accent` 글자·테두리와 `colors.selection` 배경의 배지로 강조합니다. 접힌 카드와 2열 목록에서도 `컨텍스트 25.0% 남음`, `압축됨`을 확인할 수 있습니다. 1열 카드를 펼치거나 2열 상세 보기를 열면 토큰 수·한도, 마지막 관측 시점과 압축 기록을 확인할 수 있습니다. 원본 로그에 없는 값은 미확인으로 표시합니다.

## 형식 version 1

전체 JSON은 위 예제를 복사하세요.

| 키 | 허용 값과 의미 |
|---|---|
| version | 1 |
| name | 1~48자, 제어문자 제외 |
| colors.background | 창 배경, #RRGGBB |
| colors.surface | 목록·편집 영역 배경 |
| colors.text | 작업·제목 글자색 |
| colors.muted | 부가 정보 글자색 |
| colors.selection | 선택한 행의 배경 |
| colors.accent | 강조선·강조 글자색 |
| font_size | 12~24, 96 DPI 기준 논리 픽셀. Daylight 기본값은 14이고 프로젝트 제목은 3픽셀 더 큽니다. |
| padding | 4~24, 카드 안쪽 여백. 기본 10 |
| compact | 2열 목록에 적용. 기본 true: 첫 줄에 project와 agent, 아래에 task 두 줄, 오른쪽에 상태와 그 아래 model. activity는 별도 줄. false면 fields 순서대로 각 필드를 표시하고 task는 두 줄 |
| line_gap | 2~12, 항목 사이 추가 간격 |
| fields | task, project, status, agent, model, activity 중 원하는 순서. 중복 불가. task·project·status 필수. compact에서는 위치가 고정되며 agent·model·activity의 포함 여부를 반영 |
| icons | waiting, working, error, idle, done, unknown 각각 1~4자 기호. compact가 false일 때 상태 옆에 표시 |
| state_colors | 위 여섯 상태 각각의 #RRGGBB 색상 |

`compact: false`에서는 아래 순서로 모델을 에이전트보다 먼저 표시할 수 있습니다.

```json
"fields": ["task", "project", "status", "model", "agent", "activity"]
```

`compact: false`에서 `icons.waiting`을 `"☀"`로 바꿀 수 있습니다. 해당 기호를 표시하는 Windows 글꼴이 필요합니다. 컬러 이모지·캐릭터 이미지·애니메이션은 현재 지원하지 않으며 상태별 기호로 외형을 교체합니다. 글꼴은 앱에 포함된 Pretendard SemiBold·Bold이며 로드 실패 시 맑은 고딕과 Windows 대체 글꼴을 사용합니다.

글자·상태·강조색은 창·목록·선택 배경과 명암비 4.5:1 이상이어야 합니다. 낮은 대비, 필수 항목 누락, 잘못된 JSON, 128 KiB 초과 파일은 거부하고 기존 화면을 유지합니다. 상태 이름 자체는 템플릿이 바꾸지 못합니다. 카드의 네 모서리는 둥글게 그리며 하네스 로고는 상태나 템플릿과 독립적으로 표시합니다. 창 제목과 일반 버튼도 템플릿 색상을 사용합니다. 기본 입력란·체크박스·파일 대화상자는 Windows 컨트롤입니다. 투명도는 창 전체에 적용되는 별도 설정이며 템플릿의 대비 검사와 구분됩니다.

## 복구와 저장 위치

적용본은 `%LOCALAPPDATA%\waid\settings.json`의 template 문자열에 저장합니다. 가져왔던 원본 JSON을 수정해도 자동 반영되지 않으므로 다시 가져오세요. 미리보기는 앱을 닫으면 사라집니다.

저장된 설정이 손상되면 안내와 함께 기본 화면으로 시작합니다. 이 경우 고정·숨김·종결·필터도 메모리에서 기본값으로 열립니다. **기본값 복구**로 유효한 기본 템플릿을 저장할 수 있습니다.

샘플 데이터만 보고 싶다면 별도 설정 폴더로 실행하세요.

```powershell
$env:WAID_DATA_DIR = Join-Path $env:TEMP 'waid-template-preview'
.\desktop\target\x86_64-pc-windows-gnu\release\waid-desktop.exe --demo
```

샘플 모드는 수집 코어를 실행하지 않으며 창에 샘플이라고 표시합니다. 환경변수는 해당 PowerShell 세션에 남으므로 실제 설정을 쓰려면 새 PowerShell에서 일반 실행하세요.
