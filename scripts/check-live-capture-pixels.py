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
DOC_X0 = 420
DOC_X1 = 980
DOC_Y0 = 190
DOC_Y1 = 670


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
    for y in range(DOC_Y0, min(DOC_Y1, h)):
        for x in range(DOC_X0, min(DOC_X1, w)):
            r, g, b = im.getpixel((x, y))
            if is_muted_marker_rgb(r, g, b):
                count += 1
    return count


def is_muted_marker_rgb(r: int, g: int, b: int) -> bool:
    if not (95 <= r <= 150 and 100 <= g <= 155 and 110 <= b <= 165):
        return False
    return luma(r, g, b) < 135


def muted_marker_delta(active: Image.Image, reference: Image.Image) -> int:
    """Muted ink present in active but not reference at the same pixel."""
    w, h = active.size
    if reference.size != (w, h):
        raise SystemExit("capture size mismatch for muted delta")
    count = 0
    for y in range(DOC_Y0, min(DOC_Y1, h)):
        for x in range(DOC_X0, min(DOC_X1, w)):
            ar, ag, ab = active.getpixel((x, y))
            rr, rg, rb = reference.getpixel((x, y))
            if is_muted_marker_rgb(ar, ag, ab) and not is_muted_marker_rgb(rr, rg, rb):
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


def marker_delta_band(active: Image.Image, reference: Image.Image) -> tuple[int, int, int, int] | None:
    """Row band where active shows extra muted ink vs reference (live marker widgets)."""
    w, h = active.size
    if reference.size != (w, h):
        return None
    best_y = None
    best = 0
    for y in range(DOC_Y0, min(DOC_Y1, h)):
        row_extra = 0
        for x in range(DOC_X0, min(DOC_X1, w)):
            ar, ag, ab = active.getpixel((x, y))
            rr, rg, rb = reference.getpixel((x, y))
            if is_muted_marker_rgb(ar, ag, ab) and not is_muted_marker_rgb(rr, rg, rb):
                row_extra += 1
        if row_extra > best:
            best = row_extra
            best_y = y
    if best_y is None or best < 8:
        return None
    return CONTENT_X0, best_y - 12, min(CONTENT_X1, w), best_y + 36


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
    heading_markers = muted_marker_pixels(images["live-cursor-in-heading.png"])
    bold_markers = muted_marker_pixels(images["live-cursor-in-bold.png"])

    heading_extra = muted_marker_delta(
        images["live-cursor-in-heading.png"],
        images["live-cursor-outside.png"],
    )
    bold_extra = muted_marker_delta(
        images["live-cursor-in-bold.png"],
        images["live-cursor-outside.png"],
    )
    outside_extra_vs_heading = muted_marker_delta(
        images["live-cursor-outside.png"],
        images["live-cursor-in-heading.png"],
    )

    if heading_extra < 12:
        print(
            f"heading capture missing dim marker ink vs outside ({heading_extra} delta pixels)",
            file=sys.stderr,
        )
        return 1

    if bold_extra < 12:
        print(
            f"bold capture missing dim marker ink vs outside ({bold_extra} delta pixels)",
            file=sys.stderr,
        )
        return 1

    if heading_extra <= outside_extra_vs_heading:
        print(
            "outside capture still shows marker-class ink comparable to heading "
            f"(heading_extra={heading_extra}, outside_extra={outside_extra_vs_heading})",
            file=sys.stderr,
        )
        return 1

    if heading_markers < 12:
        print(
            f"heading capture missing dim marker ink ({heading_markers} muted pixels)",
            file=sys.stderr,
        )
        return 1

    if bold_markers < 12:
        print(
            f"bold capture missing dim marker ink ({bold_markers} muted pixels)",
            file=sys.stderr,
        )
        return 1

    if outside_markers > heading_markers:
        print(
            f"outside capture shows more muted ink than heading ({outside_markers} > {heading_markers})",
            file=sys.stderr,
        )
        return 1

    diff = diff_bbox(images["live-cursor-in-heading.png"], images["live-cursor-outside.png"])
    if diff is None:
        print("heading vs outside captures are identical", file=sys.stderr)
        return 1
    band = marker_delta_band(
        images["live-cursor-in-heading.png"],
        images["live-cursor-outside.png"],
    )
    if band is None:
        print("could not locate heading marker band in heading/outside delta", file=sys.stderr)
        return 1
    bx0, by0, bx1, by1 = band

    in_band_marker_diff = 0
    outside_band_heading_marker = 0
    w, h = images["live-cursor-in-heading.png"].size
    for y in range(DOC_Y0, min(DOC_Y1, h)):
        for x in range(DOC_X0, min(DOC_X1, w)):
            head_px = images["live-cursor-in-heading.png"].getpixel((x, y))
            out_px = images["live-cursor-outside.png"].getpixel((x, y))
            if head_px != out_px and by0 <= y <= by1 and bx0 <= x <= bx1:
                in_band_marker_diff += 1
            ar, ag, ab = head_px
            rr, rg, rb = out_px
            if is_muted_marker_rgb(ar, ag, ab) and not is_muted_marker_rgb(rr, rg, rb):
                if y < by0 or y > by1 or x < bx0 or x > bx1:
                    outside_band_heading_marker += 1

    if in_band_marker_diff < 12:
        print(
            f"heading/outside captures do not differ enough inside marker band "
            f"({in_band_marker_diff} pixels, band={band})",
            file=sys.stderr,
        )
        return 1

    if outside_band_heading_marker > 24:
        print(
            "heading capture shows marker-class ink outside the heading band vs outside "
            f"({outside_band_heading_marker} pixels, band={band})",
            file=sys.stderr,
        )
        return 1

    print("live capture pixel gate: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
