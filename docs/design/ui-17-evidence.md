# UI-17 evidence

Build and capture:

- `scripts/capture-ui-17.sh`
- The script ran `npm run build` before capture.
- Both PNG files are 1280x800.
- The two PNG files have distinct SHA-256 hashes.

## Literal descriptions

### `unknown-file-support-light-1280.png`

The light-theme window has an active `unknown.ubunknown` file tab. The main
file surface remains visible and empty because no handler is installed. A
300 px support dock is joined to its right edge. The dock contains the heading
`Open support`, an unsupported-extension explanation, `Open as text`, `Open as
hex`, one installable-mod suggestion, and `Browse mods`. The dock and file
surface do not overlap. All controls are within the dock boundary.

### `mod-portal-discover-light-1280.png`

The light-theme Mod portal shows Installed, Discover, and Updates tabs with
Discover selected. The active tab title reads Mod portal. The Command Manager
context label reads Mods with Search, Install, Updates, Permissions, and Open VSX
network (distinct refresh, shield, and globe icons). Browse file operations are
not shown. Open VSX network state is shown as status text tied to the toolbar
toggle; there is no duplicate checkbox. Suggested and catalogue cards show a
state badge, metadata, and a quiet Install or Open action per card. Six
catalogue results fill the content width in a responsive grid. Installed
extensions begins below the grid. No region overlap or clipped control text.

## Grades

| Capture | Grade | Reason |
| --- | --- | --- |
| Unknown file support | Pass | The current file remains present beside a bounded support dock; actions are readable and do not cover content. |
| Mod Portal Discover | Pass | Tabs, search, recommendation metadata, and catalogue cards form one readable page with no overlap or clipping. |

UI-17 grade: Pass.
