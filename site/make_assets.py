"""Static pictures for fuckyouflow.app: favicon, logo, social card.
Run from the project root: python site/make_assets.py
"""
import os
from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON = os.path.join(ROOT, "design", "icon-fu-dark.png")
OUT = os.path.join(ROOT, "site", "assets")
BG = (12, 12, 12)
ACCENT = (200, 255, 90)
FG = (242, 242, 242)
MUTED = (143, 143, 143)

def font(size, bold=False):
    for name in (["consolab.ttf", "consola.ttf"][0 if bold else 1], "CascadiaMono.ttf", "cour.ttf"):
        p = os.path.join(os.environ.get("WINDIR", "C:/Windows"), "Fonts", name)
        if os.path.exists(p):
            return ImageFont.truetype(p, size)
    return ImageFont.load_default()

icon = Image.open(ICON).convert("RGBA")
for s in (32, 64, 180, 256, 512):
    icon.resize((s, s), Image.LANCZOS).save(os.path.join(OUT, f"icon-{s}.png"))
icon.resize((48, 48), Image.LANCZOS).save(os.path.join(OUT, "favicon.ico"), sizes=[(16, 16), (32, 32), (48, 48)])

# social card 1200x630
im = Image.new("RGB", (1200, 630), BG)
d = ImageDraw.Draw(im)
mark = icon.resize((300, 300), Image.LANCZOS)
im.paste(mark, (80, 165), mark)
d.text((430, 170), "Fuck You Flow", font=font(72, True), fill=FG)
d.text((430, 262), "0.9 beta", font=font(30), fill=ACCENT)
d.text((430, 330), "Free dictation for Windows.", font=font(38), fill=FG)
d.text((430, 380), "Greek and English. Runs on your PC.", font=font(38), fill=FG)
d.text((430, 470), "fuckyouflow.app", font=font(30), fill=MUTED)
d.rectangle([0, 622, 1200, 630], fill=ACCENT)
im.save(os.path.join(OUT, "og.png"), optimize=True)
for f in sorted(os.listdir(OUT)):
    print(f, os.path.getsize(os.path.join(OUT, f)))
