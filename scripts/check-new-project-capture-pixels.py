#!/usr/bin/env python3
"""Verify new-project capture PNGs: size, uniqueness (md5), minimum size, DOM gates."""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/screenshots/new-project"

EXPECTED = (
    "new-tab-paper-card-v5-1280.png",
    "template-gallery-pick-v5-1280.png",
    "template-gallery-pick-office-v5-1280.png",
    "template-gallery-configure-v5-1280.png",
    "empty-project-tab-v5-1280.png",
    "office-template-project-v5-1280.png",
    "presentation-template-project-v5-1280.png",
    "save-as-template-v5-1280.png",
    "docx-tab-breadcrumb-v5-1280.png",
    "new-tab-paper-card-v5-web-1280.png",
    "template-gallery-pick-v5-web-1280.png",
)

FOCUS_CM = {
    "office-template-project-v5-1280.png": "office",
    "empty-project-tab-v5-1280.png": "empty",
    "presentation-template-project-v5-1280.png": "presentation",
}


def file_md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def dimensions(path: Path) -> str:
    out = subprocess.check_output(
        ["identify", "-format", "%wx%h", str(path)],
        text=True,
    ).strip()
    return out


def dom_gate_ok(cm: dict, kind: str) -> str | None:
    dom = cm.get("dom") or {}
    if kind == "office":
        footer = str(dom.get("documentCardFooterText") or "")
        match = re.search(r"(\d+)\s+words?\b", footer, re.I)
        if not match:
            return f"office footer missing word count in '{footer}'"
        if int(match.group(1)) <= 0:
            return f"office footer word count must be > 0 in '{footer}'"
    if kind == "presentation":
        if dom.get("presentationToolbarThemed") is not True:
            return "presentation toolbar buttons must use themed Lucide controls"
    if kind == "empty":
        card_h = dom.get("projectHomeCardHeight")
        if card_h is not None and float(card_h) >= 300:
            return f"empty project card height {card_h}px must be < 300"
        if dom.get("horizontalOverflow") is True:
            return "empty project capture has horizontal overflow"
    return None


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

        if name in FOCUS_CM:
            cm_path = OUT / f".{name.replace('-v5-1280.png', '')}.cm.json"
            alt_cm = path.with_suffix(".cm.json")
            cm_file = cm_path if cm_path.is_file() else alt_cm
            if not cm_file.is_file():
                print(f"missing CM diagnostics for {name}: {cm_file}", file=sys.stderr)
                ok = False
            else:
                cm = json.loads(cm_file.read_text())
                gate_err = dom_gate_ok(cm, FOCUS_CM[name])
                if gate_err:
                    print(f"{gate_err} ({cm_file.name})", file=sys.stderr)
                    ok = False

        print(f"{name}: {dim} md5={digest} bytes={len(data)}")
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
