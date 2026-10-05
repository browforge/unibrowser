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

## Recovery precedence (launch)

On startup the shell may do **both** session restore (Rust, before the UI paints) and a **project recovery** prompt (frontend, after the first state snapshot).

| Kind | When | Precedence |
| --- | --- | --- |
| `.unibrow` `project.json` backup | Project autosave or crash recovery enabled; debounced backup exists | **Recovery wins** — session restore **skips** tabs for that project path so you are not double-opened. Accepting recovery restores `project.json` and opens or refreshes the matching tab. |
| Text / SVG / JS crash backups | Crash recovery enabled; debounced backup on disk | **Recovery wins** — session restore **skips** tabs for that file path (same dedupe as `.unibrow`). Use **Restore** on the backup row in Settings or About; restore reuses an open tab when the path is already open. Dismiss backups you do not need. |

Dismiss project recovery backups if you prefer the on-disk project without the autosaved `project.json`.

## Related

- [multi-window.md](multi-window.md)
- [onboarding.md](onboarding.md)
