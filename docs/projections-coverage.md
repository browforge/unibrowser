# Document projections coverage (ADR 0025)

Text and raster surfaces for AI, search, OCR, and accessibility. Implementation: `crates/unibrowser-projections`.

| Viewer / backend | Text projection | Raster projection | Notes |
| --- | --- | --- | --- |
| Text / code (`base-text`, `base-js`) | File buffer | — | Full UTF-8 |
| Markdown (`toy-markdown`) | Source markdown | — | |
| Web (`base-web`) | DOM text from shell (redacted) or URL fallback | — | Password / cc fields stripped |
| DOCX (`base-docx`) | `unibrowser-docx::extract_plain_text` | — | |
| EPUB (`base-epub`) | Chapter text from reader or path stub | — | Shell passes `epub_chapter_text` when available |
| PDF (`base-pdf`) | pdf.js page text from shell or path/page stub | File decode when path is image-like | |
| SVG (`base-svg`) | `<text>` nodes + truncated markup | resvg raster | |
| PNG / JPEG (`base-png`) | `tEXt` annotation + OCR sidecar | Image RGBA | |
| DXF (`base-dxf`) | TEXT/MTEXT + layer list | — | |
| CSV (`base-csv`) | File contents | — | |
| Hex (`base-hex`) | Formatted dump | — | |
| `.unibrow` (`unibrow-project`) | Timeline summary + title-card text | — | `ProjectManifest` from package |
| Audio / video | Status line only | — | Future: waveform API (ADR 0025) |

Shell command: `document_projection_snapshot` (Tauri) merges `ProjectionExtras` from the active webview / OCR cache.
