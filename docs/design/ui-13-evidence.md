# UI-13 evidence

Captured with `scripts/capture-ui-13.sh`. The script uses real theme, contrast,
and UI scale preferences, requires the requested full-window dimensions, and
rejects duplicate PNG hashes.

## Result grid

| Evidence | Width and scale | Result | Notes |
| --- | --- | --- | --- |
| File browser, light | 1600, 100% | Pass | Tree, list, selected SVG preview, actions, CM, and status bar are visible. |
| File browser, dark | 1600, 100% | Pass | All three docked regions remain distinct without card nesting. |
| File browser, high contrast | 1600, 100% | Pass | High-contrast borders and focus treatment are distinct from light mode. |
| File browser, light | 1600, 150% | Pass | Complete labels remain visible; the list scrolls instead of clipping. |
| Developer collapsed, light | 800, 100% | Pass | Category rail, search, collapsed Developer and Advanced groups, and content scroll remain usable. |
| Developer expanded, light | 800, 100% | Pass | Inventory and all four Developer actions fit without horizontal clipping. |
| Developer expanded, light | 1600, 100% | Pass | Rail, setting rows, and wide content relationship remain clear. |
| Developer expanded, dark | 1600, 100% | Pass | Text, controls, dividers, and selected category remain legible. |
| Developer expanded, high contrast | 1600, 100% | Pass | White content, dark rail, stronger dividers, and selected category use distinct tokens. |
| Developer expanded, light | 1600, 150% | Pass | Scaled rail and setting controls remain reachable with no clipped labels. |

No Fail is present.

## Region notes

### File browser

Top chrome: the complete title bar and Command Manager are visible. Groups stay
on one row, and the bookmark bar labels retain contrast at 100% and 150%.

Left: Places contains recent files and pinned destinations. It has one joined
boundary with the file list, not an independent card.

Centre: breadcrumb, sort, search, Browse actions, selection actions, and the
file table form one scrollable region. File actions remain in Browse context.

Right: selecting `sample.svg` opens a docked Preview with the rendered asset,
type, size, modified time, extension, and full path. Metadata remains readable
in light and dark modes.

Bottom: status remains visible. At 150%, the file list continues below the
viewport by scrolling; no label is cut at a region edge.

### Settings

Top chrome: title bar, Command Manager, settings lead, and search remain
visible at both evidence widths.

Left: the category rail shows General, Appearance, Files and downloads,
Editing, Accessibility, Privacy and AI, Mods, and Developer. Developer has a
text label and selected surface, so selection does not depend on colour alone.

Centre: Developer is a native dismissible disclosure. Collapsed evidence shows
the closed group; expanded evidence shows mod inventory, demo download, layout
import and export, and capture diagnostics. Advanced remains separately
collapsed.

Lower content: keyboard shortcuts, window, About, updates, and diagnostics
remain in the same scrollable settings surface. No setting is wrapped in a
decorative card.

High contrast: black and white surfaces, selected-category fill, focus outline,
and stronger section borders differ visibly from standard light mode.

## Drift

None recorded for UI-13. The file browser uses existing blue file links and
compact native controls rather than introducing a second visual language.
