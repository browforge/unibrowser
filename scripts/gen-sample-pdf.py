#!/usr/bin/env python3
"""Regenerate examples/demo-pack/sample.pdf (multi-page VQA fixture)."""
from pathlib import Path

from reportlab.lib.pagesizes import letter
from reportlab.pdfgen import canvas

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "examples" / "demo-pack" / "sample.pdf"

PAGES = [
    (
        "Field report — overview",
        [
            "This sample PDF demonstrates fit-width viewing and multi-page navigation.",
            "Representative body text wraps within the printable area.",
            "Use the thumbnail strip and page controls to move between pages.",
        ],
    ),
    (
        "Methods and observations",
        [
            "Page two contains structured paragraphs for visual QA.",
            "Annotations and highlights should remain readable at paper scale.",
            "The warm workspace surround frames the page without shrinking it.",
        ],
    ),
    (
        "Summary",
        [
            "Final page confirms export and annotation tooling.",
            "Status bar should report page index and zoom after fit-width.",
        ],
    ),
]


def main() -> None:
    c = canvas.Canvas(str(OUT), pagesize=letter)
    w, h = letter
    for i, (title, paras) in enumerate(PAGES, 1):
        c.setFont("Helvetica-Bold", 16)
        c.drawString(72, h - 72, title)
        c.setFont("Helvetica", 11)
        y = h - 108
        for para in paras:
            c.drawString(72, y, para)
            y -= 18
        c.setFont("Helvetica", 9)
        c.drawRightString(w - 72, 48, f"Page {i} of {len(PAGES)}")
        c.showPage()
    c.save()
    print(f"wrote {OUT} ({len(PAGES)} pages)")


if __name__ == "__main__":
    main()
