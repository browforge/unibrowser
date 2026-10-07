# Quiet Utility mockup parity audit

This audit compares the approved design source on the left with the current
light-theme application capture on the right. The labelled composites are in
`docs/screenshots/parity/`. A context-matrix crop is used where the approved
set defines only layout hierarchy rather than detailed visual styling.

## New tab

Sources: `mockups/quiet-utility/new-tab.png` and
`screenshots/shell-density/new-tab-light-1280.png`.

- Layout: the mockup uses a 784 px work column with four recent rows and two
  plain download rows. The app uses a narrower centred column, eight recent
  rows, and two filled download bands.
- Spacing and density: the app leaves more empty space beside the work column
  but packs more rows into it. The mockup gives section changes more space and
  keeps row density consistent.
- Type: the app heading is smaller and heavier relative to its body copy. The
  mockup has clearer heading, section-label, primary-row, and metadata levels.
- Borders and radii: the mockup uses rules between rows and one restrained
  radius on actions. The app gives each download a large grey rounded strip and
  uses browser-default button treatment.
- Colour: the mockup uses a warm work surface and blue only for primary action
  and links. The app is nearly white and gives the download rows a stronger
  neutral block than their importance warrants.
- Icons: the mockup uses quiet file-type abbreviations. The app uses dark filled
  type badges, which compete with filenames.
- Panel headers: both are one-page layouts, but the mockup aligns Recent and
  Downloads headers to their trailing actions.
- Status bar: the mockup shows local/online and tab state. The app shows AI,
  path, command context, and alerts, with weaker information grouping.
- Empty states: neither shot is empty. The mockup communicates active download
  progress; the app shows only completed rows.

## File browser

Sources: the File browser tile in `mockups/context-matrix.png` and
`screenshots/shell-density/file-browser-light-1280.png`.

- Layout: the approved layout has favourites, file list, and preview as three
  joined regions. The app capture has Places and the file list but no preview
  region at 1280.
- Spacing and density: the app list is compact and usable, but the Places column
  is much denser than the schematic hierarchy and recent files dominate it.
- Type: filenames, metadata, Places headings, and Places entries are close in
  size and weight. The approved hierarchy separates region labels from data.
- Borders and radii: the app correctly avoids cards, but nearly every file row
  has a rule while the approved layout relies first on region boundaries.
- Colour: selection and favourite colour are restrained; the filled file-type
  badges are visually heavier than the approved quiet preview/list treatment.
- Icons: the app mixes Lucide chrome icons with text-filled rectangular type
  badges. The approved layout treats file identity as supporting information.
- Panel headers: Places has no strong region header and the missing Preview
  region has no header or empty state.
- Status bar: the app repeats the full folder path already present in the
  location field and breadcrumb, then adds AI and command context.
- Empty states: no preview-selection empty state is present because the preview
  region is absent.

## Markdown and rich text

Sources: `mockups/quiet-utility/document-command-compact.png` and
`screenshots/shell-density/markdown-light-1280.png`.

- Layout: the mockup demonstrates the complete document workbench with editor,
  right AI dock, and bottom Problems panel. The app capture gives the editor the
  full work area and adds a third local Preview/Export/Print row.
- Spacing and density: the two-row global chrome is close to the mockup, but the
  local action row spends another 32 px and the status row carries more fields.
- Type: Source Code Pro is present in both. App editor text and line numbers are
  smaller and lighter, while mockup panel headings and diagnostic rows have
  stronger levels.
- Borders and radii: the app editor is correctly flat. The mockup uses purposeful
  dock boundaries; the app shot has no dock boundaries because those regions
  are closed.
- Colour: the mockup uses a warm editor surface and blue selected rows. The app
  editor is white, making the light shell feel colder and less integrated.
- Icons: both use restrained line icons in chrome. The app local action row is
  text-only and visually detached from the Command Manager.
- Panel headers: the mockup has compact AI and Problems headers with actions in
  the header line. The app has only the generic local document toolbar.
- Status bar: the mockup groups save time, language, caret, problems, AI, and
  encoding. The app spreads AI, encoding, statistics, font size, alerts, and
  context across the row without the same left/right grouping.
- Empty states: the comparison is state-mismatched. The app does not show how
  closed AI or Problems regions are invited without becoming permanent chrome.

## PDF

Sources: `mockups/quiet-utility/pdf-annotation.png` and
`screenshots/shell-density/pdf-light-1280.png`.

- Layout: both use Pages, centred canvas, and Annotations. The app gives the page
  more width, narrows Pages, and uses the right dock for a list rather than a
  selected-annotation inspector.
- Spacing and density: the mockup page has a wider quiet surround and larger
  dock padding. The app packs three thumbnails and two annotation cards into
  less horizontal space.
- Type: the app dock labels, card metadata, and actions are smaller. The mockup
  gives the selected annotation title and field labels clearer weight.
- Borders and radii: the mockup reserves shadow for the physical page and uses
  one border around the selected annotation. The app outlines every annotation
  card and reply field.
- Colour: both use warm canvas and semantic yellow/red. The app uses blue card
  borders on every annotation, making selection less clear.
- Icons: the mockup Command Manager has explicit grouped PDF tools. The app
  exposes a smaller subset and puts more tools in overflow.
- Panel headers: the mockup headers include counts and dock actions. App headers
  concatenate label and count and omit close/more affordances.
- Status bar: mockup status is PDF-specific. App status also includes AI,
  generic command context, Alerts, absolute path, byte count, and scale.
- Empty states: neither is empty; the app lacks the mockup's selected-tool and
  selected-annotation property state.

## Image

Sources: the Image tile in `mockups/context-matrix.png` and
`screenshots/shell-density/image-light-1280.png`.

