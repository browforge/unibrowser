# Quiet Utility mockup parity audit

This audit compares the approved design source on the left with the current
light-theme application capture on the right. The labelled composites are in
`docs/screenshots/parity/`. A context-matrix crop is used where the approved
set defines only layout hierarchy rather than detailed visual styling.

## Final fresh acceptance grading

These grades use the final 1280x800 captures regenerated on 2026-10-07. Each
capture passed the contrast, pixel CM, text-overlap, icon, duplicate-control,
and tooltip-clearing capture guards. The descriptions below were written from
the opened parity composites before assigning grades.

| Ticket | Capture SHA-256 | Literal description | Grade |
| --- | --- | --- | --- |
| UI-21 Settings | `9bca09bbbc3c91f2de51f3ffe405ac897d0991480002fa0aca2d6460db6c3b61` | General shows, in order, On startup, Downloads, Workspace, and Auto-save as ruled label-left/control-right rows with helper copy. The Developer callout follows. Search appears once in the category rail; the CM contains Keybindings and Diagnostics, not another search. AI and onboarding controls are absent from General. | Pass |
| UI-22 Image | `3fa17365ce85d3a056ebe24be57d8536ebbbfc4ec91b46fc9db827855bbc41f6` | History is a stable left dock, the fitted raster occupies the centre canvas, and Adjust is a stable right dock. Both docks show quiet truthful empty text. No local image toolbar duplicates the Image CM. | Pass |
| UI-23 New tab | `f9019a815072c9047b2cf35b45b4dfd5f983eedfb9a34cb5a951a44d4f555df9` | Open file is the filled accent action beside two flat-bordered secondary actions (Open folder, Start a new project). Four known recent files use quiet text type marks. Downloads are unfilled ruled rows; the first has a real 64% progress bar and percentage, and the second is complete. No unknown fixture appears. | Pass |
| UI-24 Workshop | `f70caf2c6f41599fe41666ef7408eff49ddbb000a257c0442c10928bcbb0d10a` | The programme frame contains a selected lower third. The Inspector header reads Lower third / Title clip, track 2 above Heading, Subtitle, Duration, and Position. A timecode transport sits below the stage. Video/Image, Title, and Audio/MIDI tracks are all fully visible without scrolling; light theme uses pale lanes and clips. | Pass |
| UI-25 PDF | `284aa736ea53c00370c24c2ed1a79aced6b073f95fc312039aab1512456df843` | Pages is on the left, the white page is centred, and Annotations is on the right. The yellow annotation has the selected outline and a populated Opening summary inspector; the other area annotation is visibly subdued and indexed separately. | Pass |
| UI-26 File browser | `301deffaf7467169eb052e9e1fef8892543d5be47f6562b677d77dc4da9cb0a9` | Two-row chrome only (no bookmark bar under the address field). The title tab strip shows one `sample.svg` tab plus a visible + control (no stretched ghost tab slot). Places shows RECENT FILES and PINNED section labels; `sample.svg` is selected with the 1 selected hint in the status bar, not a content banner. Places, list, and Preview are visible; `unknown.ubunknown` is absent. Isolated empty config (fresh defaults). | Pass |
| UI-27 Document workbench | `505f9bb90e548f9b92e6950648a84a8cb28230174859a4389c2f300600ecd9ff` | The editor occupies the main region, AI Assistant is docked at right with document context and composer, and Problems is docked below with three readable rows. Region boundaries are joined and no third local document toolbar appears. | Pass |
| UI-28 Mod Portal | `37832f09f516075b8187b0af78c1ddb36229baeb4f5ca31442781e8e59b7677a` | Discover for `ext=step` uses the plain intro sentence about local mods and network consent. The SUGGESTED RESULT card is a flex row: 40px CAD tile, then a growing text column starting ~12px after the tile with left-aligned semibold CAD file support title, body-sized description and licence lines, and closed Contribution details; a flat teal Install (no shadow) sits on the right with whitespace between text and button, not between tile and text. No in-card Suggested badge. Title bar shows `sample.step`, Mod portal, and +. Capture guard checks tile-to-text gap and flat Install. | Pass |

The detailed sections below record the pre-ticket baseline at `dc46d54`; they
are retained as the discrepancy history that produced UI-21 through UI-28.

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

## GPT-ready drift summary (2026-10-07)

UI-21 through UI-28 and UI-29 acceptance shots now pass on fresh captures with
isolated per-run `XDG_CONFIG_HOME` (no `UNIBROWSER_DOC_CAPTURE` prefs forks).
Remaining **Drift** is
cosmetic or schematic-only; nothing blocks the Quiet Utility parity tickets.

| Surface | Drift (still unlike mockup, acceptable) |
| --- | --- |
| Settings | Category rail width and page title scale are slightly lighter than the mockup; behaviour matches. |
| New tab | Centred column is narrower than the mockup work column; eight-row density is lower (four recents shown). |
| Workshop | Mockup uses a schematic frame; app shows real programme content and clip labels. Inspector matches hierarchy. |
| PDF | Annotation list uses more border weight than the schematic; selection and inspector hierarchy match. |
| File browser | Approved source is a three-region schematic without file rows; app is denser with real data. Places labels are uppercase section headers, not the mockup's single Favourites block. |
| Markdown / document | Mockup is layout-only; docked editor + AI + Problems match hierarchy. |
| Image | History/Adjust docks match; mockup uses flatter empty copy. |
| Mod portal | Schematic shows one-line licence under title; app keeps description, licence, and Contribution details. Discover catalogue grid and longer intro remain denser than the matrix tile. |
| Status bar | Still shows AI, path, command context, and Alerts where the mockup often shows only local/online; contrast guard passes (UI-29). |

**Fail rules (light captures):** mod portal featured card must place text within 24px of the CAD tile (no wide dead band before the copy column); Install must use flat primary styling (`box-shadow: none`); large empty gaps between related elements inside a card fail grading. `2956146` failed the featured-card layout rule before this fix.
