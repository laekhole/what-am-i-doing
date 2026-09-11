waid 로고 분리본
================

2026-09-10 사용자가 교체한 물음표 마스코트 원본입니다.
"what am I doing?"을 표현하는 물음표와 원본 비율·투명도를 유지합니다.
원본 해상도는 보존하고 Windows용 PNG/ICO만 필요한 크기로 축소합니다.

waid-horizontal.png : 가로형 로고 / 투명 배경
waid-vertical.png   : 세로형 로고 / 투명 배경
waid-mascot.png     : 마스코트 단독 / 투명 배경
waid-app-icon.png   : 앱 아이콘 / 네이비 타일 바깥 투명
waid-wordmark.png   : 워드마크 단독 / 투명 배경
waid-on-dark.png    : 다크 배경 적용 예시 / 배경 카드 유지
waid-on-light.png   : 라이트 배경 적용 예시 / 배경 카드 유지

워드마크 단독은 기존 파일을 유지합니다.
old/는 사용자가 보관한 이전 로고이며 앱에서는 참조하지 않습니다.
SVG, EPS, AI 등의 벡터 파일은 포함하지 않았습니다.

PNG 크기
--------
waid-horizontal.png: 2172 × 724 px
waid-vertical.png: 1122 × 1402 px
waid-mascot.png: 1254 × 1254 px
waid-app-icon.png: 1254 × 1254 px
waid-wordmark.png: 808 × 311 px
waid-on-dark.png: 1536 × 1024 px
waid-on-light.png: 1536 × 1024 px

적용 위치
---------
README: waid-horizontal.png (밝은 테마), waid-on-dark.png (어두운 테마), 표시 너비 440px
Windows 앱 상단: desktop/assets/waid-mascot.png (96 × 96px 투명 마스코트, 표시 크기 32 논리 픽셀, DPI에 따라 조정)
Windows 창 / 작업표시줄: desktop/assets/waid.png (256 × 256px 앱 타일)
실행 파일 / 트레이 / 설치·제거 / 시작 메뉴 / Windows 설정의 앱 아이콘: waid.ico
waid.ico 크기: 16, 20, 24, 32, 40, 48, 64, 128, 256px
Windows 아이콘 재생성: 저장소 루트에서 powershell -File assets/build-icons.ps1
macOS Dock / Finder: desktop/package-macos.sh가 waid-app-icon.png에서 16~1024px ICNS 생성
desktop/icons/와 desktop/icon.svg는 현재 네이티브 앱에서 참조하지 않는 이전 자산입니다.
아이콘은 빌드에 내장되므로 기존 EXE 재시작만으로 교체되지 않습니다. 새 빌드가 필요합니다.
