#!/usr/bin/env python3
"""Verify new-project capture PNGs: size, uniqueness (md5), minimum size."""
from __future__ import annotations

import hashlib
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/screenshots/new-project"

EXPECTED = (
    "new-tab-paper-card-1280.png",
    "template-gallery-pick-1280.png",
    "template-gallery-configure-1280.png",
    "empty-project-tab-1280.png",
    "office-template-project-1280.png",
    "save-as-template-1280.png",
    "docx-tab-breadcrumb-1280.png",
    "new-tab-paper-card-web-1280.png",
    "template-gallery-pick-web-1280.png",
)


def file_md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def dimensions(path: Path) -> str:
    out = subprocess.check_output(
        ["identify", "-format", "%wx%h", str(path)],
        text=True,
    ).strip()
    return out


def main() -> int:
    ok = True
    seen: dict[str, str] = {}
    for name in EXPECTED:
        path = OUT / name
        if not path.is_file():
            print(f"missing {path}", file=sys.stderr)
            ok = False
            continue
        data = path.read_bytes()
        if len(data) < 4096:
            print(f"too small {path} ({len(data)} bytes)", file=sys.stderr)
            ok = False
        dim = dimensions(path)
        if dim != "1280x800":
            print(f"bad size {path}: {dim}", file=sys.stderr)
            ok = False
        digest = file_md5(path)
        if digest in seen:
            print(
                f"duplicate md5 {digest}: {name} matches {seen[digest]}",
                file=sys.stderr,
            )
            ok = False
        else:
            seen[digest] = name
        print(f"{name}: {dim} md5={digest} bytes={len(data)}")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
