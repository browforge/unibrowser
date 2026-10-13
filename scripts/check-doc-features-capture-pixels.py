#!/usr/bin/env python3
"""Gate doc-feature v3 PNG captures: unique MD5s, CM chrome, and DOM layout assertions."""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

NAMES = [
    "doc-wysiwyg-blockquote-desktop-v3.png",
    "doc-split-apply-banner-desktop-v3.png",
    "doc-opaque-card-desktop-v3.png",
    "doc-visual-docx-desktop-v3.png",
    "doc-history-panel-desktop-v3.png",
    "doc-wysiwyg-blockquote-web-v3.png",
    "doc-split-apply-banner-web-v3.png",
    "doc-opaque-card-web-v3.png",
    "doc-visual-docx-web-v3.png",
    "doc-history-panel-web-v3.png",
]


def file_md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def rail_flyout_visible(cm: dict) -> bool:
    width = cm.get("railFlyoutWidth")
    if width is None:
        return False
    return float(width) > 8


def full_command_manager_row_visible(cm: dict) -> bool:
    dom = cm.get("dom") or {}
    if dom.get("commandManagerVisible") is True:
        return True
    controls = cm.get("controls") or []
    visible = [
        c
        for c in controls
        if (c.get("height") or 0) >= 24
        and (c.get("width") or 0) >= 24
        and (c.get("labelBounds") or {}).get("y", 0) < 120
    ]
    groups_width = (cm.get("pipeline") or {}).get("groupsWidth") or 0
    return len(visible) >= 4 and groups_width > 200


def dom_assertions_ok(cm: dict) -> str | None:
    dom = cm.get("dom")
    if not isinstance(dom, dict):
        return "missing dom diagnostics in capture CM JSON"
    count = dom.get("visibleDocumentCardCount")
    if count != 1:
        return f"expected visibleDocumentCardCount=1, got {count!r}"
    if dom.get("commandManagerVisible") is True:
        return "commandManagerVisible must be false"
    if dom.get("footerUsesCardTokens") is not True:
        return "footerUsesCardTokens must be true"
    if dom.get("modeMatchesVisibleHost") is False:
        return (
            f"modeMatchesVisibleHost false "
            f"(mode={dom.get('activeMode')!r}, host={dom.get('visibleEditorHost')!r})"
        )
    overlap = dom.get("outsideOverlapCount")
    if isinstance(overlap, int) and overlap > 0:
        return f"outsideOverlapCount={overlap}"
    return None


def main() -> int:
    roots = [Path(p) for p in sys.argv[1:]] if len(sys.argv) > 1 else [
        Path("/opt/cursor/artifacts"),
        Path("docs/screenshots/doc-features"),
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

    for name, path in paths.items():
        cm_path = path.with_suffix(".cm.json")
        if not cm_path.is_file():
            print(f"missing CM diagnostics for {name}: {cm_path}", file=sys.stderr)
            return 1
        cm = json.loads(cm_path.read_text())
        if full_command_manager_row_visible(cm):
            print(
                f"full Command Manager row visible in {name} (see {cm_path.name})",
                file=sys.stderr,
            )
            return 1
        if rail_flyout_visible(cm):
            print(
                f"activity rail flyout visible in {name} (see {cm_path.name})",
                file=sys.stderr,
            )
            return 1
        dom_err = dom_assertions_ok(cm)
        if dom_err:
            print(f"{dom_err} in {name} (see {cm_path.name})", file=sys.stderr)
            return 1

    print(
        "doc-feature capture pixel gate: ok "
        f"({len(NAMES)} unique files, DOM + CM chrome checks passed)"
    )
    for name in NAMES:
        print(f"  {name} md5={digests[name]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
