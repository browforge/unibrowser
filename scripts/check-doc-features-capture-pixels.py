#!/usr/bin/env python3
"""Gate doc-feature PNG captures: unique MD5s, CM chrome, DOM layout, and anti-reuse guards."""
from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOC_FEATURES_DIR = ROOT / "docs" / "screenshots" / "doc-features"
CAPTURE_SPEC = ROOT / "crates" / "unibrowser-desktop" / "tests" / "doc-features-capture.spec.ts"

NAMES = [
    "doc-wysiwyg-blockquote-desktop-v5.png",
    "doc-split-apply-banner-desktop-v5.png",
    "doc-opaque-card-desktop-v5.png",
    "doc-visual-docx-desktop-v6.png",
    "doc-history-panel-desktop-v5.png",
    "doc-wysiwyg-blockquote-web-v5.png",
    "doc-split-apply-banner-web-v5.png",
    "doc-opaque-card-web-v5.png",
    "doc-visual-docx-web-v6.png",
    "doc-history-panel-web-v5.png",
]

DOC_VISUAL_STEM_RE = re.compile(
    r"^doc-visual-docx-(desktop|web)-v(\d+)\.png$",
)


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


def dom_assertions_ok(cm: dict, png_name: str) -> str | None:
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
    rich_violations = dom.get("richEditorSubtreeViolations")
    if isinstance(rich_violations, int) and rich_violations > 0:
        return f"richEditorSubtreeViolations={rich_violations}"
    if "doc-visual-docx" in png_name:
        footer = dom.get("documentCardFooterText")
        if isinstance(footer, str) and "Word" not in footer:
            return "doc-visual-docx capture footer must include Word kind"
        if dom.get("docxEmulationBadgeVisible") is not True:
            return "doc-visual-docx capture must show ≈ emulation badge"
        if dom.get("cardImageNaturalWidthOk") is not True:
            return "doc-visual-docx capture requires card images with naturalWidth > 0"
        status = dom.get("statusBarDocumentLanguage")
        if isinstance(status, str) and "Word document" not in status:
            return "doc-visual-docx capture status bar must show Word document"
    return None


def check_capture_spec_no_patch_status_item() -> str | None:
    if not CAPTURE_SPEC.is_file():
        return f"missing capture spec: {CAPTURE_SPEC}"
    text = CAPTURE_SPEC.read_text()
    if "patchStatusItem" in text:
        return (
            f"{CAPTURE_SPEC.relative_to(ROOT)} must not call patchStatusItem; "
            "status bar must come from the app"
        )
    return None


def check_doc_visual_not_reused(paths: dict[str, Path]) -> str | None:
    prior_by_platform: dict[str, list[tuple[int, str, Path]]] = {
        "desktop": [],
        "web": [],
    }
    if DOC_FEATURES_DIR.is_dir():
        for path in sorted(DOC_FEATURES_DIR.glob("doc-visual-docx-*.png")):
            match = DOC_VISUAL_STEM_RE.match(path.name)
            if not match:
                continue
            platform, version = match.group(1), int(match.group(2))
            prior_by_platform[platform].append((version, file_md5(path), path))

    for name, path in paths.items():
        match = DOC_VISUAL_STEM_RE.match(name)
        if not match:
            continue
        platform, version = match.group(1), int(match.group(2))
        digest = file_md5(path)
        for old_version, old_digest, old_path in prior_by_platform[platform]:
            if old_version >= version:
                continue
            if digest == old_digest:
                return (
                    f"{name} md5 {digest} matches earlier capture {old_path.name} "
                    f"(version v{old_version}); recapture required"
                )
    return None


def main() -> int:
    spec_err = check_capture_spec_no_patch_status_item()
    if spec_err:
        print(spec_err, file=sys.stderr)
        return 1

    roots = [Path(p) for p in sys.argv[1:]] if len(sys.argv) > 1 else [
        Path("/opt/cursor/artifacts"),
        DOC_FEATURES_DIR,
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

    reuse_err = check_doc_visual_not_reused(paths)
    if reuse_err:
        print(reuse_err, file=sys.stderr)
        return 1

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
        dom_err = dom_assertions_ok(cm, name)
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
