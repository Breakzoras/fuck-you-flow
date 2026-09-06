"""End-to-end smoke test: hotkey -> (fake microphone) -> whisper -> cleanup -> paste into Notepad.

Requires the app running in DEBUG mode (pnpm tauri dev). The microphone is
replaced by a WAV file through %LOCALAPPDATA%\\Lalia\\fake_mic.txt; everything
else (keyboard hook, overlay, engine, insertion, history) is the real thing.

Usage: python eval/e2e_notepad.py [item-id ...]
"""
import ctypes
import ctypes.wintypes as wt
import json
import os
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
CORPUS = os.path.join(HERE, "corpus")
FAKE = os.path.join(os.environ["LOCALAPPDATA"], "Lalia", "fake_mic.txt")
sys.path.insert(0, os.path.join(HERE, "tools"))
import press_keys  # noqa: E402

user32 = ctypes.windll.user32
user32.SetProcessDPIAware()


def find_window(title_part: str):
    result = []

    @ctypes.WINFUNCTYPE(ctypes.c_bool, wt.HWND, wt.LPARAM)
    def cb(hwnd, _):
        if user32.IsWindowVisible(hwnd):
            n = user32.GetWindowTextLengthW(hwnd)
            buf = ctypes.create_unicode_buffer(n + 1)
            user32.GetWindowTextW(hwnd, buf, n + 1)
            if title_part.lower() in buf.value.lower():
                result.append(hwnd)
        return True

    user32.EnumWindows(cb, 0)
    return result[0] if result else None


def edit_text(hwnd_parent) -> str:
    edit = user32.FindWindowExW(hwnd_parent, None, "Edit", None)
    if not edit:
        # Windows 11 Notepad uses RichEditD2DPT; try that
        edit = user32.FindWindowExW(hwnd_parent, None, "RichEditD2DPT", None)
    if not edit:
        return ""
    WM_GETTEXTLENGTH, WM_GETTEXT = 0x000E, 0x000D
    n = user32.SendMessageW(edit, WM_GETTEXTLENGTH, 0, 0)
    buf = ctypes.create_unicode_buffer(n + 1)
    user32.SendMessageW(edit, WM_GETTEXT, n + 1, buf)
    return buf.value


def window_title(hwnd) -> str:
    n = user32.GetWindowTextLengthW(hwnd)
    buf = ctypes.create_unicode_buffer(n + 1)
    user32.GetWindowTextW(hwnd, buf, n + 1)
    return buf.value


def foreground_title() -> str:
    return window_title(user32.GetForegroundWindow())


def focus(hwnd):
    user32.ShowWindow(hwnd, 9)
    # Windows refuses SetForegroundWindow from a background process unless the
    # caller pressed a key recently; a synthetic Alt tap makes it allowed.
    user32.keybd_event(0x12, 0, 0, 0)
    user32.keybd_event(0x12, 0, 2, 0)
    user32.SetForegroundWindow(hwnd)
    time.sleep(0.4)
    if user32.GetForegroundWindow() != hwnd:
        print("  (focus) foreground is", repr(foreground_title()), "instead of Notepad")


def main():
    manifest = json.load(open(os.path.join(CORPUS, "manifest.json"), encoding="utf-8"))
    wanted = sys.argv[1:] or [m["id"] for m in manifest if m["language"] != "none"]
    items = [m for m in manifest if m["id"] in wanted]

    proc = subprocess.Popen(["notepad.exe"])
    time.sleep(1.2)
    hwnd = find_window("Notepad") or find_window("Σημειωματάριο")
    if not hwnd:
        print("Notepad window not found")
        sys.exit(2)
    results = []
    for m in items:
        wav = os.path.join(CORPUS, m["file"])
        with open(FAKE, "w", encoding="utf-8") as f:
            f.write(wav)
        focus(hwnd)
        before = edit_text(hwnd)
        # hold the chord roughly as long as the audio so the level meter looks real
        dur = max(1.0, min(6.0, os.path.getsize(wav) / 32000.0))
        t0 = time.time()
        press_keys.hold(dur)
        released = time.time()
        # wait for text to appear (max 20 s)
        got = before
        while time.time() - released < 20:
            time.sleep(0.15)
            got = edit_text(hwnd)
            if got != before:
                time.sleep(0.4)
                got = edit_text(hwnd)
                break
        inserted = got[len(before):].strip() if got.startswith(before) else got.strip()
        latency = round((time.time() - released) * 1000)
        ok = got != before
        if not ok:
            print("  (debug) foreground now:", repr(foreground_title()), "| notepad text len", len(got))
        results.append({"id": m["id"], "inserted": inserted, "expected": m["expected"], "latency_ms_observed": latency, "ok": ok, "notepad_text": got})
        print(f"[{m['id']}] {'OK ' if ok else 'FAIL'} {latency} ms | {inserted!r}")
        time.sleep(1.0)
    try:
        os.remove(FAKE)
    except OSError:
        pass
    with open(os.path.join(HERE, "e2e-last.json"), "w", encoding="utf-8") as f:
        json.dump(results, f, ensure_ascii=False, indent=2)
    print("inserted", sum(1 for r in results if r["ok"]), "of", len(results))
    try:
        proc.terminate()
    except Exception:
        pass


if __name__ == "__main__":
    main()