- Layout: the approved shell has History, canvas, and Adjust regions. The app
  raster capture is canvas-only, so both supporting docks are absent.
- Spacing and density: the app maximises image area and leaves only a narrow
  checkerboard margin. The approved layout protects a stable workbench around
  the canvas.
- Type: the raster fixture contains large test text, but there are no panel
  labels or property values with which to establish the approved type hierarchy.
- Borders and radii: the app canvas is flat and unframed, which is appropriate;
  the missing docks also remove the purposeful region boundaries.
- Colour: the vivid test fixture dominates the whole application. The approved
  layout contains image colour inside a neutral workbench.
- Icons: the Command Manager uses consistent line icons, but Resize and Export
  are muted beside enabled image tools and there are no dock action icons.
- Panel headers: History and Adjust headers are absent.
- Status bar: the app shows fit zoom and image dimensions, but also generic AI,
  command context, and alerts.
- Empty states: there is no no-selection Adjust state or empty History state.

## Workshop

Sources: `mockups/quiet-utility/unibrow-create.png` and
`screenshots/shell-density/workshop-light-1280.png`.

- Layout: both use Sources, Stage, Inspector, and a bottom timeline. The app
  gives more height to the timeline and less width to Sources and Inspector.
- Spacing and density: the app stage and inspector controls are tighter. The
  mockup leaves more space between property groups and keeps the timeline tracks
  shallower.
- Type: the mockup uses stronger region headers, clip labels, and property
  labels. App helper text and timeline labels are smaller and greyer.
- Borders and radii: both correctly use joined regions. App inputs retain
  browser-default bevels and each timeline clip has a stronger filled block.
- Colour: the mockup stage contains real programme colour and a selected lower
  third. The app stage is a near-black title card, so the overall frame reads
  darker and less representative.
- Icons: app Command Manager icons are consistent; source type badges use a
  second visual language of boxed abbreviations.
- Panel headers: mockup headers carry counts or panel actions. App Sources,
  Stage, Inspector, and Timeline headers are labels only.
- Status bar: the mockup uses project-specific save, workspace, format, time,
  and audio data. App uses AI, command context, alerts, project name, and node
  count.
- Empty states: neither is empty. The app does not show missing-source or
  no-selection states in this acceptance shot.

## Settings

Sources: `mockups/settings-mods-1280.png` and
`screenshots/shell-density/settings-light-1280.png`.

- Layout: the mockup is one centred Settings workbench with a 183 px category
  rail and aligned setting rows. The app spans edge to edge and uses a 153 px
  rail. The unrelated Print preview row found during the audit is now removed.
- Spacing and density: the mockup groups five common decisions with consistent
  64 px rows. The app uses larger section gaps but packs labels, selects,
  checkboxes, and helper text inconsistently within each group.
- Type: the mockup has a clear Settings title, page title, row title, and helper
  hierarchy. The app has no page title and gives labels and helper copy similar
  weight.
- Borders and radii: the mockup has one 6 px workbench boundary and row rules.
  The app is correctly mostly flat but native selects and fields use mixed border
  and radius treatment.
- Colour: both use warm neutrals and restrained blue. App selected category and
  status accent are paler.
- Icons: the mockup category rail is intentionally text-only. The app follows
  that choice; chrome icons remain consistent.
- Panel headers: the app lacks the mockup's Settings and General headings and
  descriptive subtitle at the top of the work surface.
- Status bar: the mockup says settings save immediately and shows zoom. The app
  shows AI, command context, Alerts, and Application preferences.
- Empty states: not applicable in General; app does not show search no-results
  or an unavailable-setting state.

## Mod portal

Sources: the Mod Portal tile in `mockups/context-matrix.png` and
`screenshots/shell-density/mod-portal-light-1280.png`.

- Layout: the approved hierarchy is search, one focused result, and its install
  action. The app adds Installed/Discover/Updates tabs, a network notice, one
  suggested result, and a six-card catalogue in the same viewport.
- Spacing and density: the app is substantially denser and uses smaller card
  padding and smaller inter-row gaps.
- Type: result title, package id, source, licence, permissions, and description
  are too close in size and colour. The approved tile makes title and install
  action dominant.
- Borders and radii: every app result is a bordered rounded card. The approved
  hierarchy needs only a bounded result object, not a dashboard grid.
- Colour: semantic Built-in, Available, and Suggested labels are restrained but
  too small. The app's many equal surfaces dilute selected and recommended
  meaning.
- Icons: the app Command Manager uses Lucide, while result cards rely on text
  labels with no consistent product or source icon.
- Panel headers: Mod portal and its tabs are separate rows; search and network
  state are not integrated into one task header.
- Status bar: the app repeats the internal `unibrowser://mods` location and
  generic AI/command/alert fields rather than portal task state.
- Empty states: offline search guidance is present. Installed-empty,
  no-results, update-empty, and network-error states are not shown.

## Impact ranking

1. Settings: weak page and row hierarchy after removing leaked document chrome.
2. Image: missing History and Adjust workbench regions.
3. New tab: primary action, row, and download hierarchy.
4. Workshop: stage/timeline proportions and non-representative stage content.
5. PDF: annotation inspector hierarchy and over-bordered annotation list.
6. File browser: missing 1280 preview region and preview empty state.
7. Markdown/rich text: extra local toolbar and missing docked-workbench evidence.
8. Mod portal: card-grid density and weak result metadata hierarchy.
9. Shared status bar: generic fields compete with context-specific status. The
   earlier pale AI state is fixed in the audited captures.

## Status-bar contrast result

The 18 final shell-density captures contain 56 visible status segments. The PNG
guard samples every segment against its rendered bar background. All pass
4.5:1; the minimum is `AI: Read` in `new-tab-light-1280.png` at 5.08:1.
