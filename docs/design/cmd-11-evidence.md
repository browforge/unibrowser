# CMD-11 evidence

All evidence is a 1600 x 1000 full-window capture under
`docs/screenshots/cmd-11/`. `scripts/capture-cmd-11.sh` rejects images smaller
than 1200 x 700 and rejects duplicate SHA-256 output hashes.

Legend: **Pass** proves the implemented model. **Drift** records an explicit
current-runtime gap against the target family list. **Fail** blocks Done.

## Integrity checks

- All 24 PNGs are 1600 x 1000 and have distinct SHA-256 hashes.
- `plain-text-light-150.png` Git blob:
  `69093fcb3653909850d88055dabc2e39631daffd`.
- `markdown-light-150.png` Git blob:
  `388e06e8d7bd1b854520267af085f698d3b8cbe0`.
- Rust Monaco pixel validation is fatal again. Text and code captures passed
  with non-zero ink and row counts after capture geometry was fixed.
- Light vs high-contrast absolute changed-pixel counts:
  Plain text 153230; Markdown 149023; rich document 525607; code 129378;
  PDF 450773; image 543362. High contrast uses persisted accessibility
  settings, black/white surface tokens, 2 px borders, and yellow focus.
- 150% uses the persisted **Interface text scale** setting (100/125/150) from
  Settings, not capture-only CSS.

## Region-by-region grid

Every row was opened and inspected. “Chrome” includes the complete title bar
and Save. “CM” includes the context, complete visible controls, overflow, and
right-side Follow/Pin controls.

| PNG | Chrome | Command Manager | Content top / status | Verdict |
| --- | --- | --- | --- | --- |
| `plain-text-light-100` | Full light title bar; Save visible | Text; formatting greyed in place; Strikethrough focused | Reason tooltip visible; Monaco gutter/text; status visible | Pass |
| `plain-text-dark-100` | Full dark title bar; Save visible | Same stable positions; disabled contrast remains readable | Dark Monaco; reason tooltip; status visible | Pass |
| `plain-text-hc-100` | Yellow focus + strong borders; Save visible | Same controls, high-contrast focus on Strikethrough | White canvas; reason tooltip; status visible | Pass |
| `plain-text-light-150` | Full width at persisted 150%; Save visible | No clipped labels; overflow and Follow/Pin visible | Larger tooltip/status; Monaco top visible | Pass |
| `markdown-light-100` | Full light title bar; Save visible | Text; Subscript focused with subtle dotted marker | Approximation tooltip (`<sub>`/font scaling); Monaco visible | Pass |
| `markdown-dark-100` | Full dark title bar; Save visible | Same family geometry and emulation marker | Dark Monaco; approximation tooltip | Pass |
| `markdown-hc-100` | Yellow focus + strong borders; Save visible | Subscript focus and marker remain distinguishable | White canvas; approximation tooltip | Pass |
| `markdown-light-150` | Full width at 150%; Save visible | Labels, overflow, Follow/Pin fit | Larger approximation tooltip/status; distinct from plain text | Pass |
| `rich-document-light-100` | Full light title bar; Save visible | Office; Font/Size/emphasis/baseline visible | DOCX paper and status visible | Pass |
| `rich-document-dark-100` | Full dark title bar; Save visible | Same supported controls | Dark well around readable paper | Pass |
| `rich-document-hc-100` | High-contrast focus/borders; Save visible | Stable Office controls | High-contrast outer surfaces; paper readable | Pass |
| `rich-document-light-150` | Full width at 150%; Save visible | No clipping; Follow/Pin visible | Enlarged paper text/status | Pass |
| `code-light-100` | Full light title bar; Save visible | Code; editor Font/Size, Find, Check, Export | Monaco syntax top and status visible | Pass |
| `code-dark-100` | Full dark title bar; Save visible | Same code family | Dark Monaco syntax and status | Pass |
| `code-hc-100` | High-contrast focus/borders; Save visible | Stable code family | White Monaco with syntax | Pass |
| `code-light-150` | Full width at 150%; Save visible | No clipping; Follow/Pin visible | Enlarged Monaco/status | Pass |
| `pdf-annotation-light-100` | Full title bar; Save visible | PDF Annotation; Select/Highlight/Note/Area/Page/zoom/Export PDF | Pages, canvas, annotation panel, status | **Drift** |
| `pdf-annotation-dark-100` | Full dark title bar; Save visible | Same implemented PDF controls | Dark surrounds; PDF remains readable | **Drift** |
| `pdf-annotation-hc-100` | Strong borders/focus; Save visible | Same implemented PDF controls | HC pages/annotation boundaries | **Drift** |
| `pdf-annotation-light-150` | Full width at 150%; Save visible | No clipped PDF labels; Follow/Pin visible | Enlarged viewer sidebars/status | **Drift** |
| `image-light-100` | Full title bar; Save visible | Image; Undo/Redo/Find/Check/Export/Curves | SVG canvas, selection inspector, status | **Drift** |
| `image-dark-100` | Full dark title bar; Save visible | Same implemented image controls | Dark canvas/inspector | **Drift** |
| `image-hc-100` | Strong borders/focus; Save visible | Same implemented image controls | White HC surround; canvas remains distinct | **Drift** |
| `image-light-150` | Full width at 150%; Save visible | No clipped labels; Follow/Pin visible | Enlarged inspector/status | **Drift** |
| CAD/3D | No capture: `.dxf` resolves to CAD/3D but no CAD family layout exists | Pending | Pending | **Drift** |

PDF drift is explicit: Hand, Ink, Stamp, and Appearance controls from the
target family list are not implemented. Image drift is explicit: Crop,
Transform, extended Adjust, and conditional Text Formatting are not
implemented. CAD/3D has context detection only.

## Corrections

1. **Identical 150% blobs:** the old 560 x 140 shell-only crops excluded the
   differing editor and most controls. The source files did switch, but the
   captured pixels did not include evidence of that switch. Full-window
   capture plus tooltip focus now proves plain-text unsupported vs Markdown
   emulated state.
2. **560 x 140 crop:** doc capture reapplied persisted window geometry after
   Xvfb launch. Capture mode now bypasses saved geometry and sets the real
   window to 1600 x 1000.
3. **Pixel gate warning:** restored to fatal. The previous empty sample was a
   coordinate mismatch caused by the 560 x 140 PNG paired with a 1600-wide DOM
   editor rectangle; fixed geometry and proportional sampling now pass.
4. **High contrast:** previous mode changed little beyond a flag/border. It now
   has distinct persisted tokens and measured pixel differences above.
5. **Runtime/doc disagreement:** built-in Markdown emulation providers were
   incorrectly sent through the mod runtime and became “provider unavailable”.
   Built-in `unibrowser.*` semantic providers now retain the resolver’s
   Emulated state and approximation note. Plain/Markdown size is labelled
   “Editor text size”. PDF, Image, and CAD target/current differences remain
   documented as Drift.

## Tests

- `command_system::worked_context_contract` verifies worked-context states and
  reasons.
- `save_stays_supported_in_every_worked_context` verifies Save remains
  supported.
- Capture diagnostics verify CM contrast and Monaco paint before writing PNG.
