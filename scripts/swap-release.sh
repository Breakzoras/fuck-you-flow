#!/usr/bin/env bash
# Replaces the running release build with src-tauri/target/release/fuckyouflow.exe.
# Refuses while a dictation is in progress: the pipeline's last event in the
# log must be a finished one, and no key may have started a recording in the
# last few seconds. Usage: bash scripts/swap-release.sh
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# The log rolls and stamps in local time (see src-tauri/src/logging.rs), so the
# guard below has to read it in local time too. Reading it as UTC made every
# key press look three hours old here, which quietly disabled the guard.
LOG="$LOCALAPPDATA/FuckYouFlow/logs/fuckyouflow.log.$(date +%Y-%m-%d)"
EXE="$ROOT/src-tauri/target/release/fuckyouflow.exe"

[ -f "$EXE" ] || { echo "no built binary at $EXE"; exit 2; }

# An app that is not running cannot be dictating. Without this check, a crash
# in the middle of a dictation leaves "recording started" as the last event in
# the log and the guard below refuses every swap for the rest of the day, which
# is exactly what happened on 7 September 2026 at 19:15.
RUNNING=$(tasklist //FI "IMAGENAME eq fuckyouflow.exe" //NH 2>/dev/null | grep -ci "fuckyouflow.exe" || true)
if [ -f "$LOG" ] && [ "${RUNNING:-0}" -gt 0 ]; then
  # the most recent pipeline event decides whether a recording is open
  last=$(grep -E "recording started|dictation (success|failed|copied)|no speech|Cancelled|cancelled" "$LOG" | tail -1)
  case "$last" in
    *"recording started"*)
      # and only if it started recently: nobody dictates for ten minutes in one breath
      started=$(echo "$last" | cut -c1-19)
      ss=$(date -d "${started/T/ }" +%s 2>/dev/null || echo 0)
      if [ $(( $(date +%s) - ss )) -lt 600 ]; then
        echo "REFUSED: a recording is in progress ($(echo "$last" | cut -c12-19))"; exit 3
      fi
      ;;
  esac
  # a key press in the last 5 s means the user is mid-gesture
  now=$(date +%s)
  lastkey=$(grep -E "hotkey event Pressed" "$LOG" | tail -1 | cut -c1-19)
  if [ -n "$lastkey" ]; then
    ks=$(date -d "${lastkey/T/ }" +%s 2>/dev/null || echo 0)
    if [ $((now - ks)) -lt 5 ]; then echo "REFUSED: key pressed $((now - ks)) s ago"; exit 3; fi
  fi
fi

taskkill //F //IM fuckyouflow.exe >/dev/null 2>&1 && echo "stopped running instance"
sleep 2
if tasklist //FI "IMAGENAME eq fuckyouflow.exe" //NH 2>/dev/null | grep -qi "fuckyouflow.exe"; then
  echo "REFUSED: the running copy would not close (started as administrator?). Quit it from the tray, then run this again."; exit 4
fi
taskkill //F //IM whisper-server.exe >/dev/null 2>&1
# Run from a copy outside the build tree (so the next build can replace the
# exe) AND outside AppData. A shell inside the Claude desktop app lives in its
# MSIX container: anything it writes under AppData lands in a private store
# only Claude can see, and anything it launches runs inside that container too.
# Until 11 September 2026 this script did both, so the "installed" copy and
# its files existed for Claude alone, the Run key pointed at a path Windows
# could not find, and nothing started at logon.
RUN_DIR="/c/Claude Projects/fyf-run"; mkdir -p "$RUN_DIR"
cp "$EXE" "$RUN_DIR/fuckyouflow.exe"
if [ ! -d "$RUN_DIR/bundled/engine-vulkan" ] && [ -d "$ROOT/src-tauri/target/release/bundled" ]; then
  cp -r "$ROOT/src-tauri/target/release/bundled" "$RUN_DIR/"
fi
# Started by Task Scheduler, the process is outside the container. The task
# hands the exe to Explorer, so the app is Explorer's child like any click and
# is not bound to the task's run-time limit.
powershell -NoProfile -Command "\$a = New-ScheduledTaskAction -Execute \"\$env:WINDIR\\explorer.exe\" -Argument '\"C:\\Claude Projects\\fyf-run\\fuckyouflow.exe\"'; \$s = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit (New-TimeSpan -Minutes 5); Register-ScheduledTask -TaskName FYFRun -Action \$a -Settings \$s -Force | Out-Null; Start-ScheduledTask -TaskName FYFRun" && echo "launched $RUN_DIR/fuckyouflow.exe outside the container"
sleep 8
grep -E "starting|hook installed|whisper-server ready|settings.json|ERROR" "$LOG" | tail -3
rm -f "$ROOT/src-tauri/target/release/fuckyouflow-running.exe"
