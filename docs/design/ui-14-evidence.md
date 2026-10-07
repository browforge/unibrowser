# UI-14 evidence

Captured with `scripts/capture-ui-14.sh` at 1280x800, light theme, standard
contrast, 100 percent display scale, compact Command Manager, and collapsed
activity rail. The script checks exact dimensions and rejects duplicate hashes.
Every PNG below was opened after the final capture.

## Literal image descriptions

### `pdf-annotation-light-1280.png`

The PDF shell has a three-thumbnail Pages dock, one white page, and a right
Annotations dock. The visible CM row is Undo, Redo, Highlight, Underline,
Strike, page 1 / 3, zoom 133%, and Export PDF. There is no spelling control and
no bottom page or zoom toolbar. The yellow highlight covers the third text line,
not the empty area below it. The red area outline is lower on the page and does
not cross text. Each annotation card shows a type icon, colour swatch, title,
author and time, Page 1 link, reply field, and quiet Style, Post reply, and
Resolve actions. No raw CSS value or raw tool field is visible. No text,
control, dock, or page edge overlaps or clips.

### `pdf-annotation-100-light-1280.png`

The same page is rendered at 100%. The CM and status read 100%. The yellow
highlight still covers the same third text line, with the rectangle reduced in
the same proportion as the page. The annotation cards and docks remain clear.
Nothing overlaps or clips.

### `pdf-annotation-overflow-light-1280.png`

The main PDF row remains visible behind an open overflow menu. The menu shows
Tools (Select, Hand), Add (Note, Area, Ink, Text box, Stamp), Annotations
(Previous annotation, Next annotation), and Appearance (Stroke, Fill, Width).
All entries fit above the page content without an internal cut-off. Unsupported
entries remain legible and expose their provider reason through the command
tooltip and focus announcement.

### `image-light-1280.png`

The SVG shell shows Undo and Redo, then Crop, Rotate, Resize, Exposure,
Contrast, Colour, Curves, and Export. Fit and zoom remain in the overflow group.
Raster-only operations are muted on SVG and retain specific unsupported
reasons. The Preview and Source switch sits above the full vector fixture. The
Selection inspector occupies the right dock. No legacy image toolbar duplicates
the CM.

Raster PNG evidence (`image-raster-light-1280.png`, `image-raster-dark-1280.png`,
`image-raster-hc-1280.png`, and `../shell-density/image-{light,dark}-1280.png`)
shows Fit and 100% in the CM, utilities in overflow only, a centred 1600×1000
procedural fixture on a checkerboard viewport, and a status zoom that matches fit
(for example Image 67%). No duplicate image toolbar row appears.

### `dxf-proving-light-1280.png`

The CAD / 3D row shows Select, Orientation, Pan, Orbit, Fit view, Zoom out,
Zoom in, and Measure. Section, Appearance, and Export STEP remain in overflow.
Measure is focused and its tooltip says that a full CAD geometry engine is
required. The Layers dock, wireframe canvas well, and Geometry dock fill the
viewer column; layer/pan guidance lives in the status hint, not a flush-left
canvas note. No control or canvas edge clips.

## Grade

| Context | Result | Basis |
| --- | --- | --- |
| PDF annotation | Pass | Complete target set is visible or in overflow; provider states are honest; 100% and 133% highlights cover the same text; duplicate navigation is gone. |
| Image / SVG / raster | Pass | Complete requested target set is present; raster tools have providers; SVG-only limits have reasons; duplicate local controls are gone. |
| CAD / DXF | Pass | Complete target set is present; Fit and zoom work; unavailable geometry tools remain discoverable with reasons. |

Totals: 3 Pass, 0 Drift, 0 Fail.

## Automated contract

- `pdfAnnotationGeometry.test.mjs` checks normalised annotation geometry at 100%
  and 133%.
- `worked_context_contract.rs` checks every PDF, Image, and CAD target control is
  present, raster Fit/Crop/Rotate are supported, and unavailable providers have
  reasons.
- Overflow controls stay keyboard-focusable and announce unavailable reasons.
