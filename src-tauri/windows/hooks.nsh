; Extra steps for the Tauri NSIS installer (bundle.windows.nsis.installerHooks).

; Builds from before 10 September 2026 run as lalia.exe. The installer's own
; "is the app running" check looks only for fuckyouflow.exe, so an old lalia.exe
; kept running, held the shared single-instance lock, and the freshly installed
; program exited the moment it started: the user stayed on the old version
; without being told (audit, 11 September 2026). Its speech engine sits in a
; kill-on-close job and goes with it.
;
; An old copy started "as administrator" cannot be closed from here (Windows
; answers Access is denied; seen on 11 September 2026). Then the user is asked
; to quit it from the tray and press Retry, and always learns why the old
; version is still there.
;
; A killed process takes a moment to leave the process list, so the list is
; read again for three seconds before anyone is asked anything. Asking at once
; showed the dialog for a copy that was already on its way out, and a silent
; install took that as Cancel (pre-release review, 11 September 2026).
!macro NSIS_HOOK_PREINSTALL
  Push $0
  Push $1
  nsExec::Exec 'taskkill /IM lalia.exe /F'
  Pop $0
  StrCpy $1 0
  fyf_check_old:
    nsExec::Exec 'cmd /c tasklist /FI "IMAGENAME eq lalia.exe" /NH | find /I "lalia.exe"'
    Pop $0
    StrCmp $0 "0" 0 fyf_old_gone
    IntCmp $1 6 fyf_ask 0 fyf_ask
    IntOp $1 $1 + 1
    Sleep 500
    Goto fyf_check_old
  fyf_ask:
    MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "An older Fuck You Flow is still running and Windows will not let the installer close it.$\r$\n$\r$\nRight-click its icon next to the clock, choose Quit, then press Retry." /SD IDCANCEL IDRETRY fyf_retry
    Abort
  fyf_retry:
    StrCpy $1 0
    Goto fyf_check_old
  fyf_old_gone:
  Pop $1
  Pop $0
!macroend

; The Windows startup entry can still name lalia.exe, a file this installer
; has just replaced. When the entry exists, point it at the new program.
!macro NSIS_HOOK_POSTINSTALL
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Fuck You Flow"
  StrCmp $0 "" +2 0
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Fuck You Flow" '"$INSTDIR\fuckyouflow.exe" --autostart'
!macroend

; Uninstalling leaves no startup entry pointing at a deleted program.
!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Fuck You Flow"
!macroend
