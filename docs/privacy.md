# Privacy

Unibrowser is designed for local-first work. This document describes what leaves your machine.

## No telemetry

This project ships **no analytics SDKs** (no Google Analytics, Segment, Sentry crash reporting, or similar). The application does **not** send usage events, crash reports, or profiling data to Unibrowser contributors or third parties.

What you do in the app stays on your device unless **you** trigger network access (for example opening an `https://` page, downloading a file, or searching Open VSX when you opt in).

## Data on your device

| Data | Location | Notes |
| --- | --- | --- |
| Preferences | Platform config dir (`preferences.json`) | Theme, autosave, safe mode, workspace root, etc. |
| Keybindings / toolbars | Same config dir | Plain JSON |
| Session tabs | Optional session snapshot on exit | When restore is enabled |
| Notification history | In-memory only (last 20) | Cleared when you quit; not written to disk |
| Diagnostic error ring | In-memory for `unibrowser://diagnostics` | Support debugging; not uploaded |
| Panic crash logs | `crash-logs/` under config dir | Written on panic only; local; never uploaded |
| VSIX extensions | Config dir when installed | Only if you install extensions |
| User mods | `mods/` under config | Only if you sideload mods |

## Network you control

- **Web tabs** load remote content like any browser.
- **Open VSX** search runs only when you enable network in the mod portal and search.
- **Downloads** follow your download preference (in-app or system browser).

There is no background sync, account system, or cloud backup in the core product.

## Safe mode and mods

Safe mode (`UNIBROWSER_SAFE_MODE=1` or Settings → disable user mods) stops loading **user** native mods. Bundled in-tree mods still register. This does not change the no-telemetry policy.

## Questions

For FOSS contributors, see [CONTRIBUTING.md](CONTRIBUTING.md). For product goals, see [vision.md](vision.md) and [ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md).
