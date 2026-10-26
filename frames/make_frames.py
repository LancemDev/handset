#!/usr/bin/env python3
"""Generate phone bezel PNGs (+ .frame metadata) for `scrcpy --frame`.

Each frame is an RGBA PNG whose screen area is fully transparent. The
sidecar `<name>.frame` holds "x y w h" of that area in PNG pixels.
"""
import sys
from pathlib import Path
from PIL import Image, ImageChops, ImageDraw, ImageFilter

SS = 3  # supersampling factor

SCREEN_W, SCREEN_H = 1080, 2400
SCREEN_R = 96          # screen corner radius
BEZEL = 28             # black glass border around the screen
RAIL = 14              # metal frame thickness
MARGIN = 70            # transparent room for buttons + shadow

VARIANTS = {
    # name: (rail light, rail dark, bezel)
    "obsidian": ((92, 95, 102), (28, 29, 33), (6, 6, 8)),
    "porcelain": ((236, 236, 232), (170, 170, 166), (14, 14, 16)),
    "hazel": ((176, 168, 140), (96, 90, 70), (8, 8, 8)),
}


def rrect(draw, box, r, fill):
    draw.rounded_rectangle(box, radius=r, fill=fill)


def vgrad(size, top, bottom):
    w, h = size
    col = Image.new("RGB", (1, h))
    for y in range(h):
        t = y / max(1, h - 1)
        col.putpixel((0, y), tuple(int(top[i] + (bottom[i] - top[i]) * t) for i in range(3)))
    return col.resize((w, h))


def build(name, light, dark, bezel):
    s = SS
    body_w = SCREEN_W + 2 * (BEZEL + RAIL)
    body_h = SCREEN_H + 2 * (BEZEL + RAIL)
    W, H = body_w + 2 * MARGIN, body_h + 2 * MARGIN
    body_r = SCREEN_R + BEZEL + RAIL - 8
    bx, by = MARGIN, MARGIN

    img = Image.new("RGBA", (W * s, H * s), (0, 0, 0, 0))

    # --- soft drop shadow
    sh = Image.new("L", img.size, 0)
    rrect(ImageDraw.Draw(sh), [bx * s, (by + 14) * s, (bx + body_w) * s, (by + body_h + 14) * s], body_r * s, 150)
    sh = sh.filter(ImageFilter.GaussianBlur(26 * s))
    shadow = Image.new("RGBA", img.size, (0, 0, 0, 0))
    shadow.putalpha(sh)
    img = Image.alpha_composite(img, shadow)

    # --- side buttons (right: power + volume), drawn under the rail
    btn = Image.new("RGBA", img.size, (0, 0, 0, 0))
    bd = ImageDraw.Draw(btn)
    right = (bx + body_w) * s
    for top, h in ((by + 520, 170), (by + 800, 300)):
        rrect(bd, [(bx + body_w - 6) * s, top * s, (bx + body_w + 9) * s, (top + h) * s], 6 * s, (*dark, 255))
        rrect(bd, [(bx + body_w - 6) * s, top * s, (bx + body_w + 4) * s, (top + h) * s], 5 * s, (*light, 255))
    img = Image.alpha_composite(img, btn)

    # --- metal rail with vertical gradient + edge highlight
    mask = Image.new("L", img.size, 0)
    rrect(ImageDraw.Draw(mask), [bx * s, by * s, (bx + body_w) * s, (by + body_h) * s], body_r * s, 255)
    grad = vgrad((body_w * s, body_h * s), light, dark).convert("RGBA")
    rail = Image.new("RGBA", img.size, (0, 0, 0, 0))
    rail.paste(grad, (bx * s, by * s))
    rail.putalpha(mask)
    img = Image.alpha_composite(img, rail)

    # thin specular line on the outer edge
    edge = Image.new("L", img.size, 0)
    ed = ImageDraw.Draw(edge)
    rrect(ed, [bx * s, by * s, (bx + body_w) * s, (by + body_h) * s], body_r * s, 255)
    rrect(ed, [(bx + 3) * s, (by + 3) * s, (bx + body_w - 3) * s, (by + body_h - 3) * s], (body_r - 3) * s, 0)
    hi = Image.new("RGBA", img.size, (255, 255, 255, 0))
    hi.putalpha(edge.point(lambda v: int(v * 0.35)))
    img = Image.alpha_composite(img, hi)

    # --- black glass bezel
    ix, iy = bx + RAIL, by + RAIL
    iw, ih = body_w - 2 * RAIL, body_h - 2 * RAIL
    glass = Image.new("RGBA", img.size, (0, 0, 0, 0))
    rrect(ImageDraw.Draw(glass), [ix * s, iy * s, (ix + iw) * s, (iy + ih) * s], (body_r - RAIL) * s, (*bezel, 255))
    img = Image.alpha_composite(img, glass)

    # --- punch the screen hole
    sx, sy = ix + BEZEL, iy + BEZEL
    hole = Image.new("L", img.size, 255)
    rrect(ImageDraw.Draw(hole), [sx * s, sy * s, (sx + SCREEN_W) * s, (sy + SCREEN_H) * s], SCREEN_R * s, 0)
    img.putalpha(ImageChops.multiply(img.getchannel("A"), hole))

    # --- punch-hole camera, drawn over the screen area
    cam = Image.new("RGBA", img.size, (0, 0, 0, 0))
    cd = ImageDraw.Draw(cam)
    cx, cy, cr = sx + SCREEN_W // 2, sy + 62, 24
    cd.ellipse([(cx - cr - 3) * s, (cy - cr - 3) * s, (cx + cr + 3) * s, (cy + cr + 3) * s], fill=(0, 0, 0, 255))
    cd.ellipse([(cx - 11) * s, (cy - 11) * s, (cx + 11) * s, (cy + 11) * s], fill=(18, 22, 40, 255))
    cd.ellipse([(cx - 6) * s, (cy - 6) * s, (cx + 6) * s, (cy + 6) * s], fill=(8, 10, 22, 255))
    cd.ellipse([(cx - 7) * s, (cy - 8) * s, (cx - 2) * s, (cy - 3) * s], fill=(90, 110, 170, 200))
    img = Image.alpha_composite(img, cam)

    img = img.resize((W, H), Image.LANCZOS)
    return img, (sx, sy, SCREEN_W, SCREEN_H)


def main():
    out = Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    out.mkdir(parents=True, exist_ok=True)
    for name, (light, dark, bezel) in VARIANTS.items():
        img, (x, y, w, h) = build(name, light, dark, bezel)
        img.save(out / f"frame-{name}.png")
        (out / f"frame-{name}.frame").write_text(f"{x} {y} {w} {h}\n")
        print("wrote", out / f"frame-{name}.png", (x, y, w, h), img.size)


if __name__ == "__main__":
    main()
