# Layout specification

See [UX architecture](ux.md) for region jobs and [UI specification](ui-spec.md) for tokens/components.

## Region map

### 1280 x 800 reference

```text
+------------------------------------------------------------------------------+ 32
| File Edit View Go Tools Help | Browse Text Create Live | Undo Redo [Save] ...| global top
+------+-----------------------------------------------------------------------+ 34
|      | [tab] [tab] [active tab * x]                               [ + ] [v] |
| work +-----------------------------------------------------------------------+ 40
| space| Back Fwd Reload [ location / search                       ] Find  ... |
| rail +-----------------------------------------------------------------------+ 72 max
|      | Browse Text PDF Image Create       Following: Text [Pin]              |
| Files| Undo Redo | Find | Font - 14 pt + | Proofing | Export                 |
+------+--------------+--------------------------------------+-----------------+
|      | left sidebar |                                      | right sidebar   |
|      | 0 / 240 px   |          content surface             | 0 / 320 px      |
|      |              |                                      | AI Tools Assets |
+------+--------------+--------------------------------------+-----------------+
|      | bottom panel 0 / 220 px: Problems | Output | Terminal                |
+------------------------------------------------------------------------------+ 24
| saved 2m ago          Ln 12, Col 8              3 Problems  100%  UTF-8     | status
+------------------------------------------------------------------------------+
```

The workspace rail is 58 px at 1280 and occupies the left edge below the global
top bar to above the status bar. Tabs, navigation, Command Manager, sidebars,
content, and bottom panel are inset by the rail. The rail is a stable workspace
destination, not a second command toolbar.

Nominal content height with manager expanded and no panel is 598 px. The shell
must not spend more than 188 px above content in Compact density.

The top bar reconciles native application structure with the reference
concepts:

- Menus remain first for predictable desktop access.
- The workspace switcher shares the top bar and does not create another
  permanent row.
- Save is the only filled primary shell action.
- Undo, Redo, AI, and overflow may appear at the right when width permits.
- The Command Manager remains contextual and detailed below navigation. It is
  not replaced by the workspace switcher.

### 1600 x 1000 wide reference

```text
+ global top -----------------------------------------------------------------+
+ rail | tabs ----------------------------------------------------------------+
+ rail | nav/omnibox ---------------------------------------------------------+
+ rail | Command Manager tabs + commands ------------------------------------+
+------+-----------+-----------------------------------------+----------------+
| rail | 260 left  |           content / preview             | 340 right      |
|      | sidebar   |                                         | sidebar        |
+------+-----------+-----------------------------------------+----------------+
| rail | 260 bottom panel / timeline / mixer                                  |
+ status ---------------------------------------------------------------------+
```

Wide layouts may show both sidebars. They do not add another permanent top row.

## Docking model

Dock targets:

- Workspace rail destinations activate named layouts; they are not drop
  targets.
- Left sidebar.
- Right sidebar.
- Bottom panel.
- Command Manager top dock.
- Detached owned window.

Rules:

- Dragging begins only from a labelled handle.
- Drop targets appear after drag starts, not persistently.
- Sidebars push content and never overlap it.
- A docked sidebar may contain tabs, e.g. Files / Outline or Properties / AI.
- The right AI region may contain AI Assistant / Tools / Assets tabs. These are
  views in one dock, not floating cards.
- Opening a second view in the same region switches its tab; Shift+open splits only on widths >=1440.
- At content width under 480 px, opening a sidebar replaces the other sidebar.
- Minimum dock width 220 px, default left 260, right 320, maximum 40% of window.
- Bottom panel default 220 px, min 140, max 55% window.
- Detached windows remember monitor, normalised position, size, and dock origin.

## Responsive behaviour

### >=1440 px

- Both sidebars supported.
- Workspace rail is 64 px; labels stay visible.
- Command Manager command labels visible in Compact mode.
- Wide Live/Create workspaces use three columns.

### 1024-1439 px

- Workspace rail is 58 px with persistent labels.
- One sidebar per side, but opening both requires content >=480 px.
- Low-priority navigation actions move to overflow.
- Command groups may scroll horizontally; they do not wrap into a second row.

### 800-1023 px

- Workspace rail is 52 px; labels may collapse only when content would fall
  below 480 px.
- Only one sidebar visible at a time.
- Command Manager labels hide for recognised icons; tooltips/accessible names remain.
- Workspace switcher moves to View menu/palette if necessary.
- Bottom panel opens to 35% height.

### <800 px

- Supported as a compact utility layout, not primary editing.
- Workspace rail moves to the View menu and command palette.
- Command Manager collapses to tabs + overflow.
- Sidebar/panel becomes a full-height dock replacing content, with Back to document.
- Tab strip scrolls.

## Context layouts

All contexts retain global top bar, workspace rail at desktop widths, tab
strip, navigation, Command Manager frame, and status bar.

### New tab page

- No sidebars by default.
- Content width max 920 px.
- Primary: Open.
- Secondary: New project.
- Recent files/projects as a simple list with location and last opened.
- Downloads row only when recent/active.
- No product news, account prompt, mod inventory, or promotional cards.

