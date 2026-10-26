#!/usr/bin/env python3
"""Build app/packaging/AppIcon.icns: the obsidian frame on a rounded square."""
import subprocess, sys
from pathlib import Path
from PIL import Image, ImageDraw

root = Path(__file__).resolve().parent.parent
frame = Image.open(root / "app/assets/frame-obsidian.png").convert("RGBA")
hx, hy, hw, hh = map(int, (root / "app/assets/frame-obsidian.frame").read_text().split())
# the metadata is in pixels of the original 1304x2624 image; the embedded PNG is the same size
assert frame.size == (1304, 2624), frame.size

S = 1024
# background: deep blue -> violet diagonal gradient on a macOS-style rounded square
bg = Image.new("RGBA", (S, S))
px = bg.load()
for y in range(S):
    for x in range(S):
        t = (x + y) / (2 * S)
        px[x, y] = (int(24 + 46 * t), int(28 + 22 * t), int(60 + 90 * t), 255)
mask = Image.new("L", (S, S), 0)
ImageDraw.Draw(mask).rounded_rectangle([40, 40, S - 40, S - 40], radius=185, fill=255)
icon = Image.new("RGBA", (S, S), (0, 0, 0, 0))
icon.paste(bg, (0, 0), mask)

# the phone, with a wallpaper gradient behind the transparent screen
ph = 800
k = ph / frame.height
phone = frame.resize((round(frame.width * k), ph), Image.LANCZOS)
wall = Image.new("RGBA", phone.size, (0, 0, 0, 0))
wp = wall.load()
x0, y0, x1, y1 = (round(v * k) for v in (hx, hy, hx + hw, hy + hh))
for y in range(y0, y1):
    for x in range(x0, x1):
        t = (y - y0) / max(1, y1 - y0)
        wp[x, y] = (int(255 - 60 * t), int(150 - 100 * t + 40 * (x - x0) / (x1 - x0)), int(90 + 120 * t), 255)
wall.alpha_composite(phone)
icon.alpha_composite(wall, ((S - wall.width) // 2, (S - wall.height) // 2 + 6))

out = root / "app/packaging"
iconset = out / "AppIcon.iconset"
iconset.mkdir(parents=True, exist_ok=True)
for size in (16, 32, 128, 256, 512):
    icon.resize((size, size), Image.LANCZOS).save(iconset / f"icon_{size}x{size}.png")
    icon.resize((size * 2, size * 2), Image.LANCZOS).save(iconset / f"icon_{size}x{size}@2x.png")
subprocess.run(["iconutil", "-c", "icns", str(iconset), "-o", str(out / "AppIcon.icns")], check=True)
for f in iconset.iterdir():
    f.unlink()
iconset.rmdir()
icon.resize((256, 256), Image.LANCZOS).save(out / "icon-preview.png")
print("wrote", out / "AppIcon.icns")
