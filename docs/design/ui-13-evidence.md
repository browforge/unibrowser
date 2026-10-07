# UI-13 evidence

Captured with `scripts/capture-ui-13.sh`. The script uses real theme, contrast,
and UI scale preferences, requires the requested full-window dimensions, and
rejects duplicate PNG hashes. Each image below was opened after the final
capture.

## Literal image descriptions

### `files-browser-preview-light-100.png`

Top: one `demo-pack` tab, folder path, Save, and an 80 px title plus Command
Manager stack. The Command Manager is labelled Browse and contains file
commands from New folder through Hidden files, with a final overflow button.

Left: the collapsed activity rail borders a pale Places dock containing recent
files and pinned destinations. Centre: quiet breadcrumbs, Name and Ascending
sort controls, hidden-files checkbox, filter, selection note, and a file table.
`sample.svg` is selected in a blue-grey row. Right: a joined Preview dock shows
the SVG, metadata, a wrapped full path, and Open. All three surfaces continue to
the status bar; the unused centre area is the remainder of the directory list.

Bottom: the status bar says `Commands: Browse (auto)` and shows the folder path.

### `files-browser-preview-dark-100.png`

The regions and content match the light image. Places, the file table, empty
list remainder, and Preview use joined dark workbench surfaces. File names use
body text; only the selected `sample.svg` row uses the selection treatment.
Metadata wraps inside Preview and Open remains visible.

### `files-browser-preview-hc-100.png`

The same Browse layout uses white content, dark chrome, black row dividers,
strong panel borders, and a yellow focus outline on the active tab. Selection
still has a row fill and bold label, so it is not communicated by colour alone.
No control or file label is cut.

### `files-browser-preview-light-150.png`

Chrome stays on one row while workspace type and controls use the real 150%
scale. Places is wider, the file table shows fewer rows, and Preview is wider.
The path wraps over two lines, Open remains visible, and the table and docks
reach the status bar. No label ends at a panel edge.

### `markdown-actions-light-100.png`

Top: the `sample.md` tab and Text Command Manager show the Markdown text family,
including Subscript, Proving chip, Superscript, Find, and Check. Below it,
Preview, Export HTML, and Print preview sit in one flat dark contextual strip,
not raised buttons.

Centre: real Monaco content fills the work area with line numbers, Markdown
text, link underlines, overview markers, and a vertical scrollbar. The status
bar shows the document path, Text context, word count, and text state.

### `settings-developer-collapsed-light-800.png`

Top: the Settings Command Manager contains Search settings, Keybindings, and
Diagnostics, and both the follow label and status bar say Settings. The lead
and filter remain fixed above a two-column layout.

Left: the category rail shows all categories through Developer, with Developer
selected by label, fill, and left border. Right: the scroll position begins at
the end of Reset local data, then shows Developer and Advanced collapsed,
followed by Keyboard shortcuts, Windows, About, Updates, and Open diagnostics.
The controls are flat, bordered Quiet Utility controls. A visible scrollbar
shows that this is a scrolled page, not clipped content.

### `settings-developer-expanded-light-800.png`

The same 800 px shell has Developer expanded. Its explanation, complete
22-mod inventory, and Demo download, Export layout, Import layout, and Capture
visible tab controls wrap without horizontal clipping. Advanced stays
collapsed. Windows, About, and the start of Updates remain below in the same
scrolling surface.

### `settings-developer-expanded-light-1600.png`

The full-width image shows the complete Settings shell, selected Developer
category, Reset local data, expanded Developer inventory and four actions,
Advanced, Keyboard shortcuts, Windows, About, Updates, and diagnostics. Content
uses a bounded reading width; the remaining right side is the settings canvas,
not another card or panel.

### `settings-developer-expanded-dark-1600.png`

The same sections use a dark category dock and dark content surface. Text,
muted explanations, flat controls, selected category, disclosure focus border,
and dividers remain distinct. No light native button faces remain.

### `settings-developer-expanded-hc-1600.png`

The same layout uses a white content surface, black control borders, strong
full-width section rules, a pale selected category, and a yellow active-tab
focus outline. These pixels are visibly distinct from standard light mode.

### `settings-developer-expanded-light-150.png`

Real 150% scale increases the rail, text, controls, and section spacing.
Developer inventory and all four actions remain complete. The content column
scrolls vertically; Updates continues below the viewport without horizontal
clipping.

## Grade against Quiet Utility and design rules

| Evidence | Result | Basis |
| --- | --- | --- |
| File browser light 100% | Pass | Browse-only CM, 80 px chrome, joined docks, body-colour names, quiet breadcrumbs and selection note, full-height surfaces. |
| File browser dark 100% | Pass | Joined dark workbench surfaces and readable unselected names. |
| File browser high contrast 100% | Pass | Distinct high-contrast borders and redundant focus and selection cues. |
| File browser light 150% | Pass | Real scaling, wrapped metadata, reachable Open, no clipped labels. |
| Markdown actions light 100% | Pass | Flat contextual action strip and visible Monaco pixel gate. |
| Developer collapsed light 800 | Pass | Settings-only CM, usable rail and scroll, quiet controls. |
| Developer expanded light 800 | Pass | Inventory and actions wrap; lower content remains vertically reachable. |
| Developer expanded light 1600 | Pass | Bounded content and complete section relationships. |
| Developer expanded dark 1600 | Pass | Real dark tokens with legible controls and dividers. |
| Developer expanded high contrast 1600 | Pass | Real high-contrast tokens, borders, and focus treatment. |
| Developer expanded light 150% | Pass | Real scale and vertical reachability without horizontal clipping. |

Totals: 11 Pass, 0 Drift, 0 Fail.
