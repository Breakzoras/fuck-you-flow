#!/bin/sh
# Runs as root after the .deb is installed: load the virtual keyboard driver
# and apply the udev rule to the devices that are already plugged in, so the
# shortcut works without logging out.
modprobe uinput 2>/dev/null || true
if command -v udevadm >/dev/null 2>&1; then
  udevadm control --reload-rules 2>/dev/null || true
  udevadm trigger --subsystem-match=input --subsystem-match=misc 2>/dev/null || true
fi
exit 0
