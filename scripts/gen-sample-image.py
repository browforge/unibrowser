#!/usr/bin/env python3
"""Regenerate examples/demo-pack/sample.png (raster image VQA fixture)."""
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "examples" / "demo-pack" / "sample.png"
WIDTH = 1600
HEIGHT = 1000


def _font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    for path in (
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
    ):
        if Path(path).is_file():
            return ImageFont.truetype(path, size)
    return ImageFont.load_default()


def main() -> None:
    img = Image.new("RGB", (WIDTH, HEIGHT))
    px = img.load()
    for y in range(HEIGHT):
        t = y / (HEIGHT - 1)
        for x in range(WIDTH):
            u = x / (WIDTH - 1)
            r = int(40 + 180 * u)
            g = int(90 + 120 * (1 - u) * t)
            b = int(200 - 140 * t + 60 * u)
            px[x, y] = (r, g, b)

    draw = ImageDraw.Draw(img)
    draw.rectangle((80, 80, WIDTH - 80, HEIGHT - 80), outline=(255, 255, 255), width=4)
    draw.rectangle((120, 120, WIDTH - 120, HEIGHT - 120), outline=(20, 20, 28), width=2)
    for i in range(12):
        x0 = 160 + i * 110
        draw.line((x0, 200, x0 + 40, HEIGHT - 200), fill=(255, 240, 200), width=3)
    draw.ellipse((WIDTH // 2 - 180, HEIGHT // 2 - 120, WIDTH // 2 + 180, HEIGHT // 2 + 120), outline=(255, 255, 255), width=5)
    title_font = _font(56)
    sub_font = _font(28)
    draw.text((140, 160), "North Bay colour study", fill=(255, 255, 255), font=title_font)
    draw.text(
        (140, 240),
        "1600 x 1000  gradient and line study",
        fill=(240, 248, 255),
        font=sub_font,
    )
    draw.text((140, HEIGHT - 200), "Generated locally", fill=(30, 30, 40), font=sub_font)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    img.save(OUT, format="PNG", optimize=True)
    print(f"wrote {OUT} ({WIDTH}x{HEIGHT})")


if __name__ == "__main__":
    main()
