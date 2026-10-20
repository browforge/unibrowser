#!/usr/bin/env python3
"""PIXEL gates for docs/screenshots/pkg-v2 (desktop captures; status bar must stay visible)."""
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/screenshots/pkg-v2"

EXPECTED = (
    "open-mode-dialog-v2-1280.png",
    "history-location-notice-v2-1280.png",
    "unpack-progress-v2-1280.png",
    "file-menu-pack-unpack-v2-1280.png",
)

PANEL_BOX = (340, 120, 940, 680)
STATUS_BAR_BOX = (0, 760, 1280, 800)
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
    sd = crop_stddev(path, (0, 24, 420, 520))
    if sd < 0.015:
        return f"{path.name}: file menu region flat (stddev {sd:.4f})"
    return None


def status_bar_visible(path: Path) -> str | None:
    sd = crop_stddev(path, STATUS_BAR_BOX)
    if sd < 0.008:
        return f"{path.name}: status bar crop flat (stddev {sd:.4f})"
    return None


def main() -> int:
    paths = [OUT / name for name in EXPECTED]
    for p in paths:
        if not p.is_file():
            print(f"missing {p}", file=sys.stderr)
            return 1
    digests = {file_md5(p) for p in paths}
    if len(digests) != len(paths):
        print("duplicate PNG md5 among pkg-v2 captures", file=sys.stderr)
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
        err = status_bar_visible(p)
        if err:
            print(err, file=sys.stderr)
            return 1
        if "history-location" in p.name:
            sd = crop_stddev(p, (24, 700, 1256, 792))
            if sd < 0.01:
                print(
                    f"{p.name}: history toast region flat (stddev {sd:.4f})",
                    file=sys.stderr,
                )
                return 1
        elif "file-menu" not in p.name:
            err = panel_has_content(p)
            if err:
                print(err, file=sys.stderr)
                return 1
        else:
            err = menu_has_items(p)
            if err:
                print(err, file=sys.stderr)
                return 1
    print("pkg-v2 capture pixel gates ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
