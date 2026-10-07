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
| New tab | The 80px chrome occupies one title row and one CM row. Start actions and recent items remain in the work area with no overlap. | Pass | The start content remains bounded while the surrounding work area grows; action controls do not stretch. | Pass |
| File browser | Places, the file table, and the preview/property dock remain separate. The CM shows complete Browse labels and a chevron for whole overflow groups. | Pass | The file table receives the added width; the navigation and property docks retain bounded widths. | Pass |
| Settings | The category rail remains readable. Field copy and controls occupy separate columns; selects and the 32x18 switch keep their intrinsic sizes. | Pass | The settings work area widens while the rail, selects, switch, and action link remain bounded. | Pass |
| Markdown | Editor, AI dock, and populated Problems panel remain separate. The CM is one row with complete labels and an overflow chevron. | Pass | The editor receives the added width; the AI and Problems regions keep their documented dock dimensions. | Pass |
| PDF | The thumbnail strip, page work area, and annotation dock remain distinct. Annotation fields are readable and the PDF page scrolls in the centre. | Pass | The page work area grows between unchanged thumbnail and annotation docks; form controls do not stretch. | Pass |
| Image | History and adjustment docks remain bounded around the image stage. The image is contained without a duplicate toolbar. | Pass | Added width goes to the image stage; both docks retain their bounded widths. | Pass |
| SVG | Preview and Source stay in one local mode row. The preview stage and inspector remain separate; CM labels are complete and overflow uses the chevron. | Pass | The preview stage grows while the inspector keeps its bounded width and form controls remain content-sized. | Pass |
| DXF | Layer panel, canvas, and geometry panel remain readable as three regions. CM overflow contains complete groups. | Pass | Added width goes to the canvas; both side panels retain their declared widths. | Pass |
| Workshop | Sources, stage, inspector, transport, and timeline remain separate. The 0s and 10s labels stay inside the ruler and the Intro marker does not cover 0s. | Pass | Added width goes to the stage and timeline; source and inspector docks remain bounded. | Pass |
| Mod portal | Tabs, search, suggestion copy, and the flat Install action remain readable. The result card does not overlap the rail or status bar. | Pass | The result card remains bounded rather than stretching across the full work area; Install stays next to the card content. | Pass |
| Audio | The waveform and transport occupy the work area below a single CM row. Mute remains visible without a duplicate media toolbar. | Pass | The waveform work area grows; transport controls keep their intrinsic dimensions. | Pass |
| Video | The black video stage remains contained with transport below it. Mute and picture-in-picture remain in one CM row. | Pass | Added width goes to the video stage; transport and CM controls do not stretch. | Pass |

Files are under `docs/screenshots/responsive-and-states/` as
`<surface>-light-1024.png` and `<surface>-light-1920.png`. Each PNG has a
matching `.cm.json` diagnostic file.

## Interaction-state sheets

| Evidence | Literal description | Grade |
| --- | --- | --- |
| `states-light.png` | Rows show Command Manager, document tab, activity rail, portal tab, input, button, Install, switch, select, and settings input. Columns show default, hover, keyboard focus, and pointer-down states. The CM focus cell uses the existing Text Find keyboard-focus path; the other CM cells use Mods Search. Hover changes border or surface without lift. Focus uses a 2px blue ring. Pressed uses the selected surface or accent border. | Pass |
| `states-dark.png` | The same production controls use dark surfaces with brighter borders or selected fills on hover and press. The blue 2px focus ring remains distinct from the dark chrome and panel backgrounds. Install remains flat in every state. | Pass |
| `states-dark-hc.png` | The same rows retain black surfaces and white boundaries. Keyboard focus is a high-contrast yellow ring, 2px minimum, and remains separate from selection. | Pass |
| `states-light-disabled.png` | An unsupported CM control remains in place, greyed but readable, with its capability reason tooltip. Turning Auto-save off leaves the switch off and disables the interval select without removing either control. | Pass |
| `states-dark-disabled.png` | Unsupported and disabled controls remain present and readable against dark surfaces; no opacity stack turns labels grey-on-grey. | Pass |
| `states-dark-hc-disabled.png` | Unsupported and disabled controls retain white boundaries and readable labels. The reason tooltip and disabled interval remain distinguishable from enabled controls. | Pass |

`cm-focus-light.png`, `cm-focus-dark.png`, and `cm-focus-dark-hc.png`
preserve the full Text surface for the CM focus cells. Find has real keyboard
focus and remains in the correct Text family.

Inputs do not claim a separate pressed mode: pointer down retains the hover
border and places the caret. Select pointer down may expose the native option
list. These are literal platform interactions, not decorative pressed
animation.

## Automatic guards

- `validateResponsiveShellForCapture` rejects compact chrome outside 80px
  (+/-2px), a compact CM row outside 40px (+/-1px), vertical CM overflow,
  partial groups, clipped labels, and missing overflow chevrons.
- `partitionGroupsByAvailableWidth` tests assert stable source order, complete
  groups, and that low-priority groups cannot displace normal-priority groups
  at 1024px.
- `validateFocusRingForCapture` rejects a focused production control without a
  visible outline, an outline below 2px, or ring contrast below 3:1.
- Unit tests cover dark, light, dark high-contrast, light high-contrast, and
  shared chrome focus-token pairs. Their measured ratios are 8.27:1, 5.89:1,
  16.65:1, 6.92:1, and 8.37:1 respectively.
- Existing guards also reject dark bevels, missing select chevrons, malformed
  switches, timeline escape/overlap, blank editors, and duplicate image chrome.

## Flagged issues and dispositions

| Flagged issue | Disposition |
| --- | --- |
| Unsupported CM controls explicitly removed their focus outline. | Fixed in the shared CM rule. Unsupported controls now keep an inset focus ring while remaining greyed and exposing the reason tooltip. |
| Document tabs and activity-rail destinations had no hover rule. | Fixed in `compactChrome.css` with flat selected-surface hover feedback. |
| The global focus rule used an unrelated `--accent` fallback. | Fixed with `--ub-focus` and `--ub-focus-on-chrome` tokens for every captured theme. |
| Switch focus used the action accent rather than the focus token. | Fixed in the shared switch rule. |
| Inputs and selects had no shared pointer-hover boundary change. | Fixed with a shared accent-border hover rule that does not move or raise the control. |
| General buttons, tabs, and button roles had no consistent pointer-down acknowledgement. | Fixed with a shared accent-border and selected-surface pressed rule; no transform, shadow, or animation was added. |
| Disabled form controls could inherit browser opacity and become grey-on-grey. | Fixed with opaque token-based disabled foreground, border, and well surface styles. |
| The first state-sheet pass sampled the settings switch before the surface finished painting and missed two portal targets. | Capture tooling fixed by waiting for the real surface and correcting pointer coordinates. No application behaviour was changed. |
| Capture cleanup blurred the intentional UI-19 keyboard target after its focus guard passed. | Fixed in the capture pipeline by preserving real focus only for the explicit focus-evidence path. No production style or layout changes by capture mode. |

## Verification

- `npm run test:contrast`
- `npm run test:overflow`
- `npm run build`
- `scripts/capture-responsive-shells.sh`
- `scripts/capture-interaction-states.sh`

The final responsive matrix and state-sheet scripts exited 0.
