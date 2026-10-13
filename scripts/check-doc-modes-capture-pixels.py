#!/usr/bin/env python3
"""Gate doc-mode PNG captures: unique MD5s and split right-half differs from source."""
from __future__ import annotations

import hashlib
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:  # pragma: no cover
    print("check-doc-modes-capture-pixels: Pillow required (pip install Pillow)", file=sys.stderr)
    sys.exit(2)

NAMES = [
    "doc-modes-wysiwyg-desktop-v3.png",
    "doc-modes-source-desktop-v3.png",
    "doc-modes-split-desktop-v3.png",
    "doc-modes-picker-desktop-v3.png",
    "doc-modes-wysiwyg-web-v3.png",
    "doc-modes-source-web-v3.png",
    "doc-modes-split-web-v3.png",
    "doc-modes-picker-web-v3.png",
]


def file_md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def card_body_right_half_digest(path: Path) -> str:
    im = Image.open(path).convert("RGB")
    w, h = im.size
    x0 = int(w * 0.34)
    x1 = int(w * 0.93)
    y0 = int(h * 0.17)
    y1 = int(h * 0.86)
    mid = (x0 + x1) // 2
    crop = im.crop((mid, y0, x1, y1))
    return hashlib.md5(crop.tobytes()).hexdigest()


def main() -> int:
    roots = [Path(p) for p in sys.argv[1:]] if len(sys.argv) > 1 else [
        Path("/opt/cursor/artifacts"),
        Path("docs/screenshots/doc-modes"),
    ]
    paths: dict[str, Path] = {}
    for name in NAMES:
        found = None
        for root in roots:
            candidate = root / name
            if candidate.is_file():
                found = candidate
                break
        if not found:
            print(f"missing capture: {name}", file=sys.stderr)
            return 1
        paths[name] = found

    digests = {name: file_md5(path) for name, path in paths.items()}
    seen: dict[str, str] = {}
    for name, digest in digests.items():
        if digest in seen:
            print(
                f"duplicate md5 {digest}: {name} matches {seen[digest]}",
                file=sys.stderr,
            )
            return 1
        seen[digest] = name

    for platform in ("desktop", "web"):
        source_name = f"doc-modes-source-{platform}-v3.png"
        split_name = f"doc-modes-split-{platform}-v3.png"
        source_right = card_body_right_half_digest(paths[source_name])
        split_right = card_body_right_half_digest(paths[split_name])
        if source_right == split_right:
            print(
                f"split {platform} right-half matches source right-half "
                f"(md5 {split_right}); preview pane not visible",
                file=sys.stderr,
            )
            return 1

    print("doc-mode capture pixel gate: ok (8 unique files, split right-halves differ)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
