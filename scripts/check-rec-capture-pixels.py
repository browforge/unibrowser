#!/usr/bin/env python3
"""Pixel gates for docs/screenshots/rec/*.png (REC-01 captures)."""
from __future__ import annotations

import json
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


def region_stats(
    pixels: list,
    width: int,
    height: int,
    region: dict,
) -> dict[str, float | int]:
    x0 = int(region["x"])
    y0 = int(region["y"])
    w = int(region["width"])
    h = int(region["height"])
    x1 = min(width, x0 + w)
    y1 = min(height, y0 + h)
    squiggle = 0
    greenish = 0
    luma_total = 0.0
    n = 0
    for y in range(max(0, y0), max(0, y1)):
        for x in range(max(0, x0), max(0, x1)):
            r, g, b, a = pixels[y * width + x]
            if a < 200:
                continue
            n += 1
            luma_total += (r + g + b) / 3.0
            if r > g + 35 and r > b + 20 and r > 120:
                squiggle += 1
            elif b > r + 25 and b > g + 10 and b > 100:
                squiggle += 1
            if g > r + 6 and g > b + 4 and g > 150:
                greenish += 1
            elif g > 130 and g >= r and b < 160:
                greenish += 1
    return {
        "pixels": n,
        "mean_luma": luma_total / max(n, 1),
        "squiggle_pixels": squiggle,
        "greenish_pixels": greenish,
    }


def recovery_band_active(pixels: list, width: int, height: int, x: int = 100, y: int = 66) -> bool:
    r, g, b, a = sample_pixel(pixels, width, height, x, y)
    total = r + g + b
    return a >= 200 and 250 <= total <= 520


def recovery_band_absent(pixels: list, width: int, height: int) -> bool:
    return not recovery_band_active(pixels, width, height)


def sample_pixel(
    pixels: list, width: int, height: int, x: int, y: int,
) -> tuple[int, int, int, int]:
    return pixels[y * width + x]


def _assert_plain_text_footer_cm(data: dict, label: str) -> None:
    banner_footer = (data.get("documentCardFooter") or "").strip()
    print(f"{label} documentCardFooter: {banner_footer!r}", file=sys.stderr)
    if "Plain text" not in banner_footer:
        raise SystemExit(f"{label}: document card footer must include Plain text, got {banner_footer!r}")
    if "Markdown" in banner_footer:
        raise SystemExit(f"{label}: document card footer must not say Markdown for spell-test.txt")


def gate_document_card_footer(path: Path, label: str) -> None:
    cm_path = path.with_suffix(".cm.json")
    if not cm_path.is_file():
        raise SystemExit(f"{label}: missing CM sidecar for document card footer gate")
    data = json.loads(cm_path.read_text(encoding="utf-8"))
    _assert_plain_text_footer_cm(data, label)


