"""Parody icon: Wispr Flow's bars, rearranged into a hand giving the finger.

Two tiles are written next to this script:
  icon-fu-light.png  black hand on a white rounded tile (the Wispr look)
  icon-fu-dark.png   white hand on a black rounded tile
Both are 1024 x 1024 with a transparent margin, ready for `pnpm tauri icon`.
"""
from PIL import Image, ImageDraw

S = 1024


def tile(fg, bg):
    img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle([64, 64, S - 64, S - 64], radius=220, fill=bg)

    # Fingers: five bars with rounded ends, the middle one raised. Same bar
    # language as the original mark, different message.
    w = 74
    gap = 34
    tops = {"index": 470, "middle": 250, "ring": 470, "pinky": 520}
    order = ["index", "middle", "ring", "pinky"]
    total = len(order) * w + (len(order) - 1) * gap
    x = S / 2 - total / 2 + 40
    bottom = 720
    for name in order:
        d.rounded_rectangle([x, tops[name], x + w, bottom], radius=w / 2, fill=fg)
        x += w + gap

    # Palm: one wide rounded block the fingers grow out of.
    palm_left = S / 2 - total / 2 + 40 - 10
    palm_right = palm_left + total + 20
    d.rounded_rectangle([palm_left, 620, palm_right, 800], radius=70, fill=fg)

    # Thumb: a bar standing on the palm's left edge, tilted outwards. Drawn on
    # its own layer and rotated around its base so it stays attached.
    tx = palm_left + 34
    thumb = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    td = ImageDraw.Draw(thumb)
    td.rounded_rectangle([tx - 37, 470, tx + 37, 760], radius=37, fill=fg)
    thumb = thumb.rotate(34, resample=Image.BICUBIC, center=(tx, 740))
    img.alpha_composite(thumb)
    return img


tile((20, 20, 20, 255), (255, 255, 255, 255)).save("design/icon-fu-light.png")
tile((255, 255, 255, 255), (16, 16, 16, 255)).save("design/icon-fu-dark.png")
print("ok")
