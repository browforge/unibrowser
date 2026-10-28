#!/usr/bin/env python3
"""Pixel gates for docs/screenshots/live/*.png using cm.json liveBlock metrics."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:  # pragma: no cover
    print("check-live-capture-pixels: Pillow required", file=sys.stderr)
    sys.exit(2)

NAMES = [
    "live-all-rendered-desktop.png",
    "live-heading-marker-desktop.png",
    "live-bold-marker-desktop.png",
]


def file_md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def pixel_at(im: Image.Image, x: int, y: int) -> tuple[int, int, int]:
    px = im.getpixel((x, y))
    if isinstance(px, tuple) and len(px) >= 3:
        return int(px[0]), int(px[1]), int(px[2])
    return int(px), int(px), int(px)


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("docs/screenshots/live")
    paths = {name: root / name for name in NAMES}
    for name, path in paths.items():
        if not path.is_file():
            print(f"missing {path}", file=sys.stderr)
            return 1
        cm_path = path.with_suffix(".cm.json")
        if not cm_path.is_file():
            print(f"missing {cm_path}", file=sys.stderr)
            return 1

    digests = {name: file_md5(path) for name, path in paths.items()}
    if len(set(digests.values())) != len(digests):
        print("duplicate PNG md5 among live captures", file=sys.stderr)
        return 1

    outside = json.loads(paths["live-all-rendered-desktop.png"].with_suffix(".cm.json").read_text())
    heading = json.loads(paths["live-heading-marker-desktop.png"].with_suffix(".cm.json").read_text())
    bold = json.loads(paths["live-bold-marker-desktop.png"].with_suffix(".cm.json").read_text())

    ob = outside.get("liveBlock") or {}
    hd = heading.get("liveBlock") or {}
    bd = bold.get("liveBlock") or {}

    if ob.get("activeBlockIndex") != -1:
        print("outside capture caret should sit in inter-block gap", file=sys.stderr)
        return 1
    if ob.get("markerElementCount", 0) != 0:
        print("outside capture must not expose md-live-marker spans", file=sys.stderr)
        return 1
    if hd.get("markerElementCount", 0) < 1:
        print("heading capture missing dimmed heading markers", file=sys.stderr)
        return 1
    if bd.get("markerElementCount", 0) < 2:
        print("bold capture missing dimmed ** markers", file=sys.stderr)
        return 1

    im = Image.open(paths["live-heading-marker-desktop.png"]).convert("RGB")
    for sample in hd.get("markerSamples") or []:
        x, y = int(sample["x"]), int(sample["y"])
        r, g, b = pixel_at(im, x, y)
        if (r, g, b) != (int(sample["red"]), int(sample["green"]), int(sample["blue"])):
            print(
                f"heading marker pixel drift at ({x},{y}): "
                f"png=({r},{g},{b}) cm=({sample['red']},{sample['green']},{sample['blue']})",
                file=sys.stderr,
            )
            return 1

    print("live capture pixel gate: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
