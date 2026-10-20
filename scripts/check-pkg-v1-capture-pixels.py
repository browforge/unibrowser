#!/usr/bin/env python3
"""PIXEL gates for docs/screenshots/pkg-v1 (dialog regions must have non-background pixels)."""
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/screenshots/pkg-v1"

EXPECTED = (
    "open-mode-dialog-v1-1280.png",
    "history-location-notice-v1-1280.png",
    "unpack-progress-v1-1280.png",
    "file-menu-pack-unpack-v1-1280.png",
)

# centre crop where modal panels render (1280x800)
PANEL_BOX = (340, 120, 940, 680)


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


def panel_has_content(path: Path) -> str | None:
    sd = crop_stddev(path, PANEL_BOX)
    if sd < 0.02:
        return f"{path.name}: panel crop flat (stddev {sd:.4f})"
    return None


def menu_has_items(path: Path) -> str | None:
    sd = crop_stddev(path, (0, 24, 420, 420))
    if sd < 0.015:
        return f"{path.name}: file menu region flat (stddev {sd:.4f})"
    return None


def main() -> int:
    paths = [OUT / name for name in EXPECTED]
    for p in paths:
        if not p.is_file():
            print(f"missing {p}", file=sys.stderr)
            return 1
    digests = {file_md5(p) for p in paths}
    if len(digests) != len(paths):
        print("duplicate PNG md5 among pkg-v1 captures", file=sys.stderr)
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
        if not cm.is_file():
            print(f"missing {cm}", file=sys.stderr)
            return 1
        json.loads(cm.read_text())
        err = panel_has_content(p)
        if err and "file-menu" not in p.name:
            print(err, file=sys.stderr)
            return 1
        if "file-menu" in p.name:
            err = menu_has_items(p)
            if err:
                print(err, file=sys.stderr)
                return 1
    print("pkg-v1 capture pixel gates ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
