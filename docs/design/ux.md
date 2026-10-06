# UX architecture

This document makes interaction decisions. It is not a menu of alternatives.

## Bear may push back

These are taste-sensitive defaults. Each is switchable without changing the underlying model.

1. **Workspace switcher location:** default is the left edge of the Command Manager tab row, not a permanent app-wide page rail. Switchable to hidden (palette-only) or compact icon mode.
2. **Command Manager density:** default is Compact (44 px manager body, 28 px tabs). Comfortable adds 8 px height; Dense removes labels only when the user explicitly chooses it.
3. **Right sidebar default width:** 320 px, resizable 260-520 px. Each sidebar remembers its width.
4. **Bottom panel default height:** 220 px, resizable 140 px to 55% of window.
5. **Accent:** restrained Windows-blue by default. The user can choose accent, system accent, or no accent.
6. **Tab shape:** squared lower edge and 6 px top corners, closer to browser/workstation chrome than pill tabs.
7. **New tab page:** recent work plus Open/New project, not a dashboard of widgets. Sections can be hidden.

## Information architecture

### Stable shell

The shell has six persistent regions:

1. **Title/menu row**: Windows title controls plus File, Edit, View, Go, Tools, Help. Linux uses the same command grouping with platform window chrome.
2. **Tab strip**: files, folders, web pages, settings, downloads, projects.
3. **Navigation row**: Back, Forward, Reload/Stop, omnibox, tab-local actions, overflow.
4. **Command Manager**: workspace/context tabs, Pin/Follow, command groups.
5. **Work area**: optional left sidebar, content, optional right sidebar, optional bottom panel.
6. **Status bar**: concise document/system state and panel toggles.

Only tab strip, navigation row, content, and a 24 px status bar are always visible. The Command Manager may collapse to its tab row. Sidebars and bottom panels are opt-in and docked.

### Content types

- **Start:** New tab page.
- **Browse:** folders and Downloads.
- **Web:** remote page.
- **Read/Edit:** text, code, DOCX, EPUB, PDF, image, DXF, audio, video.
- **Project:** `.unibrow` board, timeline, live studio.
- **System:** Settings, Mod Portal, download history.

These are tab content types, not separate apps.

### Workspaces

- **Browse:** files/web navigation; minimal panels.
- **Write:** outline/files left; content centre; Problems bottom on demand.
- **Create:** sources left; preview centre; inspector right; timeline bottom.
- **Live:** sources left; programme preview centre; properties/chat right; mixer/status bottom.

Workspace controls arrangement. Document context controls commands. Neither changes file format or closes tabs.

## Chrome region jobs

### 1. Title/menu row

Job: app/window commands, durable menus, workspace switcher in narrow layouts.

Contains:

- App icon and document/window title.
- Menus with canonical command locations.
- Window controls.

Does not contain:

- Loaded mods, demo actions, document tools, Save status, AI grants.

### 2. Tab strip

Job: identity and switching among all open locations.

Rules:

- File, folder, web, project, and system tabs use the same geometry.
- Icon identifies broad content class, not technical backend.
- Dirty state is a dot; pinned state is a pin icon plus tooltip.
- Audio/live activity has text/icon redundancy.
- Close button appears on active/hovered tabs; keyboard focus always exposes it.
- Overflow becomes a searchable tab list, grouped by window.

### 3. Navigation row

Job: location and navigation.

Left:

- Back, Forward, Reload/Stop.

Centre:

- Omnibox with human-readable path/URL.
- Breadcrumb mode for folders when not focused.

Right:

- Bookmark/Favourite where applicable.
- Find.
- Downloads progress.
- Overflow menu.

No permanent Home/Downloads/Settings button row. Those live in the New tab page, menus, palette, and optional user toolbar.

### 4. Command Manager

Job: visible, context-relevant commands with stable placement.

Structure:

- 28 px tab row: workspace switcher, context tabs, spacer, **Follow/Pin**, customise overflow.
- 44 px command row (Compact): grouped actions with separators.
- Tab labels: Browse, Text, Office, PDF, Image, Draw, Video, Live. Only relevant/baseline tabs show; user-pinned custom tabs remain.

Interpretation of "grey out, do not hide":

- Commands in a selected/known tab remain in fixed slots and grey out.
- Context tabs not relevant to the installed capability set need not all occupy the bar. They remain reachable through tab overflow and customisation.
- Disabled tooltip states cause and remedy: "Increase font size is unavailable: this image has no text layer."

