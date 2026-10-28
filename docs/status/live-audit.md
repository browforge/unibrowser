# Live block editing — audit

**Status (2026-10-28):** **LIVE-01 done (ProseMirror option a).** Markdown **Visual** shows dimmed syntax markers only in the block that contains the caret; all other blocks stay clean WYSIWYG on the shared rich-text model. Monaco remains for **Source** and **Split** only. Prior Monaco-overlay attempt stays reverted (see below).

## Approved design (option a)

- **No new mode.** Visual on `.md` is live editing via `Decoration.widget` spans (`md-live-marker`) on the existing ProseMirror session.
- **Active block only:** headings (`## `), emphasis/strong/code marks, lists, blockquotes, fenced code.
- **One undo history:** `prosemirror-history` plus input rules for `#` headings and `**` bold on the same document.
- **Round-trip:** serialize through `unibrowser-document-model` (unchanged); byte-stable fixtures in `round_trip.rs` / `mode_switch.rs`.
- **Proofing:** unchanged dictionary and grammar scoping (no fixture vocabulary).

## Verification shipped with LIVE-01

| Check | Location |
|-------|----------|
| Decoration / input / undo unit tests | `crates/unibrowser-desktop/tests/markdownLiveBlock.test.mjs` |
| Desktop captures 1280×800, AI closed | `scripts/capture-live-desktop.sh` → `docs/screenshots/live/live-cursor-*.png` |
| Pixel gates (double-ink, marker delta vs outside, heading band) | `scripts/check-live-capture-pixels.py` |

## Capture notes

- Doc-capture applies `live-cursor-*` tokens without remounting the ProseMirror session on every pass; paint readiness waits for `.md-live-marker` DOM matching the token (outside = none, heading = `## `, bold = `**`).
- Fixed `scrollTop` during doc-capture keeps the three PNGs aligned; pixel gates compare muted marker ink **relative to** the outside capture rather than absolute grey counts (body anti-aliasing also hits the muted tone band).

## What we learned (reverted Monaco overlay)

1. **Monaco overlay + hidden source does not replace the line.** Inactive blocks kept full-opacity Monaco tokens while HTML overlays drew on the same vertical band. Captures showed duplicated paragraphs and stacked markers — unreadable.
2. **Old pixel gates passed while UX failed.** Typography heuristics without **double-ink / overlap** checks or masked diffs were insufficient.
3. **Do not tune dictionaries to capture fixtures.** Expanding test `en-GB.dic` with words from capture samples masked spelling gaps.
4. **Product:** fewer modes, not a fifth “Live” mode. LIVE-01 folds into Visual only.

## Ignored probes (kept in tree)

| Ticket | File | Intent |
|--------|------|--------|
| LIVE-02 | `crates/unibrowser-document-model/tests/live_block_probes.rs` | Code fence live render + smart Enter close |
| LIVE-03 | same | Table click mapping |
| LIVE-04 | same | Math/diagram error retention |

Run with `cargo test -p unibrowser-document-model -- --ignored` when implementing.

## Spelling (separate track, not LIVE)

Markdown proofing uses **prose-only byte ranges** and the bundled hunspell list — not fixture vocabulary. Grammar stays enabled on prose ranges.

## References (historical, reverted SHAs)

- Chunk 1 (Monaco): `d7c9088709446f697dfd57279da63b99536637ec`
- Fix attempt: `2a7b31dc6db64bb5bab2243a4bc590c6ea0bf3b1`
- Reverts: see `git log` after `764c2b8`
