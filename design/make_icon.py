from PIL import Image, ImageDraw
import math
S = 1024
img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(img)
# background: deep teal rounded square (original identity, not Wispr's)
d.rounded_rectangle([64, 64, S-64, S-64], radius=220, fill=(16, 84, 96, 255))
# sound wave bars in warm amber
cx, cy = S/2, S/2
bars = [0.32, 0.55, 0.85, 0.62, 1.0, 0.7, 0.45]
w = 64; gap = 44
total = len(bars)*w + (len(bars)-1)*gap
x = cx - total/2
for h in bars:
    hh = 420*h
    d.rounded_rectangle([x, cy-hh/2, x+w, cy+hh/2], radius=32, fill=(255, 190, 92, 255))
    x += w + gap
img.save("design/icon-src.png")
print("ok")
