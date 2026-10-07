#!/usr/bin/env python3
"""Validate rendered Command Manager label contrast from a capture PNG."""

from __future__ import annotations

import argparse
import json
import math
import re
import sys
from pathlib import Path

from PIL import Image


RGB_RE = re.compile(r"rgba?\((\d+),\s*(\d+),\s*(\d+)")


def luminance(rgb: tuple[int, int, int]) -> float:
    def channel(value: int) -> float:
        normal = value / 255.0
        if normal <= 0.04045:
            return normal / 12.92
        return ((normal + 0.055) / 1.055) ** 2.4

    red, green, blue = rgb
    return (
        0.2126 * channel(red)
        + 0.7152 * channel(green)
        + 0.0722 * channel(blue)
    )


def contrast(first: tuple[int, int, int], second: tuple[int, int, int]) -> float:
    high = max(luminance(first), luminance(second))
    low = min(luminance(first), luminance(second))
    return (high + 0.05) / (low + 0.05)


def parse_rgb(value: str) -> tuple[int, int, int] | None:
    match = RGB_RE.search(value)
    if not match:
        return None
    return tuple(int(match.group(index)) for index in range(1, 4))


def percentile(values: list[float], fraction: float) -> float:
    if not values:
        return 1.0
    ordered = sorted(values)
    index = min(len(ordered) - 1, math.floor((len(ordered) - 1) * fraction))
    return ordered[index]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("png", type=Path)
    parser.add_argument("json", type=Path)
    args = parser.parse_args()

    image = Image.open(args.png).convert("RGB")
    data = json.loads(args.json.read_text(encoding="utf-8"))
    viewport_width = max(1, int(data.get("viewportWidth", image.width)))
    viewport_height = max(1, int(data.get("viewportHeight", image.height)))
    scale_x = image.width / viewport_width
    scale_y = image.height / viewport_height
    failures: list[str] = []

    for control in data.get("controls", []):
        bounds = control.get("labelBounds", {})
        x0 = max(0, math.floor(float(bounds.get("x", 0)) * scale_x))
        y0 = max(0, math.floor(float(bounds.get("y", 0)) * scale_y))
        x1 = min(
            image.width,
            math.ceil((float(bounds.get("x", 0)) + float(bounds.get("width", 0))) * scale_x),
        )
        y1 = min(
            image.height,
            math.ceil((float(bounds.get("y", 0)) + float(bounds.get("height", 0))) * scale_y),
        )
        background = parse_rgb(control.get("background", "")) or (23, 37, 50)
        ratios: list[float] = []
        label_pixels = image.crop((x0, y0, x1, y1)).get_flattened_data()
        for red, green, blue in label_pixels:
            pixel = (red, green, blue)
            delta = sum(abs(pixel[index] - background[index]) for index in range(3))
            if delta >= 18:
                ratios.append(contrast(pixel, background))

        # The 90th percentile samples glyph interiors and ignores anti-aliased edges.
        sampled = percentile(ratios, 0.90)
        control["pixelContrast"] = round(sampled, 3)
        control["pixelInkCount"] = len(ratios)
        control["pixelBounds"] = {"x": x0, "y": y0, "width": x1 - x0, "height": y1 - y0}
        supported_enabled = (
            control.get("capabilityState") in ("supported", "emulated")
            and bool(control.get("enabled"))
        )
        if supported_enabled and sampled < 4.5:
            failures.append(
                f"{control.get('controlId', '?')}: label contrast {sampled:.2f}:1 < 4.5:1"
            )

    args.json.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    if failures:
        print(f"CM pixel contrast failed for {args.png}:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1
    print(f"CM pixel contrast OK: {args.png}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
