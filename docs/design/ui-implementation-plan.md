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
- Define title-bar global commands: Undo, Redo, Save, layout overflow.
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
