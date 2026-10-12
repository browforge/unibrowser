# UX architecture

This document makes interaction decisions. It is not a menu of alternatives.

## Bear may push back

These are taste-sensitive defaults. Each is switchable without changing the underlying model.

1. **Custom title bar:** default is a 40 px row combining menu access, tabs,
   location/command field, Save/Undo/Redo, and window controls. Native title
   bars remain an accessibility/compatibility preference.
2. **Command Manager density:** Compact is the default 40 px grouped row. Large
   uses 64 px with icon-above-label controls. Collapse hides it entirely,
   leaving 40 px total chrome above content.
3. **Right sidebar default width:** 320 px, resizable 260-520 px. Each sidebar remembers its width.
4. **Bottom panel default height:** 220 px, resizable 140 px to 55% of window.
5. **Accent:** restrained Windows-blue by default. The user can choose accent, system accent, or no accent.
6. **Activity rail:** 40 px icon mode by default at 1280, with expanded,
   auto-hidden, and hidden options. It opens side panels only.
7. **New tab page:** recent work plus Open/New project, not a dashboard of widgets. Sections can be hidden.

## Information architecture

### Stable shell

The shell has five persistent regions:

1. **Unified title bar**: compact menu access, tabs, location/command field,
   global document actions, layout controls, and platform window controls.
2. **Command Manager**: one file-type-driven tool row with Follow/Pin.
3. **Activity rail + work area**: rail opens side panels; sidebars, content, and
   bottom panel remain docked.
4. **Status bar**: concise document/system state and panel toggles.
5. **Transient overlays**: palette, menus, and tooltips only.

Only the unified title bar, content, and a 24 px status bar are always visible.
The Command Manager collapses completely. The activity rail may auto-hide.
Sidebars and bottom panels are opt-in and docked.

### Content types

- **Start:** New tab page.
- **Browse:** folders and Downloads.
- **Web:** remote page.
- **Read/Edit:** text, code, DOCX, EPUB, PDF, image, DXF, audio, video.
- **Project:** `.unibrow` board, timeline, live studio.
- **System:** Settings, Mod Portal, download history.

These are tab content types, not separate apps.

### Saved layouts

- **Browse:** files/web navigation; minimal panels.
- **Write:** outline/files left; content centre; Problems bottom on demand.
- **Create:** sources left; preview centre; inspector right; timeline bottom.
- **Live:** sources left; programme preview centre; properties/chat right; mixer/status bottom.

These are saved arrangements reached from View > Workspaces and the palette.
They are not a permanent mode switcher. The active file type alone controls the
Command Manager. Layout changes neither file format nor open tabs.

## Evidence behind the compact chrome

The decision is synthesis, not imitation:

