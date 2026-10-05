# Onboarding

Unibrowser follows [ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md): onboarding may help, but must **never block** open, browse, or save.

## First-run tour

- A small **floating card** (not a modal) appears on first launch when `onboarding_tour_skipped_forever` is false in preferences.
- **Skip tour** (one click) sets `onboarding_tour_skipped_forever` and hides the card for all future launches, reinstalls, and profile resets on this device config dir.
- The tour does not capture pointer events outside the card; the rest of the window stays usable.

## Tip banner

- Optional strip under the header (`onboarding_dismissed` when dismissed).
- Dismiss is independent of the tour skip flag.

## Settings checklist

- Optional steps in **Settings** (palette, browse, open file, split, settings visit).
- **Hide checklist forever** sets `onboarding_checklist_skip_forever` and dismisses the tip banner.
- Checklist never covers the document surface.

## Preferences (disk)

| Key | Meaning |
| --- | --- |
| `onboarding_tour_skipped_forever` | First-run tour must not show again |
| `onboarding_dismissed` | Tip banner dismissed |
| `onboarding_checklist_skip_forever` | Settings checklist hidden |
| `onboarding_checklist_completed` | Step ids ticked by the user |

Stored in the usual `preferences.json` under the Unibrowser config directory.
