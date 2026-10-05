# ADR 0011: Lazy conversion — native until encode

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

Most specialised apps **reject** “wrong” media in a given context (PDF on a video timeline, MIDI in an NLE, DOCX in a photo editor). Unibrowser’s product direction is the opposite: **mix unusual formats in one project** and **convert only at the boundary** where the destination representation requires it.

Bear’s examples:

- **PDF on a timeline** — keep vector/text objects native in the project; rasterise only for **video encode** (or use a display-resolution proxy while scrubbing).
- **MIDI in a project** — keep MIDI events in the model; **audition on scrub** via a synth; **render to PCM** only when baking to an audio track or on final export.

The same rule applies elsewhere: **source of truth stays in the mod’s native model** until preview or export forces a transform.

## Decision

### Principles

1. **Native-first project model** — tabs and project graphs reference `BrowserLocation` + backend-native handles (paths, structured data, not “everything is PNG”.
2. **Two pipelines** — **Preview** (interactive, may downsample, proxy, or partial render) vs **Export** (full fidelity, deterministic, may be slow).
3. **Conversion at boundaries** — adapters produce derived artifacts (proxy bitmap, rendered WAV) with provenance (`derived_from`, `purpose: preview | export`).
4. **Plugin responsibility** — format mods declare what they can **hold**, **preview**, and **encode to**; the host orchestrates jobs and cache, not one global transcoder god-object.
5. **No silent flattening** — flattening (e.g. PDF → full-page raster for edit) is an explicit user or export step, not the default open path.

### Host responsibilities (engine / shell)

| Concern | Owner |
|--------|--------|
| Command + tab chrome | Host |
| Native document state | Mod backend |
| Preview proxy cache | Host job queue + mod callbacks |
| Export graph | Project mod (future) + host scheduler |
| Licence of encoders (ffmpeg, etc.) | Sidecar / optional mod, labelled in manifest |

### Mod API direction (not all implemented)

- `PreviewSurface` — what the UI shows at interaction time (may be lower res).
- `ExportContribution` — what this clip/layer needs at encode time.
- `ConversionEdge` — typed transform (e.g. `Midi → AudioPcm`, `PdfPage → Raster`) with cache key.

Phase 1 does **not** implement timelines or PDF-on-timeline; this ADR locks the **design contract** for when those mods land.

## Consequences

- Roadmap phases for **media** and **office** must not assume “import = permanent rasterise”.
- CI and docs should describe preview vs export for each new format mod.
- AGPL/LGPL encoders stay out of the Apache core; sidecars produce derived blobs the host references.

## References (product research)

- Video NLE “wrong format” behaviour (contrast): most reject non-audio/video on timeline — Unibrowser explicitly diverges.
- Proxy editing in professional video (offline proxies vs full-res export) — analogue for PDF/MIDI preview.

## Open questions

- Single global **asset cache** vs per-project cache directory?
- Undo across conversion edges (revert proxy regen when source PDF edits)?
- Maximum proxy resolution defaults per platform (GPU memory)?
