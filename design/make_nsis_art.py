"""The two pictures the Windows installer shows.

NSIS wants BMP files at fixed sizes: a 150x57 banner at the top of every page
and a 164x314 panel on the welcome and finish pages. Both are drawn from the
app icon on the same black ground the app uses.

Run from the project root: python design/make_nsis_art.py
"""
import os

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON = os.path.join(ROOT, "design", "icon-fu-dark.png")
OUT = os.path.join(ROOT, "src-tauri", "icons")
BG = (12, 12, 12)
ACCENT = (200, 255, 90)


def load_mark(size):
    """The white hand alone, without its black tile: the installer pages are
    already black, so the tile would read as a square hole."""
    im = Image.open(ICON).convert("RGBA")
    bbox = im.split()[3].getbbox()
    im = im.crop(bbox)
    # keep the light pixels, drop the dark tile
    px = im.load()
    w, h = im.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = px[x, y]
            px[x, y] = (255, 255, 255, a if (r + g + b) > 250 else 0)
    hand = im.crop(im.split()[3].getbbox())
    ratio = min(size / hand.width, size / hand.height)
    return hand.resize((max(1, int(hand.width * ratio)), max(1, int(hand.height * ratio))), Image.LANCZOS)


def header():
    w, h = 150, 57
    im = Image.new("RGB", (w, h), BG)
    mark = load_mark(h - 14)
    im.paste(mark, (w - mark.width - 10, (h - mark.height) // 2), mark)
    return im


def sidebar():
    w, h = 164, 314
    im = Image.new("RGB", (w, h), BG)
    mark = load_mark(110)
    im.paste(mark, ((w - mark.width) // 2, 70), mark)
    for x in range(w):
        im.putpixel((x, h - 3), ACCENT)
        im.putpixel((x, h - 2), ACCENT)
    return im


os.makedirs(OUT, exist_ok=True)
header().save(os.path.join(OUT, "nsis-header.bmp"))
sidebar().save(os.path.join(OUT, "nsis-sidebar.bmp"))
for f in ("nsis-header.bmp", "nsis-sidebar.bmp"):
    p = os.path.join(OUT, f)
    print(f, Image.open(p).size, os.path.getsize(p), "bytes")
