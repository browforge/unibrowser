# UI batch 3b (Composer)

Captures: `./scripts/capture-ui-batch-3b.sh` (rebuilds release via `ensure-release-desktop.sh`, renders mockups if needed).

## Files

| File | Description |
|------|-------------|
| `ui-document-mockup-light.png` / `-dark.png` | `sync-notes.ts`, Text CM pinned, Problems (3 seeded), AI docked |
| `ui-document-mockup-side-by-side-*.png` | App left, design mockup right, each 1280x800 |
| `ui-pdf-annotation-app.png` | `examples/demo-pack/sample.pdf` |
| `ui-pdf-annotation-side-by-side.png` | App vs `pdf-annotation.png` |
| `ui-multimodal-workshop-app.png` | Placeholder (editor-only); not the CAD workshop |
| `ui-multimodal-workshop-side-by-side.png` | Honest mismatch vs `multimodal-workshop-wide.png` |

## Remaining visual gaps (ranked)

### `document-problems-ai` (light/dark)

1. **Editor paint in headless capture** – Monaco often snapshots as a flat dark region under WebKitGTK/Xvfb; layout and paint-ready gates are in place but pixel parity with the mockup editor is not reliable in CI captures.
2. **Title bar** – Single tab and file URL omnibox vs mockup multi-tab strip, breadcrumb path, and primary Save.
3. **Command Manager labels** – Separate Increase/Decrease font commands vs mockup combined “14 pt” split control; Format remains a separate group (mockup nests Format under Type).
4. **AI sidebar** – Generic assistant chrome vs mockup provider line, grant chips, starter card, and prefilled composer question.
5. **Status bar** – Rich mockup status (saved time, problem count, AI mode) vs app diagnostics row.
6. **Problems panel chrome** – Filter row and table styling differ; seeded rows match messages but density/selection differ.

### `pdf-annotation`

1. **No PDF page thumbnails or annotation sidebar** – App opens PDF in viewer; mockup shows page rail, on-page highlights, and annotation list.
2. **Office/PDF command tab** – Sparse Office CM vs PDF Tools/Annotate/Highlight/Note/Area groups.
3. **Second tab** – Mockup shows `field-report.pdf` + `sync-notes.ts`; capture uses `sample.pdf` only.

### `multimodal-workshop-wide`

1. **Not implemented** – No CAD viewport, timeline, or multimodal workshop layout; capture is a text document only.
2. **Resolution** – Mockup is 1920x1080 workshop; side-by-side scales both to 1280x800 for comparison only.
