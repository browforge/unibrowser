# UI batch 3c

Regenerate: `./scripts/capture-ui-batch-3c.sh` (uses `WEBKIT_DISABLE_DMABUF_RENDERER=1` and `WEBKIT_DISABLE_COMPOSITING_MODE=1`).

## Black editor root cause (batch 3c)

1. **Layout:** `#text-editor-host` was `position: absolute` inside a flex column without height, so Monaco often laid out to a single line while the rest read as empty canvas.
2. **Capture path:** Doc PNG selection preferred an **X11 root composite** that did not include WebKit’s painted Monaco layer; capture now **prefers the shell WebKitGTK snapshot**.
3. **Paint checks:** DOM `.view-lines` text is the source of truth; GPU canvas readback is often **0×0** under WebKitGTK, so variance checks only apply when canvas samples are non-empty.

## Assertions at capture

- `data-theme` and computed `--ub-base` for light demos (`#f1f0ec`).
- Monaco DOM diagnostics logged as JSON before finish.
- Abort if view-lines are missing/short or shell chrome audit fails.

## Remaining visual gaps (ranked)

1. Second tab (`field-report.pdf`) not in capture yet (opening a second tab breaks paint wait until tab focus is wired through `applyState`).
2. Title breadcrumb vs raw URL until post-demo `refreshState` (fixed in app; re-capture to verify).
3. CM label density vs mockup (“14 pt” split vs long titles on some builds).
4. AI sidebar copy/provider strings vs mockup art.
5. Problems panel still shows a stale “No problems” line above the table in some builds (empty-state sync).
