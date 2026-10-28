#!/usr/bin/env python3
"""Pixel-only gates for docs/screenshots/live/live-cursor-*.png (LIVE-01)."""
from __future__ import annotations

import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("check-live-capture-pixels: Pillow required", file=sys.stderr)
    sys.exit(2)

NAMES = [
    "live-cursor-outside.png",
    "live-cursor-in-heading.png",
    "live-cursor-in-bold.png",
]

# Document card rich-text band (1280x800 capture-ready layout).
CONTENT_Y0 = 200
CONTENT_Y1 = 720
CONTENT_X0 = 120
CONTENT_X1 = 1160


def load_rgb(path: Path) -> Image.Image:
    return Image.open(path).convert("RGB")


def luma(r: int, g: int, b: int) -> float:
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def ink_columns(row_pixels: list[tuple[int, int, int]], x0: int, x1: int) -> list[tuple[int, int]]:
    """Return (start, end) x ranges of dark ink on one row."""
    runs: list[tuple[int, int]] = []
    start: int | None = None
    for x in range(x0, x1):
        r, g, b = row_pixels[x]
        if luma(r, g, b) < 118:
            if start is None:
                start = x
        elif start is not None:
            runs.append((start, x - 1))
            start = None
    if start is not None:
        runs.append((start, x1 - 1))
    return runs


def row_double_ink(row_pixels: list[tuple[int, int, int]], x0: int, x1: int) -> bool:
    """True when two ink runs overlap in x (double-strike / stacked glyphs)."""
    runs = ink_columns(row_pixels, x0, x1)
    if len(runs) < 2:
        return False
    for i, (a0, a1) in enumerate(runs):
        for b0, b1 in runs[i + 1 :]:
            overlap = min(a1, b1) - max(a0, b0)
            if overlap >= 3:
                return True
            gap = b0 - a1
            if 0 <= gap <= 4 and (a1 - a0) >= 6 and (b1 - b0) >= 6:
                return True
    return False


def scan_double_ink(im: Image.Image) -> list[int]:
    bad: list[int] = []
    w, h = im.size
    x1 = min(CONTENT_X1, w)
    for y in range(CONTENT_Y0, min(CONTENT_Y1, h)):
        row = [im.getpixel((x, y)) for x in range(w)]
        if row_double_ink(row, CONTENT_X0, x1):
            bad.append(y)
    return bad


def muted_marker_pixels(im: Image.Image) -> int:
    count = 0
    w, h = im.size
    for y in range(CONTENT_Y0, min(CONTENT_Y1, h)):
        for x in range(CONTENT_X0, min(CONTENT_X1, w)):
            r, g, b = im.getpixel((x, y))
            if not (95 <= r <= 150 and 100 <= g <= 155 and 110 <= b <= 165):
                continue
            if luma(r, g, b) > 135:
                continue
            count += 1
    return count


def diff_bbox(a: Image.Image, b: Image.Image) -> tuple[int, int, int, int] | None:
    w, h = a.size
    if b.size != (w, h):
        raise SystemExit("capture size mismatch for diff")
    min_x, min_y = w, h
    max_x, max_y = 0, 0
    found = False
    for y in range(CONTENT_Y0, min(CONTENT_Y1, h)):
        for x in range(CONTENT_X0, min(CONTENT_X1, w)):
            if a.getpixel((x, y)) != b.getpixel((x, y)):
                found = True
                min_x = min(min_x, x)
                max_x = max(max_x, x)
                min_y = min(min_y, y)
                max_y = max(max_y, y)
    if not found:
        return None
    return min_x, min_y, max_x, max_y


def heading_band(im: Image.Image) -> tuple[int, int, int, int] | None:
    w, h = im.size
    top = bottom = None
    for y in range(CONTENT_Y0, min(CONTENT_Y1, h)):
        for x in range(CONTENT_X0, min(CONTENT_X1, w)):
            r, g, b = im.getpixel((x, y))
            if luma(r, g, b) < 118:
                continue
            if top is None:
                top = y
            bottom = y
            break
        if top is not None and bottom is not None:
            break
    if top is None:
        return None
    for y in range(top, min(CONTENT_Y1, h)):
        row_ink = False
        for x in range(CONTENT_X0, min(CONTENT_X1, w)):
            r, g, b = im.getpixel((x, y))
            if luma(r, g, b) < 118:
                row_ink = True
                break
        if not row_ink and y > top + 4:
            bottom = y - 1
            break
        bottom = y
    return CONTENT_X0, top - 6, CONTENT_X1, (bottom or top) + 10


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("docs/screenshots/live")
    paths = {name: root / name for name in NAMES}
    for name, path in paths.items():
        if not path.is_file():
            print(f"missing {path}", file=sys.stderr)
            return 1

    images = {name: load_rgb(path) for name, path in paths.items()}

    for name, im in images.items():
        bad_rows = scan_double_ink(im)
        if bad_rows:
            print(f"{name}: double-ink rows at {bad_rows[:8]}", file=sys.stderr)
            return 1

    outside_markers = muted_marker_pixels(images["live-cursor-outside.png"])
    if outside_markers > 40:
        print(
            f"outside capture shows too many muted marker pixels ({outside_markers})",
            file=sys.stderr,
        )
        return 1

    heading_markers = muted_marker_pixels(images["live-cursor-in-heading.png"])
    if heading_markers < 12:
        print(
            f"heading capture missing dim marker ink ({heading_markers} muted pixels)",
            file=sys.stderr,
        )
        return 1

    bold_markers = muted_marker_pixels(images["live-cursor-in-bold.png"])
    if bold_markers < 12:
        print(
            f"bold capture missing dim marker ink ({bold_markers} muted pixels)",
            file=sys.stderr,
        )
        return 1

    diff = diff_bbox(images["live-cursor-in-heading.png"], images["live-cursor-outside.png"])
    if diff is None:
        print("heading vs outside captures are identical", file=sys.stderr)
        return 1
    band = heading_band(images["live-cursor-in-heading.png"])
    if band is None:
        print("could not locate heading band in heading capture", file=sys.stderr)
        return 1
    bx0, by0, bx1, by1 = band
    dx0, dy0, dx1, dy1 = diff
    if dx0 < bx0 or dy0 < by0 or dx1 > bx1 or dy1 > by1:
        print(
            f"heading/outside diff spills outside heading band: diff={diff} band={band}",
            file=sys.stderr,
        )
        return 1

    print("live capture pixel gate: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
