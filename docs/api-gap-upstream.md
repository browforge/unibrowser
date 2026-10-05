# API gap and upstream request process

When a mod author needs something the engine does not expose, do **not** patch private engine internals. File an upstream request so the mod API stays small and stable.

## Steps

1. Search existing issues for `api-request` and `unibrowser-mod-api`.
2. Open a new issue with:
   - **Mod:** `name` from `info.json`
   - **Scenario:** user-visible behaviour (one paragraph)
   - **Proposal:** trait method, context key, or host event
   - **Workaround today:** sidecar, fork, or disabled command
3. Maintainers tag with target milestone or `api-defer`.

## Version negotiation

| Host API | Mod `unibrowser_api` | Loader behaviour |
| --- | --- | --- |
| 0.1.x | 0.1.y | Load if mod minor ≤ host minor |
| 0.2.0 | 0.1.x | Reject (major bump) |
| 0.1.0 | 0.2.0 | Reject (mod requires newer host) |

## Current known gaps (skeleton)

| Gap | Workaround | Tracking |
| --- | --- | --- |
| Real WebView document surface | egui placeholder | ADR 0001, Phase 1 Tauri |
| WASM mod tier | T0 native only | ADR 0002 |
| Unified undo stack | Per-backend undo | Phase 2+ |
| `when` clause expressions | Rust enablement fns | Phase 1 |

## Capability flags (initial)

- `font_size` — increase/decrease font on text selection
- `save` — backend can persist document
- More added via api-request process only
