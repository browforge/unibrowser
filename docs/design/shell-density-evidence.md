# Shell density evidence (Quiet Utility)

Capture: `scripts/capture-shell-density.sh`. The matrix contains 18 PNGs at
1280x800: light and dark captures for eight surfaces, plus light and dark
high-contrast workshop captures. Doc-capture rejects scoped small text below
WCAG AA 4.5:1 and intersecting sibling text in mod cards.

## Literal descriptions and grades

### `new-tab-light-1280.png`

The Start CM uses distinct File, Folder, Project, and Recent icons. Eight recent
rows are about 35px high and use SVG, DOC, PDF, TS, DXF, ?, IMG, and MD badges.
There is no duplicate type label at right. Two Downloads rows sit fully above
the status bar. No row or button overlaps. Grade: Pass.

### `new-tab-dark-1280.png`

The same eight recent rows and two Downloads rows appear on the dark surface.
Badges, names, paths, times, and Open buttons remain legible and fully inside
the viewport. Grade: Pass.

### `file-browser-light-1280.png`

The Places column is on the left. The main table shows directories and twelve
files with one checkbox and one shared DIR, TXT, DOC, DXF, PDF, JSN, IMG, SVG,
TS, or ? badge per row. Name, size, and modified columns do not collide. Preview
metadata colours are readable. Grade: Pass.

### `file-browser-dark-1280.png`

The same table uses dark rows and separators. Badges, names, sizes, dates, and
Places text remain legible. The final unknown row sits above the status bar.
Grade: Pass.

### `markdown-light-1280.png`

The Markdown editor begins directly below Preview, Export HTML, and Print
preview. Lines 1 through 11, syntax colours, spelling marks, minimap markers,
and the right scrollbar are visible. The editor reaches the status bar without
an extra blank dock. Grade: Pass.

### `markdown-dark-1280.png`

The same eleven lines fill the dark editor. Text, links, syntax colours, line
numbers, and minimap markers remain distinguishable. Nothing clips. Grade:
Pass.

### `pdf-light-1280.png`

The visible PDF CM row is Undo, Redo, Highlight, Underline, Strike, page 1 / 3,
zoom 133%, and Export PDF. Three thumbnails fit in the left dock. The yellow
highlight covers the third text line. Two redesigned annotation cards fit in
the right dock. There is no bottom navigation row. Grade: Pass.

### `pdf-dark-1280.png`

The same page, highlight, area outline, thumbnails, cards, and CM controls
remain legible on dark chrome. The white PDF page is contained between the two
docks and status bar. Grade: Pass.

### `image-light-1280.png`

The Image CM shows Fit, 100%, zoom, Crop, Rotate, Resize, and Export, with
Exposure, Contrast, Colour, Curves, Flip, OCR, Copy image, Histogram, and
Eyedropper in overflow. There is no second command row under the CM. The
1600×1000 procedural fixture is centred on a checkerboard viewport that fills the
region between the CM and status bar. The status readout reports the true fit
zoom (for example Image 67%), not a false 100%. Pan/crop guidance sits in the
status hint with padding, not flush on the canvas. Grade: Pass.

### `image-dark-1280.png`

The same single CM and centred fit viewport appear on dark chrome. Utilities
remain in overflow only; the checkerboard well fills the viewer column. Status
zoom and hints stay legible. Grade: Pass.

### `settings-light-1280.png`

The Settings CM shows Search settings, Keybindings, and Diagnostics. General is
selected in the left category list. Language, Theme, Appearance, Focus mode,
and Bookmark bar labels are dark on the light form. The page has a scrollbar
for content below the status bar. Grade: Pass.

### `settings-dark-1280.png`

The same General form appears on a dark panel. Category names, field labels,
select values, checkbox labels, and helper text remain separated. Grade: Pass.

### `mod-portal-light-1280.png`

The Discover page shows a search field, a suggestion card, and a three-column
catalogue. The suggestion card puts the mod id and unsupported-extension reason
on separate lines. Catalogue columns have 16px gutters and equal outer insets.
Metadata wraps above a separate action row. Grade: Pass.

### `mod-portal-dark-1280.png`

The same suggestion and catalogue use dark cards. The id, reason, source,
licence, permissions, and actions remain separate. No card borders touch and no
text intersects. Grade: Pass.

### `workshop-light-1280.png`

Sources, Stage, Inspector, and Timeline headers are dark on light strips.
TTL, IMG, PDF, and MID badges, source sub-labels, Video / Image, Title, and
Audio / MIDI track labels, inspector labels, and the Title track meta label are
legible. All three timeline tracks fit above the status bar. Grade: Pass.

### `workshop-dark-1280.png`

The same four-region workbench uses dark panels. Headers, badges, source
sub-labels, inspector labels, and track labels remain readable. No track is
covered by the status bar. Grade: Pass.

### `workshop-light-hc-1280.png`

Light high contrast adds strong panel, input, timeline, and focus boundaries.
All headers and small labels remain legible. Geometry matches the standard
light capture and nothing clips. Grade: Pass.

### `workshop-dark-hc-1280.png`

Dark high contrast uses white region boundaries, input outlines, track
separators, and a cyan primary-action outline. All small labels remain visible
and all tracks fit. Grade: Pass.

Overall: 18 Pass, 0 Drift, 0 Fail.

## Automation

- `npm run test:contrast` checks token pairs and rectangle intersection.
- Doc-capture samples computed foreground and background colours for workshop
  headers, track labels, badges, source sub-labels, inspector labels, selection
  meta, title breadcrumb, file metadata, and settings labels.
- Doc-capture checks sibling text rectangles inside mod cards for intersections.
