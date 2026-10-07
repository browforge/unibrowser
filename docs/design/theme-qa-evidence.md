# Theme QA evidence

Independent visual review of the final isolated-config dark and high-contrast
captures. Each row gives the literal visible result before the grade.

## Grading rules

A capture fails for overlap, clipping, unreadable or grey-on-grey text, the
wrong Command Manager family, duplicated controls, developer text, raised or
90s-style controls, glassy controls, inconsistent theme surfaces, or a large
dead gap inside a card.

## Shell density

| Capture | Literal description | Grade |
| --- | --- | --- |
| `file-browser-dark-1280.png` | Dark Browse shell with Places, a dense file table, and SVG preview and properties rails. Headings, sort labels, property labels, the selected row, and the flat Open action are readable. Breadcrumb separators have equal space on both sides and Save is an accent button. | Pass |
| `file-browser-light-hc-1280.png` | White high-contrast Browse shell with strong table rules and dark text. The selected SVG row, preview metadata, address, and flat Open action are distinct. | Pass |
| `file-browser-dark-hc-1280.png` | Black high-contrast Browse shell with bright table text and rules. The selected SVG row, preview metadata, and path remain readable against their panels. | Pass |
| `image-dark-1280.png` | Dark Image shell with History and Adjust rails. A complete fitted North Bay raster sits on a checkerboard canvas. Title-bar Undo and Redo are absent; the Image Command Manager retains those commands. | Pass |
| `image-light-hc-1280.png` | Light high-contrast Image shell with white side rails and a fitted colour raster. History and Adjust instructions are dark and readable. | Pass |
| `image-dark-hc-1280.png` | Dark high-contrast Image shell with outlined rails and the complete fitted raster. Rail copy, image status, and toolbar controls are visible. | Pass |
| `markdown-dark-1280.png` | Dark Text shell with Markdown source, AI rail, and a three-row Problems panel. Editor text, assistant copy, filters, and problem rows do not collide; disabled Send is readable and visibly unavailable. | Pass |
| `markdown-light-hc-1280.png` | White high-contrast Text shell with outlined AI composer and Problems controls. Source text, messages, disabled Send, and diagnostics remain legible. | Pass |
| `markdown-dark-hc-1280.png` | Black high-contrast Text shell with white panel boundaries, Markdown source, AI rail, and Problems rows. The disabled Send state remains visible without claiming an available action. | Pass |
| `mod-portal-dark-1280.png` | Dark Mods shell with search controls, offline notice, and one CAD support card. The CAD tile, adjacent left-aligned metadata, free middle space, and flat compact Install action form one row. | Pass |
| `mod-portal-light-hc-1280.png` | White high-contrast Mods shell with a cream CAD card. Its tile and text start together at the left while the flat Install action stays at the right. | Pass |
| `mod-portal-dark-hc-1280.png` | Black high-contrast Mods shell with a thin outlined CAD card. Card copy, licence metadata, contribution link, and compact Install action are readable. | Pass |
| `new-tab-dark-1280.png` | Dark Start shell with primary Open file, secondary start actions, recent files, and two download rows. Actions are flat and the content forms one compact column. | Pass |
| `new-tab-light-hc-1280.png` | Cream high-contrast Start shell with a bounded recent-work area. Primary, secondary, recent, and download actions are distinct and do not overlap. | Pass |
| `new-tab-dark-hc-1280.png` | Black high-contrast Start shell with bright copy, recent rows, download progress, and a compact Open action. No control is clipped. | Pass |
| `pdf-dark-1280.png` | Dark PDF Annotation shell around a white document page. The Page 1 chip, Label input, Colour swatch, Reply textarea, and other-annotation card use dark panel surfaces with flat 1px borders and no inset bevel. Breadcrumbs are evenly spaced and Save is a flat accent button. | Pass |
| `pdf-light-hc-1280.png` | White high-contrast PDF Annotation shell with outlined thumbnails and annotation card. Page content, reply placeholder, and status values remain readable. | Pass |
| `pdf-dark-hc-1280.png` | Black high-contrast PDF Annotation shell around a white document page. The selected thumbnail, highlight, dark annotation fields, reply placeholder, and actions are distinct. | Pass |
| `settings-dark-1280.png` | Dark Settings shell with category rail and General rows. Auto-save is a 32x18 flat switch with accent on track and the knob at the right beside the interval select. Advanced settings is an inline row rather than a mostly empty card. | Pass |
| `settings-light-1280.png` | Light Settings shell with ruled General rows. Auto-save shows the same flat switch in the on state with an accent track and the knob at the right beside the interval select. Changes save immediately appears at the lower left. | Pass |
| `settings-light-hc-1280.png` | White high-contrast Settings shell with ruled General rows, flat controls, and an inline Advanced settings action. Auto-save uses the accent on switch beside the interval select. | Pass |
| `settings-dark-hc-1280.png` | Black high-contrast Settings shell with bright row rules and a clearly highlighted General category. Auto-save reads as an accent on switch with the knob at the right. | Pass |
| `workshop-dark-1280.png` | Dark Project shell with source rail, preview stage, lower-third inspector, and three timeline tracks. The 10s label is complete at the right edge and the Intro flag occupies a row above the 0s label. | Pass |
| `workshop-light-hc-1280.png` | Light high-contrast Project shell with outlined stage, inspector fields, and timeline rows. The selected title clip, endpoint labels, and lower-third controls remain distinct. | Pass |
| `workshop-dark-hc-1280.png` | Black high-contrast Project shell with bright region boundaries and a full timeline. Both endpoint labels are inside the ruler and the marker does not cover 0s. | Pass |

