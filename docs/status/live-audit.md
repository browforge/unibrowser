# LIVE block editing audit

Date: 2026-10-28. Base: `764c2b8`. Reference behaviour: Sarala (GPL-3.0, studied from README only; no code copied).

## Sarala-aligned behaviours

| Behaviour | Current (pre-LIVE) | Content model / editor pieces | Risks |
| --- | --- | --- | --- |
| One page, no split preview; blocks render until caret enters | **Partial**: WYSIWYG (ProseMirror) always rendered; Source is raw Monaco; Split is side-by-side | `documentEditorModes.ts`, `richTextProseMirror.ts`, `textEditor.ts` (Monaco), `documentEditorHost.ts` | Split mode contradicts single-surface goal; LIVE mode adds fifth surface on same model |
| Active block shows dimmed markdown markers; inactive blocks stay rendered | **Missing** (design note only in `document-editor-modes.md`) | LIVE: `markdownLiveEditor.ts` + `markdownLiveMapping.ts` on Monaco; markers via `deltaDecorations` | Decoration churn on cursor move; must not fight proofing decorations |
| Click rendered text maps to source offset (not block start) | **Missing** in Source/Split; WYSIWYG uses PM positions not MD bytes | `buildPlainRenderMap`, `sourceOffsetFromRenderClick`; view-zone click handlers | Nested emphasis/links; RTL bidi reordering |
| Smart Enter (lists, quotes, numbering, fence close); Shift+Enter soft break | **Partial**: Monaco default Enter only; PM `baseKeymap` in WYSIWYG | `applySmartEnter` in `markdownLiveMapping.ts`; LIVE keymap in `markdownLiveEditor.ts` | Undo must stay Monaco-native (one stack per LIVE session) |
| Broken math/diagram keeps last good render + error marker | **Missing** | Future: mod render cache + block widget (`LIVE-04`) | Stale cache vs model edits |
| IME-safe (no re-render during composition) | **Missing** in LIVE | `onDidCompositionStart/End` gate in `markdownLiveEditor.ts` | CJK + decoration refresh |

## Chunk 1 (this change)

| Block kinds | LIVE treatment |
| --- | --- |
| Paragraph, heading, emphasis, bullet/ordered list, blockquote | Render when inactive; source + dimmed markers when active |
| Table, code fence, math, diagram | Unchanged source lines (no LIVE render); tickets **LIVE-02** (code), **LIVE-03** (tables), **LIVE-04** (math/diagram) |

## Undo and history

| Path | Undo stack | Disk history |
| --- | --- | --- |
| WYSIWYG | ProseMirror `history` | Snapshot on commit via `commitModelToMarkdown` |
| Source / Split | Monaco | Merge banner + `documentSourceMerge.ts` |
| LIVE (new) | Monaco only | Same sync as Source through `onMonacoMarkdownContentChanged` / `syncDocumentModelFromMarkdown` |

Risk: mode switch WYSIWYG to LIVE must round-trip through model without splitting undo stacks (accept separate stacks per mode like Source vs WYSIWYG today).

## Probe tests (ignored)

| Ticket | Test location | Intent |
| --- | --- | --- |
| LIVE-02 | `crates/unibrowser-document-model/tests/live_block_probes.rs` | Code fence live render + smart Enter close |
| LIVE-03 | same | Table block click mapping |
| LIVE-04 | same | Math/diagram error retention |

## Performance

Long documents: view zones per inactive block; refresh debounced. Risk: O(blocks) DOM on 10k-line files; cap or virtualise in **LIVE-05**.

## i18n / RTL

Marker dimming is punctuation-only today. RTL paragraph direction not mirrored in render map (**LIVE-06**).
