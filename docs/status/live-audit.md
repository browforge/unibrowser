# LIVE block editing audit

Date: 2026-10-28 (updated). Reference behaviour: Sarala (GPL-3.0, studied from README only; no code copied).

## Mode design (Bear / Obsidian / Typora)

**Decision:** Markdown **Visual** (`wysiwyg`) is live-block editing on the shared Monaco buffer — not a fifth toolbar mode and not ProseMirror for `.md` files. ProseMirror Visual remains for Word/DOCX-shaped documents only (`documentModeHistory.ts`, `documentEditorHost.ts`). Split and Source still show raw markdown in Monaco; Visual toggles live-block overlays on the same model and undo stack.

## Sarala-aligned behaviours

| Behaviour | Status after chunk 1 | Pieces | Risks |
| --- | --- | --- | --- |
| One page, no split preview; blocks render until caret enters | **Visual (md)** on Monaco overlays | `markdownLiveEditor.ts`, `markdownLiveRender.ts` | Split still side-by-side |
| Active block dimmed markers; inactive rendered | **Implemented** (overlay render + hidden source lines) | `deltaDecorations`, `.md-live-overlay-layer` | Decoration + overlay churn |
| Click rendered → source offset | **Mapping unit tests** | `markdownLiveMapping.ts` | RTL (**LIVE-06**) |
| Smart Enter / Shift+Enter | **Implemented** | `applySmartEnter` | Fence close in **LIVE-02** |
| Broken math/diagram last good render | **Deferred** | **LIVE-04** | — |
| IME-safe composition | **Implemented** | composition gate | — |

## Chunk 1 scope

| Block kinds | Treatment |
| --- | --- |
| Paragraph, heading, emphasis, lists, blockquote | Inactive: HTML overlay; active: source + dimmed markers |
| Code fence | Raw source (**LIVE-02** deliberate styling) |
| Table, math, diagram | **LIVE-03**, **LIVE-04** |

## Undo and history

Markdown editing surfaces that mount Monaco (**Visual, Source, Split, Read**) share one Monaco undo stack when switching modes (`modeSwitchPreservesMonacoUndoStack` in `documentModeHistory.ts`, tested in `documentModeHistory.test.mjs`). DOCX Visual still uses ProseMirror history separately.

## Proofing

Markdown checks run only on prose byte ranges (`markdown_prose_ranges` in `unibrowser-diagnostics`) so `#`, fences, and code blocks are not spell/grammar targets.

## Probe tests (ignored)

| Ticket | Location | Intent |
| --- | --- | --- |
| LIVE-02 | `live_block_probes.rs` | Code fence live render + smart Enter close |
| LIVE-03 | same | Table click mapping |
| LIVE-04 | same | Math/diagram error retention |

## Performance

Overlays per inactive block; debounced refresh. **LIVE-05** for virtualisation on huge files.

## Captures

`scripts/check-live-capture-pixels.py` gates PNG pixels only (heading ink height, hash/`**` marker ink, dimmed marker tone, squiggle colours).