- Chromium defines tabs and the tab strip as browser-window chrome, including
  toolbar overlap and maximised hit targets. This supports tabs in the title
  region rather than a separate application row:
  [Chromium layout constants](https://chromium.googlesource.com/chromium/src/+/refs/tags/128.0.6613.18/chrome/browser/ui/layout_constants.h).
- Edge documents horizontal tabs at the top of the browser window as its
  compact default, with vertical tabs as an optional high-tab-count layout:
  [Edge horizontal and vertical tabs](https://www.microsoft.com/en-us/edge/learning-center/vertical-tabs-versus-horizontal-tabs).
- VS Code puts Command Center and layout controls in its custom title bar and
  supports classic, toggle, compact, and hidden menu presentation:
  [VS Code Custom Layout](https://code.visualstudio.com/docs/configure/custom-layout).
- Windows CommandBar defaults to a compact row of primary icons plus overflow;
  Microsoft explicitly recommends overflow and context-only commands when
  space is scarce:
  [Command bar](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar) and
  [Toolbar guidelines](https://learn.microsoft.com/en-us/windows/win32/uxguide/cmd-toolbars).
- Windows 11 File Explorer applies that model in practice: common operations
  remain immediate, compact file spacing is optional, and older commands remain
  under "Show more options":
  [File Explorer in Windows](https://support.microsoft.com/en-us/windows/experience/fileexplorer/file-explorer-in-windows).
- Office's Simplified Ribbon keeps common commands on one line, groups related
  commands under split buttons, and sends the remainder to overflow:
  [Use the Simplified Ribbon](https://support.microsoft.com/en-us/office/use-the-simplified-ribbon-44bef9c3-295d-4092-b7f0-f471fa629a98).
- Firefox Compact Mode reduces tab and toolbar spacing specifically to return
  room to content:
  [Firefox compact design](https://blog.mozilla.org/en/firefox/new-firefox-design-is-here/).
- Zed exposes title-bar, tab-bar, breadcrumb, and toolbar visibility
  independently. Its maintainers also document why per-pane tabs cannot always
  be merged into a per-window title bar; Unibrowser therefore keeps split-pane
  tabs local to the pane:
  [Zed visual customisation](https://zed.dev/docs/visual-customization) and
  [title-bar/tab-bar design discussion](https://github.com/zed-industries/zed/issues/5066).
- Arc proves the value of hiding a navigation surface and summoning actions
  from the command bar, but its sidebar combines tabs and spaces. Unibrowser
  does not copy that overlap: its rail only opens panels:
  [Arc Command Bar actions](https://start.arc.net/command-bar-actions).

Result: two rows and one context authority. The title row answers "what is open
and where am I?" The Command Manager answers "what can I do to this file?"
The rail answers "which supporting panel do I need?"

## Chrome region jobs

### 1. Unified title bar

Job: window movement/controls, tab identity, location, and global document
commands in one 40 px row.

Contains:

- Compact menu button. Alt reveals/focuses the full File/Edit/View/Go/Tools/Help
  menu bar without changing command taxonomy.
- Scrollable tabs. A split pane owns any secondary local tab strip.
- Flexible location/command field: breadcrumb for local files/folders, URL for
  web, friendly identity for internal pages.
- Undo, Redo, Save, layout overflow.
- Windows controls on the right in Windows order; Linux follows the desktop
  environment when server-side decorations are used.
- Explicit drag regions between controls. Interactive controls are excluded
  from drag hit testing.

Does not contain:

- Workspace modes, file-type modes, loaded mods, demo actions, contextual
  document tools, Save status, or AI grants.

### Tab behaviour

Job: identity and switching among all open locations.

Rules:

- File, folder, web, project, and system tabs use the same geometry.
- Icon identifies broad content class, not technical backend.
- Dirty state is a dot; pinned state is a pin icon plus tooltip.
- Audio/live activity has text/icon redundancy.
- Close button appears on active/hovered tabs; keyboard focus always exposes it.
- Overflow becomes a searchable tab list, grouped by window.

### Conditional navigation

Job: location and navigation.

Back, Forward, and Reload/Stop appear at the leading edge of the location field
only for web and folder tabs. Text, PDF, image, media, and project tabs do not
pay for browser navigation they cannot use. Find is a Command Manager command
for the active file type. Downloads progress appears in the status area and
Downloads tab, not as a permanent button.

### 2. Command Manager

Job: visible, context-relevant commands with stable placement.

Structure:

- One grouped row: 40 px Compact (default) or 64 px Large.
- Active file-type context label at the left, followed by named command groups
  with Lucide icon + label controls.
- **Following: Text / PDF / Image / Create** and **Pin** at the right.
- Context choices may open from the context label for preview/customisation, but
  no second permanent tab row exists.
- The default context follows the active tab's file type automatically.

Interpretation of "grey out, do not hide":

- Commands in the active/pinned context remain in fixed slots and grey out.
- Other file-type contexts do not occupy permanent chrome. They remain
  reachable through the context menu and customisation.
- Disabled tooltip states cause and remedy: "Increase font size is unavailable: this image has no text layer."
- Split buttons keep a default action and expose variants from a separate
  chevron target. Least-used complete groups enter overflow before any group
  wraps or breaks apart.

Pin model:

- **Following: PDF** is a quiet status in the tab row.
- Clicking another context tab previews it.
- **Pin** is a labelled toggle adjacent to the status.
- Pinned state reads **Pinned: Image** and survives tab changes.

### 3. Activity rail and work area

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

- Rail entries open side panels only. Core entries: Files, Search, Project,
  Assets, Settings. Extensions may add a destination only for a distinct panel.
- CAD, Video, Audio, Code, and Paint are not rail entries merely because those
  file types exist.
- Rail modes: expanded 188 px, collapsed 40 px, auto-hide, hidden.
- Sidebars push content.
- Minimum content width is 480 px. Below it, only one sidebar may be open; opening another replaces the current sidebar.
- Bottom panel pushes content upward.
- Sidebar/panel headers contain title, tab switch if needed, More, Close.
- Detached panels are owned windows and never always-on-top by default.

### 4. Status bar

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

### Native project storage mode (`.unibrow`)

[ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md) defines three modes; the user always sees which one is active.

**Status bar (left slot, high priority when a native project tab is active):**

| Mode | Primary text | Tooltip |
| --- | --- | --- |
| View only | `View only` | No changes are written. Choose **Start editing** or **Save As** to save work. |
| Work in archive | `Archive` | Editing `filename.unibrow` in place. |
| Folder mode | `Folder` | Project data in `.unibrow/` inside this folder. |

After the user’s first mutating edit (or **Start editing** from view only), append a second left item until dismissed or saved:

- **History will be saved in** `<absolute or project-relative path>`  
  (archive path to the `.unibrow` file, or `…/ProjectName/.unibrow/` in folder mode.)

**Large project hint (non-blocking):** when the project exceeds the configured threshold ([ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md) §4), show a `kind: warning` right-slot item:

- Copy: **This project is large. Folder mode can be faster for big media.**  
- Action: **Unpack to folder** (command link; does not switch mode without confirmation.)

Never auto-switch to folder mode. Threshold off → no hint.

**Commands (File menu and palette; stable ids TBD in implementation):**

| Label | Behaviour |
| --- | --- |
| **Unpack to folder** | Explicit conversion from archive to folder project; confirmation names destination. |
| **Pack as .unibrow** | Build OPC `.unibrow` from folder project; confirmation names output file. |
| **Collect into project** | Copy linked media into `assets/` (embed); grey when nothing is referenced. |

Mode indicator stays visible during pack/unpack; Save is never covered by the hint.

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

- Unified-title-bar focus order is menu, tabs, conditional navigation,
  location/command field, Undo, Redo, Save, layout overflow, then window
  controls where the platform exposes them to web content.
- Alt focuses/reveals the canonical menu; arrow keys traverse menus and tabs.
- Every icon-only rail/title action has an accessible name and tooltip.
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

- Regions have landmarks and names: Window navigation, Tabs, Location,
  Command Manager, Activity rail, Left sidebar, Document, Right sidebar,
  Bottom panel, Status.
- The custom title bar exposes separate named landmarks for menu, tabs, and
  location even though they share one visual row.
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
- Use familiar Windows menus, shortcuts, right-aligned minimise/maximise/close,
  snap-layout-compatible maximise behaviour, drag regions, and density.
- Linux defaults to server-side decorations when required by the compositor.
  With client-side decorations, control side/order follow the desktop setting;
  never hard-code Windows controls on Linux.
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
