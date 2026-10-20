#!/usr/bin/env python3
"""Gate history panel/jump PNG pair: pixel checks on bottom dock + document diff."""
from __future__ import annotations

import hashlib
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "screenshots" / "history"
PANEL = OUT / "history-panel-v4-1280.png"
JUMP = OUT / "history-jump-v4-1280.png"


def md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def document_crop_box(width: int, height: int) -> tuple[int, int, int, int]:
    """Paper card body (excludes chrome and bottom dock)."""
    return (80, 100, width - 80, int(height * 0.55))


def bottom_dock_crop_box(width: int, height: int) -> tuple[int, int, int, int]:
    top = int(height * 0.72)
    return (0, top, width, height)


def is_panel_surface_pixel(rgb: tuple[int, int, int]) -> bool:
    r, g, b = rgb
    if r < 210 or r > 248:
        return False
    if abs(r - g) > 8:
        return False
    return 220 <= b <= 245


def diff_fraction(a_path: Path, b_path: Path, box: tuple[int, int, int, int]) -> float:
    from PIL import Image

    a = Image.open(a_path).convert("RGB").crop(box)
    b = Image.open(b_path).convert("RGB").crop(box)
    if a.size != b.size:
        return 1.0
    pixels_a = a.getdata()
    pixels_b = b.getdata()
    changed = sum(1 for p, q in zip(pixels_a, pixels_b) if p != q)
    return changed / max(len(pixels_a), 1)


def bottom_panel_gate(path: Path) -> None:
    from PIL import Image

    im = Image.open(path).convert("RGB")
    w, h = im.size
    if w != 1280 or h != 800:
        raise SystemExit(f"{path.name}: expected 1280x800, got {w}x{h}")
    crop = im.crop(bottom_dock_crop_box(w, h))
    pixels = list(crop.getdata())
    panel_frac = sum(1 for p in pixels if is_panel_surface_pixel(p)) / max(len(pixels), 1)
    if panel_frac < 0.12:
        raise SystemExit(
            f"{path.name}: bottom dock panel surface {panel_frac:.1%} < 12% (history dock not visible)",
        )


def ocr_contains_restored(path: Path) -> None:
    try:
        out = subprocess.check_output(
            ["tesseract", str(path), "stdout", "-l", "eng"],
            stderr=subprocess.DEVNULL,
            text=True,
        )
    except FileNotFoundError:
        print("tesseract missing; skip OCR gate for Restored to", file=sys.stderr)
        return
    except subprocess.CalledProcessError as exc:
        raise SystemExit(f"{path.name}: tesseract failed ({exc})") from exc
    if "Restored to" not in out:
        raise SystemExit(f"{path.name}: OCR did not find 'Restored to' in jump shot")


def main() -> None:
    for path in (PANEL, JUMP):
        if not path.is_file():
            raise SystemExit(f"missing {path}")
    if md5(PANEL) == md5(JUMP):
        raise SystemExit("panel and jump PNGs are identical")
    frac = diff_fraction(PANEL, JUMP, document_crop_box(1280, 800))
    if frac <= 0.01:
        raise SystemExit(f"document region diff {frac:.4f} <= 1%")
    bottom_panel_gate(PANEL)
    bottom_panel_gate(JUMP)
    ocr_contains_restored(JUMP)
    print(f"history capture gate ok (document diff {frac:.2%})")


if __name__ == "__main__":
    main()
