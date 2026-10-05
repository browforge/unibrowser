# PDF highlight annotations (stub)

Unibrowser stores **non-destructive** highlights beside the PDF file. The original PDF bytes are never modified.

## Sidecar file

For `report.pdf`, annotations live in:

`report.pdf.ub-annotations.json`

The JSON includes a `highlights` array with normalised rectangles (`x`, `y`, `w`, `h` on 0–1) per page.

## Commands

| Command | Status |
| --- | --- |
| **Add PDF highlight** | Arm highlight mode, then drag on the page |
| **PDF stamp** | Greyed (not implemented) |
| **PDF ink** | Greyed (not implemented) |

Open a PDF, run **Add PDF highlight** from the palette, then drag on the page to save a highlight to the sidecar.

## Limitations (blitz)

- Highlights are viewer overlays only (not embedded in the PDF).
- No export to PDF/A with annotations burned in.
- Stamp and ink tools are placeholders for a future pass.
