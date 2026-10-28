# Live block editing — audit (paused)

**Status (2026-10-28):** Work on `d7c9088` / `2a7b31d` was **reverted** on `main`. Markdown **Visual** again uses the pre-LIVE behaviour (ProseMirror rich-text for document-card markdown, not Monaco live overlays). No live desktop captures or pixel gates ship on `main` until a redesign is approved.

## What we learned (reverted attempt)

1. **Monaco overlay + hidden source does not replace the line.** Inactive blocks kept full-opacity (or partially visible) Monaco tokens while HTML overlays drew on the same vertical band. Captures showed `# Live…` under `Live block capture sample`, duplicated paragraphs, `**bold**` over **bold**, quote lines stacked on fences — unreadable. Root cause: `opacity`/`visibility` on decoration classes does not reliably suppress Monaco’s painted glyphs; overlays were scroll-synced siblings, not a single surface.

2. **Pixel gates passed while UX failed.** `check-live-capture-pixels.py` measured ink height, grey marker tone, and squiggle colour counts. It never asserted “one text line, one visual row” or absence of double ink. Any future gate must include **double-ink / overlap** checks or screenshot diff against a golden mask, not typography heuristics alone.

3. **Do not tune dictionaries to capture fixtures.** Expanding the ~21-line test `en-GB.dic` with words from `live-block-sample.md` (`Live`, `caret`, `Quote`, …) made Problems count hit zero without fixing real spelling. Grammar was also disabled for `surface=markdown`, which hid grammar noise instead of scoping checks to prose.

4. **Product:** Bear asked for **fewer modes**, not a fifth “Live” mode. Folding live into Visual without solving (1) still broke Visual for every markdown user.

## Ignored probes (kept in tree)

| Ticket | File | Intent |
|--------|------|--------|
| LIVE-02 | `crates/unibrowser-document-model/tests/live_block_probes.rs` | Code fence live render + smart Enter close |
| LIVE-03 | same | Table click mapping |
| LIVE-04 | same | Math/diagram error retention |

Run with `cargo test -p unibrowser-document-model -- --ignored` when implementing.

## Spelling (separate track, not LIVE)

Markdown proofing should use **prose-only byte ranges** (`markdown_prose.rs`: skip fences, strip `#`/list/`>` prefixes, exclude URLs/code spans where applicable) and a **real** bundled hunspell list — not fixture vocabulary. Grammar stays enabled on those prose ranges.

---

## Design choice for Bear (no implementation until decided)

**Goal:** Typora / Obsidian Live Preview feel — rendered block until caret enters; one undo history; markdown Visual must not ship double-painted UI.

### (a) ProseMirror / shared rich-text model (recommended)

- **Idea:** Markdown Visual stays on the **document model + ProseMirror** surface used for Word-shaped content. Each block node stores markdown source (or round-trips through existing parse/serialize). While the selection is **inside** a block, show **decorations** (dimmed `#`, `**`, list markers); when the selection is **outside**, render WYSIWYG for that block only (Milkdown/Typora-style “node view” or mark-hiding decorations).
- **Fits:** ADR “one rich-text model, format is only how it’s saved”; mode switches don’t fork buffers; undo is ProseMirror’s stack (already separate from Monaco Source — mode switch policy still needed, but no overlay sync).
- **Risks:** Mapping click-in-rendered → source offset must stay in sync with `markdownLiveMapping`-style tests; tables/math/fences need explicit node types (LIVE-02..04); performance on huge files via node views / virtualization.

### (b) Monaco overlay on the raw `.md` buffer (not recommended)

- **Idea:** Keep one Monaco model for Source/Split/Visual; paint HTML in a layer; hide source lines with decorations.
- **Why it failed:** Monaco always paints the token layer; decoration CSS cannot guarantee zero ink; scroll/layout coupling duplicated lines; capture proved **systematic overprint**, not a one-line CSS fix.
- **Risks if retried:** IME, proofing squiggles, bidi (LIVE-06), accessibility, and pixel CI would all fight the dual layer; would still contradict “Visual = rich surface” unless markdown abandons ProseMirror entirely.

**Recommendation:** **(a)** — implement live-block as **rich-text block decorations / node views** on the existing markdown document session, and keep Monaco for **Source** and **Split** only. Reuse mapping unit-test ideas from the reverted branch as pure logic, not as `markdownLiveEditor.ts`.

**Open product questions for Bear**

- Should Visual on `.md` remain ProseMirror-only (current `main`) until (a) lands, with no Monaco in Visual?
- Should Split preview stay read-only rich while Source edits Monaco (today), or share one model via sync?

---

## References (historical, reverted SHAs)

- Chunk 1: `d7c9088709446f697dfd57279da63b99536637ec`
- Fix attempt: `2a7b31dc6db64bb5bab2243a4bc590c6ea0bf3b1`
- Reverts: see `git log` after `764c2b8`