## SVG, raster, and DXF

| Capture | Literal description | Grade |
| --- | --- | --- |
| `image-svg-dark-1280.png` | Dark Image shell with Preview and Source tabs, a complete North Bay vector study, and a Selection inspector. Helper copy and empty field guidance are visible. | Pass |
| `image-svg-hc-1280.png` | Light high-contrast Image shell with a dark vector artwork area and white inspector. Controls use flat backgrounds, one-pixel outlines, and no shadow. | Pass |
| `image-svg-dark-hc-1280.png` | Dark high-contrast Image shell with outlined inspector fields and a complete vector artwork. No vector edge or inspector control is clipped. | Pass |
| `image-raster-dark-1280.png` | Dark Image shell with full-height History and Adjust workbench rails. Their empty-state copy is visible. The complete North Bay colour study is fitted inside the checkerboard canvas and its line study begins below the subtitle. | Pass |
| `image-raster-hc-1280.png` | Light high-contrast Image shell with white workbench rails and a fully fitted raster. Artwork text and lines remain inside the image boundary. | Pass |
| `image-raster-dark-hc-1280.png` | Dark high-contrast Image shell with bright rail boundaries and a fully fitted raster. The canvas, artwork, empty-state guidance, and status values remain distinct. | Pass |
| `dxf-proving-dark-1280.png` | Dark CAD / 3D shell with Layers, a centred outline-and-circle drawing, and a Geometry count rail. Layer labels and entity counts use readable theme text. | Pass |
| `dxf-proving-hc-1280.png` | Light high-contrast CAD / 3D shell with white side rails and a dark drawing canvas. Layer toggles and entity counts are clearly separated. | Pass |
| `dxf-proving-dark-hc-1280.png` | Black high-contrast CAD / 3D shell with bright region boundaries and cyan geometry. Layer names, table headings, counts, zoom, and status remain visible. | Pass |

## Audio and video

| Capture | Literal description | Grade |
| --- | --- | --- |
| `audio-dark-1280.png` | Dark Audio shell with a compact player centred in the open media workspace. The player contains a full-width cyan waveform, flat Play action, seek rail, duration display, and audio status without an internal dead gap. | Pass |
| `audio-light-hc-1280.png` | White high-contrast audio workspace with a dark waveform panel and outlined focus state. Mute, Play, seek, time, and status remain readable. | Pass |
| `audio-dark-hc-1280.png` | Dark high-contrast audio workspace with a bright waveform boundary and flat controls. No video-only command, native control set, or developer copy is present. | Pass |
| `video-dark-1280.png` | Dark Audio-family shell with Mute and Picture-in-picture commands. The bounded stage shows Press Play to watch above one flat playback row. | Pass |
| `video-light-hc-1280.png` | White high-contrast video workspace with a black stage, explicit play prompt, and one playback row. Stage and controls are fully bounded. | Pass |
| `video-dark-hc-1280.png` | Dark high-contrast video workspace with a bright stage outline, explicit play prompt, and flat playback controls. Native or proving controls are not duplicated. | Pass |

## Live workshop

