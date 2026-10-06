# UI batch 3 screenshots

Compact chrome (batch3-v1): unified 40px title bar, grouped command manager, activity rail.

## Capture with doc capture

From a built desktop binary with demo state:

```bash
cd crates/unibrowser-desktop
npm run build
cargo build -p unibrowser-desktop
UB_DOC_CAPTURE=docs/screenshots/ui-batch-3/compact-1280.png \
  ./target/debug/unibrowser-desktop
```

## Measure chrome heights in DevTools

With the app loaded:

```javascript
window.__unibrowserMeasureChromeHeights?.()
window.__unibrowserRunShellChromeAudit?.()
```

Expected compact totals: title 40px, command manager 40px, total 80px (audit allows 78-82px).

## Manual checklist

- Mixed tabs at 1280 with Text context and collapsed rail
- Folder tab showing back/forward/reload in title bar
- Expanded rail (188px) and auto-hide edge reveal
- Command manager collapsed (Ctrl+F1) showing 40px chrome only