### File browser

- Left: optional favourites/tree, 240 px.
- Centre: breadcrumb + list/grid. View controls are content-local.
- Right: optional preview/properties, 320 px.
- Browse commands: New folder, Cut/Copy/Paste, Rename, Sort, View, Archive.
- Folder tab identity remains in global tab strip.

### Web page

- No sidebars by default.
- Omnibox uses URL mode.
- Command Manager: Find, Zoom, Save page, Capture, Print/Export.
- Downloads progress in navigation row; developer tools in Tools/Developer.
- Permission/trust notices are inline below navigation.

### Text / DOCX

- Left: optional Files/Outline.
- Centre: editor/reader.
- Right: optional AI/Properties.
- Bottom: Problems/Output/Terminal.
- Text commands: Undo/Redo, Find, font size, proofing, Export. Save remains the
  one global primary action.
- DOCX-specific controls join Office tab, not another toolbar.

### Image

- Centre: checkerboard/canvas with fit-to-window default.
- Left optional: layers/history when supported.
- Right: properties/adjustments.
- Bottom optional: filmstrip for multi-image contexts.
- Commands: Crop, Rotate, Resize, Adjust, Copy, Export.
- Unsupported layer/text commands remain disabled with reason.

### PDF

- Left optional: pages/outline.
- Centre: page canvas.
- Right optional: comments/AI.
- Bottom: Problems/search results when invoked.
- Commands: Select, Pan, Find, Zoom, Highlight, Comment, Export.

### DXF

- Left: layers/structure.
- Centre: drawing canvas.
- Right: properties.
- Bottom: Problems/console on demand.
- Commands: Select, Pan, Zoom, Measure, Layers, text size, Export.
- CAD terms appear only in this context and properties.

### Video timeline (`.unibrow` Create)

- Left 260: Sources.
- Centre: preview.
- Right 320: Inspector.
- Bottom 260: timeline, required and resizable.
- Command Manager: Add, Split, Trim, Title, Audio, Export.
- Tab/navigation remain visible; workspace does not become a separate app.

### Multimodal workshop

- Named workspace for simultaneous CAD/3D, video, code, paint/compositing, and
  diagnostics work. It is not the default document arrangement.
- Workspace rail provides stable context entry; top switcher shows `Workshop`.
- Centre prioritises the active CAD/3D viewport and programme preview.
- Left dock: scene/project tree and source assets.
- Right dock: Inspector, colour/effects/audio tabs with numeric fields and
  restrained sliders.
- Bottom dock: code plus Console/Problems and a `.unibrow` timeline. At 1280,
  secondary panes collapse behind labelled tabs rather than shrinking text.
- At 1920, Layers/Brushes/Scopes may occupy additional joined panes.
- Every pane has a 28-32 px labelled header and one shared boundary. No panel
  receives a card shadow.

### Live studio

- Left 280: Sources/scenes and health.
- Centre: programme preview.
- Right 320: Properties or AI tab.
- Bottom 220: audio mixer / events.
- Persistent studio header inside content: Preview, **Go LIVE**, Record, elapsed state.
- LIVE and Recording also appear in status bar.
- No toast can cover preview or studio header.

### Settings

- Opens in a tab.
- Content max width 1120 px.
- Left 220: categories.
- Centre 640-760: searchable setting rows.
- Right absent by default; help is inline.
- Developer category is last and collapsed/hidden unless enabled.

### Mod Portal

- Opens in a tab or right sidebar from an unknown-file prompt.
- Top content row: search, Installed/Discover/Updates.
- Card/list hybrid: icon, name, capability, publisher, licence, source, compatibility, permissions, action.
- Technical manifest details are expandable.
- Install progress occupies the card.

## Overlay policy

Allowed transient overlays:

- Command palette.
- Context menus.
- Native text selection/caret aids.
- Drag ghost/drop targets.
- Tooltips.

These are user-invoked, short-lived, focus-managed, and dismissible. Everything else docks or renders inline.

The AI assistant is never an overlay. Problems is never an overlay. Settings and Mod Portal are never modals.

## Mockups

Rendered sources live in [`mockups/`](mockups/):

- [`main-document-1280.svg`](mockups/main-document-1280.svg) / PNG: text/PDF-style shell at 1280x800.
- [`live-studio-wide.svg`](mockups/live-studio-wide.svg) / PNG: Live workspace at 1600x1000.
- [`new-tab-files-1280.svg`](mockups/new-tab-files-1280.svg) / PNG: New tab and file-browser hierarchy.
- [`settings-mods-1280.svg`](mockups/settings-mods-1280.svg) / PNG: Settings and Mod Portal hierarchy.
- [`context-matrix.svg`](mockups/context-matrix.svg) / PNG: all eleven requested contexts and their dock regions.

Wireframes use final region proportions and representative labels, not final visual polish.

High-fidelity Quiet Utility studies live in
[`mockups/quiet-utility/`](mockups/quiet-utility/README.md). They cover the five
required ordinary contexts plus 1280x800 and 1920x1080 multimodal workspaces.
