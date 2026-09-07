"""Photograph the drawn art pages and write the WebP copies the site reads.

Two pictures are drawn as HTML rather than generated as images, because both
have to be exact:

    keyboard.html  the ANSI bottom row with the Alt to the RIGHT of the spacebar
                   lit. An image model cannot place a keycap correctly.
    receipt.html   the bill. Set as text, so every figure on it can be read.

Run from the project root: python press/make_key.py
"""
import io
import os
import subprocess
import time

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HERE = os.path.join(ROOT, "press")
SITE = os.path.join(ROOT, "site", "assets")
WORK = os.path.join(os.environ.get("TEMP", "."), "ffart")
CHROME = r"C:\Program Files\Google\Chrome\Application\chrome.exe"

# source page -> (width, height, asset stem, webp quality)
JOBS = [
    ("keyboard.html", 1600, 900, "art-key-macro", 88),
    ("receipt.html", 1000, 1250, "art-receipt", 90),
]


def shoot(src_html, w, h, png):
    if os.path.exists(png):
        os.remove(png)
    subprocess.run([CHROME, "--headless=new", "--disable-gpu", "--hide-scrollbars",
                    "--user-data-dir=" + os.path.join(WORK, "profile"),
                    f"--window-size={w},{h}", f"--screenshot={png}",
                    "--force-device-scale-factor=1",
                    "file:///" + src_html.replace("\\", "/")],
                   check=False, capture_output=True, timeout=180)
    # headless Chrome returns before the child process has written the file
    for _ in range(120):
        if os.path.exists(png) and os.path.getsize(png) > 0:
            break
        time.sleep(0.5)
    return os.path.exists(png)


if __name__ == "__main__":
    os.makedirs(WORK, exist_ok=True)
    os.makedirs(SITE, exist_ok=True)
    failed = []
    for page, w, h, stem, q in JOBS:
        src = os.path.join(HERE, page)
        png = os.path.join(WORK, stem + ".png")
        if not shoot(src, w, h, png):
            failed.append(stem)
            print(stem, "-> MISSING")
            continue
        im = Image.open(png).convert("RGB")
        if (im.width, im.height) != (w, h):
            failed.append(stem + " (wrong size %sx%s)" % (im.width, im.height))
        dst = os.path.join(SITE, stem + ".webp")
        im.save(dst, "WEBP", quality=q, method=6)
        # a readable PNG stays next to the source for checking
        im.save(os.path.join(HERE, "shots", stem + ".png"))
        print("%-16s %sx%s  %s KB -> site/assets/%s.webp"
              % (stem, im.width, im.height, round(os.path.getsize(dst) / 1024), stem))
    if failed:
        raise SystemExit("FAILED: " + ", ".join(failed))
    print("art written")
