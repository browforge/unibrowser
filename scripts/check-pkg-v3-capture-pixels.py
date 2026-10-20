#!/usr/bin/env python3
"""PIXEL gates for docs/screenshots/pkg-v3 (desktop shell + contrast)."""
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/screenshots/pkg-v3"

EXPECTED = (
    "open-mode-dialog-v3-1280.png",
    "history-location-notice-v3-1280.png",
    "unpack-progress-v3-1280.png",
    "file-menu-pack-unpack-v3-1280.png",
)

PANEL_BOX = (340, 120, 940, 680)
STATUS_BAR_BOX = (8, 768, 900, 798)
CONTENT_CHROME_BOX = (0, 72, 1280, 88)
TOAST_BOX = (700, 680, 1260, 760)


def file_md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def crop_stddev(path: Path, box: tuple[int, int, int, int]) -> float:
    x0, y0, x1, y1 = box
    w, h = x1 - x0, y1 - y0
    out = subprocess.check_output(
        [
            "convert",
            str(path),
            "-crop",
            f"{w}x{h}+{x0}+{y0}",
            "+repage",
            "-colorspace",
            "Gray",
            "-format",
            "%[fx:standard_deviation]",
            "info:",
        ],
        text=True,
    ).strip()
    return float(out or "0")


def crop_mean(path: Path, box: tuple[int, int, int, int]) -> float:
    x0, y0, x1, y1 = box
    w, h = x1 - x0, y1 - y0
    out = subprocess.check_output(
        [
            "convert",
            str(path),
            "-crop",
            f"{w}x{h}+{x0}+{y0}",
            "+repage",
            "-colorspace",
            "Gray",
            "-format",
            "%[fx:mean]",
            "info:",
        ],
        text=True,
    ).strip()
    return float(out or "0") * 255.0


def panel_has_content(path: Path) -> str | None:
    sd = crop_stddev(path, PANEL_BOX)
    if sd < 0.02:
        return f"{path.name}: panel crop flat (stddev {sd:.4f})"
    return None


def menu_has_items(path: Path) -> str | None:
    sd = crop_stddev(path, (0, 24, 420, 520))
    if sd < 0.015:
        return f"{path.name}: file menu region flat (stddev {sd:.4f})"
    return None


def status_bar_has_text(path: Path) -> str | None:
    sd = crop_stddev(path, STATUS_BAR_BOX)
    if sd < 0.012:
        return f"{path.name}: status bar text region flat (stddev {sd:.4f})"
    return None


def no_content_chrome_bar(path: Path) -> str | None:
    mean = crop_mean(path, CONTENT_CHROME_BOX)
    if mean < 55:
        return f"{path.name}: dark content chrome bar (mean gray {mean:.1f})"
    return None


def toast_contrast_ok(path: Path) -> str | None:
    sd = crop_stddev(path, TOAST_BOX)
    if sd < 0.04:
        return f"{path.name}: toast region low contrast (stddev {sd:.4f})"
    return None


def main() -> int:
    paths = [OUT / name for name in EXPECTED]
    for p in paths:
        if not p.is_file():
            print(f"missing {p}", file=sys.stderr)
            return 1
    digests = {file_md5(p) for p in paths}
    if len(digests) != len(paths):
        print("duplicate PNG md5 among pkg-v3 captures", file=sys.stderr)
        return 1
    for p in paths:
        dim = subprocess.check_output(
            ["identify", "-format", "%wx%h", str(p)],
            text=True,
        ).strip()
        if dim != "1280x800":
            print(f"{p.name} dimensions {dim}", file=sys.stderr)
            return 1
        cm = p.with_suffix(".cm.json")
        if cm.is_file():
            json.loads(cm.read_text())
        err = status_bar_has_text(p)
        if err:
            print(err, file=sys.stderr)
            return 1
        err = no_content_chrome_bar(p)
        if err:
            print(err, file=sys.stderr)
            return 1
        if "history-location" in p.name:
            err = toast_contrast_ok(p)
            if err:
                print(err, file=sys.stderr)
                return 1
        if "file-menu" in p.name:
            err = menu_has_items(p)
            if err:
                print(err, file=sys.stderr)
                return 1
        elif "history-location" not in p.name:
            err = panel_has_content(p)
            if err:
                print(err, file=sys.stderr)
                return 1
    print("pkg-v3 capture pixel gates ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