def region_mean_luma(pixels: list, width: int, x0: int, x1: int, y0: int, y1: int) -> float:
    total = 0.0
    n = 0
    for y in range(max(0, y0), min(y1, len(pixels) // width)):
        for x in range(max(0, x0), min(x1, width)):
            r, g, b, _a = pixels[y * width + x]
            total += (r + g + b) / 3.0
            n += 1
    return total / max(n, 1)


def gate_spell_teh_region(
    path: Path,
    label: str,
    pixels: list,
    width: int,
    height: int,
    data: dict,
) -> None:
    region = data.get("spellTehRegion")
    if not isinstance(region, dict):
        raise SystemExit(f"{label}: missing spellTehRegion in capture sidecar (layout probe only)")
    stats = region_stats(pixels, width, height, region)
    print(
        f"{label} spellTehRegion={region} stats={stats}",
        file=sys.stderr,
    )
    if int(stats["pixels"]) < 8:
        raise SystemExit(f"{label}: spellTehRegion too small to sample ({stats['pixels']} pixels)")
    if int(stats["squiggle_pixels"]) < 3:
        raise SystemExit(
            f"{label}: expected visible squiggle pixels under 'teh', got {stats['squiggle_pixels']} "
            f"in region mean_luma={stats['mean_luma']:.1f}",
        )
    calm_y = int(region["y"]) + int(region["height"]) + 14
    if calm_y < height - 2:
        calm = count_spell_squiggle_pixels(pixels, width, calm_y)
        print(f"{label} calm row y={calm_y} squiggle pixels: {calm}", file=sys.stderr)
        if calm > 12:
            raise SystemExit(f"{label}: too many squiggles away from 'teh' ({calm} pixels)")


def gate_diff_line4_region(
    path: Path,
    pixels: list,
    width: int,
    height: int,
    data: dict,
) -> None:
    region = data.get("diffLine4Region")
    if not isinstance(region, dict):
        raise SystemExit("show-changes: missing diffLine4Region in capture sidecar (layout probe only)")
    stats = region_stats(pixels, width, height, region)
    print(f"show-changes diffLine4Region={region} stats={stats}", file=sys.stderr)
    if int(stats["pixels"]) < 20:
        raise SystemExit(f"show-changes: diffLine4Region too small ({stats['pixels']} pixels)")
    if int(stats["greenish_pixels"]) < 8:
        raise SystemExit(
            "show-changes: expected green insert tint on line 4 row, "
            f"got greenish_pixels={stats['greenish_pixels']} mean_luma={stats['mean_luma']:.1f}",
        )


def gate_file(path: Path, label: str) -> None:
    width, height, pixels = read_png_rgba(path)
    print(f"{label}: {path.name} {width}x{height}", file=sys.stderr)
    tabs = count_tab_strip_pixels(pixels, width, height)
    print(f"  tab-strip signal pixels: {tabs}", file=sys.stderr)
    if tabs < 80:
        raise SystemExit(f"{label}: expected visible title tab strip, got {tabs} signal pixels")
    if not status_bar_has_document_meta(pixels, width, height):
        raise SystemExit(f"{label}: status bar lacks document metadata pixels on the left")


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

    banner_cm = json.loads(banner.with_suffix(".cm.json").read_text(encoding="utf-8"))
    restored_cm = json.loads(restored.with_suffix(".cm.json").read_text(encoding="utf-8"))
    show_cm = json.loads(show_changes.with_suffix(".cm.json").read_text(encoding="utf-8"))

    _w, h, px_banner = read_png_rgba(banner)
    gate_spell_teh_region(banner, "banner", px_banner, _w, h, banner_cm)
    gate_document_card_footer(banner, "banner")

    br, bg, bb, ba = sample(banner, 100, 66)
    print(f"banner sample (100,66): {br},{bg},{bb},{ba}", file=sys.stderr)
    if ba < 200 or not recovery_band_active(px_banner, _w, h):
        raise SystemExit(f"banner: expected recovery band at (100,66), got {br},{bg},{bb},{ba}")

    _w, h, px = read_png_rgba(restored)
    gate_spell_teh_region(restored, "restored", px, _w, h, restored_cm)
    gate_document_card_footer(restored, "restored")

    if not recovery_band_absent(px, _w, h):
        raise SystemExit("restored: recovery banner still visible near y=66")

    _w, h, px_sc = read_png_rgba(show_changes)
    gate_diff_line4_region(show_changes, px_sc, _w, h, show_cm)

    edge_luma = region_mean_luma(px_sc, _w, 1240, 1270, 120, h - 120)
    print(f"show-changes right-edge mean luma x=1240-1270: {edge_luma:.1f}", file=sys.stderr)
    if edge_luma < 175:
        raise SystemExit(f"show-changes: dark vertical band on right edge (luma {edge_luma:.1f})")

    corner_luma = region_mean_luma(px_sc, _w, 1180, 1270, max(0, h - 80), h - 8)
    print(f"show-changes bottom-right mean luma: {corner_luma:.1f}", file=sys.stderr)
    if corner_luma < 175:
        raise SystemExit(f"show-changes: dark block bottom-right (luma {corner_luma:.1f})")

    if not recovery_band_active(px_sc, _w, h):
        br, bg, bb, ba = sample(show_changes, 100, 66)
        raise SystemExit(
            f"show-changes: expected recovery banner at (100,66), got {br},{bg},{bb},{ba}",
        )
    print(f"show-changes recovery band sample (100,66): {sample(show_changes, 100, 66)}", file=sys.stderr)

    print("rec capture pixel gates ok")


if __name__ == "__main__":
    main()
