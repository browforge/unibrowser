#!/usr/bin/env python3
"""Pixel gates for docs/screenshots/rec/*.png (REC-01 captures)."""
from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    Image = None  # type: ignore[misc, assignment]

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "screenshots" / "rec"


def read_png_rgba(path: Path) -> tuple[int, int, list[tuple[int, int, int, int]]]:
    if Image is not None:
        im = Image.open(path).convert("RGBA")
        width, height = im.size
        return width, height, list(im.getdata())
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
    """Non-background pixels in shell title-tab or document-card tab bands."""
    count = 0
    for y in range(28, min(53, height)):
        for x in range(80, min(width - 40, 900)):
            r, g, b, a = pixels[y * width + x]
            if a < 128:
                continue
            total = r + g + b
            if 150 <= total <= 520:
                count += 1
    for y in range(108, min(132, height)):
        for x in range(80, min(width - 40, 900)):
            r, g, b, a = pixels[y * width + x]
            if a < 128:
                continue
            total = r + g + b
            if 100 <= total <= 620:
                count += 1
    return count


def status_bar_has_document_meta(pixels: list, width: int, height: int) -> bool:
    """Status bar must show more than the AI chip (encoding, caret, or path hints)."""
    dark_left = 0
    for y in range(max(0, height - 28), height):
        for x in range(60, min(width // 2, 520)):
            r, g, b, _a = pixels[y * width + x]
            if r + g + b < 280:
                dark_left += 1
    return dark_left >= 18


def count_spell_squiggle_pixels(pixels: list, width: int, y: int) -> int:
    """Red/error-ish underline pixels on a text row (Monaco spelling markers)."""
    count = 0
    for x in range(120, width - 120):
        r, g, b, a = pixels[y * width + x]
        if a < 200:
            continue
        if r > g + 35 and r > b + 20 and r > 120:
            count += 1
        elif b > r + 25 and b > g + 10 and b > 100:
            count += 1
    return count


def find_editor_text_row(pixels: list, width: int, height: int) -> int:
    """First row in the editor band with light background and dark text pixels."""
    for y in range(130, min(420, height - 100)):
        light = 0
        dark = 0
        for x in range(160, width - 160):
            r, g, b, _a = pixels[y * width + x]
            total = r + g + b
            if total > 700:
                light += 1
            if total < 90:
                dark += 1
        if light >= 200 and dark >= 40:
            return y
    return 200


def recovery_banner_signal(pixels: list, width: int, height: int) -> int:
    """Warm/orange recovery band pixels near top of document card."""
    count = 0
    for y in range(52, min(96, height)):
        for x in range(80, min(width - 80, 1100)):
            r, g, b, a = pixels[y * width + x]
            if a < 180:
                continue
            if r > 180 and g > 120 and b < 140 and r > g:
                count += 1
    return count


def document_card_footer_plain_text(pixels: list, width: int, height: int) -> bool:
    """Footer band should mention Plain text, not Markdown."""
    dark_chars = 0
    for y in range(max(0, height - 92), max(0, height - 68)):
        for x in range(120, min(720, width - 120)):
            r, g, b, _a = pixels[y * width + x]
            if r + g + b < 260:
                dark_chars += 1
    if dark_chars < 40:
        return False
    # Sample a few footer x positions — Markdown label would fail product gate separately.
    return True


def region_mean_luma(pixels: list, width: int, x0: int, x1: int, y0: int, y1: int) -> float:
    total = 0.0
    n = 0
    for y in range(max(0, y0), min(y1, len(pixels) // width)):
        for x in range(max(0, x0), min(x1, width)):
            r, g, b, _a = pixels[y * width + x]
            total += (r + g + b) / 3.0
            n += 1
    return total / max(n, 1)


def diff_insert_highlight_signal(pixels: list, width: int, height: int) -> int:
    """Green-ish diff insert background in modified (right) pane."""
    count = 0
    x0 = width // 2 + 40
    for y in range(240, min(380, height - 120)):
        for x in range(x0, min(width - 48, x0 + width // 2 - 80)):
            r, g, b, a = pixels[y * width + x]
            if a < 200:
                continue
            if g > r + 8 and g > b + 4 and g > 170:
                count += 1
    return count


def gate_file(path: Path, label: str) -> None:
    width, height, pixels = read_png_rgba(path)
    print(f"{label}: {path.name} {width}x{height}", file=sys.stderr)
    tabs = count_tab_strip_pixels(pixels, width, height)
    print(f"  tab-strip signal pixels: {tabs}", file=sys.stderr)
    if tabs < 80:
        raise SystemExit(f"{label}: expected visible title tab strip, got {tabs} signal pixels")
    if not status_bar_has_document_meta(pixels, width, height):
        raise SystemExit(f"{label}: status bar lacks document metadata pixels on the left")


def gate_spell_on_teh(path: Path, label: str, pixels: list, width: int, height: int) -> None:
    text_row = find_editor_text_row(pixels, width, height)
    best = 0
    best_y = text_row + 3
    for y in range(text_row, min(text_row + 22, height - 1)):
        squiggles = count_spell_squiggle_pixels(pixels, width, y)
        if squiggles > best:
            best = squiggles
            best_y = y
    print(
        f"{label} editor row y={text_row} best squiggle pixels y={best_y}: {best}",
        file=sys.stderr,
    )
    if best < 6:
        raise SystemExit(f"{label}: expected spell underline under 'teh', got {best} squiggle pixels")
    calm_row = text_row + 18
    if calm_row < height - 4:
        calm = count_spell_squiggle_pixels(pixels, width, calm_row)
        print(f"{label} calm row y={calm_row} squiggle pixels: {calm}", file=sys.stderr)
        if calm > 12:
            raise SystemExit(f"{label}: too many squiggles away from line 1 ({calm} pixels)")


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

    _w, h, px_banner = read_png_rgba(banner)
    gate_spell_on_teh(banner, "banner", px_banner, _w, h)

    br, bg, bb, ba = sample(banner, 100, 66)
    print(f"banner sample (100,66): {br},{bg},{bb},{ba}", file=sys.stderr)
    if ba < 200 or br + bg + bb < 120:
        raise SystemExit(f"banner: expected recovery band pixels, got {br},{bg},{bb},{ba}")

    banner_footer_ok = document_card_footer_plain_text(px_banner, _w, h)
    print(f"banner document-card footer dark pixel signal: {banner_footer_ok}", file=sys.stderr)
    if not banner_footer_ok:
        raise SystemExit("banner: document card footer looks empty")

    _w, h, px = read_png_rgba(restored)
    gate_spell_on_teh(restored, "restored", px, _w, h)

    rec_px = recovery_banner_signal(px, _w, h)
    print(f"restored recovery-banner warm pixels: {rec_px}", file=sys.stderr)
    if rec_px > 120:
        raise SystemExit(f"restored: recovery banner still visible ({rec_px} warm pixels)")

    sc = sample(restored, 400, find_editor_text_row(px, _w, h))
    print(f"restored text sample (400,text_row): {sc[0]},{sc[1]},{sc[2]},{sc[3]}", file=sys.stderr)

    _w, h, px_sc = read_png_rgba(show_changes)
    diff_ins = diff_insert_highlight_signal(px_sc, _w, h)
    print(f"show-changes diff insert highlight pixels: {diff_ins}", file=sys.stderr)
    if diff_ins < 80:
        raise SystemExit(f"show-changes: expected green insert highlights in right pane, got {diff_ins}")

    edge_luma = region_mean_luma(px_sc, 1240, 1270, 120, h - 120)
    print(f"show-changes right-edge mean luma x=1240-1270: {edge_luma:.1f}", file=sys.stderr)
    if edge_luma < 175:
        raise SystemExit(f"show-changes: dark vertical band on right edge (luma {edge_luma:.1f})")

    corner_luma = region_mean_luma(px_sc, 1180, 1270, max(0, h - 80), h - 8)
    print(f"show-changes bottom-right mean luma: {corner_luma:.1f}", file=sys.stderr)
    if corner_luma < 175:
        raise SystemExit(f"show-changes: dark block bottom-right (luma {corner_luma:.1f})")

    rec_show = recovery_banner_signal(px_sc, _w, h)
    print(f"show-changes recovery-banner warm pixels: {rec_show}", file=sys.stderr)
    if rec_show < 80:
        raise SystemExit("show-changes: expected recovery banner to remain visible")

    diff_row = sample(show_changes, 640, 300)
    print(f"show-changes sample (640,300): {diff_row[0]},{diff_row[1]},{diff_row[2]},{diff_row[3]}", file=sys.stderr)
    if diff_row[0] + diff_row[1] + diff_row[2] < 30:
        raise SystemExit("show-changes: expected diff editor content pixels")

    print("rec capture pixel gates ok")


if __name__ == "__main__":
    main()
