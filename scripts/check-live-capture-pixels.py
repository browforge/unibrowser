#!/usr/bin/env python3
"""Pixel gates for docs/screenshots/live/*.png (no cm.json pass/fail fields)."""
from __future__ import annotations

import hashlib
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:  # pragma: no cover
    print("check-live-capture-pixels: Pillow required", file=sys.stderr)
    sys.exit(2)

NAMES = [
    "live-all-rendered-desktop.png",
    "live-heading-marker-desktop.png",
    "live-bold-marker-desktop.png",
]

INK = 118
SQUIGGLE_COLORS = [(194, 46, 46), (0, 120, 212), (199, 46, 46), (0, 120, 215)]


def file_md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def is_ink(px: tuple[int, int, int]) -> bool:
    r, g, b = px
    return r < INK and g < INK and b < INK


def near(px: tuple[int, int, int], target: tuple[int, int, int], tol: int = 36) -> bool:
    return all(abs(int(px[i]) - target[i]) <= tol for i in range(3))


def is_squiggle(px: tuple[int, int, int]) -> bool:
    return any(near(px, c, 42) for c in SQUIGGLE_COLORS)


def find_editor_bbox(im: Image.Image) -> tuple[int, int, int, int]:
    w, h = im.size
    best = (0, 0, 0, 0, 0)
    step = 4
    for y in range(140, h - 80, step):
        for x in range(240, w - 120, step):
            px = im.getpixel((x, y))
            if px[0] < 245 or px[1] < 245 or px[2] < 240:
                continue
            x2 = x
            y2 = y
            while x2 < w - 1 and im.getpixel((x2, y))[0] > 240:
                x2 += 8
            while y2 < h - 1 and im.getpixel((x, y2))[0] > 240:
                y2 += 8
            area = (x2 - x) * (y2 - y)
            if area > best[4]:
                best = (x, y, x2, y2, area)
    if best[4] < 40_000:
        return (280, 200, 1040, 680)
    return (best[0], best[1], best[2], best[3])


def ink_row_bands(im: Image.Image, box: tuple[int, int, int, int]) -> list[tuple[int, int, int]]:
    x1, y1, x2, y2 = box
    bands: list[tuple[int, int, int]] = []
    in_band = False
    start = y1
    for y in range(y1, y2):
        count = sum(1 for x in range(x1 + 40, x2 - 20) if is_ink(im.getpixel((x, y))))
        if count >= 8:
            if not in_band:
                start = y
                in_band = True
        elif in_band:
            bands.append((start, y, y - start))
            in_band = False
    if in_band:
        bands.append((start, y2, y2 - start))
    return [b for b in bands if b[2] >= 6]


def band_ink_height(im: Image.Image, box: tuple[int, int, int, int], y_start: int, y_end: int) -> int:
    x1, _, x2, _ = box
    ys = []
    for y in range(y_start, y_end):
        for x in range(x1 + 40, x2 - 20):
            if is_ink(im.getpixel((x, y))):
                ys.append(y)
    if not ys:
        return 0
    return max(ys) - min(ys) + 1


def leftmost_ink_x(im: Image.Image, box: tuple[int, int, int, int], y_start: int, y_end: int) -> int | None:
    x1, _, x2, _ = box
    best = None
    for y in range(y_start, y_end):
        for x in range(x1 + 8, x2 - 20):
            if is_ink(im.getpixel((x, y))):
                if best is None or x < best:
                    best = x
    return best


def count_hash_marker_rows(im: Image.Image, box: tuple[int, int, int, int], y_start: int, y_end: int) -> int:
    """Rows with a '#' glyph: short horizontal bar mid-line plus left vertical stroke."""
    x0 = leftmost_ink_x(im, box, y_start, y_end)
    if x0 is None:
        return 0
    hits = 0
    mid_y = (y_start + y_end) // 2
    for y in range(y_start, y_end):
        cols = [x for x in range(x0, min(x0 + 10, box[2])) if is_ink(im.getpixel((x, y)))]
        if len(cols) < 2 or cols[-1] - cols[0] > 8:
            continue
        # Hash crossbar: ink on the same row near the left column.
        if y == mid_y and len(cols) >= 2:
            hits += 1
            continue
        # Double-hash heading marker: two tight columns on consecutive rows.
        if len(cols) <= 4 and y > y_start and y < y_end - 1:
            prev = [x for x in range(x0, min(x0 + 10, box[2])) if is_ink(im.getpixel((x, y - 1)))]
            if prev and abs(prev[0] - cols[0]) <= 2:
                hits += 1
    return hits


def count_asterisk_pairs(im: Image.Image, box: tuple[int, int, int, int], y_start: int, y_end: int) -> int:
    x1, _, x2, _ = box
    pairs = 0
    for y in range(y_start, y_end):
        run = 0
        for x in range(x1 + 40, x2 - 20):
            px = im.getpixel((x, y))
            if is_ink(px) and max(px) - min(px) < 8:
                run += 1
                if run == 2:
                    pairs += 1
                    run = 0
            else:
                run = 0
    return pairs


def squiggle_pixels(im: Image.Image, box: tuple[int, int, int, int]) -> int:
    x1, y1, x2, y2 = box
    n = 0
    for y in range(y1, y2):
        for x in range(x1 + 30, x2 - 20):
            if is_squiggle(im.getpixel((x, y))):
                n += 1
    return n


