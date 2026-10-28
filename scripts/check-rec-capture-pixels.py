#!/usr/bin/env python3
"""Pixel gates for docs/screenshots/rec/*.png (REC-01 captures)."""
from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "screenshots" / "rec"


def read_png_rgba(path: Path) -> tuple[int, int, list[tuple[int, int, int, int]]]:
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"{path}: not a PNG")
    pos = 8
    raw = b""
    width = height = 0
    color_type = 6
    while pos < len(data):
        length = struct.unpack(">I", data[pos : pos + 4])[0]
        pos += 4
        chunk = data[pos : pos + 4]
        pos += 4
        chunk_data = data[pos : pos + length]
        pos += length
        pos += 4
        if chunk == b"IHDR":
            width, height = struct.unpack(">II", chunk_data[:8])
            color_type = chunk_data[9]
        elif chunk == b"IDAT":
            raw += chunk_data
        elif chunk == b"IEND":
            break
    bpp = 4 if color_type in (6, 4) else 3
    inflated = zlib.decompress(raw)
    pixels: list[tuple[int, int, int, int]] = []
    off = 0
    for _y in range(height):
        off += 1
        row = inflated[off : off + width * bpp]
        off += width * bpp
        for x in range(width):
            i = x * bpp
            if bpp == 4:
                pixels.append((row[i], row[i + 1], row[i + 2], row[i + 3]))
            else:
                pixels.append((row[i], row[i + 1], row[i + 2], 255))
    return width, height, pixels


def sample(path: Path, x: int, y: int) -> tuple[int, int, int, int]:
    width, height, pixels = read_png_rgba(path)
    if x < 0 or y < 0 or x >= width or y >= height:
        raise SystemExit(f"{path}: sample ({x},{y}) out of bounds {width}x{height}")
    return pixels[y * width + x]


def row_mean_luma(pixels: list, width: int, y: int) -> float:
    total = 0.0
    for x in range(width):
        r, g, b, _a = pixels[y * width + x]
        total += (r + g + b) / 3.0
    return total / width


def count_tab_strip_pixels(pixels: list, width: int, height: int) -> int:
    """Non-background pixels in title-tab band (y 28..52)."""
    count = 0
    for y in range(28, min(53, height)):
        for x in range(80, min(width - 40, 900)):
            r, g, b, a = pixels[y * width + x]
            if a < 128:
                continue
            if r + g + b < 80 or r + g + b > 740:
                continue
            count += 1
    return count


def status_bar_dark_band(pixels: list, width: int, height: int) -> bool:
    """True if a full-width dark strip sits in the status-bar band."""
    for y in range(max(0, height - 55), height - 35):
        dark = 0
        for x in range(40, width - 40):
            r, g, b, _a = pixels[y * width + x]
            if r + g + b < 60:
                dark += 1
        if dark > (width - 80) * 0.55:
            return True
    return False


def status_bar_has_text(pixels: list, width: int, height: int) -> bool:
    y = height - 18
    if y < 0:
        return False
    dark = 0
    for x in range(60, width - 60):
        r, g, b, _a = pixels[y * width + x]
        if r + g + b < 120:
            dark += 1
    return dark >= 12


def count_spell_squiggle_pixels(pixels: list, width: int, y: int) -> int:
    """Blue-ish pixels on a text row (Monaco spelling markers)."""
    count = 0
    for x in range(120, width - 120):
        r, g, b, a = pixels[y * width + x]
        if a < 200:
            continue
        if b > r + 25 and b > g + 10 and b > 100:
            count += 1
    return count


def find_editor_text_row(pixels: list, width: int, height: int) -> int:
    """First row in the editor band with dark text pixels."""
    for y in range(130, min(420, height - 100)):
        dark = 0
        for x in range(160, width - 160):
            r, g, b, _a = pixels[y * width + x]
            if r + g + b < 90:
                dark += 1
        if dark >= 40:
            return y
    return 200


def gate_file(path: Path, label: str) -> None:
    width, height, pixels = read_png_rgba(path)
    print(f"{label}: {path.name} {width}x{height}", file=sys.stderr)
    tabs = count_tab_strip_pixels(pixels, width, height)
    print(f"  tab-strip signal pixels: {tabs}", file=sys.stderr)
    if tabs < 80:
        raise SystemExit(f"{label}: expected visible title tab strip, got {tabs} signal pixels")
    if status_bar_dark_band(pixels, width, height):
        raise SystemExit(f"{label}: dark band in status-bar region")
    if not status_bar_has_text(pixels, width, height):
        raise SystemExit(f"{label}: status bar row lacks text-like dark pixels")


def main() -> None:
    banner = OUT / "rec-recovery-banner.png"
    restored = OUT / "rec-restored-document.png"
    show_changes = OUT / "rec-recovery-show-changes.png"
    for p in (banner, restored, show_changes):
        if not p.is_file():
            raise SystemExit(f"missing capture {p}")

    gate_file(banner, "banner")
    gate_file(restored, "restored")
    gate_file(show_changes, "show-changes")

    br, bg, bb, ba = sample(banner, 100, 66)
    print(f"banner sample (100,66): {br},{bg},{bb},{ba}", file=sys.stderr)
    if ba < 200 or br + bg + bb < 120:
        raise SystemExit(f"banner: expected recovery band pixels, got {br},{bg},{bb},{ba}")

    _w, h, _ = read_png_rgba(banner)
    centre = sample(banner, 640, min(400, h - 1))
    print(f"banner centre (640,{min(400, h - 1)}): {centre[0]},{centre[1]},{centre[2]},{centre[3]}", file=sys.stderr)

    _w, h, px = read_png_rgba(restored)
    text_row = find_editor_text_row(px, _w, h)
    squiggles = count_spell_squiggle_pixels(px, _w, text_row + 3)
    print(f"restored editor row y={text_row} blue squiggle pixels: {squiggles}", file=sys.stderr)
    if squiggles > 80:
        raise SystemExit(f"restored: too many spell underline pixels ({squiggles}) on line 1")

    sc = sample(restored, 400, text_row)
    print(f"restored text sample (400,{text_row}): {sc[0]},{sc[1]},{sc[2]},{sc[3]}", file=sys.stderr)
    if sc[0] + sc[1] + sc[2] < 40:
        raise SystemExit("restored: editor text row looks blank")

    diff_row = sample(show_changes, 640, 300)
    print(f"show-changes sample (640,300): {diff_row[0]},{diff_row[1]},{diff_row[2]},{diff_row[3]}", file=sys.stderr)
    if diff_row[0] + diff_row[1] + diff_row[2] < 30:
        raise SystemExit("show-changes: expected diff editor content pixels")

    print("rec capture pixel gates ok")


if __name__ == "__main__":
    main()
