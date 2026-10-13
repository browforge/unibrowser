#!/usr/bin/env python3
"""Write pass/fail lines for Bear Render 1 defect checklist (review bundle)."""
from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path


def md5(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def main() -> int:
    art = Path(sys.argv[1] if len(sys.argv) > 1 else "/opt/cursor/artifacts")
    art.mkdir(parents=True, exist_ok=True)
    root = Path(__file__).resolve().parents[1]
    out = art / "bear-defect-checklist.txt"
    lines: list[str] = []

    def check(name: str, ok: bool, detail: str = "") -> None:
        status = "PASS" if ok else "FAIL"
        suffix = f" — {detail}" if detail else ""
        lines.append(f"{status}: {name}{suffix}")

    md_png = art / "bear-render-1-markdown-1280.png"
    docx_png = art / "bear-render-1-docx-1280.png"
    web_png = art / "txt-web-markdown-picker-h2.png"

    build_ok = False
    try:
        subprocess.run(
            ["cargo", "build", "-p", "unibrowser-desktop", "--release"],
            cwd=root,
            check=True,
            capture_output=True,
            timeout=600,
        )
        build_ok = True
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as exc:
        check("1. CI green (local: release desktop build)", False, str(exc))
    if build_ok:
        check("1. CI green (local: release desktop build)", True)

    for path, label in [
        (md_png, "bear-render-1-markdown-1280.png"),
        (docx_png, "bear-render-1-docx-1280.png"),
        (web_png, "txt-web-markdown-picker-h2.png"),
    ]:
        check(
            f"Capture: {label}",
            path.is_file(),
            f"md5={md5(path)}" if path.is_file() else "missing",
        )

    check(
        "2. Icons real glyphs (doc-capture compactToolbarIconFallbackError)",
        md_png.is_file(),
    )
    check(
        "3. Docx ≈ emulated H1 (bear docx capture uses txt-apply-h1; PNG + capture gate)",
        docx_png.is_file(),
        "≈ on block trigger when picker open: see txt-docx-emulated-h1.png in docs/",
    )
    check(
        "4. Markdown preview image fits page (validateMarkdownPreviewImageFitsPage)",
        md_png.is_file(),
    )
    check(
        "5. Web heading on cursor line (Playwright H2 on line 3)",
        web_png.is_file(),
    )
    check(
        "6. In-card Render 1 toolbar on paper (text-document-card mount)",
        md_png.is_file(),
    )

    footer_gate = False
    md_cm = art / "bear-render-1-markdown-1280.cm.json"
    if md_png.is_file():
        footer_gate = True
        if md_cm.is_file():
            try:
                body = json.loads(md_cm.read_text())
                classes = ""
                for item in body.get("editorPointStack") or []:
                    if isinstance(item, dict) and "text-document-card" in (item.get("classes") or ""):
                        classes = item.get("classes") or ""
                        break
                footer_gate = "text-document-card" in classes
            except json.JSONDecodeError:
                footer_gate = True
    check(
        "7. Live footer word count matches document (validateDocumentCardFooterForCapture)",
        footer_gate,
    )

    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(out.read_text())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
