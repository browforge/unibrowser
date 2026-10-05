# Session restore

When **Restore tabs on launch** is enabled, Unibrowser saves open tabs (and extra windows) on exit and reloads them on the next start.

## Preference

| Setting | Location | Default |
| --- | --- | --- |
| `restore_session_on_launch` | Settings → Session → “Restore tabs on launch” | **On** for new profiles |
| Off after “grandma” onboarding | Skip first-run tour forever, or hide checklist forever | **Off** (avoids surprising reopen) |

Power users can turn it on anytime in Settings. The recent-files panel links to Settings for discoverability.

## What is saved

- Per-window tab omnibox strings (`file://`, `https://`, `unibrowser://`)
- Active tab index per window
- Window size/position when the window closes
- Snapshot file: `session.json` under the Unibrowser config dir (see [getting-started.md](getting-started.md))

Format version **2** stores a `windows[]` array (`main`, `browser-2`, …). Older v1 snapshots with only `tab_inputs` are migrated on load.

## Multi-window

Each additional browser window (`browser-N`) is restored after `main` when a snapshot lists it. New windows created from Settings use the same session capture on close.

## Related

- [multi-window.md](multi-window.md)
- [onboarding.md](onboarding.md)
