"""Web-ready art for fuckyouflow.app.

Takes the four GPT renders from the run folder, writes lean WebP copies into
site/assets/, and builds two candidate social cards from them.

Run from anywhere:  python site/make_art.py
"""
import os
from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = r"C:\Claude Projects\generated-images\fuckyouflow-site-2026-09-06"
OUT = os.path.join(ROOT, "site", "assets")
ICON = os.path.join(ROOT, "design", "icon-fu-dark.png")

BG = (12, 12, 12)
ACCENT = (200, 255, 90)
FG = (242, 242, 242)
MUTED = (143, 143, 143)

# name -> target width on the page (2x of the biggest slot it fills)
TARGETS = {
    "art-key-macro": 1600,
    "art-wave-letters": 1920,
    "art-receipt": 1000,
    "art-hand-hero": 1000,
}


def font(size, bold=False):
    names = ["consolab.ttf"] if bold else ["consola.ttf"]
    names += ["CascadiaMono.ttf", "cour.ttf"]
    for name in names:
        p = os.path.join(os.environ.get("WINDIR", "C:/Windows"), "Fonts", name)
        if os.path.exists(p):
            return ImageFont.truetype(p, size)
    return ImageFont.load_default()


def cover(im, w, h, anchor_x=0.5):
    """Scale and crop im to exactly w x h. anchor_x 0 keeps the left edge."""
    r = max(w / im.width, h / im.height)
    im = im.resize((round(im.width * r), round(im.height * r)), Image.LANCZOS)
    left = round((im.width - w) * anchor_x)
    top = (im.height - h) // 2
    return im.crop((left, top, left + w, top + h))


loaded = {}
print("== webp ==")
for stem, width in TARGETS.items():
    src = os.path.join(SRC, stem + ".png")
    im = Image.open(src).convert("RGB")
    loaded[stem] = im
    if im.width > width:
        h = round(im.height * width / im.width)
        out_im = im.resize((width, h), Image.LANCZOS)
    else:
        out_im = im
    dst = os.path.join(OUT, stem + ".webp")
    out_im.save(dst, "WEBP", quality=82, method=6)
    print("%-20s %sx%s  %s KB" % (stem + ".webp", out_im.width, out_im.height,
                                  round(os.path.getsize(dst) / 1024)))

# ---- social cards, 1200x630 ----------------------------------------------
def card(art, path, art_width=430, anchor_x=0.5, fade=150):
    im = Image.new("RGB", (1200, 630), BG)
    panel = cover(art, art_width, 630, anchor_x)
    # fade the art into the black on its right edge, so there is no seam
    mask = Image.new("L", (art_width, 630), 255)
    md = ImageDraw.Draw(mask)
    for i in range(fade):
        x = art_width - fade + i
        md.line([(x, 0), (x, 630)], fill=255 - round(255 * (i + 1) / fade))
    im.paste(panel, (0, 0), mask)
    d = ImageDraw.Draw(im)
    x = art_width + 60
    d.text((x, 170), "Fuck You Flow", font=font(66, True), fill=FG)
    d.text((x, 256), "0.9 beta", font=font(28), fill=ACCENT)
    d.text((x, 320), "Free dictation for Windows.", font=font(34), fill=FG)
    d.text((x, 366), "Greek and English. Runs on your PC.", font=font(34), fill=FG)
    d.text((x, 470), "fuckyouflow.app", font=font(28), fill=MUTED)
    d.rectangle([0, 622, 1200, 630], fill=ACCENT)
    im.save(path, optimize=True)
    print("%-26s %s KB" % (os.path.basename(path), round(os.path.getsize(path) / 1024)))


print("== social cards ==")
card(loaded["art-hand-hero"], os.path.join(OUT, "og-hand.png"), anchor_x=0.5)
card(loaded["art-wave-letters"], os.path.join(OUT, "og-wave.png"), art_width=520, anchor_x=0.0)
# the hand card is the one the pages point at. Swap the source here to change it.
card(loaded["art-hand-hero"], os.path.join(OUT, "og.png"), anchor_x=0.5)
