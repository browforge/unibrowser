# UI implementation plan

Ordered Composer-sized tickets. Each ticket should land independently with `npm run build`, relevant Rust tests, and the named screenshot check. Product code changes should not start before [UX](ux.md), [layout](layout.md), and [UI spec](ui-spec.md) are accepted.

## Definition of done for every ticket

- Keyboard and pointer paths work.
- Light, dark, and high-contrast checked where relevant.
- 100% and 150% Windows scaling checked.
- No new modal, content overlap, or duplicate command implementation.
- Existing command ids are reused.
- Screenshot at 1280x800; wide screenshot when layout changes.

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

- First 106 px contain title/menu, tabs, navigation only.
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
- Add Compact/Comfortable density variable.
- Replace one-off shell colours/sizes only; do not restyle every viewer yet.

Acceptance:

- Stable region dimensions match `layout.md`.
- Light/dark tokens pass contrast spot checks.
- 125/150/200% scaling does not clip shell regions.

Screenshot:

- Annotated 1280 shell-region screenshot in light/dark.

### UI-04: Menus and canonical navigation row

Likely files:

- `src/main.ts`
- `src/chromeI18n.ts`
- `src/i18n/*.json`
- command registry/menu bindings

Work:

- Add File/Edit/View/Go/Tools/Help commands.
- Navigation row: Back, Forward, Reload/Stop, omnibox, Find, Downloads, overflow.
- Move Home/Settings/Mods/Layout actions into menus and palette.
- Keep Windows/Linux menu conventions.

Acceptance:

- No duplicated action appears in permanent chrome.
- All menu entries execute registry commands.
- Alt/menu keyboard access works.

Screenshot:

- View menu open, showing Panels, Workspaces, Customise.

## Phase B: Command Manager and tab/navigation clarity

### UI-05: Rebuild Command Manager frame

Likely files:

- `src/commandManager.ts`
- `src/styles.css`
- `src-tauri/src/ui_command.rs`
- `src-tauri/src/commands.rs`

Work:

- Separate 28 px context-tab row from 44 px commands.
- Add explicit Following/Pinned status and labelled Pin toggle.
- Group commands; remove dashed debug container.
- Prevent wrapping; horizontal scroll/overflow.

Acceptance:

- Manual tab click previews without pinning.
- Pin persists.
- Slots do not shift when disabled.

Screenshot:

- Text context with enabled/disabled commands; pinned Image while text tab active.

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

### UI-07: Tab strip and omnibox

Likely files:

- tab rendering in `src/main.ts`
- `src/styles.css`
- omnibox suggestion module

Work:

- Apply tab dimensions/states.
- Add file/folder/web/project icons.
- Dirty/pinned/audio/LIVE redundancy.
- Breadcrumb display for local paths; full path on focus.
- Add tab overflow/search.

Acceptance:

- 12 tabs remain operable at 1280.
- Keyboard tab movement and close work.
- Local path and URL identities are unambiguous.

Screenshot:

- Mixed tabs: folder, PDF, web, `.unibrow`, dirty text.

### UI-08: Full command palette

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

Screenshot:

- "font" query with enabled and disabled results.

## Phase C: panels, feedback, and ordinary journeys

### UI-09: Standard docked panel component

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

Screenshot:

- Problems open at 1280 with editor visible above.

### UI-10: Problems redesign

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

Screenshot:

- Spelling + code/project problems, selected row.

### UI-11: Feedback ladder

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

Screenshot:

- Recovery inline bar and Closed tab Undo toast.

### UI-12: New tab and Downloads

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

Screenshot:

- Empty first launch; populated recents/downloads.

## Phase D: context layouts

### UI-13: File browser and settings layouts

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

Screenshot:

- File browser with preview; Settings > Developer collapsed/expanded.

### UI-14: PDF/image/DXF context shells

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

Screenshot:

- Three 1280 context comparisons.

### UI-15: `.unibrow` Create workspace

Likely files:

- `src/documentViewer.ts`
- timeline/board modules
- workspace layout state

Work:

- Sources left, preview centre, Inspector right, timeline bottom.
- Workspace switch without closing/changing tabs.
- Save vs Export hierarchy.

Acceptance:

- Title-card journey completes without modal.
- Missing source appears in timeline + Problems; Save stays enabled.

Screenshot:

- 1600x1000 project with title selected.

### UI-16: Live workspace

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

Screenshot:

- Preview, LIVE, Recording states at 1600x1000.

### UI-17: Mod Portal and unknown-handler journey

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

Screenshot:

- Unknown file + support sidebar; Mod Portal search result.

## Phase E: customisation, accessibility, polish

### UI-18: Workspace/layout customisation

Likely files:

- `src/commandManager.ts`
- `src/workspaceLayoutPanel.ts`
- `src-tauri/src/toolbar_windows.rs`
- preferences/layout persistence

Work:

- Explicit edit mode, drag handles, docking overlays.
- Named layouts and import/export management.
- Monitor-loss recovery.

Acceptance:

- Two-monitor detach/restart test.
- No accidental drag from command body.

Screenshot:

- Edit mode with docking targets; detached toolbar window.

### UI-19: Accessibility and scaling pass

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

Screenshot:

- HC at 150%; focus-visible keyboard sequence.

### UI-20: Remove legacy CSS/components and update evidence

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

Screenshot:

- Final 1280 and 1600 shell matrix matching `mockups/context-matrix.png`.
