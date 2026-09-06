"""Photograph the ten shader candidates, dark and light, and check for errors.

Run: "C:/Python314/python.exe" shoot.py [01 05 ...]
"""
import base64
import os
import sys
import time

from selenium import webdriver
from selenium.webdriver.chrome.options import Options

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "shots")
os.makedirs(OUT, exist_ok=True)

PAGES = sorted(f for f in os.listdir(HERE) if f.startswith("s") and f.endswith(".html"))
WANT = sys.argv[1:]
if WANT:
    PAGES = [p for p in PAGES if p[1:3] in WANT]


def driver(w, h):
    o = Options()
    for a in ("--headless=new", "--use-angle=swiftshader", "--enable-unsafe-swiftshader",
              "--hide-scrollbars", "--force-device-scale-factor=1", f"--window-size={w},{h}"):
        o.add_argument(a)
    o.set_capability("goog:loggingPrefs", {"browser": "ALL"})
    return webdriver.Chrome(options=o)


def url(page):
    return "file:///" + os.path.join(HERE, page).replace("\\", "/").replace(" ", "%20")


def shot(d, path, w=1440, h=980):
    r = d.execute_cdp_cmd("Page.captureScreenshot", {
        "captureBeyondViewport": True,
        "clip": {"x": 0, "y": 0, "width": w, "height": h, "scale": 1}})
    with open(path, "wb") as f:
        f.write(base64.b64decode(r["data"]))


def main():
    bad = 0
    for page in PAGES:
        stem = page[:-5]
        d = driver(1440, 900)
        try:
            d.get(url(page))
            time.sleep(4.5)
            state = d.execute_script(
                "var e=document.getElementById('shader-state');"
                "var h=document.getElementById('shader-host');"
                "return [e?e.textContent:'', h?h.className:'', "
                "!!(h&&h.querySelector('canvas')), h?h.style.opacity:''];")
            shot(d, os.path.join(OUT, stem + "-dark.png"))
            d.execute_script(
                "document.documentElement.setAttribute('data-theme','light');"
                "document.dispatchEvent(new CustomEvent('fyf-theme'));")
            time.sleep(2.5)
            shot(d, os.path.join(OUT, stem + "-light.png"))
            errs = [e for e in d.get_log("browser") if e.get("level") == "SEVERE"]
        finally:
            d.quit()
        note, cls, has_canvas, op = state
        ok = has_canvas and "no-gl" not in cls and not note
        if not ok:
            bad += 1
        print(f"{stem:26s} canvas={has_canvas} opacity={op or '1'} "
              f"{'OK' if ok else 'PROBLEM: ' + (note or cls)}")
        for e in errs[:3]:
            print("    ", e.get("message", "")[:220])
    print(f"\n{len(PAGES)} pages, {bad} with a problem")


if __name__ == "__main__":
    main()
