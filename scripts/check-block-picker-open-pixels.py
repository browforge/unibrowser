#!/usr/bin/env python3
"""Assert block-style picker listbox is visibly open in capture PNGs."""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

from PIL import Image


def fingerprint_region(
    image: Image.Image,
    bounds: dict[str, float],
    viewport_width: float,
    viewport_height: float,
) -> dict[str, float | int]:
    scale_x = image.width / max(1.0, viewport_width)
    scale_y = image.height / max(1.0, viewport_height)
    x = float(bounds.get("x", 0))
    y = float(bounds.get("y", 0))
    width = float(bounds.get("width", 0))
    height = float(bounds.get("height", 0))
    x0 = max(0, math.floor(x * scale_x))
    y0 = max(0, math.floor(y * scale_y))
    x1 = min(image.width, math.ceil((x + width) * scale_x))
    y1 = min(image.height, math.ceil((y + height) * scale_y))
    if x1 <= x0 or y1 <= y0:
        return {"meanR": 0, "meanG": 0, "meanB": 0, "inkPixels": 0, "sampledPixels": 0}

    sum_r = 0
    sum_g = 0
    sum_b = 0
    ink = 0
    sampled = 0
    crop = image.crop((x0, y0, x1, y1))
    pixel_data = (
        crop.get_flattened_data()
        if hasattr(crop, "get_flattened_data")
        else crop.getdata()
    )
    for red, green, blue in pixel_data:
        sum_r += red
        sum_g += green
        sum_b += blue
        sampled += 1
        dr = red - 255
        dg = green - 255
        db = blue - 255
        if dr * dr + dg * dg + db * db >= 28 * 28 * 3:
            ink += 1
    if sampled == 0:
        return {"meanR": 0, "meanG": 0, "meanB": 0, "inkPixels": 0, "sampledPixels": 0}
    return {
        "meanR": sum_r / sampled,
        "meanG": sum_g / sampled,
        "meanB": sum_b / sampled,
        "inkPixels": ink,
        "sampledPixels": sampled,
    }


def rects_overlap(a: dict[str, float], b: dict[str, float]) -> bool:
    a_right = a["x"] + a["width"]
    a_bottom = a["y"] + a["height"]
    b_right = b["x"] + b["width"]
    b_bottom = b["y"] + b["height"]
    return a["x"] < b_right and a_right > b["x"] and a["y"] < b_bottom and a_bottom > b["y"]


def fingerprint_distance(
    left: dict[str, float | int], right: dict[str, float | int]
) -> float:
    return math.sqrt(
        (float(left["meanR"]) - float(right["meanR"])) ** 2
        + (float(left["meanG"]) - float(right["meanG"])) ** 2
        + (float(left["meanB"]) - float(right["meanB"])) ** 2
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("png", type=Path)
    parser.add_argument("json", type=Path, nargs="?", default=None)
    parser.add_argument("--baseline-png", type=Path, default=None)
    args = parser.parse_args()

    image = Image.open(args.png).convert("RGB")
    block: dict[str, object] | None = None
    viewport_width = image.width
    viewport_height = image.height
    if args.json and args.json.is_file():
        data = json.loads(args.json.read_text(encoding="utf-8"))
        viewport_width = int(data.get("viewportWidth", image.width))
        viewport_height = int(data.get("viewportHeight", image.height))
        block = data.get("blockPicker")

    if block is None and args.baseline_png is None:
        print("block picker pixel check skipped (no blockPicker in JSON)", file=sys.stderr)
        return 0

    failures: list[str] = []
    probe = (block or {}).get("probe") if block else None
    if not isinstance(probe, dict):
        failures.append("blockPicker.probe missing from CM JSON")
        probe = {}

    open_fp = fingerprint_region(image, probe, viewport_width, viewport_height)

    closed_fp: dict[str, float | int] | None = None
    if block and isinstance(block.get("closedFingerprint"), dict):
        closed_fp = block["closedFingerprint"]
    elif args.baseline_png and args.baseline_png.is_file():
        baseline = Image.open(args.baseline_png).convert("RGB")
        closed_fp = fingerprint_region(
            baseline,
            {"x": 0, "y": 0, "width": baseline.width, "height": baseline.height},
            baseline.width,
            baseline.height,
        )

    if closed_fp is not None:
        ink_delta = int(open_fp["inkPixels"]) - int(closed_fp["inkPixels"])
        mean_delta = fingerprint_distance(open_fp, closed_fp)
        if ink_delta < 25 and mean_delta < 6.0:
            failures.append(
                "probe below block trigger unchanged vs closed state "
                f"(ink delta {ink_delta}, mean rgb delta {mean_delta:.2f})"
            )
    else:
        failures.append("blockPicker.closedFingerprint missing (cannot prove listbox open)")

    menu = block.get("menu") if block else None
    if not isinstance(menu, dict):
        failures.append("blockPicker.menu bounds missing while picker should be open")
    else:
        menu_fp = fingerprint_region(image, menu, viewport_width, viewport_height)
        if int(menu_fp["inkPixels"]) < 60:
            failures.append(
                f"block picker menu region lacks visible list ink (ink={menu_fp['inkPixels']})"
            )

    label = block.get("label") if block else None
    badge = block.get("badge") if block else None
    if isinstance(label, dict) and isinstance(badge, dict):
        if rects_overlap(label, badge):
            failures.append("docx emulation badge bounding box overlaps label in CM metrics")
        if float(badge["x"]) < float(label["x"]) + float(label["width"]) - 0.5:
            failures.append("docx emulation badge is not placed after label in CM metrics")

    if failures:
        print(f"Block picker pixel check failed for {args.png}:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1

    print(f"Block picker pixel check OK: {args.png}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
