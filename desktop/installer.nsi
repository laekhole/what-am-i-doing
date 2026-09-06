Unicode true
!include "MUI2.nsh"
!include "x64.nsh"
Name "waid"
!ifndef WAID_VERSION
    !error "Use desktop/build.ps1 to supply WAID_VERSION."
!endif
!ifndef WAID_RELEASE_DIR
    !error "Use desktop/build.ps1 to supply WAID_RELEASE_DIR."
!endif
OutFile "${WAID_RELEASE_DIR}\bundle\waid_${WAID_VERSION}_x64-setup.exe"
InstallDir "$LOCALAPPDATA\waid"
RequestExecutionLevel user
SetCompressor /SOLID lzma
!define MUI_ICON "${__FILEDIR__}\..\assets\waid.ico"
!define MUI_UNICON "${__FILEDIR__}\..\assets\waid.ico"
!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\waid-desktop.exe"
!define MUI_FINISHPAGE_RUN_NOTCHECKED
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Korean"

Function .onInit
    ${IfNot} ${RunningX64}
        MessageBox MB_OK|MB_ICONSTOP "This build requires 64-bit Windows."
        Abort
    ${EndIf}
FunctionEnd

Section "waid"
    SetShellVarContext current
    SetOutPath "$INSTDIR"
    File "${WAID_RELEASE_DIR}\waid-desktop.exe"
    File "${WAID_RELEASE_DIR}\waid.exe"
    WriteUninstaller "$INSTDIR\uninstall.exe"
    CreateShortcut "$SMPROGRAMS\waid.lnk" "$INSTDIR\waid-desktop.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\waid" "DisplayName" "waid"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\waid" "DisplayVersion" "${WAID_VERSION}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\waid" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\waid" "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\waid" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\waid" "NoRepair" 1
SectionEnd

Section "Uninstall"
    SetShellVarContext current
    Delete "$INSTDIR\waid-desktop.exe"
    Delete "$INSTDIR\waid.exe"
    Delete "$INSTDIR\uninstall.exe"
    Delete "$SMPROGRAMS\waid.lnk"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\waid"
    RMDir "$INSTDIR" ; Non-recursive: never remove user files.
SectionEnd
