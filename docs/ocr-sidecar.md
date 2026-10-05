# OCR (Tesseract CLI)

Unibrowser uses the **tesseract** binary on PATH (override with `UNIBROWSER_TESSERACT_PATH`). No bundled engine.

## Commands

| Command | When |
| --- | --- |
| **Copy text from image (OCR)** | Active PNG/JPEG/WebP/BMP/GIF tab, or PDF (page 1 via `pdftoppm` + tesseract) |
| Image toolbar **Extract text (OCR)** | Same, when tesseract is detected |

If tesseract is missing, controls stay disabled with an install notice (non-blocking).

## Tests

- `unibrowser-ocr`: `ocr_smoke_when_tesseract_present` skips when tesseract is absent.

## Limits

- Language fixed to `eng` in blitz.
- Scanned PDFs need Poppler `pdftoppm` for page rasterization.
- No layout boxes or hOCR export.
