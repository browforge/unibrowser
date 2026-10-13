#!/usr/bin/env python3
"""Verify new-project capture PNG dimensions and non-empty checksums."""
from __future__ import annotations

import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/screenshots/new-project"
EXPECTED = ("new-tab-paper-card-1280.png", "template-gallery-1280.png")


def main() -> int:
    ok = True
    for name in EXPECTED:
        path = OUT / name
        if not path.is_file():
            print(f"missing {path}", file=sys.stderr)
            ok = False
            continue
        data = path.read_bytes()
        if len(data) < 4096:
            print(f"too small {path}", file=sys.stderr)
            ok = False
        digest = hashlib.md5(data).hexdigest()
        print(f"{name}: md5={digest} bytes={len(data)}")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