Pin model:

- **Following: PDF** is a quiet status in the tab row.
- Clicking another context tab previews it.
- **Pin** is a labelled toggle adjacent to the status.
- Pinned state reads **Pinned: Image** and survives tab changes.

### 5. Work area

Job: user content and supporting views.

Layout:

```text
+ left sidebar +------------ content ------------+ right sidebar +
| optional     | never covered by docked views  | optional      |
+--------------+---------------------------------+---------------+
| optional bottom panel: Problems | Output | Terminal           |
+----------------------------------------------------------------+
```

Rules:

- Sidebars push content.
- Minimum content width is 480 px. Below it, only one sidebar may be open; opening another replaces the current sidebar.
- Bottom panel pushes content upward.
- Sidebar/panel headers contain title, tab switch if needed, More, Close.
- Detached panels are owned windows and never always-on-top by default.

### 6. Status bar

Job: glanceable state and panel entry points.

Left:

- Current file/folder or page identity (ellipsised).
- Save/autosave state.

Centre:

- Cursor/page/selection or playhead/timecode, depending on context.

Right:

- Problems count.
- AI access indicator when AI mod is configured and relevant.
- LIVE/Recording state.
- Encoding/zoom.
- Panel toggles in overflow on narrow windows.

The status bar never carries a full error sentence. Clicking a problem opens Problems.

## Command placement rules

| Surface | Use for | Do not use for |
| --- | --- | --- |
| Command Manager | Frequent, visible, context-relevant actions; stable muscle memory | Settings, diagnostics, one-off developer actions |
| Palette | Every command; infrequent actions; direct navigation; settings search | Persistent status or multi-step forms |
| Context menu | Actions on the clicked object/selection; max 12 top-level entries | Global navigation, unrelated commands, configuration |
| Status bar | Passive state and one-click panel entry | Primary actions, prose errors, onboarding |
| Settings | Durable preferences, grants, keybindings, workspace management | Actions on current selection/document |
| Sidebar | Supporting work that must remain visible: Files, Outline, AI, Properties, Sources | Alerts, transient confirmation, duplicate command ribbons |
| Bottom panel | Problems, Output, Terminal, timeline/mixer when workspace requires it | Permanent global navigation |
| Inline bar | Recovery, unknown handler, trust/risk tied to current tab | Marketing, routine success, global notices |
| Toast | Brief low-risk feedback with optional Undo | Required decisions, long errors, progress that must persist |

### Canonical command rule

Every action has one command id and one canonical label. Surfaces reference it. Surfaces never implement parallel actions with different wording.

Examples:

- `Save` is File > Save, Ctrl+S, palette Save, optional toolbar Save.
- `Find` is Edit > Find, Ctrl+F, navigation-row icon, palette Find.
- `Export` is File > Export, Command Manager Export, palette Export.

## Progressive disclosure

### Default view

- No mod list.
- No Demo download.
- No Export/Import layout buttons.
- No permanent AI window.
- One Command Manager row.
- Status bar shows only relevant state.

### Learn-once depth

- Tooltip on first disabled command explains that tools follow content.
- First manual context-tab click reveals Pin/Follow explanation once.
- Workspace customisation enters an explicit edit mode.
- Advanced export sections are collapsed.

### Expert depth

- Palette provides all commands.
- Keybindings editor and JSON coexist.
- Developer settings reveal mod inventory, host logs, demo/capture actions, and reload commands.
- Toolbar layout import/export lives in View > Workspaces and Developer settings.

## Keyboard model

### Global

