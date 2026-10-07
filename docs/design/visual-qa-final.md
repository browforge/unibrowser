# Visual QA final matrix

Date: 2026-10-07 (S1-S3 closure pass)

## Method

The release application was rebuilt before capture. Each application image is
1280 x 800. Each labelled comparison places the application on the left and the
reference on the right at 1280 x 800 per half. Every PNG in
`docs/screenshots/vqa-final/` was opened and inspected.

Grades:

- **Pass**: the named structure and behaviour correspond to the reference.
- **Minor drift**: the structure is present, with spacing, density, typography,
  colour, or fixture differences.
- **Fail**: a required state is absent or reports false information.

## Matrix

| Screen | Theme | Chrome | Command Manager | Rail | Main work region | Right dock | Bottom / status | Overall |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Document + Problems + AI | Light | Minor drift | Pass | Pass | Pass | Minor drift | Pass | Minor drift |
| Document + Problems + AI | Dark | Minor drift | Pass | Pass | Pass | Minor drift | Pass | Minor drift |
| PDF annotation | Light | Minor drift | Pass | Pass | Pass | Minor drift | Minor drift | Minor drift |
| PDF annotation | Dark | Minor drift | Pass | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Image / SVG | Light | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Image / SVG | Dark | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Rich document | Light | Minor drift | Minor drift | Pass | Pass | Pass | Minor drift | Minor drift |
| Rich document | Dark | Minor drift | Minor drift | Pass | Pass | Pass | Minor drift | Minor drift |
| New tab | Light | Minor drift | Pass | Pass | Pass | Pass | Minor drift | Minor drift |
| New tab | Dark | Minor drift | Pass | Pass | Pass | Pass | Minor drift | Minor drift |
| Rail collapsed | Light | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Rail collapsed | Dark | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Rail expanded | Light | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Rail expanded | Dark | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Chrome collapsed | Light | Pass | Pass | Pass | Pass | Minor drift | Minor drift | Pass |
| Chrome collapsed | Dark | Pass | Pass | Pass | Pass | Minor drift | Minor drift | Pass |

### Before / after (S1-S3 tickets)

| Ticket | Before | After |
| --- | --- | --- |
| S1 PDF CM page | **Fail** (`1 / 1`) | **Pass** (`1 / 3` matches viewer) |
| S2 New tab data | Minor drift (no times, empty downloads) | Minor drift (seeded times + download rows; mockup still richer) |
| S2 Rich doc paper | Minor drift (short paper) | Pass (paper stretches with warm surround) |
| S2 SVG fixture | Minor drift (120x80) | Pass (960x540 fixture) |
| S3 Shell density | Minor drift | Minor drift (looser CM/status/title; not pixel-matched) |
| S3 Problems fixture | Minor drift (16 spell rows) | **Pass** (3 mixed-severity mockup rows) |

## Region findings

### Document with Problems and AI

- **Chrome / CM / rail:** stable two-row shell with Code commands visible.
- **Main:** Monaco paints with CM mounted; pixel gate passes.
- **Problems:** three rows (spell error, TypeScript warning, project warning).
  Mock seed is protected from editor proofing overwrite.
- **AI:** dock open with composer; narrower than reference.

### PDF annotation

- **Command Manager:** page control reads `1 / 3`, aligned with thumbnails,
  in-viewer nav, and status `Page 1 of 3`.
- **Main:** three-page fixture, fit-width paper, annotations and inspector.
- **Right:** AI closed.

### Image / SVG

- **Main:** large vector preview with inspector; source hidden by default.
- **Fixture:** 960 x 540 artwork fills the stage.

### Rich document

- **Main:** centred paper with warm surround; paper height follows the reader.
- **Right:** AI closed.

### New tab

- **Main:** Start actions, Recent with relative timestamps, Downloads with
  representative completed items when history was empty.
- **Right:** AI closed.

### Rail and chrome collapsed

- Unchanged from prior pass: 40 px collapsed rail, 188 px expanded rail,
  40 px title-only collapsed chrome with Monaco intact.

## Remaining work (out of S1-S3 scope)

- **CMD-04 / CMD-08 / CMD-09:** roving focus and screen-reader matrix, runtime
  mod provider, detached mod toolbars (see `docs/design/ui-implementation-plan.md`).
- **Polish:** New-tab download list still lacks an in-progress row matching the
  mockup; shell spacing is closer but not identical to Quiet Utility boards.

## Verdict

S1-S3 tickets are closed with evidence under `docs/screenshots/vqa-final/`.
No false-state Fail remains in the matrix. CMD partials were not started in
this run.
