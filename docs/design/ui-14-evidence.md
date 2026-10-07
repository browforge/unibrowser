# UI-14 evidence

Captured with `scripts/capture-ui-14.sh` at 1280x800, light theme, standard
contrast, 100% scale, compact Command Manager, and collapsed activity rail. PNG
and SVG paths now resolve to `WorkContext::Image` so capabilities match the
Image context shell. The script requires exact dimensions and rejects duplicate
hashes. Every PNG was opened after the final capture.

## Literal image descriptions

### `pdf-annotation-light-1280.png`

`sample.pdf` with PDF Annotation Command Manager (Undo, Redo, Find, Check, Export,
Select, page control, zoom, overflow). Joined Pages dock with three thumbnails;
recessed PDF canvas with highlight and area annotation; right Annotations dock.
Status: PDF Annotation, page 1 of 3.

### `image-light-1280.png`

`sample.svg` with Image Command Manager: Undo, Redo, Export, Zoom out, and Zoom
in use magnifier icons; Find, Check, and Curves stay visible but muted with
unsupported reasons (not available for images / spell check / raster curves on
SVG). Flat segmented Preview | Source control above recessed vector canvas with
full fixture art. Right: joined Selection inspector (fill, stroke, width,
disabled Apply). Status: Image context, SVG dimensions, element counts, 100%
zoom.

### `dxf-proving-light-1280.png`

`sample.dxf` with CAD / 3D Command Manager; Measure focused with unsupported
reason tooltip; Layers and Geometry docks; cyan wireframe on dark canvas.

## Grade against Quiet Utility and design rules

| Context | Result | Basis |
| --- | --- | --- |
| PDF annotation | Pass | Three-region shell, aligned universal commands, overflow without clipping. |
| Image / SVG | Pass | Image context resolution, greyed Find/Check/Curves with reasons, magnifier zoom icons, segmented preview toggle, joined inspector. |
| DXF proving | Pass | CAD family, real fixture, visible unsupported reasons. |

Totals: 3 Pass, 0 Drift, 0 Fail.

## Runtime contract

- Raster and vector image extensions infer `WorkContext::Image` from the document
  path (contract test `svg_path_infers_image_work_context`).
- Image capability map greys out Find, Proofing, and Curves on `.svg` with
  documented reasons (`svg_image_context_marks_non_image_commands_unsupported`).
- CAD and PDF behaviours unchanged from the prior UI-14 slice.
