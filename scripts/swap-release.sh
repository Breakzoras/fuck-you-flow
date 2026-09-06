#!/usr/bin/env bash
# Replaces the running release build with src-tauri/target/release/lalia.exe.
# Refuses while a dictation is in progress: the pipeline's last event in the
# log must be a finished one, and no key may have started a recording in the
# last few seconds. Usage: bash scripts/swap-release.sh
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOG="$LOCALAPPDATA/Lalia/logs/lalia.log.$(date -u +%Y-%m-%d)"
EXE="$ROOT/src-tauri/target/release/lalia.exe"

[ -f "$EXE" ] || { echo "no built binary at $EXE"; exit 2; }

if [ -f "$LOG" ]; then
  # the most recent pipeline event decides whether a recording is open
  last=$(grep -E "recording started|dictation (success|failed|copied)|no speech|Cancelled|cancelled" "$LOG" | tail -1)
  case "$last" in
    *"recording started"*) echo "REFUSED: a recording is in progress ($(echo "$last" | cut -c12-19))"; exit 3 ;;
  esac
  # a key press in the last 5 s means the user is mid-gesture
  now=$(date -u +%s)
  lastkey=$(grep -E "hotkey event Pressed" "$LOG" | tail -1 | cut -c1-19)
  if [ -n "$lastkey" ]; then
    ks=$(date -u -d "${lastkey/T/ }" +%s 2>/dev/null || echo 0)
    if [ $((now - ks)) -lt 5 ]; then echo "REFUSED: key pressed $((now - ks)) s ago"; exit 3; fi
  fi
fi

taskkill //F //IM lalia.exe >/dev/null 2>&1 && echo "stopped running instance"
sleep 2
taskkill //F //IM whisper-server.exe >/dev/null 2>&1
# run from a copy outside the build tree, so the next build can replace lalia.exe
RUN_DIR="$LOCALAPPDATA/Lalia/running"; mkdir -p "$RUN_DIR"; cp "$EXE" "$RUN_DIR/lalia.exe"
cmd //c start "" "$(cygpath -w "$RUN_DIR/lalia.exe")" && echo "launched copy of $EXE"
sleep 8
grep -E "starting|hook installed|whisper-server ready|settings.json|ERROR" "$LOG" | tail -3
rm -f "$ROOT/src-tauri/target/release/lalia-running.exe"
