"""Simulate the push-to-talk chord for end-to-end tests without a human hand.

Injected keys carry no Lalia signature, so the app's keyboard hook treats them
like real presses (only Lalia's own injections are ignored).

Usage:
  python eval/tools/press_keys.py hold 4        # hold Ctrl+Win for 4 seconds
  python eval/tools/press_keys.py tap           # quick tap (hands-free toggle)
  python eval/tools/press_keys.py escape        # cancel
  python eval/tools/press_keys.py paste_last    # Shift+Alt+Z
"""
import ctypes
import sys
import time

user32 = ctypes.windll.user32
KEYEVENTF_KEYUP = 0x0002
VK = {"ctrl": 0x11, "lwin": 0x5B, "space": 0x20, "escape": 0x1B, "shift": 0x10, "alt": 0x12, "z": 0x5A}


def down(vk):
    user32.keybd_event(vk, 0, 0, 0)


def up(vk):
    user32.keybd_event(vk, 0, KEYEVENTF_KEYUP, 0)


def hold(seconds: float):
    down(VK["ctrl"])
    time.sleep(0.03)
    down(VK["lwin"])
    time.sleep(seconds)
    up(VK["lwin"])
    time.sleep(0.03)
    up(VK["ctrl"])


def tap():
    hold(0.12)


def escape():
    down(VK["escape"])
    time.sleep(0.03)
    up(VK["escape"])


def paste_last():
    down(VK["shift"])
    down(VK["alt"])
    down(VK["z"])
    time.sleep(0.05)
    up(VK["z"])
    up(VK["alt"])
    up(VK["shift"])


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "tap"
    if cmd == "hold":
        hold(float(sys.argv[2]) if len(sys.argv) > 2 else 3.0)
    elif cmd == "tap":
        tap()
    elif cmd == "escape":
        escape()
    elif cmd == "paste_last":
        paste_last()
    print("done", cmd)
