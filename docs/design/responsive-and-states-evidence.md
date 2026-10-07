# Responsive and interaction-state evidence

## Capture method

All application captures use a fresh isolated `XDG_CONFIG_HOME`, the production
release binary, 100% UI scale, and the real light, dark, or dark high-contrast
theme. Responsive captures use the normal document-capture path. State sheets
use real pointer movement, pointer down, and keyboard traversal against the
production shell. No state class, capture stylesheet, or capture-specific
layout is applied.

The compact top chrome rule is 40px title chrome plus a 40px Command Manager
row. Docks retain their declared fixed or bounded width at 1024px. The centre
work area takes the remainder. At 1920px the same dock bounds remain and the
centre work area grows. Cards and controls keep their content-sized or bounded
width.

## Responsive captures

Descriptions record what is visible in the PNG before assigning a grade.

| Surface | 1024x700 literal description | Grade | 1920x1080 literal description | Grade |
| --- | --- | --- | --- | --- |
| New tab | 80px chrome: title row plus one CM row showing Recent, File, Folder, Project. Start card has Open file, Open folder, Start a new project, a recent list, and a download row. Nothing overlaps the rail or status bar. | Pass | The same Start actions stay content-sized. Extra width is empty work area around the card, not stretched buttons. | Pass |
| File browser | Places rail, file table, and Preview dock stay separate. CM shows New folder, New text file, Open, Copy paths, then a chevron. sample.svg is selected; the preview shows the North Bay mark. | Pass | The table widens. Places and Preview keep their bounded widths. CM prefix grows to Copy to, Move to, Rename, Compress, Properties, Delete. | Pass |
| Settings | Category rail lists General through Developer with a Search settings field. General shows On startup, Downloads, Workspace, Auto-save (on, Every 5 minutes), and Open advanced settings. Selects and the 32x18 switch keep intrinsic size. | Pass | The settings column widens. Rail, selects, switch, and the advanced link stay bounded. | Pass |
| Markdown | Editor shows Demo markdown. CM is Undo, Redo, Font, 12 pt, plus a chevron. AI Assistant dock is on the right. Problems 3 is open at the bottom with three rows. | Pass | The editor takes the extra width. AI and Problems keep their dock sizes. CM prefix adds Bold through Export. | Pass |
| PDF | Thumbnails 1-3, the Field report page, and the annotation dock stay distinct. CM is Highlight, Underline, Strike, Select, Hand, plus a chevron. Undo, Redo, and Export PDF are not on the narrow row. | Pass | The page widens between the same thumbnail and annotation docks. CM prefix keeps Highlight, Underline, Strike, Select, Hand, then Note, Area, Ink, Text box, Stamp, 1 / 3, 238%, Previous annotation, Next annotation. | Pass |
| Image | History dock, checkerboard stage with the North Bay colour study, and Adjust dock stay separate. CM is Undo, Redo, Fit, 100%, Zoom out, Zoom in, plus a chevron. No second image toolbar. | Pass | The stage grows. Both docks keep their widths. CM prefix adds Crop through Curves. | Pass |
| SVG | Preview / Source local mode row sits above the North Bay vector. Inspector has Fill, Stroke, Stroke width. CM matches Image: Undo through Zoom in plus a chevron. | Pass | The preview stage grows. The inspector stays bounded. Form fields stay content-sized. | Pass |
| DXF | Layer panel, canvas, and geometry panel remain three regions. CM is Select, Orientation, Pan, Orbit, Fit view, plus a chevron. | Pass | Extra width goes to the canvas. Side panels keep their widths. CM prefix adds Zoom out through Export STEP. | Pass |
| Workshop | Sources, stage (Unibrowser title card), inspector, transport, and timeline stay separate. CM is Add asset, Add title, Split, Delete clip, plus a chevron. Ruler shows 0s through 10s. 10s sits inside the ruler, left of the scrollbar. Intro sits to the left of 0s and does not cover it. | Pass | Stage and timeline grow. Source and inspector docks stay bounded. CM prefix is Add asset, Add title, Split, Delete clip, Snap, Marker, Timeline zoom, Preview, LIVE, Record, Export. 0s and 10s remain inside the ruler. | Pass |
| Mod portal | Installed / Discover / Updates, Open VSX search, Search, the network-off note, and a flat Install on the CAD file support card all stay readable. The card does not cover the rail or status bar. | Pass | The suggested card stays content-sized. Install stays on the card, not stretched across the work area. | Pass |
| Audio | Waveform and transport sit under one CM row. Mute is the only visible media command. | Pass | The waveform widens. Mute keeps its size. | Pass |
| Video | Black video stage with transport below. CM shows Mute and Picture-in-picture in one row. | Pass | The stage widens. Transport and CM controls do not stretch. | Pass |

Files are under `docs/screenshots/responsive-and-states/` as
`<surface>-light-1024.png` and `<surface>-light-1920.png`. Each PNG has a
matching `.cm.json` diagnostic file. `scripts/check-cm-responsive-prefix.py`
passed for all 12 contexts: every 1024 visible control list is a prefix of
the 1920 list.

## Interaction-state sheets

