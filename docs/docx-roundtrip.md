# DOCX round-trip (lite)

## What works

- Open `.docx` via **base-docx** (Mammoth → HTML in the viewer).
- Experimental **contenteditable** preview when spellcheck experiment is on.
- **Save edits as HTML** writes a sibling file next to the document:

  `{name}.docx.ub-edit.html`

  The file is a minimal HTML document with your edited body markup and the current preview font size.

- **Export print HTML** / print preview still use the original Mammoth HTML at open time until you re-open the file.

## What does not work

- Saving back into `.docx` (OOXML packer) is **not** implemented in the blitz. Layout, styles, fields, and tracked changes from Word are not round-tripped.
- The `.ub-edit.html` sibling is an honest export for diffing, archiving, or opening in a browser — not a Word replacement.

## Future

- Optional zip-based DOCX writer for simple paragraph HTML only, behind a clear fidelity warning.
