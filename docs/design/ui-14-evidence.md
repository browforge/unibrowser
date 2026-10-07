# UI-14 evidence

Captured with `scripts/capture-ui-14.sh` at 1280x800, light theme, standard
contrast, 100% scale, compact Command Manager, and collapsed activity rail. The
script requires exact dimensions and rejects duplicate hashes. Every PNG below
was opened after the final capture.

## Literal image descriptions

### `pdf-annotation-light-1280.png`

Top: the `sample.pdf` tab, Save, and the 80 px compact chrome are complete. The
PDF Annotation Command Manager keeps Undo, Redo, Find, Check, Export, Select,
page, and 133% zoom visible. Lower-priority annotation commands move to the
overflow instead of clipping.

Left: a joined Pages dock shows three complete thumbnails and selected page 1.
Centre: the recessed canvas surrounds a full white PDF page with heading, body
text, a yellow highlight, and a red area annotation. A quiet footer keeps page
navigation reachable. Right: the joined Annotations dock shows two annotation
objects with page, tool, border, reply, and action controls.

Bottom: the status bar shows PDF Annotation context, page 1 of 3, zoom, file
path, and document details. No panel or control ends outside its region.

### `image-light-1280.png`

Top: the Image Command Manager keeps Undo, Redo, Find, Check, Export, Zoom out,
Zoom in, and Curves in one unclipped row.

Centre: Preview and Source are flat local mode controls above a recessed dark
canvas. The full SVG fixture is centred with clear edges and text. There is no
left dock because this fixture has no page or layer structure. Right: a joined
Selection properties dock shows Fill, Stroke, Stroke width, and a disabled
Apply to selection control until an object is selected.

Bottom: the status bar shows Image context, file path, SVG dimensions, element
counts, font size, and 100% zoom.

### `dxf-proving-light-1280.png`

Top: the CAD / 3D Command Manager contains Find entity, Fit view, Zoom out,
Zoom in, Measure, and Export STEP without clipping. Save remains present in the
title bar. The focused Measure control shows the real unsupported-reason
tooltip: `Measure needs a full CAD geometry engine.`

Below the CM: a quiet proving strip identifies the base-dxf wireframe scope and
states that Measure and STEP export stay greyed until a full engine ships.

Left: the joined Layers dock has checked Detail and Outline layers. Centre: the
recessed dark canvas shows the complete cyan rectangle, circle, and centre line
from the real ASCII DXF fixture. Right: the joined Geometry dock reports one
circle and five lines. The status bar reports CAD / 3D context, file path,
entity count, extents, and 100% zoom.

## Grade against Quiet Utility and design rules

| Context | Result | Basis |
| --- | --- | --- |
| PDF annotation | Pass | Optional left pages, recessed centre canvas, right annotations, aligned Find/Export/Page/Zoom, and group overflow with no clipping. |
| Image | Pass | Optional left dock omitted honestly, recessed canvas, joined properties dock, and aligned Find/Export/Zoom controls. |
| DXF proving | Pass | Left layers, recessed canvas, right geometry, real fixture, supported view controls, and visible unsupported reason. |

Totals: 3 Pass, 0 Drift, 0 Fail.

## Runtime contract

CAD path detection now resolves a CAD family. Fit view and both zoom controls
are supported. Find entity, Measure, and Export STEP remain in place with
provider-specific reasons. The command-system contract test checks these states
and reasons. Image now exposes Zoom out and Zoom in beside its existing Adjust
control. PDF page, zoom, and export groups are preserved before detailed
annotation groups, which overflow first.