| Evidence | Literal description | Grade |
| --- | --- | --- |
| `states-light.png` | Ten rows, four columns: default, hover, keyboard focus, pointer-down. Command Manager default/hover/pressed are Mods Search/Install on the portal surface. The focus cell is the Text Undo control with a 2px light-blue inset ring and an Undo tooltip over Demo markdown. Document tab focus is the selected Mod portal title tab with a 2px chrome-blue ring. Activity rail focus is the files destination with a 2px blue ring. Portal tab focus is Updates with a 2px blue box. Open VSX search focus is a 2px blue ring on the field (the same token as Settings search). Search focus is a 2px blue ring around the Search button. Install focus is a 2px blue ring around the flat Install control. Switch, interval select, and Settings search each show the same 2px blue ring. Hover changes border or fill without lift. Pressed uses the selected surface or a darker Install fill. | Pass |
| `states-dark.png` | The same ten controls on dark surfaces. Hover and press lighten the selected fill or border. Keyboard focus is a 2px cyan-blue ring on chrome and panel backgrounds: Undo, Mod portal tab, rail icon, Updates, Open VSX search, Search, Install, switch, select, and Settings search. Install stays flat in every state. | Pass |
| `states-dark-hc.png` | Black surfaces and white boundaries. Keyboard focus is a 2px-minimum yellow ring on Undo, Mod portal tab, rail, Updates, Open VSX search, Search, Install, switch, select, and Settings search. The yellow ring stays separate from the white selected border. | Pass |
| `states-light-disabled.png` | SVG Image CM: Fit stays in place. Hovering it shows the tooltip "Pixel zoom needs an SVG view provider." Turning Auto-save off leaves the switch off and the Every 5 minutes select present but disabled. | Pass |
| `states-dark-disabled.png` | The same Fit tooltip and disabled interval select on dark surfaces. Labels stay readable. No grey-on-grey stack. | Pass |
| `states-dark-hc-disabled.png` | Fit tooltip and disabled interval select keep white boundaries and readable labels on black. | Pass |

`cm-focus-light.png`, `cm-focus-dark.png`, and `cm-focus-dark-hc.png`
are the full Text surface used for the CM focus cells. Undo has real
keyboard focus on the visible Text prefix (Undo, Redo, Font, 12 pt). Find
is in overflow at this width, so the focus target is the first visible
command, not an overflowed Find control.

Inputs do not claim a separate pressed mode: pointer down retains the hover
border and places the caret. Select pointer down may expose the native option
list (Every 1 minute is visible under the focused select). These are literal
platform interactions, not decorative pressed animation.

## Automatic guards

- `validateResponsiveShellForCapture` rejects compact chrome outside 80px
  (+/-2px), a compact CM row outside 40px (+/-1px), vertical CM overflow,
  partial groups, clipped labels, and missing overflow chevrons.
- `partitionGroupsByAvailableWidth` keeps a declared-priority prefix. A
  narrower width can only drop trailing complete groups. It cannot substitute
  a later smaller group. `scripts/check-cm-responsive-prefix.py` checks that
  invariant on the captured `.cm.json` files.
- PDF markup (Highlight, Underline, Strike) is Critical. History and Export
  are Low on PDF, so they overflow before markup. The unit test
  `declared priority order keeps core PDF markup ahead of history and export`
  locks that order.
- `timelineEndpointClearsScrollbar` rejects a last ruler tick that enters the
  12px scrollbar inset. The workshop ruler uses `right: 14px`. The capture
  assert runs at every captured width, not only 1280.
- `validateFocusRingForCapture` rejects a focused production control without a
  visible outline, an outline below 2px, or ring contrast below 3:1.
- `scripts/check-state-focus-rings.py` measures approved focus-token pixels in
  each of the ten sampled focus cells (AA +/-14). Light and dark accept
  `#176b94` / `#83c7e6`. Dark high-contrast accepts `#ffe566`.
- Unit tests cover dark, light, dark high-contrast, light high-contrast, and
  shared chrome focus-token pairs. Their measured ratios are 8.27:1, 5.89:1,
  16.65:1, 6.92:1, and 8.37:1 respectively.
- Editor capture readiness is painted `.view-lines` text inside
  `#text-editor-host`, not a height ratio and not an overflowed Find control.
- Existing guards also reject dark bevels, missing select chevrons, malformed
  switches, timeline escape/overlap, blank editors, and duplicate image chrome.

## Reviewed Fails

| Fail | Disposition |
| --- | --- |
| Workshop CM at 1024 swapped in different commands than 1280/1920. | Fixed. Overflow is a declared-priority prefix. 1024 is Add asset, Add title, Split, Delete clip, which is the prefix of 1920. |
| PDF 1024 hid Highlight/Underline/Strike while Undo/Redo/Export stayed. | Fixed. Markup is Critical. History and Export are Low on PDF. 1024 keeps Highlight, Underline, Strike, Select, Hand. |
| Workshop 1024 "10s" clipped by the scrollbar; the ruler guard only ran at 1280. | Fixed. Ruler inset is 14px. The endpoint guard runs at every captured width. 10s is inside the ruler at 1024 and 1920. |
| Open VSX search had no 2px >=3:1 focus ring, while Settings search did. | Fixed on the shared input / `.ub-form-control` / `.mod-portal-query` `:focus-visible` rule. The Input/focus cell now shows the same 2px ring as Settings search in light, dark, and dark-hc. |

## Verification

- `npm run test:overflow`
- `scripts/check-cm-responsive-prefix.py docs/screenshots/responsive-and-states`
- `scripts/check-state-focus-rings.py` (invoked by the state-sheet script)
- `scripts/capture-responsive-shells.sh`
- `scripts/capture-interaction-states.sh`

The responsive prefix check passed for 12 contexts. The state-sheet focus-ring
check passed for 10 sampled controls on light, dark, and dark-hc.
