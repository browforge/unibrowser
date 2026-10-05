# Tab sleep / discard (stub)

## Current behaviour (blitz)

- Background **https** tabs with no activity for **10 minutes** show a **sleeping** style on the tab strip (italic label, `z` hint, lower opacity).
- **No webview is destroyed** and no navigation occurs; this is UI-only state.
- Switching back to a tab clears sleep styling immediately (`touchTabActivity`).
- Idle tracking is per-window in the Vite shell (`tabSleep.ts`), not persisted.

## Not implemented

- Suspending JS timers, tearing down WebKit/WebView2 surfaces, or serialising page state.
- Per-tab memory metrics or automatic reload on wake.

## Memory plan (post-blitz)

| Phase | Goal |
| --- | --- |
| 1 (now) | Visual sleep + idle timestamps; diagnostics line for open https tab count |
| 2 | Optional **discard** after longer idle: navigate content webview to `about:blank` while keeping tab URL in session; restore on focus |
| 3 | Platform hooks: WebKitGTK memory pressure, WebView2 `TrySuspend` where available |
| 4 | Policy prefs: never sleep pinned tabs; shorter idle on low-RSS hosts |

Pinned tabs and the active tab are never marked sleeping.
