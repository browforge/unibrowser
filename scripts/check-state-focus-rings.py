#!/usr/bin/env python3
"""Check every state-sheet focus cell for a visible approved focus ring."""

from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image


def luminance(rgb: tuple[int, int, int]) -> float:
    def channel(value: int) -> float:
        normal = value / 255.0
        if normal <= 0.04045:
            return normal / 12.92
        return ((normal + 0.055) / 1.055) ** 2.4

    red, green, blue = rgb
    return 0.2126 * channel(red) + 0.7152 * channel(green) + 0.0722 * channel(blue)


def contrast(first: tuple[int, int, int], second: tuple[int, int, int]) -> float:
    high = max(luminance(first), luminance(second))
    low = min(luminance(first), luminance(second))
    return (high + 0.05) / (low + 0.05)


def close_to(pixel: tuple[int, ...], target: tuple[int, int, int]) -> bool:
    return all(abs(pixel[index] - target[index]) <= 6 for index in range(3))


def main() -> int:
    if len(sys.argv) < 5:
        print(
            "usage: check-state-focus-rings.py <theme> <contrast> <focus-cell>...",
            file=sys.stderr,
        )
        return 2

    theme = sys.argv[1]
    contrast_mode = sys.argv[2]
    paths = [Path(value) for value in sys.argv[3:]]
    if len(paths) != 10:
        print(f"expected 10 sampled focus controls, found {len(paths)}", file=sys.stderr)
        return 1

    if contrast_mode == "high":
        approved = [((255, 229, 102), (0, 0, 0))]
    elif theme == "light":
        approved = [
            ((23, 107, 148), (255, 255, 255)),
            ((131, 199, 230), (23, 37, 50)),
        ]
    else:
        approved = [((131, 199, 230), (35, 37, 38))]

    for ring, surrounding in approved:
        ratio = contrast(ring, surrounding)
        if ratio < 3:
            print(f"approved focus token has only {ratio:.2f}:1 contrast", file=sys.stderr)
            return 1

    failures: list[str] = []
    for path in paths:
        with Image.open(path).convert("RGB") as image:
            pixels = list(image.getdata())
        matching = max(
            sum(1 for pixel in pixels if close_to(pixel, ring))
            for ring, _ in approved
        )
        if matching < 8:
            failures.append(f"{path.name}: no measured 2px focus-ring colour")

    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"State focus-ring check passed for {len(paths)} sampled controls")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
