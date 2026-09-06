#!/usr/bin/env bash
# Restart the development app cleanly: kill app, vite, orphan engines; relaunch tauri dev.
cd "$(dirname "$0")/.." || exit 1
taskkill //IM lalia.exe //F >/dev/null 2>&1
taskkill //IM whisper-server.exe //F >/dev/null 2>&1
PID=$(netstat -ano | grep ":1420 .*LISTENING" | awk '{print $5}' | head -1)
[ -n "$PID" ] && taskkill //PID "$PID" //F >/dev/null 2>&1
sleep 1
# A force-killed app cannot remove its own tray icon, so Windows keeps drawing a
# ghost. Sweep the notification area before starting a fresh instance.
powershell -NoProfile -ExecutionPolicy Bypass -File "$(dirname "$0")/tray-sweep.ps1" >/dev/null 2>&1
(pnpm tauri dev > dev.log 2>&1 &)
WAIT=${1:-75}
sleep "$WAIT"
tasklist | grep -i "lalia.exe\|whisper-server" | head -3
LOG=$(ls -t "$LOCALAPPDATA"/Lalia/logs/lalia.log.* | head -1)
grep -a "$(date -u +%Y-%m-%dT%H:%M | cut -c1-15)" "$LOG" | tail -14 | cut -c1-260