| Capture | Literal description | Grade |
| --- | --- | --- |
| `unibrow-live-on-dark-1280.png` | Dark 1280-wide Project shell in LIVE state with sources, programme stage, health and mixer rail, and three timeline tracks. Functional labels are readable, 10s is complete, and Intro is above rather than over 0s. | Pass |
| `unibrow-live-on-dark-1600.png` | Dark 1600-wide LIVE shell with the same source, stage, inspector, and timeline regions. Extra width is assigned to the stage and timeline without leaving a dead card gap. | Pass |
| `unibrow-live-recording-dark-1280.png` | Dark 1280-wide recording shell with a REC marker and Recording programme-stage copy. Timeline endpoints, clips, mixer, and source health remain visible. | Pass |
| `unibrow-live-recording-dark-1600.png` | Dark 1600-wide recording shell with an expanded stage and timeline. REC state, programme message, clip durations, and output status are readable. | Pass |
| `unibrow-live-on-light-1280.png` | Light 1280-wide LIVE shell with white source and inspector rails around a dark programme stage. Timeline tracks, endpoint labels, and status text remain distinct. | Pass |
| `unibrow-live-on-light-1600.png` | Light 1600-wide LIVE shell with expanded stage and timeline regions. Source labels, mixer state, clips, and time ruler remain aligned. | Pass |
| `unibrow-live-recording-light-1280.png` | Light 1280-wide recording shell with REC marker and Recording programme copy. No control overlaps the stage, inspector, or timeline. | Pass |
| `unibrow-live-recording-light-1600.png` | Light 1600-wide recording shell with clear REC state, programme stage, source health, mixer, and complete timeline. | Pass |

## Reviewer flags and disposition

Fixed:

- PDF dark surface leaks: a dock-wide hard-coded light rule affected form
  controls, chips, and annotation cards. Those components now use shared
  surface and text tokens. The dark-surface capture guard checks chrome and
  side-panel inputs, textareas, selects, chips, and cards against the raised
  surface token.
- Settings Auto-save state: replaced the stretched pseudo-checkbox with a shared
  `ub-switch` (32x18, 5px radius, role=switch, aria-checked). On uses an accent
  track with the knob at the right; off uses a neutral bordered track with the
  knob at the left. Auto-save defaults on for fresh isolated config.
- PDF and chrome form bevels: shared `ub-form-control` styles strip native
  appearance, box-shadow, and uneven borders on inputs, textareas, and colour
  swatches. The dark capture guard rejects uneven border colours on form fields.
- Workshop ruler clipping and collision: endpoint transforms keep 0s and 10s
  within the viewport, while marker flags and tick labels use separate rows.
  The capture guard asserts ruler bounds and marker separation.
- Breadcrumb spacing: separators are dedicated spans with equal margins.
- Dark Save styling: one all-theme rule gives Save an accent fill, same-hue
  border, 4px radius, and no shadow.
- Browse muted text: headings, sort labels, and properties now use readable
  theme text tokens.
- Markdown disabled Send: explicit themed control and text colours preserve a
  readable disabled state.
- Duplicate Image Undo and Redo: title-bar copies were removed. The commands
  remain in the context Command Manager.
- Settings Advanced gap: the former card is an inline settings row.
- Raster overlap: generated guide lines now begin below the subtitle.
- SVG helper contrast: helper and empty-field text use readable theme tokens.
- Media proving controls: audio and video now use one compact user-facing
  control row and format-specific Command Manager actions.

Reviewed with a specific non-failure reason:

- Audio surrounding space is the unallocated media workspace around a centred
  compact player. It is not a gap between related elements inside a card; the
  waveform uses the full player width.
- New tab page actions are task launchers required by the Start view. File and
  Folder in the Command Manager are the persistent keyboard command surface,
  so they are separate access paths rather than duplicated controls in one
  control group.
- Raster History and Adjust space belongs to full-height workbench rails with
  explicit empty-state copy. Neither is a card separating related content.
- SVG high-contrast outlines communicate control boundaries. They are
  one-pixel borders on flat backgrounds with no box-shadow, not raised
  controls.
- Live functional labels are 12px in the source. Capture diagnostics measured
  their status contrast between 9.49:1 and 11.12:1, above the 4.5:1 rule.
- DXF dark labels use the primary and secondary theme text tokens. The final
  capture and contrast guard both keep them readable.

## Verification

- `npm run test:contrast`: 6 tests passed, including status contrast, dark
  surface luminance, form border uniformity, timeline bounds, and marker
  separation.
- `npm run test:image-chrome`: 1 test passed.
- `npm run build`: passed.
- Final isolated capture chain: exited 0 for shell density, UI-14, UI-16, and
  theme media.
- Final isolated capture pixel checks: passed.
