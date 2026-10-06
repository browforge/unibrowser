# UI batch 3 screenshots

Compact chrome (`batch3-v1`): unified 40 px title bar, grouped command manager, activity rail.

## Capture

```bash
./scripts/capture-ui-batch-3.sh
```

Requires release binary (`scripts/ensure-release-desktop.sh`).

## Chrome metrics (runtime)

```javascript
window.__unibrowserMeasureChromeHeights?.()
```

Compact budget: title 40 px + command manager 40 px = 80 px total (audit allows 78-82 px).
Large density: 64 px command manager, 104 px total.
Collapsed command manager: 40 px title bar only.

## Files

| File | Ticket |
| --- | --- |
| `ui-04-07-document-compact.png` | UI-04, UI-07 title bar + tabs + omnibox |
| `ui-05-command-manager.png` | UI-05 grouped command manager |
| `ui-rail-expanded.png` | Activity rail 188 px |
| `ui-chrome-collapsed.png` | Ctrl+F1 collapse (40 px chrome) |
| `ui-chrome-scale-150.png` | 150% root font scaling check |
| `ui-document-mockup-side-by-side.png` | App vs design mockup |
