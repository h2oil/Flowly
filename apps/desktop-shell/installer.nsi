; Flowly Windows installer — built with plain makensis (works on Linux too;
; Tauri's own NSIS bundler needs GitHub-release downloads that some build
; environments block). Per-user install, no admin prompt. Installs the app,
; the on-device Vosk speech runtime + model, and the WebView2 Evergreen
; runtime if the machine doesn't already have it.

Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"

!define APP_NAME "Flowly"
!define APP_VERSION "0.1.0"
!define APP_EXE "Flowly.exe"
!define UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Flowly"

Name "${APP_NAME} ${APP_VERSION}"
OutFile "target\Flowly_${APP_VERSION}_x64-setup.exe"
InstallDir "$LOCALAPPDATA\Flowly"
RequestExecutionLevel user
SetCompressor /SOLID lzma

!define MUI_ICON "icons\icon.ico"
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "Start Flowly"

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Flowly" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"

  File /oname=${APP_EXE} "target\x86_64-pc-windows-msvc\release\flowly-shell.exe"
  ; On-device speech runtime (Vosk, MinGW build) — audio never leaves the PC.
  File "vendor\libvosk.dll"
  File "vendor\libgcc_s_seh-1.dll"
  File "vendor\libstdc++-6.dll"
  File "vendor\libwinpthread-1.dll"

  SetOutPath "$INSTDIR\model"
  File /r "vendor\model\*.*"
  SetOutPath "$INSTDIR"

  ; WebView2 Evergreen runtime: preinstalled on Windows 11 and serviced
  ; Windows 10; install silently only when missing.
  ReadRegStr $0 HKCU "Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${If} $0 == ""
    ReadRegStr $0 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${EndIf}
  ${If} $0 == ""
    DetailPrint "Installing Microsoft WebView2 runtime..."
    InitPluginsDir
    File /oname=$PLUGINSDIR\wv2setup.exe "vendor\MicrosoftEdgeWebView2Setup.exe"
    ExecWait '"$PLUGINSDIR\wv2setup.exe" /silent /install'
  ${EndIf}

  CreateShortcut "$SMPROGRAMS\Flowly.lnk" "$INSTDIR\${APP_EXE}"
  CreateShortcut "$DESKTOP\Flowly.lnk" "$INSTDIR\${APP_EXE}"

  WriteUninstaller "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINST_KEY}" "DisplayIcon" "$INSTDIR\${APP_EXE}"
  WriteRegStr HKCU "${UNINST_KEY}" "Publisher" "Flowly"
  WriteRegStr HKCU "${UNINST_KEY}" "UninstallString" "$\"$INSTDIR\Uninstall.exe$\""
  WriteRegDWORD HKCU "${UNINST_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINST_KEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\Flowly.lnk"
  Delete "$DESKTOP\Flowly.lnk"
  RMDir /r "$INSTDIR\model"
  Delete "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\libvosk.dll"
  Delete "$INSTDIR\libgcc_s_seh-1.dll"
  Delete "$INSTDIR\libstdc++-6.dll"
  Delete "$INSTDIR\libwinpthread-1.dll"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${UNINST_KEY}"
SectionEnd
