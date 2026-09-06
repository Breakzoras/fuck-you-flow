"""Photograph the five front-page variants and lay them side by side.

For each variants/vN.html: desktop fold (1440x900, dark), the same in light,
a full-page capture (dark) and a phone fold (390x844). Then one contact sheet
(shots/contact-sheet.png) and a local gallery (index.html) that opens by
double-click. Nothing here touches the live site.

Run from anywhere: "C:/Python314/python.exe" "C:/Claude Projects/lalia/site/variants/shoot.py" [v1 v2 ...]
"""
import base64
import os
import sys
import time

from PIL import Image, ImageDraw, ImageFont
from selenium import webdriver
from selenium.webdriver.chrome.options import Options

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "shots")
os.makedirs(OUT, exist_ok=True)

NAMES = {"v1": "Console", "v2": "The bill", "v3": "Poster", "v4": "Studio", "v5": "Manifesto"}
WANTED = sys.argv[1:] or ["v1", "v2", "v3", "v4", "v5"]


def driver(w, h):
    o = Options()
    for a in ("--headless=new", "--use-angle=swiftshader", "--enable-unsafe-swiftshader",
              "--hide-scrollbars", "--force-device-scale-factor=1", f"--window-size={w},{h}"):
        o.add_argument(a)
    o.set_capability("goog:loggingPrefs", {"browser": "ALL"})
    return webdriver.Chrome(options=o)


def url(v):
    return "file:///" + os.path.join(HERE, v + ".html").replace("\\", "/").replace(" ", "%20")


def cdp_full(d, w):
    m = d.execute_cdp_cmd("Page.getLayoutMetrics", {})
    h = int(min(m["cssContentSize"]["height"], 9000))
    r = d.execute_cdp_cmd("Page.captureScreenshot", {
        "captureBeyondViewport": True,
        "clip": {"x": 0, "y": 0, "width": w, "height": h, "scale": 1}})
    return base64.b64decode(r["data"])


def set_theme(d, t):
    d.execute_script(
        "document.documentElement.setAttribute('data-theme', arguments[0]);"
        "document.dispatchEvent(new CustomEvent('fyf-theme'));", t)
    time.sleep(1.2)


def shoot(v):
    d = driver(1440, 900)
    try:
        d.get(url(v))
        time.sleep(3)
        errors = [e for e in d.get_log("browser") if e.get("level") == "SEVERE"]
        d.save_screenshot(os.path.join(OUT, f"{v}-desktop-dark.png"))
        with open(os.path.join(OUT, f"{v}-full-dark.png"), "wb") as f:
            f.write(cdp_full(d, 1440))
        set_theme(d, "light")
        d.save_screenshot(os.path.join(OUT, f"{v}-desktop-light.png"))
        with open(os.path.join(OUT, f"{v}-full-light.png"), "wb") as f:
            f.write(cdp_full(d, 1440))
    finally:
        d.quit()
    d = driver(390, 844)
    try:
        d.get(url(v))
        time.sleep(3)
        d.save_screenshot(os.path.join(OUT, f"{v}-phone-dark.png"))
    finally:
        d.quit()
    return errors


def fit(img, w, h=None):
    r = w / img.width
    nh = int(img.height * r)
    im = img.resize((w, nh), Image.LANCZOS)
    if h and nh > h:
        im = im.crop((0, 0, w, h))
    if h and nh < h:
        canvas = Image.new("RGB", (w, h), (12, 12, 12))
        canvas.paste(im, (0, 0))
        im = canvas
    return im


