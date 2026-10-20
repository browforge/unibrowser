#!/usr/bin/env python3
"""PIXEL gates for docs/screenshots/pkg-v4 (desktop shell + contrast)."""
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/screenshots/pkg-v4"

EXPECTED = (
    "open-mode-dialog-v4-1280.png",
    "history-location-notice-v4-1280.png",
    "unpack-progress-v4-1280.png",
    "file-menu-pack-unpack-v4-1280.png",
)

PANEL_BOX = (340, 120, 940, 680)
STATUS_BAR_BOX = (8, 768, 900, 798)
TOAST_BOX = (700, 680, 1260, 760)
HAMBURGER_BOX = (0, 0, 52, 44)
# Below compact title bar + command manager (content host only).
CONTENT_TOP = 64
CONTENT_BOTTOM = 760
STRIP_HEIGHT = 6


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


def crop_compare_rmse(a: Path, b: Path, box: tuple[int, int, int, int]) -> float:
    x0, y0, x1, y1 = box
    w, h = x1 - x0, y1 - y0
    out = subprocess.check_output(
        [
            "convert",
            "(",
            str(a),
            "-crop",
            f"{w}x{h}+{x0}+{y0}",
            "+repage",
            ")",
            "(",
            str(b),
            "-crop",
            f"{w}x{h}+{x0}+{y0}",
            "+repage",
            ")",
            "-compose",
            "difference",
            "-composite",
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
    sd = crop_stddev(path, (0, 36, 420, 520))
    if sd < 0.015:
        return f"{path.name}: file menu region flat (stddev {sd:.4f})"
    return None


def status_bar_has_text(path: Path) -> str | None:
    sd = crop_stddev(path, STATUS_BAR_BOX)
    if sd < 0.012:
        return f"{path.name}: status bar text region flat (stddev {sd:.4f})"
    return None


def no_full_width_dark_band(path: Path) -> str | None:
    width = 1280
    for y0 in range(CONTENT_TOP, CONTENT_BOTTOM - STRIP_HEIGHT, STRIP_HEIGHT):
        mean = crop_mean(path, (0, y0, width, y0 + STRIP_HEIGHT))
        sd = crop_stddev(path, (0, y0, width, y0 + STRIP_HEIGHT))
        if mean < 52 and sd < 0.028:
            return (
                f"{path.name}: full-width dark band at y≈{y0}-{y0 + STRIP_HEIGHT} "
                f"(mean gray {mean:.1f}, stddev {sd:.4f})"
            )
    return None


def toast_contrast_ok(path: Path) -> str | None:
    sd = crop_stddev(path, TOAST_BOX)
    if sd < 0.04:
        return f"{path.name}: toast region low contrast (stddev {sd:.4f})"
    return None


def file_menu_below_title_bar(path: Path) -> str | None:
    # Menu panel should not paint over the title bar (y < 32).
    sd_title = crop_stddev(path, (180, 4, 420, 30))
    if sd_title < 0.02:
        return f"{path.name}: title bar region under menu is flat (menu may cover tabs)"
    mean_top = crop_mean(path, (8, 8, 120, 28))
    if mean_top < 40:
        return f"{path.name}: hamburger/title region too dark (mean {mean_top:.1f})"
    return None


def hamburger_matches_baseline(menu_path: Path, baseline_path: Path) -> str | None:
    rmse = crop_compare_rmse(menu_path, baseline_path, HAMBURGER_BOX)
    if rmse > 18:
        return f"{menu_path.name}: hamburger region differs from baseline (rmse {rmse:.1f})"
    return None


def main() -> int:
    paths = [OUT / name for name in EXPECTED]
    for p in paths:
        if not p.is_file():
            print(f"missing {p}", file=sys.stderr)
            return 1
    digests = {file_md5(p) for p in paths}
    if len(digests) != len(paths):
        print("duplicate PNG md5 among pkg-v4 captures", file=sys.stderr)
        return 1
    baseline = OUT / "history-location-notice-v4-1280.png"
    file_menu = OUT / "file-menu-pack-unpack-v4-1280.png"
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
        err = no_full_width_dark_band(p)
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
            err = file_menu_below_title_bar(p)
            if err:
                print(err, file=sys.stderr)
                return 1
            err = hamburger_matches_baseline(file_menu, baseline)
            if err:
                print(err, file=sys.stderr)
                return 1
        elif "history-location" not in p.name and "open-mode" not in p.name:
            err = panel_has_content(p)
            if err:
                print(err, file=sys.stderr)
                return 1
    print("pkg-v4 capture pixel gates ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
