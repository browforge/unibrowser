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
Discover selected. The Command Manager context label reads Mods and the family
row shows Search, Install, Updates, Permissions, and Open VSX network. Browse
file operations are not shown. One recommended handler card appears above an
opt-in Open VSX search row. Six catalogue results appear as a two-column grid of
bordered cards. Each card shows a title, identifier, reason, source, licence
reminder, and permission reminder. Installed extensions begins below the result
grid. No result, heading, or control overlaps another region. No visible text
is clipped.

## Grades

| Capture | Grade | Reason |
| --- | --- | --- |
| Unknown file support | Pass | The current file remains present beside a bounded support dock; actions are readable and do not cover content. |
| Mod Portal Discover | Pass | Tabs, search, recommendation metadata, and catalogue cards form one readable page with no overlap or clipping. |

UI-17 grade: Pass.
