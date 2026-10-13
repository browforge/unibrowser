# UI implementation plan

Ordered Composer-sized tickets. Each ticket should land independently with `npm run build`, relevant Rust tests, and the named screenshot check. Product code changes should not start before [UX](ux.md), [layout](layout.md), and [UI spec](ui-spec.md) are accepted.

## Definition of done for every ticket

- Keyboard and pointer paths work.
- Light, dark, and high-contrast checked where relevant.
- 100% and 150% Windows scaling checked.
- No new modal, content overlap, or duplicate command implementation.
- Existing command ids are reused.
- Screenshot at 1280x800; wide screenshot when layout changes.
- Screenshot passes every applicable item in [visual-language.md](visual-language.md#screenshot-drift-checklist); deliberate exceptions are documented.

## Phase A: remove prototype clutter and establish regions

### UI-01: Remove developer clutter from primary chrome

Likely files:

- `crates/unibrowser-desktop/src/main.ts`
- `crates/unibrowser-desktop/src/index.html`
- `crates/unibrowser-desktop/src/styles.css`
- command/menu registration files in `src-tauri/src/commands.rs`
- `settingsPage.ts`

Work:

- Remove loaded `Mods: ...` banner.
- Remove **Demo download** from normal chrome.
- Remove permanent Export layout / Import layout buttons.
- Remove duplicate Home, Downloads, Download manager, New project, Settings row.
- Keep commands available in palette/menus.
- Add Developer settings section for mod inventory, demo download, layout import/export, capture diagnostics.

Acceptance:

- This cleanup remains valid, but Batch 1 chrome geometry is explicitly
  temporary and will be replaced by UI-07.
- No debug/demo text in default shell.
- Every removed action remains reachable through a canonical command.

Screenshot:

- 1280x800 New tab with clean top chrome.

### UI-02: Dock AI as a right sidebar

Likely files:

- `src/aiAssistantPanel.ts`
- `src/main.ts`
- `src/styles.css`
- shell HTML/layout module

Work:

- Replace absolute/floating panel with right dock region.
- Opening AI resizes content.
- Add sidebar header, close, resize, width persistence.
- Put provider/model and grant summary in header.
- Collapse tool calls under messages; remove permanent exhaustive command list.
- Add one-sidebar-at-a-time behaviour under 1024 px.

Acceptance:

- AI never covers toolbar or content.
- Content remains at least 480 px or switches to single-pane rule.
- Focus returns to document on close.

Screenshot:

- Text file + AI at 1280x800; `.unibrow` + AI at 1600x1000.

### UI-03: Introduce shell region CSS and token foundation

Likely files:

- `src/styles.css`
- new `src/ui/tokens.css`
- `src/main.ts`
- `src/index.html`

Work:

- Define named grid regions.
- Add colour/type/spacing/radius/elevation tokens from `ui-spec.md`.
- Add Compact/Large density variable.
- Replace one-off shell colours/sizes only; do not restyle every viewer yet.

Acceptance:

- Stable region dimensions match `layout.md`.
- Light/dark tokens pass contrast spot checks.
- 125/150/200% scaling does not clip shell regions.

Screenshot:

- Annotated 1280 shell-region screenshot in light/dark.

### UI-04: Canonical menus and title-bar command model

Likely files:

- `src/main.ts`
- `src/chromeI18n.ts`
- `src/i18n/*.json`
- command registry/menu bindings

Work:

- Keep File/Edit/View/Go/Tools/Help as the canonical taxonomy.
- Add a compact menu button for the future unified title bar while retaining
  full Alt-key menu access and arrow-key traversal.
- Define title-bar global commands: Save and layout overflow. Undo and Redo
  remain in the context Command Manager to avoid duplicate controls.
- Keep Back, Forward, Reload/Stop conditional to web and folder tabs.
- Move Home/Settings/Mods/Layout actions into menus and palette.
- Put saved layouts under View > Workspaces and the palette; do not add a
  permanent workspace switcher.
- Keep Windows/Linux menu conventions.

Acceptance:

- No duplicated action appears in permanent chrome.
- All menu entries execute registry commands.
- Alt/menu keyboard access works.
- Windows and Linux window controls, drag regions, and focus order remain
  platform-correct.

Screenshot:

- View menu open, showing Panels, Workspaces, Customise.

## Phase B: Command Manager and tab/navigation clarity

### UI-05: Rebuild Command Manager as one contextual row

Likely files:

- `src/commandManager.ts`
- `src/styles.css`
- `src-tauri/src/ui_command.rs`
- `src-tauri/src/commands.rs`

Work:

- Replace the separate context-tab and command rows with one grouped Command
  Manager row.
- Active file type selects the context automatically.
- Put the context label at the left, commands next, and Follow/Pin at the right.
- Add explicit Following/Pinned status and labelled Pin toggle.
- Add Compact (40 px, default) and Large (64 px) densities. Compact uses
  16 px Lucide icon + short label inline. Large uses 20 px icons with labels
  below, like a restrained workstation ribbon.
- Arrange commands in named groups with one divider. Text: History, Type, Find,
  Proofing, Export. CAD/3D: Select, View, Measure, Section, Export.
- Add split buttons for variants and an overflow chevron.
- Prevent wrapping. Move the least-used complete group into overflow first;
  never split a group or displace context / Follow / Pin.
- Keep disabled commands greyed in place with their reason in a keyboard-
  reachable tooltip.
- Remove dashed debug container.
- Remove the stray blank control after Find.

Acceptance:

- Context-menu selection previews without pinning.
- Pin persists.
- Slots do not shift when disabled.
- No second context switcher is visible.
- The row collapses completely with Ctrl+F1.
- Compact totals 80 px of chrome; Large totals 104 px; collapsed totals 40 px.
- At 800, 1024, and 1280 widths, groups enter overflow in the specified order
  without wrapping.
- Split-button main and chevron targets are separately keyboard reachable and
  screen-reader named.

Screenshot:

- Text, PDF, and CAD/3D contexts in Compact and Large density. Text Large shows
  a disabled Format command and its reason.

### UI-06: Disabled reasons and command tooltip

Likely files:

- `src/commandManager.ts`
- palette component
- command DTO/enabled reason fields

Work:

- Standard tooltip with label, reason, remedy, shortcut.
- Reuse reason in palette.
- Ensure disabled buttons do not claim hover/press execution.

Acceptance:

- All disabled first-party commands have human-readable reasons.
- Tooltip is keyboard reachable and screen-reader described.

Screenshot:

- Disabled font command on plain image with reason.

### UI-06A: Restyle the Batch 1 shell foundation

Batch 1 established temporary regions before the visual language was binding.
Its menu/tab/navigation/Command Manager frame is superseded by UI-07; do not
polish that geometry further.

Likely files:

- `src/ui/tokens.css`
- `src/styles.css`
- `src/appMenu.ts`
- `src/commandManager.ts`
- `src/aiAssistantPanel.ts`
- shell HTML and icon imports

Work:

- Replace the Batch 1 token values with the Quiet Utility palette, Source Sans
  3 / Source Code Pro stacks, 4 to 6 px radius scale, and WCAG AA text roles.
- Restyle non-chrome shared surfaces, docked AI, panels, buttons, inputs, and
  status bar as joined workbench regions.
- Replace ad hoc text/symbol icons with Lucide at the specified sizes.
- Remove shadows from docked regions and redundant borders between adjacent
  regions.
- Preserve every Batch 1 interaction, shell audit, and stale-bundle guard.

Acceptance:

- Existing Batch 1 self-tests and release frontend stamp verification pass.
- No control or panel radius exceeds 6 px.
- Docked regions have no shadow and no doubled boundaries.
- Secondary text and control boundaries meet WCAG AA.
- Light, dark, and high-contrast screenshots pass the visual-language drift
  checklist.

Screenshot:

- Text document at 1280x800 with Command Manager and docked AI in light and dark.

### UI-07: Unified compact title bar and activity rail

**Visual-language impact:** Replaces Batch 1 chrome. One file-type context,
two rows above content, Lucide activity rail, no permanent workspace switcher.

Likely files:

- shell/title-bar rendering in `src/main.ts`
- `src/appMenu.ts`
- `src/commandManager.ts`
- `src/styles.css`
- omnibox suggestion module
- window decoration / platform integration

Work:

- Merge compact menu access, tabs, conditional browser navigation,
  location/command field, Undo/Redo/Save, layout overflow, drag regions, and
  platform window controls into one 40 px custom title-bar row.
- Keep full keyboard menus: Alt reveals/focuses
  File/Edit/View/Go/Tools/Help.
- Use one grouped Command Manager row below it: 40 px Compact (default) or
  64 px Large. Context label and commands share the row; Follow/Pin stays at
  the right.
- Active file type automatically selects the context. Remove the permanent
  Browse/Text/Create/Live switcher.
- Restrict the left rail to Files, Search, Project, Assets, Settings, and
  installed views that open a genuinely distinct panel. Remove CAD/Video/
  Audio/Code/Paint mode entries.
- Add Lucide rail icons and Expanded (188 px), Collapsed (40 px), Auto-hide,
  and Hidden options.
- Put saved layouts in View > Workspaces and the palette.
- Preserve dirty/pinned/audio/LIVE redundancy, tab search, breadcrumb/full-path
  editing, and secure origin identity.
- Back/Forward/Reload appear only for folder and web tabs.
- Ensure split panes own local secondary tabs instead of forcing all pane tabs
  into the window title bar.

Acceptance:

- Expanded Compact chrome is exactly 80 px above content; Large is 104 px;
  collapsed is 40 px.
- 12 tabs remain operable at 1280.
- Keyboard tab movement and close work.
- Local path and URL identities are unambiguous.
- No overlapping context switcher exists.
- Rail switching changes a panel, never the active file context or layout.
- Windows 11 and Linux controls, snap/maximise behaviour, drag regions,
  screen-reader names, and focus order pass.
- Chrome never covers content.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- Mixed tabs at 1280 with Text context and collapsed rail; folder tab showing
  navigation; expanded/auto-hidden rail states; Windows and Linux variants.
- Annotated before/after chrome-height comparison.

### UI-08: Full command palette

**Visual-language impact:** Revised. This is a permitted elevated overlay: 6 px radius, one border, and the specified functional shadow.

Likely files:

- palette code in `src/main.ts` or extracted `src/commandPalette.ts`
- `src/styles.css`
- command/search DTOs

Work:

- Group Commands, Open, Settings, Help.
- Add result labels, context/path, shortcuts, disabled reasons.
- Position relative to content region.
- Focus restore and Escape behaviour.

Acceptance:

- Palette is fully keyboard navigable.
- Disabled commands explain but do not execute.
- No conflict with sidebars.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- "font" query with enabled and disabled results.

## Phase C: panels, feedback, and ordinary journeys

### UI-09: Standard docked panel component

**Visual-language impact:** Revised. Docked joins use zero radius and no shadow; surfaces and one purposeful divider establish the workbench.

Likely files:

- new `src/ui/DockPanel.ts` or equivalent
- `src/problemsPanel.ts`
- terminal/output modules
- `src/styles.css`

Work:

- Header, tabs, resize, close, empty/loading/error.
- Bottom panel persistence.
- F6 region cycling.

Acceptance:

- Problems/Output/Terminal share one shell.
- Panel pushes content.
- Resize works by mouse and keyboard.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- Problems open at 1280 with editor visible above.

### UI-10: Problems redesign

**Visual-language impact:** Revised. Rows use compact density and selection surfaces, not card wrappers or a divider after every item.

Likely files:

- `src/problemsPanel.ts`
- diagnostics DTOs if needed
- `src/styles.css`

Work:

- Integrate filters into header.
- Structured rows with severity, message, source, location.
- Selection detail/actions.
- Remove full error sentences from status bar.

Acceptance:

- Selecting a row navigates.
- Empty/filter/no-results states differ.
- 200+ entries remain responsive.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- Spelling + code/project problems, selected row.

### UI-11: Feedback ladder

**Visual-language impact:** Revised. Feedback uses semantic colour with text/icon redundancy; only toasts receive functional elevation.

Likely files:

- toast module
- recovery bar
- unknown-file banner
- downloads progress
- common styles

Work:

- Standard toast, inline bar, progress, error details.
- Audit existing alerts/notices into correct rung.
- Guarantee no feedback covers caret/live header.

Acceptance:

- Close tab Undo, recovery, unknown handler, failed export, completed download each use specified surface.
- No modal/alert/confirm/prompt.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- Recovery inline bar and Closed tab Undo toast.

### UI-12: New tab and Downloads

**Visual-language impact:** Revised. The New tab page is an organised work surface, not a dashboard of rounded cards or a performative empty state.

Likely files:

- `src/newTabPage.ts`
- downloads page/module
- `src/styles.css`

Work:

- Open/New project primary actions.
- Recents and active/recent downloads.
- Remove widget/dashboard clutter.
- Downloads manager remains a system tab.

Acceptance:

- Useful empty state at first launch.
- Download opens new tab by default.
- No account/news/promo.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- Empty first launch; populated recents/downloads.

## Phase D: context layouts

### UI-13: File browser and settings layouts

**Visual-language impact:** Revised. Rails, lists, and settings groups use spacing and surface roles; cards are reserved for independent objects.

Status: done 2026-10-07 (drift pass same day). Browse CM owns sort, hidden
files, and set/clear workspace; content pane has no duplicate controls; file
names sit beside type badges with uniform row height; Places recent list is one
line with ellipsis; filter uses theme field tokens. Settings and Markdown
evidence unchanged. All 11 captures re-run with integrity checks; grades in
`docs/design/ui-13-evidence.md`.

Likely files:

- `src/filesPanel.ts`
- `src/settingsPage.ts`
- settings section modules
- styles

Work:

- File tree/list/preview docking.
- Settings category rail, search, setting-row component.
- Move Developer section content from UI-01.

Acceptance:

- Settings is usable at 800-1600 px.
- File actions live in Browse context/context menus.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- File browser with preview; Settings > Developer collapsed/expanded.

### UI-14: PDF/image/DXF context shells

**Visual-language impact:** Revised. Canvas surrounds are recessed wells, property panels are docked workbench surfaces, and annotations use semantic colour sparingly.

Status: done 2026-10-07 (drift pass same day). Image paths infer Image work
context; Find, Check, and Curves grey out on SVG with reasons; zoom uses
magnifier icons; Preview/Source is a flat segmented control. PDF and DXF shells
unchanged. Three 1280x800 captures re-run; grades in `docs/design/ui-14-evidence.md`.

Likely files:

- `src/documentViewer.ts`
- viewer-specific renderers
- Command Manager defaults

Work:

- Standard optional left structure/pages, centre canvas, right properties/comments.
- Align Find/Zoom/Export.
- Use disabled reasons for unsupported edits.

Acceptance:

- Switching among PDF/image/DXF preserves shell and expected universal-command positions.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- Three 1280 context comparisons.

### UI-15: `.unibrow` Create workspace

**Visual-language impact:** Revised. Timeline tracks are recessed wells; source, preview, inspector, and timeline are joined workbench regions without floating cards.

Status: done 2026-10-07 after workshop redesign. One fixed Sources / 16:9 Stage
/ Inspector grid sits above a three-track NLE timeline. Clip blocks are
duration-proportional and support move/trim; exact timing is only in Inspector.
The Project CM replaces Browse and Export opens in Inspector. Four light/dark,
1600/1280 captures pass in `docs/design/ui-15-evidence.md`. Normative rationale:
`docs/design/workshop-layout.md`.

Likely files:

- `src/documentViewer.ts`
- timeline/board modules
- workspace layout state

Work:

- Sources left, preview centre, Inspector right, timeline bottom.
- Saved-layout switch from View/palette without closing/changing tabs.
- Keep the one global primary Save distinct from contextual Export.

Acceptance:

- Title-card journey completes without modal.
- Missing source appears in timeline + Problems; Save stays enabled.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- 1600x1000 project with title selected.

### UI-16: Live workspace

**Visual-language impact:** Revised. Red is reserved for LIVE/recording/error and all broadcast states remain clear without colour.

Status: done 2026-10-07 after workshop redesign. Preview, LIVE, and Recording
are modes of the same Stage; LIVE/REC use restrained red state labels and
Inspector switches to source health and Mixer. Eight LIVE/Recording light/dark,
1600/1280 captures pass in `docs/design/ui-16-evidence.md`.

Likely files:

- `src/documentViewer.ts`
- live UI modules
- `src/styles.css`
- status bar

Work:

- Persistent studio header.
- Separate Preview, LIVE, Recording state.
- Source health and mixer.
- Status redundancy.

Acceptance:

- LIVE cannot be mistaken for Preview or Recording in greyscale.
- No notification/sidebar covers programme preview/header.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- Preview, LIVE, Recording states at 1600x1000.

### UI-17: Mod Portal and unknown-handler journey

**Visual-language impact:** Revised. Mod results may use independent 6 px cards; permission and risk states use restrained semantic colour and explicit labels.

Status: done 2026-10-07. Unknown files retain their file surface beside a
joined support dock with text, hex, suggested-handler, and Mod Portal paths.
Discover uses labelled result cards with source, licence, and permission
language; sideloading is contained in Developer details. Both 1280x800
captures pass in `docs/design/ui-17-evidence.md`.

Likely files:

- mod portal renderer
- unknown-file banner
- mod DTOs
- settings/mod styles

Work:

- Standard cards and capability language.
- Installed/Discover/Updates.
- Permissions/licence/source.
- Open support path beside unknown file.

Acceptance:

- Install occurs without obscuring/replacing current file.
- Sideload remains available under More/Developer.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- Unknown file + support sidebar; Mod Portal search result.

## Phase E: customisation, accessibility, polish

### UI-18: Saved layout customisation

**Visual-language impact:** Revised. Docking targets are functional overlays, with no decorative glow, glass, or oversized rounded drop zones.

Status: Drift 2026-10-07. Explicit layout edit mode now gates command dragging,
offers functional left/right/bottom dock targets, saves named layouts, and
imports or exports workspace-layout files. Light and dark captures pass
overlap and clipping checks. Detached-toolbar and two-monitor restart evidence
is still unavailable in the single-display capture environment; see
`docs/design/ui-18-evidence.md`.

Likely files:

- `src/commandManager.ts`
- `src/workspaceLayoutPanel.ts`
- `src-tauri/src/toolbar_windows.rs`
- preferences/layout persistence

Work:

- Explicit edit mode, drag handles, docking overlays.
- Named layouts and import/export management.
- View > Workspaces and palette entry; no permanent switcher.
- Monitor-loss recovery.

Acceptance:

- Two-monitor detach/restart test.
- No accidental drag from command body.
- Screenshot passes the visual-language drift checklist.

Screenshot:

- Edit mode with docking targets; detached toolbar window.

### UI-19: Accessibility and scaling pass

**Visual-language impact:** Revised. Enforce Source Sans 3 minimum sizes, WCAG AA contrast, system-colour high contrast, and non-colour state redundancy.

Status: Drift 2026-10-07. Light and dark high-contrast captures at 150% show
a complete keyboard focus ring, labelled tooltip, readable editor, and no
region clipping. Automated capture diagnostics report all nine visible Command
Manager controls above the contrast threshold. Windows scale, Linux 2x, axe,
and screen-reader checks remain outstanding; see
`docs/design/ui-19-evidence.md`.

Likely files:

- all shell components
- i18n copy
- automated a11y test config

Work:

- Landmarks, roles, labels, described disabled states, focus order.
- High-contrast tokens.
- Windows 100/125/150/200%; Linux 1x/2x.
- Reduced motion and touch mode.

Acceptance:

- Keyboard-only top journeys.
- Axe/manual screen-reader pass.
- No clipping at required scales.
- Screenshot passes the visual-language drift checklist in light, dark, and high contrast.

Screenshot:

- HC at 150%; focus-visible keyboard sequence.

### UI-20: Remove legacy CSS/components and update evidence

**Visual-language impact:** Revised. Remove legacy styling that exceeds the radius scale, introduces card nesting, duplicates dividers, or uses non-semantic colour.

Status: Drift 2026-10-07. The active shell passes 11 self-test scenarios on
three consecutive runs, `npm run build` passes, and `cargo test --workspace`
passes in normal parallel mode. Superseded workshop shell CSS was removed and
the final 1280/1600 workshop matrix passes visible integrity checks. The
inactive legacy project renderer and some old timeline rules remain in source,
so cleanup is not complete; see `docs/design/ui-20-evidence.md`.

Likely files:

- `src/styles.css`
- obsolete panel/toolbar modules
- screenshots/docs/self-test selectors

Work:

- Delete old raw button rows, floating AI CSS, debug banners.
- Update self-test selectors and doc capture scenes.
- Replace README screenshots only with feature-visible captures.

Acceptance:

- No legacy duplicate chrome in DOM.
- Ten self-test scenarios 3x green.
- `cargo test --workspace`, `npm run build` green.
- Final screenshot matrix passes the visual-language drift checklist with no undocumented exceptions.

Screenshot:

- Final 1280 and 1600 shell matrix matching `mockups/context-matrix.png`.

## Mockup-parity remediation: UI-21+

Source audit: [mockup-parity-audit.md](mockup-parity-audit.md). Ticket order is
visible-impact order. Each acceptance capture must also regenerate its labelled
comparison in `docs/screenshots/parity/`.

### UI-21: Restore Settings task hierarchy

Status: done 2026-10-07. Category rail search, page title and subtitle, aligned
field rows, Developer callout, and bookmark bar moved to Developer with working
chrome when enabled.

Work:

- Done: remove document-local actions, including Print preview, while Settings
  owns the work area.
- Add the approved Settings page title, section title, subtitle, and aligned
  setting-row hierarchy.
- Keep the category rail compact and text-led; preserve Developer disclosure.

Acceptance shot:

- `settings-light-1280.png` beside `mockups/settings-mods-1280.png`.

Fail rule:

- Fail for any unrelated document toolbar, clipped category or field, mixed
  native/custom field treatment, or helper text below 4.5:1.

### UI-22: Restore the Image workbench regions

Status: done 2026-10-07. History and Adjust docks flank the fitted raster canvas; empty
states are truthful; doc-capture resolves repo-relative `--open` paths.

Work:

- Provide stable History and Adjust docks around the raster canvas.
- Show truthful no-history and no-selection states without shrinking the image
  below the 480 px content rule.

Acceptance shot:

- `image-light-1280.png` with History, fitted canvas, and Adjust visible.

Fail rule:

- Fail if either required dock is absent, covers the image, duplicates a
  Command Manager action, or contains unreadable disabled text.

### UI-23: Match New tab action and download hierarchy

Status: done 2026-10-07. Warm 784 px work column, four recent rows, compact download
rows with truthful in-progress percent, primary Open actions strengthened.

Work:

- Strengthen Open file, Open folder, and New project as the primary action set.
- Replace filled download bands with compact rows, real progress, and trailing
  actions aligned with section headers.
- Match the approved work-column width and warm surface.

Acceptance shot:

- `new-tab-light-1280.png` with recent files, one active download, and one
  completed download.

Fail rule:

- Fail for card-grid treatment, a performative empty state, clipped metadata,
  duplicated Open actions, or download progress represented inaccurately.

### UI-24: Match Workshop stage and timeline proportions

Status: done 2026-10-07. Programme frame with selected lower third, Heading/Subtitle/
Duration/Position inspector, timecode transport under stage, lighter 1280 timeline tracks.

Work:

- Rebalance Sources, Stage, Inspector, and Timeline to the approved 1280
  proportions.
- Use representative programme content and a selected title overlay.
- Align timeline and Inspector type levels without increasing chrome height.

Acceptance shot:

- `workshop-light-1280.png` with a selected title and visible programme frame.

Fail rule:

- Fail for covered preview content, clipped tracks or Inspector fields,
  duplicated timeline controls, or status text below 4.5:1.

### UI-25: Clarify PDF annotation selection

Status: done 2026-10-07. Right dock shows one selected-annotation inspector with
editable fields; canvas selection outline; other annotations listed quietly.

Work:

- Make one annotation visibly selected and use the right dock as its inspector.
- Reduce repeated card borders and restore selected-tool, field-label, and
  annotation-action hierarchy.
- Keep page shadow as the only work-area elevation.

Acceptance shot:

- `pdf-light-1280.png` with one selected annotation and its editable inspector.

Fail rule:

- Fail if selection relies on colour alone, page or dock content clips, all
  annotations receive equal emphasis, or page controls are duplicated.

### UI-26: Keep File browser preview present at 1280

Status: done 2026-10-07. Preview pane stays visible at 1280 with selection capture,
lighter type badges, and useful unsupported-preview copy.

Work:

- Preserve Places, file list, and Preview as joined regions at 1280.
- Give no-selection and unsupported-preview states literal, useful copy.
- Reduce file-type badge weight and redundant row rules.

Acceptance shot:

- `file-browser-light-1280.png` with one file selected and Preview visible.

Fail rule:

- Fail if Preview disappears, list content falls below 480 px without applying
  the documented close/replace rule, or path/action controls duplicate.

### UI-27: Align document workbench regions

Status: done 2026-10-07. Removed the permanent Preview/Export/Print row; editor,
AI, and Problems share one capture with joined regions.

Work:

- Remove or merge the third Preview/Export/Print row into the contextual command
  model.
- Prove editor, AI, and Problems as joined regions in one current capture.
- Match approved warm editor and panel-header hierarchy.

Acceptance shot:

- `document-problems-ai-light-1280.png` with editor, AI, and three Problems rows.

Fail rule:

- Fail for a third permanent chrome row, floating or covering panels, raw
  provider text, clipped editor content, or unreadable diagnostics.

### UI-28: Reduce Mod Portal card-grid density

Status: done 2026-10-07. Task header merges search and network state; one featured
suggested result sits above a restrained catalogue.

Work:

- Make search and network state one task header.
- Establish clear title, package, source, licence, permission, and action levels.
- Keep one focused result before the wider catalogue and expose truthful empty
  and network-error states.

Acceptance shot:

- `mod-portal-light-1280.png` with one suggested result and a restrained
  catalogue below.

Fail rule:

- Fail if every result has equal emphasis, metadata is below 12 px or 4.5:1,
  install state relies on colour, or cards clip at 1280.

### UI-29: Guard status-bar semantic contrast

Status: done 2026-10-07. `AI: Read` now uses the theme accent token rather than
the legacy pale accent. Capture diagnostics record every visible status segment
and the PNG pixel guard rejects any segment below 4.5:1.

Acceptance shot:

- Light `new-tab-light-1280.png` and `workshop-light-1280.png`, with sibling
  diagnostics reporting status segment pixel contrast.

Fail rule:

- Fail if any visible status label samples below 4.5:1, if status text overlaps
  or clips, or if semantic state relies on colour alone.

## Command System: CMD series

These tickets implement [command-system.md](command-system.md) and
[ADR 0027](../adr/0027-command-system-capabilities-and-extensibility.md).
Order is architectural: registry/capability core, components, Command Manager
rendering, mod API, then layout persistence.

### CMD-01: Versioned command and capability core

Work:

- Add stable data types for Command, Control, Group, Family, Tab, and Context.
- Add capability states: supported, emulated, unsupported, hidden-by-user.
- Keep mixed selection orthogonal to capability.
- Add context revision and stale-response rejection.
- Adapt current Enabled/Disabled handlers without changing behaviour.
- Prove that capability resolution cannot affect Save, Save As, autosave, or
  recovery.

Acceptance:

- Resolver unit tests cover family defaults, per-control overrides, mixed
  state, emulation notes, missing providers, and stale callbacks.
- Unsupported results always carry a user-facing reason.
- Existing command IDs and handlers continue to execute.

### CMD-02: Provider resolution and conflict diagnostics

Work:

- Implement user/backend/mod/emulation provider precedence.
- Add deterministic dependency, priority, and lexical tie-breaking.
- Record considered providers and winner reason.
- Add callback timeout, cancellation, failure fallback, and diagnostics.

Acceptance:

- Load order alone cannot change the winning provider.
- Provider failure degrades to unsupported without blocking document input.
- Conflict output identifies contribution IDs and remediation.

### CMD-03: Host control component primitives

Work:

- Build host-rendered button, toggle, split button, dropdown, value stepper,
  colour, gallery, segmented, menu button, and indicator-action components.
- Add Compact/Large, icon/label, supported/emulated/unsupported, mixed, and
  value variants.
- Implement one tooltip/description contract for unsupported and emulated
  states.

Acceptance:

- Component gallery matches the design sheet in light, dark, and high contrast.
- No component permits arbitrary mod HTML or CSS.
- Emulated and mixed states are readable without colour.

### CMD-04: Keyboard, palette, and accessibility integration

Status: done 2026-10-07. CM roving focus (`cmRovingFocus.ts`), palette
merges resolved controls (`merge_resolved_command_palette`), unsupported
controls remain focusable with `aria-disabled`, and live announcements cover
emulated, unsupported, and mixed states. Evidence:
`tests/controlA11y.test.mjs`, `keybinding::tests::user_can_bind_palette_to_plain_s`,
`npm run build`, and `npm run test:overflow`.

Work:

- Add roving group focus, arrow/Home/End, Enter/Space, Alt+Down, and Escape.
- Render the same resolved model in the command palette.
- Expose roles, names, descriptions, shortcuts, mixed values, emulation notes,
  and unsupported reasons.
- Preserve Ctrl+Shift+P; allow user-defined S binding.

Acceptance:

- Every visible control is reachable by keyboard and palette.
- Unsupported controls explain but do not execute.
- Screen-reader checks announce emulated, unsupported, and mixed states.

### CMD-05: Family and group composition

Work:

- Define core Text Formatting, History, Find, Proofing, and Export families or
  groups with canonical order and extension points.
- Add text family context defaults for plain text, Markdown, rich document,
  and code.
- Add PDF Annotation, Image, and CAD/3D family definitions.

Acceptance:

- The same family retains order across all worked contexts.
- Strikethrough remains visible and unsupported in plain text.
- Markdown Subscript resolves emulated with its approximation note.

### CMD-06: Command Manager resolved rendering

Work:

- Replace slot-as-button rendering with ResolvedControl rendering.
- Keep context, Following/Pin, family/group order, and complete-group overflow.
- Implement the `14 pt` value stepper from registry state.
- Keep unsupported controls greyed in place.

Acceptance:

- No wrapping or third row at 800, 1024, 1280, and 1600 px.
- Complete groups/families enter overflow in deterministic order.
- Component positions do not shift when support or selection changes.

### CMD-07: Mod contribution manifest schema

Work:

- Add versioned optional `contributes` metadata for commands, controls, groups,
  families, tabs, toolbars, extension points, and capability providers.
- Validate namespaces, IDs, references, insertion constraints, and schema
  compatibility before loading payload callbacks.
- Keep manifest declarative and executable behaviour in native/WASM payloads.

Acceptance:

- Invalid contributions isolate safely and do not prevent unrelated mods or
  the shell from loading.
- Core `unibrowser.*` namespace cannot be claimed by mods.
- Schema migration and compatibility fixtures cover current manifests.

### CMD-08: Runtime mod capability and emulation API

Status: done 2026-10-07. `ModRuntimeRegistry`, versioned family/control
snapshots, typed errors/deadlines, declared permission checks, wasmi fuel, and
typed host edit requests are live. Markdown Subscript uses one host text
transaction and undo step. Evidence:
`real_wasm_snapshot_matches_native_proving_runtime`,
`real_wasm_proving_mod_loads_through_manifest_host_path`,
`runtime_cannot_override_save_control`, and
`host_wrap_transaction_is_one_undo_step`.

Work:

- Add versioned runtime callbacks for family/control capability, selection
  state, value, emulation, and execution.
- Add permissions for capability provision, selection read, document edit,
  Command Manager contribution, toolbar contribution, and detachment.
- Route edits through host transactions and undo.

Acceptance:

- Native and WASM proving mods expose equivalent resolved data.
- Capability callbacks are read-only and cannot cover content or block Save.
- Markdown Subscript proving mod executes one undoable emulation.

### CMD-09: Mod groups, families, and toolbars

Status: done 2026-10-07. Contributions merge in stable mod/ID order;
duplicates, order cycles, and keybinding displacement produce sorted advisory
diagnostics exposed non-modally on the Command Manager context control.
Detached bars persist monitor name/optional EDID and recover to the nearest
visible monitor with screen clamping. Evidence:
`order_cycle_has_sorted_diagnostic`,
`conflict_diagnostic_is_deterministic_and_displaces_old_binding`,
`missing_monitor_recovers_to_nearest_visible_monitor_and_clamps`, and
`docs/screenshots/cmd/cmd-proving-detached-light.png`.

Work:

- Merge extensions at declared family extension points.
- Render mod-contributed groups and host-owned toolbars.
- Support floating and detached toolbar windows on other monitors.
- Enforce reserved content, Save geometry, focus, and screen-bound recovery.

Acceptance:

- A proving mod adds one core-family control and one detached toolbar.
- Duplicate IDs, order cycles, and keybinding conflicts have deterministic
  diagnostics.
- Toolbars cannot cover content or strand themselves off-screen.

### CMD-10: Versioned layout persistence and migration

Work:

- Persist customisation per context and per family using stable IDs.
- Persist hidden-by-user separately from capability.
- Persist control variants, provider choices, group/family order, pinned tabs,
  and monitor anchors.
- Migrate current `toolbars.json`; preserve orphaned mod contributions.

Acceptance:

- Restart and monitor-loss tests restore safely.
- Removing and reinstalling a compatible mod restores customisation.
- Reset works per control, family, context, toolbar, and entire system.

### CMD-11: Worked-context integration and evidence

Status: done 2026-10-07. Contract tests in
`worked_context_contract.rs`, evidence matrix in `docs/screenshots/cmd-11/`,
grid in `docs/design/cmd-11-evidence.md`, mod authoring in
`docs/mods/command-system-authoring.md`. CAD/3D screenshots pending (path detect
only).

Work:

- Integrate plain text, Markdown, rich document, code, PDF annotation, image,
  and CAD/3D proving contexts.
- Add API contract tests, keyboard tests, accessibility tests, and screenshot
  matrices.
- Document provider resolution and trust diagnostics for mod authors.

Acceptance:

- Worked examples in `command-system.md` match runtime resolved models.
- Light, dark, high contrast, 100%, and 150% evidence passes drift checks.
- Saving remains available through every unsupported/provider-failure state.

## Compact text toolbar: TXT series

Implements [ADR 0034](../adr/0034-compact-text-toolbar-and-semantic-styles.md),
[text-toolbar.md](text-toolbar.md), and research in
[text-editor-toolbars.md](../research/text-editor-toolbars.md). Depends on CMD-03
primitives and CMD-06 resolved rendering. Order: shell region, block picker,
capability providers, CM split, mod surface, evidence.

### TXT-01: Compact text toolbar region and chrome coupling

Likely files:

- `crates/unibrowser-desktop/src/commandManager.ts`
- `crates/unibrowser-desktop/src/styles.css`
- shell layout in `src/main.ts`

Work:

- Add a 32 px `text-compact-toolbar` region above the Command Manager when
  `CommandContext` is Text and the document is editable rich/plain text.
- Render icon-only controls from the resolved model for surface
  `text_compact_toolbar` (empty until TXT-02 wires controls).
- Tie visibility to existing CM collapse (Ctrl+F1): both tiers hide together.
- Do not show the region for Code, Image, PDF, or CAD contexts.

Acceptance:

- Markdown at 1280 px shows 32 px + 40 px chrome; code file shows CM only.
- Collapse leaves a single 40 px hidden state (no orphan gap).
- Region uses Lucide 16 px and existing focus-ring tokens.

Screenshot:

- Markdown light 1280 with compact bar placeholder and CM strip below.

### TXT-02: Semantic block-style picker

Likely files:

- `crates/unibrowser-mod-api` (semantic ids / command registration)
- `crates/unibrowser-desktop/src/commandManager.ts`
- `mods/base-text` or `mods/toy-markdown` providers

Work:

- Implement dropdown control `unibrowser.semantic.block_style` per
  [text-toolbar.md](text-toolbar.md) § Block-style picker.
- Wire Ctrl+Alt+0–6 shortcuts; mixed block selection shows Mixed.
- Remove document font-size stepper from default Text Formatting family on CM;
  relabel editor-only size as “Editor size” where it remains.

Acceptance:

- Markdown H2 applies `##` prefix; plain text greys heading rows with reason.
- Picker trigger shows current style name, not point size.
- Keyboard and palette expose the same command id.

Screenshot:

- Markdown with picker open; plain text with disabled headings.

### TXT-03: Split Text Formatting between compact bar and CM strip

Likely files:

- `crates/unibrowser-desktop/src/commandManager.ts`
- default layout JSON / `toolbars.json` seed
- `docs/design/command-system.md` (worked examples)

Work:

- Map bold, italic, strike, inline code, link, lists, clear to
  `text_compact_toolbar`; keep History, Find, Proofing, Export on CM.
- Move underline, colour, sub/sup to compact overflow by default.
- Update canonical Text tab family list: Text Formatting inline controls no
  longer occupy CM primary row (block picker only on compact bar).

Acceptance:

- CM Text row at 1280 px fits without wrapping; no duplicate Bold on CM.
- Overflow chevron on compact bar matches deterministic width rules in design doc.
- `command-system.md` Markdown example matches runtime layout.

Screenshot:

- Text context CM + compact bar side by side, light and dark.

### TXT-04: Block-style capability matrix per host format

Likely files:

- `mods/toy-markdown` / `markdown-baseline` provider
- `mods/base-docx` (rich document)
- capability resolver tests

Work:

- Register `block_style` providers for markdown, html, docx, odt, rtf, plain
  text per ADR 0034 mapping table.
- Return `emulated` with notes for Markdown underline, task lists in ODT, etc.
- Keep `.cm-emulation-mark` on emulated compact-bar toggles.

Acceptance:

- Contract tests cover each semantic × host cell in the ADR table.
- Unsupported plain-text headings retain focusable controls with reasons.
- Save unaffected when provider times out.

### TXT-05: Mod manifest `text_compact_toolbar` surface

Likely files:

- `crates/unibrowser-mod-api/src/contributes.rs`
- mod loader validation
- `docs/mods/command-system-authoring.md`

Work:

- Extend CMD-07 schema with `surfaces: ["text_compact_toolbar"]` and
  `insert.after` extension points from [text-toolbar.md](text-toolbar.md).
- Validate content_kinds and refuse arbitrary HTML toolbars.

Acceptance:

- Sample mod contributes one icon after `unibrowser.format.link`; appears in
  compact bar with tooltip and capability state.
- Disabled mod removes control without layout crash (CMD-10 rules).

### TXT-06: Compact toolbar evidence and accessibility

Likely files:

- `e2e/specs/` or desktop self-test hooks
- `docs/screenshots/` capture scripts
- `tests/controlA11y.test.mjs`

Work:

- Capture 1280 matrices: Markdown, plain text, rich document; light/dark/hc.
- Add roving-toolbar test for Alt+F10 and arrow navigation on compact bar.
- Sibling `.cm.json` includes compact bar bounds.

Acceptance:

- axe and existing CM a11y tests pass with compact bar visible.
- Emulated underline shows marker and described approximation in SR text.
- Drift checklist passes for new PNGs.

## Phase G: context-aware status

### UI-30: Status item component model and fixed slots

Status: complete 2026-10-07. Specification:
[status-bar.md](status-bar.md).

- Replace the generic AI/path/command/alerts row with fixed left/right slots.
- Render status through `StatusItem` components with stable ids, priority,
  tooltip, semantic kind, and slot.
- Expose `registerStatusContribution()` for context and mod contributions.
- Keep path out when address or breadcrumb already identifies location.
- Hide empty AI and notification utilities.
- Preserve the 4.5:1 status contrast capture guard.

Acceptance:

- Context changes replace items without moving the two slots.
- Status never wraps, overlaps, or increases shell height.
- Every segment passes 4.5:1 in light, dark, and high contrast.

### UI-31: Per-context defaults and narrow overflow

Status: complete 2026-10-07. Specification:
[status-bar.md](status-bar.md).

- Text: save, language, caret, problems, encoding.
- PDF: page, zoom, annotations.
- Workshop: timecode, output format, sample rate/channels.
- Settings: immediate-save statement.
- Mod portal: Open VSX state and local scope.
- File browser and media/viewer contexts use task-specific state.
- Hide priority 60 and below first under 760px; truncate retained items.

Acceptance:

- No generic `Commands: X (auto)` or duplicate path appears.
- Empty alerts and ordinary AI state consume no status space.
- Narrow status retains the primary task state and active warnings.

## Phase H: Content-kind platform (ADR 0028)

Runtime and API tickets implied by [ADR 0028](../adr/0028-platform-content-kinds-and-plugin-scope.md). Docs-only direction is accepted; implementation follows here.

### CMD-12: Content-kind registry and schema

Status: open.

Work:

- Versioned `content_kind_schema` in mod API; core kinds under `unibrowser.content.*`.
- Host registry: labels, optional parent/child relationships, extension registration with `content.kind.register`.
- Manifest validation for `content_kinds` on commands, providers, panels, and status contributions.

Likely files: `unibrowser-mod-api`, `unibrowser-mod-host`, manifest schema tests.

Acceptance:

- Contract tests for canonical ids and namespace rejection rules.
- Unknown kind on a contribution fails validation with a clear diagnostic; Save unaffected.

### CMD-13: Embedded-content tree traversal

Status: open.

Work:

- Document content tree API: node ids, kind, parent, host format, blob handles.
- Selection maps to node paths (root and embedded).
- Format backends (`base-*`) expose minimal trees for proving (image in rich doc, inline asset in Markdown).

Likely files: `unibrowser-engine`, document backends, `HostContext` / selection DTOs.

Acceptance:

- Integration test: select embedded image node in a host fixture; traversal returns kind `unibrowser.content.image`.
- Mixed root + embedded selection returns deterministic kind list for resolver.

### CMD-14: Per-content-kind capability resolver

Status: open.

Work:

- Extend resolver inputs: `selection_content_kinds`, `host_format_id`, `selection_node_paths`.
- Provider declarations accept `content_kinds` and `host_formats`; matrix (kind × host) → supported / emulated / unsupported.
- Update `ResolvedControl` diagnostics to cite kind and host when greyed.

Likely files: `ui_command.rs`, capability provider registry, `command-system.md` worked examples.

Acceptance:

- Same control id: supported on PNG host, emulated or unsupported on host that cannot persist metadata — placement unchanged.
- Provider failure and timeout degrade to unsupported; Save controls bypass overlays.

### CMD-15: Plugin contribution points (panels, views, providers, services)

Status: open.

Work:

- Manifest `contributes.panels`, `views`, `status_items`, `services`, and kind-scoped `providers` merged into fixed shell slots.
- Permission gates: `ui.panel.contribute`, `ui.view.contribute`, `ui.status.contribute`, `service.register`, `provider.*`.
- Portal hints: recommend mods by missing content kind or host format (wire to existing portal cards).

Likely files: `modPortalUi.ts`, mod host loader, dock/status registration APIs.

Acceptance:

- Proving mod contributes one panel and one status item without covering content or Save.
- Disabled mod removes contributions; orphaned layout entries follow CMD-10 rules.

## Phase J: Core skeleton and capability providers (ADR 0030)

### SKEL-01: Capability id registry and manifest `provides`

Status: open.

Work:

- Versioned list of `unibrowser.capability.*` ids (open text, render PDF, project surface, …).
- Extend `info.json` with `provides` entries: capability id, optional `default`, priority, content-kind scope.
- Loader builds provider registry; log conflicts when two defaults claim the same capability.

Likely files: `unibrowser-mod-api` manifest schema, mod host resolver, plugin SDK docs.

Acceptance:

- Two proving mods with the same capability show resolver diagnostic; shell starts; Save unaffected.
- Disabled provider disappears from Open-with list without orphan tabs.

### SKEL-02: Default provider preferences and Open with

Status: open.

Work:

- Settings → Plugins → Defaults: per-capability dropdown of installed providers.
- Files tab / tab context: **Open with** remembers per-path or per-extension choice.
- Route tab open to selected provider; grey with portal link when none installed.

Likely files: preferences schema, `main.ts` tab open path, settings UI.

Acceptance:

- User sets community text mod as default; new `.md` tabs use it; bundled `base-text` still available via Open with.
- Missing provider opens read-only or files-only per capability policy without blocking Save.

Screenshot:

- Open-with menu on a text file at 1280×800 light.

### SKEL-03: About and portal copy — features as plugins

Status: open.

Work:

- About → Plugins lists bundled mods with capability chips (not “built-in Unibrowser editor”).
- Portal cards show `provides` capabilities alongside format extensions.
- Align README screenshots captions where they imply monolithic features.

Likely files: `aboutPage.ts`, `modPortalUi.ts`, docs cross-links only if UI strings change.

Acceptance:

- No user-facing string claims “Unibrowser PDF viewer” without naming `base-pdf` or active provider.

### SKEL-04: Extract legacy viewers behind capability boundaries (spike → slices)

Status: open.

Work:

- Document map: which `unibrowser-desktop` paths belong to which `base-*` capability.
- Spike: route one format (e.g. plain text) through provider registry only; no duplicate command handlers in shell.
- Track remaining inline viewers as follow-up slices (PDF, DOCX, …).

Likely files: `documentViewer.ts`, mod host activation, capability router module.

Acceptance:

- Text open path goes through registry in dev build; undo/save command ids unchanged.
- Regression: `npm run build` and existing e2e shell spec green.

## Phase K: History store and panel (ADR 0031)

Specification: [history-panel.md](history-panel.md).

### HIST-01: History file layout and atomic append

Status: open.

Work:

- Define single **`.unibrow-history`** (pin header + log marker + JSONL ops) at archive root and in folder projects ([ADR 0031](../adr/0031-edit-history-as-file-format.md)).
- Atomic append + header rewrite + partial tail recovery on open; integrate with existing zip/directory save ([ADR 0023](../adr/0023-saving-is-sacred.md)).

Likely files: `unibrowser-project`, desktop save pipeline.

Acceptance:

- Simulated crash mid-append recovers last complete op; non-modal recovery copy.
- Save always succeeds appending ops for native project tab.

### HIST-02: Host history API and undo/redo head

Status: open.

Work:

- Core API: append op, set head, list ops, load snapshot at seq.
- Wire `unibrowser.document.undo` / `redo` to head movement; plugins supply apply/invert.

Likely files: new `history_store` module, `undo_stack.rs` migration path.

Acceptance:

- Proving mod emits two ops; undo/redo matches head; grey when empty.
- Read-only open when replay stops leaves head at last good seq.

### HIST-03: Pin header update and missing-plugin banner

Status: open.

Work:

- Rewrite `.unibrow-history` pin header on each new plugin/op_schema pair; then append op.
- Open flow: read header, replay log until failure; banner with Install version Y / Continue read-only ([ADR 0031](../adr/0031-edit-history-as-file-format.md) §4).

Likely files: project open, mod portal deep link, i18n.

Acceptance:

- Fixture project with unpinned mod opens read-only with exact version in banner.
- Install matching mod replays without data loss.

### HIST-04: History panel UI

Status: open.

Work:

- Bottom dock tab per [history-panel.md](history-panel.md): list, rollback, squash/clean entry points.
- Keyboard navigation; inline confirm for rollback branch discard.

Likely files: new `historyPanel.ts`, dock registration, styles.

Acceptance:

- Rollback to seq updates document; Problems-style density; does not cover Save.
- Empty state copy matches design doc for session-only undo.

Screenshot:

- History panel with three ops at 1280×800 dark.

### HIST-05: Foreign format sidecar and export without history

Status: open.

Work:

- Optional sidecar path for `.docx` / raster saves; preference per workspace.
- Export / Save Copy checkbox **Omit edit history** with privacy explainer.

Likely files: save dialogs, sidecar naming, export pipeline.

Acceptance:

- Save `.docx` with sidecar enabled creates companion history store; foreign bytes remain latest state.
- Export without history produces file with no `.unibrow-history` part or sidecar.

### HIST-06: Squash and clean-before-sharing flows

Status: open.

Work:

- Commands squash log to snapshot and clean wizard (strip log + old snapshots).
- Plugin `migrate_op` registration hook for schema bumps.

Likely files: history commands, mod API, settings.

Acceptance:

- Squash reduces log size; user confirm names redo depth loss.
- Clean before sharing removes recoverable deleted content from bundle per ADR §8.

## Phase M: One document model and editor modes (ADR 0035)

Specification: [document-editor-modes.md](document-editor-modes.md), [ADR 0035](../adr/0035-one-document-model-wysiwyg.md). Research: [markdown-editing-modes.md](../research/markdown-editing-modes.md).

Depends on TXT-01–TXT-04 (compact toolbar) and HIST-02 (history API) for full integration; DOC-01–03 can start in parallel.

### DOC-01: Document model schema and session host

Status: open.

Work:

- Publish `unibrowser.document.model` v1 JSON schema (blocks, marks, opaque, metadata).
- Core session object: model + optional `source_map` + dirty block set; validate on load.

Likely files: `unibrowser-mod-api`, new document model module, `base-text` types.

Acceptance:

- Fixture model round-trips JSON; invalid block rejected with actionable error.
- Content tree exposes `unibrowser.content.rich_text` root per ADR 0028.

### DOC-02: Shared WYSIWYG surface (ProseMirror host)

Status: open.

Work:

- `base-text` (or dedicated editor mod) hosts ProseMirror view bound to model; transactions → model ops.
- Paper card layout per [document-editor-modes.md](document-editor-modes.md); integrate compact toolbar slot.

Likely files: `documentViewer.ts`, new `richTextEditor.ts`, styles for paper card.

Acceptance:

- Open sample `.md` in WYSIWYG; bold/heading picker mutates model; undo uses session stack until HIST-02 wires log.

Screenshot: WYSIWYG light 1280×800 with mode strip.

### DOC-03: Markdown parse/serialize with source_map

Status: open.

Work:

- `parse(bytes) → { model, source_map }` and `serialize` with byte-identical untouched blocks ([ADR 0035](../adr/0035-one-document-model-wysiwyg.md) §4).
- Opaque blocks for unknown syntax; GFM baseline.

Likely files: `mods/base-text`, `crates/unibrowser-projections` or new markdown crate.

Acceptance:

- Corpus of 20+ fixtures: no edit → SHA-256 unchanged on save.
- Edit single paragraph → only that span may change cosmetics.

### DOC-04: Source and Split panes

Status: open.

Work:

- Source projection buffer with apply/discard banner; re-parse merge.
- Split layout with scroll sync and focus rules.

Likely files: `documentViewer.ts`, Monaco or CodeMirror host for source.

Acceptance:

- Split at 50/50; toolbar bold works in source pane; dirty guard blocks silent WYSIWYG switch.

### DOC-05: Mode strip UI and persistence

Status: open.

Work:

- Segmented control + status chip; workspace layout stores per-tab mode and split ratio.
- Default `document.default_mode = wysiwyg` for new profiles.

Likely files: chrome components, preferences schema, i18n.

Acceptance:

- Reload restores last mode per path; Reading disables edit toolbar.

### DOC-06: Reading mode projection

Status: open.

Work:

- Read-only render sharing WYSIWYG typography; `Ctrl+E` toggle; links active.

Likely files: document viewer, a11y live region.

Acceptance:

- No caret in Reading; switch back restores prior edit mode.

### DOC-07: History ops on document model

Status: open.

Work:

- Register `base-text.block.*`, `base-text.text.*`, `base-text.opaque.*` op types; snapshot includes model + map blob ref.

Likely files: `base-text`, history store integration (HIST-02).

Acceptance:

- Two heading changes append ops; replay on cold open matches session.

### DOC-08: DOCX serialiser plugin (shared model)

Status: open.

Work:

- `base-docx` maps model ↔ OOXML; capability matrix from ADR 0034; no `source_map` byte rule.

Likely files: `mods/base-docx`.

Acceptance:

- Same toolbar edits `.docx` and `.md` with consistent semantic labels; DOCX save opens in Word.

### DOC-09: Opaque block UX and “Parse as Markdown” command

Status: open.

Work:

- Opaque card UI; explicit parse command with diff preview; never auto-promote.

Likely files: editor components, command registry.

Acceptance:

- Raw HTML block survives WYSIWYG ↔ Source ↔ save cycle.

### DOC-10: Round-trip and mode-switch regression suite

Status: open.

Work:

- Automated tests: byte identity, mode switch without delimiter drift, emulation markers.
- Document in [markdown-editing-modes.md](../research/markdown-editing-modes.md) corpus appendix.

Likely files: `e2e/`, `mods/base-text/tests/`.

Acceptance:

- CI gate on fixture folder; documents known intentional normalisation inside edited blocks.

## Phase L: `.unibrow` package modes and OPC container (ADR 0032)

Specification: [ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md), [ux.md](ux.md) (storage mode section).

### PKG-01: OPC archive layout and compression policy

Status: open.

Work:

- Implement OPC zip writer/reader for `.unibrow`: root `manifest.json`, `project.json`, `.unibrow-history`, content-addressed `assets/<hash>`.
- STORE for already-compressed media parts; DEFLATE for JSON/text and `.unibrow-history`; round-trip with standard unzip.

Likely files: `unibrowser-project` zip bundle, format validation tests.

Acceptance:

- Fixture archive opens in `unzip -l`; duplicate asset bytes dedupe to one `assets/` entry.
- Manifest and `.unibrow-history` paths match [ADR 0031](../adr/0031-edit-history-as-file-format.md) and [ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md).

### PKG-02: Folder mode — plain vs unpacked on-disk IO

Status: open.

Work:

- **Plain folder (ADR 0032 §2a):** open any directory; never write `manifest.json`, `project.json`, or `assets/` on open or edit; first mutating edit may create **only** `.unibrow-history`.
- **Unpacked folder (§2b):** detect root `manifest.json`; read/write same paths as archive interior.
- Regression guard: automated test that opening + editing a temp folder leaves no package files except optional `.unibrow-history`.

Likely files: project open/save paths, path security for project root.

Acceptance:

- Open `MyShow/` (no manifest), edit timeline → only `.unibrow-history` appears on disk.
- Open `MyShow/` after **Unpack to folder** → `manifest.json` + `project.json` present; save updates them and appends history.
- Plain folder open remains ordinary browse when user does not start native project editing.

### PKG-03: Storage mode state (view / archive / folder)

Status: open.

Work:

- Session flag for three modes; view only blocks disk writes; archive vs folder routes IO per ADR §3.
- **Start editing** transitions from view only with writable target check.

Likely files: document session, project tab backend, command enablement.

Acceptance:

- Open `.unibrow` read-only volume → view only; Save As offers writable path.
- Folder mode on plain dir: first edit may add `.unibrow-history` only; no manifest until unpack/collect.
- Mode label matches ADR names in status contribution.

### PKG-04: Status bar mode indicator and first-edit history path

Status: open.

Work:

- Left-slot items per [ux.md](ux.md): mode label + **History will be saved in …** after first edit.
- No status text on open-before-edit beyond mode when view only.

Likely files: `statusBar.ts`, project status contributions, i18n.

Acceptance:

- First text edit in archive mode shows history path pointing at the `.unibrow` file.
- View only shows `View only` with tooltip from design doc.

### PKG-05: Unpack to folder and Pack as .unibrow

Status: open.

Work:

- File menu + palette commands with confirmation and progress for large trees.
- **Unpack** writes §2b layout (`manifest.json`, `project.json`, `assets/`, `.unibrow-history`, …) without deleting source unless user confirms.
- **Pack** serialises session + history + user files into OPC; **`manifest.json` only inside the new `.unibrow`**, never dropped into a plain source folder.

Likely files: project conversion service, progress UI.

Acceptance:

- Pack from plain folder: before/after listing shows no new `manifest.json` in source dir; output `.unibrow` contains manifest part.
- Round-trip unpack → edit → pack preserves `.unibrow-history` seq continuity.
- Cancel mid-unpack leaves no broken half-project without user-visible failed state.

### PKG-06: Large-project folder hint (configurable threshold)

Status: open.

Work:

- Settings → Projects: size and/or asset-count threshold; off disables hint.
- Non-blocking status warning + **Unpack to folder** link; never auto-switch mode.

Likely files: settings schema, project open estimator, status bar warning item.

Acceptance:

- Fixture above threshold shows hint in archive mode only.
- Threshold set to off → no warning item.

### PKG-07: Archive edit journal and atomic save

Status: open.

Work:

- Journal beside file or cache per preference; fold into zip on save/autosave with atomic replace.
- Crash open recovers journal per [ADR 0032](../adr/0032-unibrow-container-archive-and-folder-modes.md) §5.

Likely files: save pipeline, recovery banner copy.

Acceptance:

- Kill process during edit; reopen offers recovery count; Save succeeds after accept.
- Document tradeoff: multi-GB archive save may be slow; folder mode avoids full repack in acceptance notes.

### PKG-08: Referenced media and Collect into project

Status: open.

Work:

- `refs/` entries in graph; missing-file banner; **Collect into project** copies into `assets/<hash>` and may materialise §2b files (`manifest.json`, `project.json`, `assets/`) on disk when user confirms.

Likely files: asset index, file ops, Command Manager entry.

Acceptance:

- Plain folder: Collect is the only path (besides Unpack) that creates `manifest.json` / `project.json` / `assets/` on disk.
- Link large video → ref only until Collect; Collect embeds once under `assets/`.
- Collect grey when no refs in project.

## Phase I: Plugin permissions and network (ADR 0029)

Specification: [plugin-permissions-ui.md](plugin-permissions-ui.md). Runtime policy: [ADR 0029](../adr/0029-plugin-permissions-and-network-security.md).

### PERM-01: Permission manifest schema and validation

Status: open.

Work:

- Add versioned `permissions` block to `info.json` (`permission_schema` semver).
- Validate `network.domains`, `filesystem.scopes`, `clipboard`, `process.launch`, per-kind `content.read` / `content.write`, `secrets` slots, `background`, and ADR 0028 UI/command keys before loading payload.
- Require `permissions.network.reason` and `permissions.network.disclosure` when any domain is declared.

Likely files: manifest schema tests, `unibrowser-mod-host` loader, plugin SDK docs.

Acceptance:

- Invalid or overbroad wildcard without flag fails load with a clear diagnostic; Save unaffected.
- Proving fixture mod with two domains loads and lists both in portal chips.

### PERM-02: Grant store and version reset

Status: open.

Work:

- Persist grants in `permission-grants.json` under config dir.
- Implement Allow once (session), Always allow, and explicit deny.
- On plugin version bump with new domains/scopes/secrets/process entries, reset persistent grants and show portal banner copy from design doc.

Likely files: new `permission_grants.rs` (or prefs module), desktop settings bindings.

Acceptance:

- Restart preserves Always grants; Allow once does not survive quit.
- Upgrade fixture from v1 to v2 with a new domain clears Always grant and requires new consent.

### PERM-03: Host-mediated network gate

Status: open.

Work:

- Central fetch/WebSocket API for mods; deny when domain not declared or not granted.
- Honour global Offline mode kill switch with design-doc copy.
- Integrate Open VSX and portal search with the same gate and logging hooks.

Likely files: mod host network shim, `network.md` behaviour parity tests, portal fetch paths.

Acceptance:

- Undeclared origin fails closed without crashing the shell.
- Offline mode blocks mod catalogue fetch; user-initiated https tab still loads.

### PERM-04: Network consent banner (first use)

Status: open.

Work:

- Non-modal banner per [plugin-permissions-ui.md](plugin-permissions-ui.md) §1.
- Queue concurrent requests; wire Allow once / Always / Don't allow to PERM-02.
- Assistant variant uses disclosure bullets; no focus trap.

Likely files: `permissionConsentBanner.ts`, shell layout CSS, i18n strings.

Acceptance:

- Banner does not cover Save or block typing in the editor.
- Esc maps to Don't allow; keyboard path reaches all three actions.

Screenshot:

- Light and dark 1280×800 with banner open over a text document.

### PERM-05: Settings Permissions page

Status: open.

Work:

- Settings → Plugins → Permissions list cards with revoke per origin.
- Show filesystem, clipboard, process, and content-kind summary per plugin.
- Footer links to Offline mode and activity log.

Likely files: `settingsPage.ts`, new permissions section module.

Acceptance:

- Revoke updates grant store immediately; greyed commands refresh without restart.
- Full-trust native note appears only for unsigned T1 mods.

Screenshot:

- Permissions page with two plugins at 1280×800.

### PERM-06: Network activity log

Status: open.

Work:

- Append JSONL log with rotation cap per ADR 0029.
- UI table with filters, Save log, Clear log (inline confirm).

Likely files: `networkActivityLog.ts`, log writer in host.

Acceptance:

- Each gated fetch writes one row with bytes and grant type; secrets never logged.
- Clear log empties view and file; Save As exports valid JSON.

Screenshot:

- Activity log with two rows.

### PERM-07: Keychain-backed plugin secrets

Status: open.

Work:

- `secrets.put/get/delete` host API; OS keychain service naming per plugin slot.
- Block plain-text secret fields in mod prefs paths.

Likely files: `unibrowser-mod-api`, platform keychain crate/wrapper.

Acceptance:

- Proving mod stores and retrieves a token; grep of config dir shows no secret value.
- Activity log records slot use without value.

### PERM-08: Mod Portal permission chips

Status: open.

Work:

- Derive chips from manifest for cards and expanded detail (Network count, Sandboxed, Full trust).
- List domains and kinds in expanded view in plain language.

Likely files: `modPortalUi.ts`, catalog row DTO.

Acceptance:

- WASM fixture shows Sandboxed; unsigned native shows Full trust warning before Enable.
- Install flow does not auto-grant network.

### PERM-09: Command Manager grant affordance

Status: open.

Work:

- Extend disabled reason DTO with `grant_action` (plugin id, capability, origin).
- Tooltip + palette remedy **Grant…** opens PERM-04 banner; no modal on Save path.

Likely files: `commandManager.ts`, `ui_command.rs`, palette component.

Acceptance:

- Network-gated mod command grey with reason; Grant completes pending action.
- Save control never shows grant UI.

Screenshot:

- Greyed command with tooltip at 1280 (text context).

### PERM-10: WASM default vs native full-trust install path

Status: open.

Work:

- Portal Enable flow: T2 copy “Runs sandboxed”; T1 unsigned secondary warning panel per design doc.
- Align extension-host Node trust with grant store (follow-up: remove blocking trust dialog).

Likely files: `modPortalUi.ts`, extension-host prefs migration note.

Acceptance:

- Decline full-trust warning leaves mod disabled; document editing and Save work.
- T2 install does not show full-trust warning.

## Visual QA remediation: VQA series

Source audit: [visual-qa-2026-10-06.md](visual-qa-2026-10-06.md).
Each ticket is one Composer commit and must regenerate only its named evidence.

### VQA-01: Remove the split/black text capture surface

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa-monaco/`.

- Make `.txt` and Markdown captures show one continuous Monaco editor surface.
- Remove the capture-only `pre`/native-layer split once WebKitGTK paint is
  reliable.
- Reject captures containing a large low-luma rectangle in any editor quadrant.
- Likely files: `textEditor.ts`, `docCaptureAsserts.ts`, `webview_capture.rs`,
  `styles.css`.
- Evidence: light `.txt`, light Markdown, dark Markdown.

### VQA-02: Restore real editor geometry in evidence

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa-monaco/`.

- Show gutter, line numbers, caret-ready text layer, and real syntax styling.
- Match Source Code Pro metrics and light/dark editor tokens.
- Likely files: `textEditor.ts`, Monaco theme integration, font loading.
- Evidence: `.txt` and Markdown at 1280x800.

### VQA-03: Route viewer kind to the correct CM context

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-03-*.png`.

- Resolve PDF to PDF Annotation and SVG/raster to Image.
- Prove that active viewer context replaces Text without a second switcher.
- Likely files: `main.ts`, `commandManager.ts`, Rust context resolver.
- Evidence: PDF and SVG with context label and resolved model diagnostics.

### VQA-04: Make compact CM groups overflow without label collisions

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-04-*.png`.

- Measure and overflow complete groups before any label overlaps.
- Keep context and Follow/Pin fixed; preserve stable control positions.
- Likely files: `commandManager.ts`, `commandControls.ts`,
  `compactChrome.css`.
- Evidence: Text, PDF, and Image at 800, 1024, and 1280 widths.

### VQA-05: Restore dark-slate shell identity in light theme

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-05-*.png`.

- Use dark slate for title bar, CM, and rail while retaining warm document
  surfaces.
- Verify icon, text, and window-control contrast.
- Likely files: `tokens.css`, `compactChrome.css`,
  `quietUtilityShell.css`.
- Evidence: light text document and PDF at 1280x800.

### VQA-06: Render the rich DOCX fixture as a document

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-06-rich-docx.png`.

- Replace the black/fallback region with a centred readable page.
- Preserve bold/italic hierarchy and expose mixed rich-text states.
- Remove capture-only flattened fallback text.
- Likely files: `documentViewer.ts`, DOCX viewer CSS, demo fixture.
- Evidence: `text-family-rich-docx.png`.

### VQA-07: Build the PDF evidence shell

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-07-pdf-annotation.png`.

- Render page thumbnails, centred page, annotation overlay, and Annotations
  dock.
- Add page, zoom, annotation, and Export PDF controls from the PDF family.
- Likely files: `documentViewer.ts`, PDF renderer, PDF command definitions.
- Evidence: `pdf-annotation-cm.png`.

### VQA-08: Render the image evidence shell

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-08-image-svg.png`.

- Show the actual SVG/raster on a recessed canvas with zoom/metadata.
- Replace Text formatting controls with Image family controls.
- Likely files: `documentViewer.ts`, SVG/image renderer, Image family defaults.
- Evidence: `image-cm.png`.

### VQA-09: Match activity rail icon and contrast rules

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-09-rail-*.png`.

- Use 16 px Lucide icons, consistent stroke, visible selected state, and at
  least 3:1 contrast.
- Keep the rail limited to side-panel destinations.
- Likely files: activity rail renderer, `compactChrome.css`.
- Evidence: light/dark text document with collapsed and expanded rail.

### VQA-10: Replace AI cards with transcript structure

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-10-ai-*.png`.

- Remove the rounded response card inside the dock.
- Use a subtle speaker edge/surface change and preserve a visibly bounded
  composer input.
- Likely files: `aiAssistantPanel.ts`, `styles.css`.
- Evidence: light and dark text document with AI dock.

### VQA-11: Make Problems truthful and proportional

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-11-problems-ai-*.png`.

- Collapse an empty Problems/Output panel.
- When opened for evidence, show at least three structured rows and no duplicate
  `No problems` message.
- Likely files: `problemsPanel.ts`, `scriptOutputPanel.ts`, capture presets.
- Evidence: `document-problems-ai` light/dark equivalents.

### VQA-12: Load and enforce the Source font pair

Status: complete 2026-10-06. Evidence: `docs/screenshots/vqa/vqa-12-text-*.png`.

- Load Source Sans 3 and Source Code Pro and use the documented 12/13/14 px
  scale.
- Verify no functional text below 12 px and no clipped labels at 100%/150%.
- Likely files: font assets, `tokens.css`, `styles.css`.
- Evidence: command gallery and text document at 100%/150%.

### VQA-13: Strengthen screenshot acceptance gates

Status: complete 2026-10-06. The PNG pixel gate rejects blank text regions and
wrong theme surfaces.

- Add region-level checks for continuous editor/viewer paint, expected context
  label, no overlaps, chrome height, and required viewer landmarks.
- Fail rather than substituting misleading fallback surfaces.
- Likely files: `docCaptureAsserts.ts`, `shellPaintReady.ts`,
  capture scripts.
- Evidence: one intentional failing fixture per gate plus the passing matrix.

### VQA-14: Retire superseded visual evidence

Status: complete 2026-10-06.

- Mark `cmd-batch/` and `ui-batch-3c/` as historical failures in their READMEs.
- Remove them from current review/report links without deleting useful
  regression artefacts.
- Likely files: screenshot READMEs and documentation indexes.

### VQA-15: Publish the final visual QA matrix

- Recapture Text plain/Markdown/rich, PDF, Image, Problems+AI, light/dark, and
  high contrast at full review size.
- Re-run the visual QA against every applicable drift-checklist item.
- Likely files: capture scripts, `docs/screenshots/`, visual QA report.
- Acceptance: no undocumented S0/S1 defects.

## Workflow walkthrough QA tickets

From [workflow-walkthrough-qa.md](workflow-walkthrough-qa.md) at 1280 light.

### QA-01: Make PDF Highlight create a visible mark

Status: open.

- Arming Highlight and dragging across body text did not add a rect or
  change the annotation count.
- If Highlight needs a text selection, the armed tool must say so. If it
  accepts a drag, the mark must appear and the tab must go dirty.
- Save must stay available.
- Likely files: `documentViewer.ts`, PDF annotation commands.
- Evidence: `docs/screenshots/workflow-walkthrough/02-pdf-annotated.png`.

### QA-02: Open workshop Export without hunting overflow

Status: open.

- At 1280 the Project prefix is Add asset, Add title, Split, Delete clip.
  Export is overflow-only. This walk did not reach a delivery panel.
- Keep a reachable Export that opens the non-modal delivery panel from
  US-10. Do not cover the timeline.
- Likely files: workshop command families, export panel.
- Evidence: `docs/screenshots/workflow-walkthrough/04-workshop-export.png`.

### QA-03: Finish catalogue Install through to Open with

Status: open. In-card stub copy shipped in `d4dbb9f`.

- CAD file support Install does not install a handler. sample.step never
  offers Open with [mod].
- Install must stay on the card, show progress there, and leave the file
  tab in place (US-13).
- Likely files: `modPortalUi.ts`, `main.ts` portal handlers.
- Evidence: `docs/screenshots/workflow-walkthrough/05-portal-install.png`.

### QA-04: Make the Auto-save interval select reliable to change

Status: open.

- General shows Auto-save on and Every 5 minutes. A pointer open of the
  themed list did not keep a new interval in the walkthrough.
- Changing the interval must persist and stay visible. The switch must
  not steal the select hit target.
- Likely files: `settingsPage.ts`, `themedSelect.ts`.
- Evidence: `docs/screenshots/workflow-walkthrough/06-settings-autosave.png`.

### QA-05: Keep the floating command window off the Settings rail

Status: open.

- Unibrowser commands (Dock in main window, Markdown proving) can cover
  Appearance and block Theme.
- Theme stays under Appearance. Switching Light to Dark must remain
  possible without dismissing a stray toolbar first.
- Likely files: `toolbar_windows.rs`, settings page, command pin.
- Evidence: `docs/screenshots/workflow-walkthrough/07-theme-appearance.png`.
