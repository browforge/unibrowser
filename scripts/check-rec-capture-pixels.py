#!/usr/bin/env python3
"""Pixel gates for docs/screenshots/rec/*.png (REC-01 captures)."""
from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "screenshots" / "rec"


def read_png_pixels(path: Path) -> list[tuple[int, int, int, int]]:
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
    return pixels


def sample(path: Path, x: int, y: int) -> tuple[int, int, int, int]:
    pixels = read_png_pixels(path)
    width = 1280
    return pixels[y * width + x]


def main() -> None:
    banner = OUT / "rec-recovery-banner.png"
    restored = OUT / "rec-restored-document.png"
    for p in (banner, restored):
        if not p.is_file():
            raise SystemExit(f"missing capture {p}")
    # Recovery bar row background (light theme band under chrome)
    r, g, b, a = sample(banner, 100, 66)
    if a < 200 or r + g + b < 120 or r + g + b > 720:
        raise SystemExit(f"banner: expected recovery band pixels, got {r},{g},{b},{a}")
    # Restored doc: Monaco surface (not empty chrome)
    r2, g2, b2, a2 = sample(restored, 400, 50)
    if a2 < 200 or r2 + g2 + b2 < 80:
        raise SystemExit(f"restored: expected editor content pixels, got {r2},{g2},{b2},{a2}")
    print("rec capture pixel gates ok")


if __name__ == "__main__":
    main()
