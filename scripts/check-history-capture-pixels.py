#!/usr/bin/env python3
"""Gate history panel/jump PNG pair: unique files, document crop diff > 1%, panel chrome."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "screenshots" / "history"
PANEL = OUT / "history-panel-v2-1280.png"
JUMP = OUT / "history-jump-v2-1280.png"
PANEL_CM = OUT / "history-panel-v2-1280.cm.json"
JUMP_CM = OUT / "history-jump-v2-1280.cm.json"


def md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def document_crop_box(height: int) -> tuple[int, int, int, int]:
    # Document area above ~35% bottom dock on 1280x800 captures.
    top = 0
    bottom = int(height * 0.62)
    return (0, top, 1280, bottom)


def diff_fraction(a_path: Path, b_path: Path, box: tuple[int, int, int, int]) -> float:
    try:
        from PIL import Image
    except ImportError:
        print("PIL missing; install Pillow for history capture gate", file=sys.stderr)
        return 1.0
    a = Image.open(a_path).convert("RGB").crop(box)
    b = Image.open(b_path).convert("RGB").crop(box)
    if a.size != b.size:
        return 1.0
    pixels_a = a.getdata()
    pixels_b = b.getdata()
    changed = sum(1 for p, q in zip(pixels_a, pixels_b) if p != q)
    return changed / max(len(pixels_a), 1)


def assert_cm_dom(cm_path: Path, expect_edit_three: bool, expect_restored_top: bool) -> None:
    raw = json.loads(cm_path.read_text(encoding="utf-8"))
    dom = raw.get("dom") or {}
    if dom.get("historyCaptureReady") is not True:
        raise SystemExit(f"{cm_path.name}: historyCaptureReady not true")
    if dom.get("historyToolbarVisible") is not True:
        raise SystemExit(f"{cm_path.name}: historyToolbarVisible not true")
    if dom.get("historyRestoreVisible") is not True:
        raise SystemExit(f"{cm_path.name}: historyRestoreVisible not true")
    if expect_edit_three and dom.get("historyDocHasEditThree") is not True:
        raise SystemExit(f"{cm_path.name}: expected Edit three in document")
    if not expect_edit_three and dom.get("historyDocHasEditThree") is True:
        raise SystemExit(f"{cm_path.name}: document must not contain Edit three after jump")
    if expect_restored_top and dom.get("historyTopSummary", "").find("Restored to #") < 0:
        raise SystemExit(f"{cm_path.name}: top summary must be Restored to #N")


def main() -> None:
    for path in (PANEL, JUMP, PANEL_CM, JUMP_CM):
        if not path.is_file():
            raise SystemExit(f"missing {path}")
    if md5(PANEL) == md5(JUMP):
        raise SystemExit("panel and jump PNGs are identical")
    frac = diff_fraction(PANEL, JUMP, document_crop_box(800))
    if frac <= 0.01:
        raise SystemExit(f"document region diff {frac:.4f} <= 1%")
    assert_cm_dom(PANEL_CM, expect_edit_three=True, expect_restored_top=False)
    assert_cm_dom(JUMP_CM, expect_edit_three=False, expect_restored_top=True)
    print(f"history capture gate ok (document diff {frac:.2%})")


if __name__ == "__main__":
    main()