def sheet(vs):
    col_w, gap, label_h = 560, 24, 44
    rows = [("desktop-dark", 350), ("desktop-light", 350), ("phone-dark", 520), ("full-dark", 900)]
    W = gap + len(vs) * (col_w + gap)
    H = gap + label_h + sum(r[1] + gap for r in rows) + 8
    im = Image.new("RGB", (W, H), (0, 0, 0))
    dr = ImageDraw.Draw(im)
    try:
        font = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 26)
    except OSError:
        font = ImageFont.load_default()
    for i, v in enumerate(vs):
        x = gap + i * (col_w + gap)
        dr.text((x, gap), f"{v}  {NAMES.get(v, '')}", fill=(200, 255, 90), font=font)
        y = gap + label_h
        for kind, h in rows:
            p = os.path.join(OUT, f"{v}-{kind}.png")
            if os.path.exists(p):
                src = Image.open(p).convert("RGB")
                if kind == "phone-dark":
                    t = fit(src, 240, h)
                    tile = Image.new("RGB", (col_w, h), (12, 12, 12))
                    tile.paste(t, ((col_w - 240) // 2, 0))
                else:
                    tile = fit(src, col_w, h)
                im.paste(tile, (x, y))
                dr.rectangle((x, y, x + col_w - 1, y + h - 1), outline=(42, 42, 42))
            y += h + gap
    out = os.path.join(OUT, "contact-sheet.png")
    im.save(out)
    return out


def gallery(vs):
    cards = "".join(f"""
    <section>
      <h2><a href="{v}.html">{v} {NAMES.get(v, '')}</a></h2>
      <div class="row">
        <a href="shots/{v}-desktop-dark.png"><img src="shots/{v}-desktop-dark.png" alt="{v} desktop dark"></a>
        <a href="shots/{v}-desktop-light.png"><img src="shots/{v}-desktop-light.png" alt="{v} desktop light"></a>
        <a href="shots/{v}-phone-dark.png"><img class="phone" src="shots/{v}-phone-dark.png" alt="{v} phone"></a>
      </div>
      <details><summary>Whole page</summary><img src="shots/{v}-full-dark.png" alt="{v} full page"></details>
    </section>""" for v in vs)
    html = f"""<!doctype html><html lang="el"><head><meta charset="utf-8"><title>Fuck You Flow: five front pages</title>
<style>
body{{margin:0;background:#000;color:#f2f2f2;font:18px/1.5 Consolas,"Cascadia Mono",monospace;padding:24px}}
h1{{font-size:28px;margin:0 0 6px}} p{{color:#a3a3a3;margin:0 0 24px}}
.grid{{display:grid;grid-template-columns:repeat(5,1fr);gap:20px}}
section{{border:1px solid #2a2a2a;padding:12px;background:#0c0c0c}}
h2{{font-size:22px;margin:0 0 10px}} h2 a{{color:#c8ff5a}}
.row{{display:grid;gap:8px}} .row img{{width:100%;display:block;border:1px solid #2a2a2a}}
.row a:last-child{{justify-self:center}} .row img.phone{{width:44%;margin:0 auto}}
details{{margin-top:10px;color:#a3a3a3}} details img{{width:100%;margin-top:8px;border:1px solid #2a2a2a}}
@media(max-width:1500px){{.grid{{grid-template-columns:repeat(3,1fr)}}}}
@media(max-width:900px){{.grid{{grid-template-columns:1fr}}}}
</style></head><body>
<h1>Fuck You Flow: five front pages, side by side</h1>
<p>Click a title to open the real page (dark and light toggle inside). Top: desktop dark, desktop light, phone. Open "Whole page" for the full scroll.</p>
<div class="grid">{cards}</div>
</body></html>"""
    p = os.path.join(HERE, "index.html")
    with open(p, "w", encoding="utf-8") as f:
        f.write(html)
    return p


if __name__ == "__main__":
    all_errors = {}
    for v in WANTED:
        if not os.path.exists(os.path.join(HERE, v + ".html")):
            print(f"{v}: missing")
            continue
        errs = shoot(v)
        all_errors[v] = errs
        print(f"{v}: done, console errors: {len(errs)}")
        for e in errs[:5]:
            print("   ", e.get("message", "")[:200])
    present = [v for v in ["v1", "v2", "v3", "v4", "v5"] if os.path.exists(os.path.join(OUT, f"{v}-desktop-dark.png"))]
    print("sheet:", sheet(present))
    print("gallery:", gallery(present))
