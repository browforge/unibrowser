# CMD-11 evidence grid

Captures live under `docs/screenshots/cmd-11/`. Regenerate with `./scripts/capture-cmd-11.sh` after `scripts/ensure-release-desktop.sh`.

Legend: **Pass** matches worked-context intent; **Drift** honest gap vs `command-system.md` target; **Fail** blocking defect.

| Capture | Context | Theme / scale | CM context | Overflow / clip | Contrast / HC | Save | Emulation / unsupported | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| plain-text-light-100 | Plain text | light 100% | Text | Chevron; no label clip | Pass | Visible teal Save | Size active; formatting in overflow | Pass |
| plain-text-dark-100 | Plain text | dark 100% | Text | Same | Pass | Pass | Pass | Pass |
| plain-text-hc-100 | Plain text | light + HC | Text | Same | HC borders on chrome | Pass | Pass | Pass |
| plain-text-light-150 | Plain text | light 150% | Text | Same | Pass | Pass | Pass | Pass |
| markdown-light-100 | Markdown | light 100% | Text | Chevron; 15 controls resolved | Pass | Pass | Subscript emulation in overflow set | Pass |
| markdown-dark-100 | Markdown | dark 100% | Text | Same | Pass | Pass | Pass | Pass |
| markdown-hc-100 | Markdown | light + HC | Text | Same | HC chrome | Pass | Pass | Pass |
| markdown-light-150 | Markdown | light 150% | Text | Same | Pass | Pass | Pass | Pass |
| rich-document-light-100 | Rich DOCX | light 100% | Office | Chevron | Pass | Pass | Full formatting family resolved | Pass |
| rich-document-dark-100 | Rich DOCX | dark 100% | Office | Same | Pass | Pass | Pass | Pass |
| rich-document-hc-100 | Rich DOCX | light + HC | Office | Same | Pass | Pass | Pass | Pass |
| rich-document-light-150 | Rich DOCX | light 150% | Office | Same | Pass | Pass | Pass | Pass |
| code-light-100 | Code | light 100% | Code | Editor appearance + overflow | Pass | Pass | Doc formatting unsupported | Pass |
| code-dark-100 | Code | dark 100% | Code | Same | Pass | Pass | Pass | Pass |
| code-hc-100 | Code | light + HC | Code | Same | Pass | Pass | Pass | Pass |
| code-light-150 | Code | light 150% | Code | Same | Pass | Pass | Pass | Pass |
| pdf-annotation-light-100 | PDF | light 100% | PDF Annotation | PDF groups + overflow | Pass | Pass | Page/zoom in chrome | Drift |
| pdf-annotation-dark-100 | PDF | dark 100% | PDF Annotation | Same | Pass | Pass | Pass | Drift |
| pdf-annotation-hc-100 | PDF | light + HC | PDF Annotation | Same | Pass | Pass | Pass | Drift |
| pdf-annotation-light-150 | PDF | light 150% | PDF Annotation | Same | Pass | Pass | Pass | Drift |
| image-light-100 | Image | light 100% | Image | Adjust family | Pass | Pass | Text formatting absent | Drift |
| image-dark-100 | Image | dark 100% | Image | Same | Pass | Pass | Pass | Drift |
| image-hc-100 | Image | light + HC | Image | Same | Pass | Pass | Pass | Drift |
| image-light-150 | Image | light 150% | Image | Same | Pass | Pass | Pass | Drift |
| cad-3d (all variants) | CAD/3D | n/a | n/a | n/a | n/a | n/a | Path detect only; no family layout | Drift |

## Region notes (representative)

### Text / Markdown (plain-text-light-100, markdown-light-100)

- **Chrome:** Tab `sample.*`, breadcrumb, Undo/Redo, **Save** (supported), overflow menu.
- **CM:** Context `Text`; visible strip shows **Find** plus **overflow chevron**; pin row `Following: Text`. Resolved model has 15 controls with 14 in overflow (never-clip).
- **Editor band:** Monaco light surface with gutter and sample copy (pixel gate via frontend diagnostics).
- **Status:** Line/column, encoding, reading stats, Alerts.

### Code (code-light-100)

- **CM context:** `Code`; Editor Appearance controls in resolved model; document formatting unsupported with reasons in model (contract tests).

### Rich document (rich-document-light-100)

- **CM context:** `Office`; DOCX reader paper visible; formatting family supported per matrix.

### PDF (pdf-annotation-light-100)

- **CM:** `PDF Annotation` context chip; Find + overflow; PDF annotate/page/zoom/export groups in resolved model.
- **Viewer:** Page 1 of 3, zoom 52%; native PDF canvas below CM.
- **Drift:** Doc lists Hand/Ink/Stamp and Appearance groups not yet in layout (document updated).

### Image (image-light-100)

- **CM:** `Image` context; Adjust/Curves group.
- **Drift:** Crop/transform and export groups not yet implemented (document updated).

### High contrast (plain-text-hc-100, markdown-hc-100)

- **`data-high-contrast="true"`** via Settings pref + `applyHighContrastToDocument` demo token (real setting, not capture-only CSS).
- Thicker borders on chrome, CM, and tabs.

### 150% scale (markdown-light-150)

- Root `font-size: 150%` demo token; CM and Save scale with shell.

## Contract / save tests

- `unibrowser-engine::command_system::worked_context_contract::*` — matrix vs `command-system.md`.
- `save_stays_supported_in_every_worked_context` — Save capability independent of formatting failures.

## Doc vs runtime corrections (this batch)

- **Runtime:** `Editor text size` label for plain text and Markdown size control (`families.rs`).
- **Doc:** PDF, Image, and CAD sections note current vs target families (`command-system.md`).
- **Fixture:** Replaced stub `%DOCX%` with minimal OOXML `sample.docx` for rich-document captures.

## Gaps (not Fail)

- **CAD/3D:** `WorkContext::Cad3d` detected; family layout and screenshots pending.
- **PDF / Image:** Partial CM vs full worked-example family lists (tracked as Drift).
- **Rust editor pixel gate:** logs warning when composited crop cannot sample Monaco band; frontend Monaco gate remains authoritative for text captures.