def marker_tone_pixels(im: Image.Image, box: tuple[int, int, int, int], y_start: int, y_end: int) -> int:
    """Dimmed markdown marker ink (lighter grey than body text)."""
    x1, _, x2, _ = box
    n = 0
    for y in range(y_start, y_end):
        for x in range(x1 + 8, x2 - 20):
            r, g, b = im.getpixel((x, y))
            if 95 <= r <= 150 and 95 <= g <= 150 and 95 <= b <= 160 and max(r, g, b) - min(r, g, b) < 20:
                n += 1
    return n


def max_stroke_heights_in_band(
    im: Image.Image, box: tuple[int, int, int, int], y_start: int, y_end: int
) -> list[int]:
    """Return ink stroke heights for each contiguous ink run in [y_start, y_end)."""
    x1, _, x2, _ = box
    heights: list[int] = []
    y = y_start
    while y < y_end:
        row_ink = any(is_ink(im.getpixel((x, y))) for x in range(x1 + 40, x2 - 20))
        if not row_ink:
            y += 1
            continue
        y0 = y
        while y < y_end and any(is_ink(im.getpixel((x, y))) for x in range(x1 + 40, x2 - 20)):
            y += 1
        heights.append(band_ink_height(im, box, y0, y))
    return heights


def gate_outside(im: Image.Image, box: tuple[int, int, int, int]) -> tuple[bool, str]:
    x1, y1, x2, y3 = box
    mid = y1 + int((y3 - y1) * 0.38)
    top_heights = max_stroke_heights_in_band(im, box, y1, mid)
    body_heights = max_stroke_heights_in_band(im, box, mid, y3 - 24)
    if not top_heights or not body_heights:
        return False, f"missing ink strokes top={top_heights} body={body_heights}"
    h1 = max(top_heights)
    body = max(body_heights)
    if h1 < body + 2:
        return False, f"h1 ink height {h1} not taller than body {body}"
    bands = ink_row_bands(im, box)
    fence_y = y1 + int((y3 - y1) * 0.72)
    for idx, (y0, y1_band, _) in enumerate(bands[1:6], start=1):
        if y0 >= fence_y:
            continue
        if count_hash_marker_rows(im, box, y0, y1_band) >= 3:
            return False, f"band {idx} still shows hash marker ink"
        if count_asterisk_pairs(im, box, y0, y1_band) >= 1:
            return False, f"band {idx} still shows ** marker pairs"
    sq = squiggle_pixels(im, box)
    if sq > 12:
        return False, f"too many proofing squiggle pixels {sq}"
    return True, f"h1Height={h1} bodyHeight={body} squiggles={sq}"


def gate_heading_marker(im: Image.Image, box: tuple[int, int, int, int]) -> tuple[bool, str]:
    bands = ink_row_bands(im, box)
    if len(bands) < 2:
        return False, "not enough bands"
    # Active ## line is typically the band with marker tones at the left.
    best = 0
    best_band = -1
    for i, (y0, y1, _) in enumerate(bands):
        tone = marker_tone_pixels(im, box, y0, y1)
        if tone > best:
            best = tone
            best_band = i
    if best < 4:
        return False, f"dimmed heading marker pixels {best}"
    sq = squiggle_pixels(im, box)
    if sq > 12:
        return False, f"squiggles {sq}"
    return True, f"markerTone={best} band={best_band} squiggles={sq}"


def gate_bold_marker(im: Image.Image, box: tuple[int, int, int, int]) -> tuple[bool, str]:
    bands = ink_row_bands(im, box)
    if not bands:
        return False, "no bands"
    best = 0
    for y0, y1, _ in bands:
        best = max(best, marker_tone_pixels(im, box, y0, y1))
    if best < 4:
        return False, f"dimmed bold marker pixels {best}"
    sq = squiggle_pixels(im, box)
    if sq > 12:
        return False, f"squiggles {sq}"
    return True, f"markerTone={best} squiggles={sq}"


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("docs/screenshots/live")
    paths = {name: root / name for name in NAMES}
    for path in paths.values():
        if not path.is_file():
            print(f"missing {path}", file=sys.stderr)
            return 1

    digests = {name: file_md5(path) for name, path in paths.items()}
    if len(set(digests.values())) != len(digests):
        print("duplicate PNG md5 among live captures", file=sys.stderr)
        return 1

    results: list[str] = []
    outside_im = Image.open(paths["live-all-rendered-desktop.png"]).convert("RGB")
    obox = find_editor_bbox(outside_im)
    ok, msg = gate_outside(outside_im, obox)
    if not ok:
        print(f"outside: {msg}", file=sys.stderr)
        return 1
    results.append(f"outside {msg} bbox={obox}")

    heading_im = Image.open(paths["live-heading-marker-desktop.png"]).convert("RGB")
    hbox = find_editor_bbox(heading_im)
    ok, msg = gate_heading_marker(heading_im, hbox)
    if not ok:
        print(f"heading: {msg}", file=sys.stderr)
        return 1
    results.append(f"heading {msg}")

    bold_im = Image.open(paths["live-bold-marker-desktop.png"]).convert("RGB")
    bbox = find_editor_bbox(bold_im)
    ok, msg = gate_bold_marker(bold_im, bbox)
    if not ok:
        print(f"bold: {msg}", file=sys.stderr)
        return 1
    results.append(f"bold {msg}")

    for line in results:
        print(line)
    print("live capture pixel gate: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