| Shortcut | Action |
| --- | --- |
| Ctrl+L | Focus omnibox |
| Ctrl+T / Ctrl+W | New / close tab |
| Ctrl+Shift+T | Reopen closed tab |
| Ctrl+Tab / Ctrl+Shift+Tab | Next / previous tab |
| Ctrl+1...8 | Tab by position |
| Ctrl+Shift+P | Command palette |
| Ctrl+P | Quick open file/tab |
| Ctrl+S / Ctrl+Shift+S | Save / Save As |
| Ctrl+F | Find in active surface |
| Ctrl+Shift+M | Problems |
| Ctrl+` | Terminal |
| F6 / Shift+F6 | Cycle chrome, sidebars, content, panels |
| Alt+Left / Alt+Right | Back / forward |

S-key style popup is optional and unbound by default. Users may bind it to palette filtered to context commands.

### Focus rules

- Opening a docked sidebar does not steal focus unless invoked for immediate input (e.g. AI chat command).
- Closing a sidebar/panel returns focus to the originating control/content.
- Palette remembers and restores focus.
- Escape dismisses temporary surfaces in order: context menu, palette, Find; it does not close docked sidebars.
- Tab traverses within the active region; F6 changes regions.

## Discoverability

- Menus provide durable taxonomy.
- Command Manager provides frequent contextual actions.
- Palette provides search and shortcut discovery.
- Context menus provide object actions.
- Tooltips include shortcut and disabled reason.
- Empty states offer one clear primary action plus a secondary learn link.
- New features do not earn permanent chrome by existing.

## Accessibility

### Contrast and colour

- WCAG 2.2 AA minimum: 4.5:1 text, 3:1 large text and meaningful UI boundaries.
- Focus ring must reach 3:1 against adjacent colours and be at least 2 CSS px.
- Error, warning, success, selected, LIVE, and recording use icon/text/shape in addition to colour.
- High-contrast theme avoids transparency and shadows as the only boundary.

### Screen readers

- Regions have landmarks and names: Tabs, Navigation, Command Manager, Left sidebar, Document, Right sidebar, Bottom panel, Status.
- Tabs use `tablist`/`tab`/`tabpanel`.
- Disabled reasons use `aria-describedby`.
- Save/LIVE/record/progress state changes use polite live regions; errors use assertive only when immediate action is required.
- Problems expose severity, source, location, message, and actions as structured rows.

### Scaling and input

- Verify Windows scaling at 100, 125, 150, 200%.
- Do not use pixel-snapped text or icon assets.
- Pointer target minimum 32x32 px; touch mode 44x44 px.
- Resizers provide 6 px visual/10 px hit zones and keyboard alternatives.
- Reduced motion removes non-essential transitions.

## State design

### Empty

- New tab: Open, New project, recents (when any), keyboard hint.
- Sidebar: explain its job and the action that populates it.
- Problems: "No problems found" plus current filter summary.
- AI: provider setup or a prompt, never an empty box.
- Timeline: "Add a clip, title, photo, or document."

### Loading

- Under 200 ms: no indicator.
- 200 ms-2 s: local inline spinner/status.
- Longer: persistent inline progress with item name and Cancel where safe.
- Determinate progress for downloads, export, install, and encode.
- Skeletons only where final structure is known.

### Error

Every error answers:

1. What failed?
2. What remains safe?
3. What can the user do?
4. Where are details?

Error appears next to the failed task. Technical details are expandable/copyable. Save is not disabled because export or diagnostics failed.

### Success

- Silent visible state change is preferred.
- Toast only when result is otherwise invisible: "Saved copy to ..." or "Closed tab" with Undo.
- Never show a modal success.

## Notifications without modals

Attention ladder:

1. Status change.
2. Toast with optional Undo.
3. Persistent inline bar tied to a tab.
4. Sidebar/panel item.
5. OS notification only for meaningful background completion when app is unfocused.

No application modal is introduced for confirmation. Recoverable destructive actions execute with Undo. Irreversible actions use an inline confirmation row in the relevant panel.

## Windows 11 fit without Fluent imitation

- Use Segoe UI Variable when available; system UI fallback elsewhere.
- Respect system accent and light/dark preference.
- Use familiar Windows menus, shortcuts, window controls, drag regions, and density.
- Keep Unibrowser's workstation character: squared content frames, compact Command Manager, restrained radii, no pervasive acrylic or oversized cards.
- Linux uses the same information architecture with system font and window decoration.

## Principles check

| Principle | UX response |
| --- | --- |
| Trust the user | Layout, grants, keybindings, and mods remain user-controlled; no nagging reset |
| Never block saving/work | Save is independent from diagnostics/export; sidebars and bars are non-modal |
| Local-first | No account in first launch; local recents/config/autosave; network is explicit |
| Predictable across formats | One command identity and stable regions; disabled reason rather than disappearance |
| Simple durable mechanism | Menus, tabs, files, sidecars, docked panels, JSON layouts |
| Mods over core | Mods contribute capabilities through standard commands/views, not arbitrary chrome |
| Nothing covers content | Docked views push content; only invoked palette/context menu are transient overlays |
| User taste wins | Named layouts, density, accent, detached bars, and sidebar widths persist |
