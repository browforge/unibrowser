# Undo, redo, and PDF export matrix

Commands:

| ID | Title | Default chord |
| --- | --- | --- |
| `unibrowser.document.undo` | Undo | `Ctrl+Alt+Z` |
| `unibrowser.document.redo` | Redo | `Ctrl+Alt+Y` |
| `unibrowser.export.printPdf` | Export PDF (print HTML) | (none) |

## Undo / redo

| Document / view | Enabled | Mechanism | Notes |
| --- | --- | --- | --- |
| Monaco text (`base-text`) | Yes | Rust per-path undo stack on `sync_text_document` | `Ctrl+Z` / `Ctrl+Y` remain Monaco-local; document undo is the cross-save stack |
| SVG editor (`base-svg`) | Yes | Same Rust stack on `sync_svg_document` | |
| DOCX preview (`base-docx`) | Grey | — | View-only Mammoth HTML; no editable document model |
| PDF / image / video | Grey | — | No text undo lane |
| EPUB chapter HTML | Grey | — | Spine preview only |
| HTTP tab | Grey | — | Browser history is separate |
| Files browser | Grey | — | No document session |
| `.unibrow` project | Yes (timeline) | `project.json` snapshot on clip move/trim, duck, fade | `document_undo` / palette **Undo** restores last manifest; one level per edit (shared stack depth with text) |

Palette and Command Manager show the **disabled reason** from the mod registry when a handler is missing.

## Export PDF (print HTML)

Uses the same HTML body as **Print** / **Export print HTML** in the Markdown and DOCX viewers. On Linux the shell tries headless Chromium, then `wkhtmltopdf`; otherwise use **Print** and save as PDF from the system dialog.

| Document / view | Enabled | HTML source |
| --- | --- | --- |
| Markdown (`.md`, `base-text`) | Yes | Preview pane HTML when split preview is on; otherwise `<pre>` of source |
| DOCX (`base-docx`) | Yes | Mammoth article HTML (live DOM when exported) |
| Plain text / code | Grey | Enable Markdown preview path or open DOCX |
| EPUB | Grey | Use per-chapter **Print** until export is wired |
| PDF | Grey | Open PDF in a viewer that supports export |

See also [commands-cross-format.md](commands-cross-format.md) for find and zoom.
