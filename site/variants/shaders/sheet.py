"""One contact sheet of the ten candidates: dark on the left, light on the right."""
import os

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "shots")

ITEMS = [("01", "god-rays", "God rays"), ("02", "mesh-gradient", "Mesh gradient"),
         ("03", "grain-gradient", "Grain gradient"), ("04", "smoke-ring", "Smoke ring"),
         ("05", "liquid-metal", "Liquid metal"), ("06", "neuro-noise", "Neuro noise"),
         ("07", "warp", "Warp"), ("08", "dithering", "Dithering"),
         ("09", "waves", "Waves"), ("10", "gem-smoke", "Gem smoke")]

HALF, ROW_H, LABEL, GAP, COLS = 430, 300, 40, 18, 2
CELL_W = HALF * 2


def fit(path, w, h):
    im = Image.open(path).convert("RGB")
    r = w / im.width
    im = im.resize((w, int(im.height * r)), Image.LANCZOS)
    return im.crop((0, 0, w, h)) if im.height >= h else im


def main():
    rows = (len(ITEMS) + COLS - 1) // COLS
    W = GAP + COLS * (CELL_W + GAP)
    H = GAP + rows * (LABEL + ROW_H + GAP)
    sheet = Image.new("RGB", (W, H), (0, 0, 0))
    dr = ImageDraw.Draw(sheet)
    try:
        font = ImageFont.truetype("C:/Windows/Fonts/consolab.ttf", 24)
    except OSError:
        font = ImageFont.load_default()

    for i, (n, slug, title) in enumerate(ITEMS):
        cx, cy = i % COLS, i // COLS
        x = GAP + cx * (CELL_W + GAP)
        y = GAP + cy * (LABEL + ROW_H + GAP)
        dr.text((x, y + 6), f"{n}  {title}", fill=(200, 255, 90), font=font)
        for k, theme in enumerate(("dark", "light")):
            p = os.path.join(OUT, f"s{n}-{slug}-{theme}.png")
            if os.path.exists(p):
                sheet.paste(fit(p, HALF, ROW_H), (x + k * HALF, y + LABEL))
        dr.rectangle((x, y + LABEL, x + CELL_W - 1, y + LABEL + ROW_H - 1), outline=(42, 42, 42))

    path = os.path.join(OUT, "contact-sheet.png")
    sheet.save(path)
    print(path)


if __name__ == "__main__":
    main()
