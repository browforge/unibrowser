#!/usr/bin/env python3
"""Require narrow Command Manager controls to be a prefix of wider controls."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def control_ids(path: Path) -> tuple[str, list[str]]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    context = str(payload.get("contextLabel", ""))
    controls = payload.get("controls", [])
    return context, [str(control.get("controlId", "")) for control in controls]


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check-cm-responsive-prefix.py <capture-directory>", file=sys.stderr)
        return 2

    capture_dir = Path(sys.argv[1])
    narrow_files = sorted(capture_dir.glob("*-light-1024.cm.json"))
    if not narrow_files:
        print("no 1024 Command Manager diagnostics found", file=sys.stderr)
        return 1

    failures: list[str] = []
    for narrow_path in narrow_files:
        wide_path = Path(str(narrow_path).replace("-light-1024.cm.json", "-light-1920.cm.json"))
        if not wide_path.exists():
            failures.append(f"{narrow_path.name}: missing {wide_path.name}")
            continue
        narrow_context, narrow_ids = control_ids(narrow_path)
        wide_context, wide_ids = control_ids(wide_path)
        if narrow_context != wide_context:
            failures.append(
                f"{narrow_path.name}: context changed from {narrow_context!r} to {wide_context!r}"
            )
            continue
        if narrow_ids != wide_ids[: len(narrow_ids)]:
            failures.append(
                f"{narrow_path.name}: visible controls are not a prefix of {wide_path.name}"
            )

    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"Responsive CM prefix check passed for {len(narrow_files)} contexts")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
