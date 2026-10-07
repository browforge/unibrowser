# UI-16 evidence

Captured with `scripts/capture-ui-16.sh` after `npm run build` and release
desktop rebuild. Three 1600x1000 PNGs on `examples/demo-pack/sample.unibrow`
with shared compact chrome and title card selected. Studio header states differ
by demo token. Dimension and duplicate-hash checks pass.

## Literal image descriptions

### `unibrow-live-preview-light-1600.png`

Live preview disclosure collapsed at the top of the Create workspace. Board,
Sources, Inspector, and Timeline match UI-15. No red LIVE banner visible.

### `unibrow-live-on-light-1600.png`

Live preview section expanded. Studio header reads **LIVE** with red left border;
Live mode checkbox on; red LIVE badge beside controls; Start recording enabled.

### `unibrow-live-recording-light-1600.png`

Same expanded live section. Studio header reads **RECORDING** on dark red fill;
Live mode on; LIVE badge visible; recording controls reflect active capture
(Start recording disabled).

## Grade

| State | Result | Basis |
| --- | --- | --- |
| Preview | Pass | Preview header distinct from broadcast states; live section collapsed. |
| LIVE | Pass | Red semantic LIVE header and badge; distinguishable from Preview. |
| Recording | Pass | Recording header uses stronger red fill; distinct from LIVE-only state. |

Totals: 3 Pass, 0 Drift, 0 Fail.

## Notes

Mixer, source health, and status-bar redundancy from the full UI-16 spec remain
follow-up work. Demo tokens sync visible studio header when backend live APIs are
unavailable in safe capture mode.
