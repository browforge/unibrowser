# DOCX round-trip (lite)

## What works

- Open `.docx` via **base-docx** (Mammoth → HTML in the viewer).
- **Save to .docx** (toolbar): writes supported HTML back into the OOXML package via `unibrowser-docx` (`save_docx_from_preview_html`).
  - Supported markup: paragraphs, headings `h1`–`h6`, **bold** / **italic**, lists (`ul`/`ol`/`li`), line breaks.
  - Original files containing tables, drawings, fields, or content controls are **rejected** with an error (not silently dropped).
  - Unsupported HTML tags in the editor are rejected before save.
- **Save edits as HTML** still writes `{name}.docx.ub-edit.html` for diffing.

## Limits

- Body text is replaced for simple documents; Word styles, numbering, and complex layout are not round-tripped.
- Save shows a warning that only the supported subset was written.
- Mammoth conversion notes at open time still apply; re-open after save to refresh preview.

## Tests

- `crates/unibrowser-docx`: `roundtrip_simple_paragraph`, `refuses_original_table`.

## Related

- [commands-undo-redo.md](commands-undo-redo.md) (print/export)
