#!/bin/sh
# Runs as root after removal: forget the rule for the devices in use.
if command -v udevadm >/dev/null 2>&1; then
  udevadm control --reload-rules 2>/dev/null || true
  udevadm trigger --subsystem-match=input --subsystem-match=misc 2>/dev/null || true
fi
exit 0
