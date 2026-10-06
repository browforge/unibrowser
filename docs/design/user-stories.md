# User stories

Status: design baseline for the post-blitz shell redesign.  
Sequence: **stories -> workflows -> UX -> layout -> UI**.

## Product promise

Unibrowser opens files, folders, web pages, and projects in one predictable tab strip. Common actions use the same names and locations across formats. Power remains available without making ordinary users manage formats, mods, projections, or rendering internals.

## Priority definitions

- **P0:** required for the first coherent redesign.
- **P1:** required for a complete public desktop experience.
- **P2:** valuable depth after the core interaction model is stable.

## Personas

### Moana, non-technical home user

Moana is 72 and uses Windows 11. She recognises tabs, folders, search, Save, Print, and Download. She does not know whether a photo is raster or vector and should never need to.

Needs: obvious opening and saving, large targets, plain language, undo, no surprise panels, no account, and recovery after mistakes.

### Alex, FOSS power user

Alex runs Linux and Windows, keeps local files, changes keybindings, installs community mods, and expects inspectable configuration. They tolerate complexity only when it buys control.

Needs: palette, keyboard-first use, local-first settings, sideloading, visible permissions, high density, detachable toolbars, and no forced service.

### Bear, engineer and SOLIDWORKS user

Bear changes among documents, drawings, media, and projects all day. He expects a stable Command Manager, context tabs, greyed commands with reasons, predictable docking, and layouts that survive restarts and monitor changes.

Needs: dense context-aware commands, pin/follow control, multi-monitor toolbars, one command identity across formats, no content overlays, and fast context switching.

### Hana, writer and editor

Hana works in Markdown, text, DOCX, PDFs, and reference web pages. She needs spelling and grammar help without Save being held hostage.

Needs: Find, proofing, Problems navigation, autosave, word count, clean reading/editing layouts, export, and recovery.

### Jules, video and streaming hobbyist

Jules runs a backyard weather channel and streams games. They combine titles, images, PDFs, browser sources, test patterns, and live data without wanting a broadcast engineering course.

Needs: project templates, timeline, preview, LIVE state that cannot be mistaken, record controls, source health, safe local outputs, and simple recovery.

### Priya, mod author

Priya adds a specialist file handler and commands. She needs APIs and UI contribution rules that preserve consistency instead of producing a second interface inside the first.

Needs: command registry, documented surfaces, capability/grant visibility, developer diagnostics out of primary chrome, sideloading, and predictable disabled states.

## Prioritised stories

### P0: stable shell and ordinary work

#### US-01: Open anything without format decisions

As Moana, I want to open a downloaded item from Files or Downloads so it appears in a tab with the right viewer.

Acceptance criteria:

- A file, folder, URL, download, and `.unibrow` project use the same tab strip.
- The app chooses a handler without asking about file-format implementation details.
- If no handler exists, the tab remains usable and offers plain-language next steps inline.
- Opening one item never blocks another tab.

#### US-02: Know where I am

As any user, I want the active tab, location, document type, and current workspace to be obvious without reading debug text.

Acceptance criteria:

- Active tab and active Command Manager tab are visually distinct.
- The omnibox shows a human-readable path or URL and exposes the full value on focus.
- Product chrome never prints the loaded mod list.
- Technical backend details live in Properties or Developer settings.

#### US-03: Use universal commands

As Hana, I want one Find, Zoom, Increase font size, Save, and Export command to behave consistently across formats.

Acceptance criteria:

- Each concept has one command id, label, shortcut, and palette result.
- The command remains in its expected slot when unavailable.
- Disabled controls explain why and, when possible, what enables them.
- Save remains available whenever user state can be persisted.

#### US-04: Keep content unobscured

As Bear, I want sidebars and panels to resize the document surface rather than cover it.

Acceptance criteria:

- AI, Files, Properties, and other sidebars occupy docked layout regions.
- Opening a sidebar pushes/resizes content; it never floats above toolbar or document content.
- Palette and context menus are the only temporary overlays over content; both dismiss immediately and preserve work.
- Notifications never cover the active caret, selection, live controls, or Save state.

#### US-05: Save and recover with confidence

As Hana, I want Save to work even if the document has warnings, and I want my work back after a crash.

Acceptance criteria:

- Diagnostics never block Save, Save As, or autosave.
- Save feedback is proportional and truthful.
- Crash recovery is an inline bar/list, not a modal.
- Recovery offers Open recovered, Compare, Dismiss, and location details.

#### US-06: Use context tools without hunting

As Bear, I want the Command Manager to follow the active tab while keeping all expected commands in stable positions.

Acceptance criteria:

- Follow context is the default.
- Manual tab selection previews another tool set without silently pinning it.
- Pin is a labelled, persistent control adjacent to context tabs.
- Disabled commands remain visible and show a reason.

#### US-07: Fix issues from one place

As Hana, I want spelling, grammar, code, and project problems in one panel that jumps to the right location.

Acceptance criteria:

- Problems is a docked bottom panel and does not obscure content.
- Severity, source, and text filters are available without consuming a full toolbar row.
- Selecting a problem navigates and focuses the relevant location.
- Fix, ignore, and learn actions appear near the selected problem.

#### US-08: Reach every action by keyboard

As Alex, I want Ctrl+Shift+P to find commands, files, settings, and help without memorising menus.

Acceptance criteria:

- Results are grouped and keyboard navigable.
- Disabled commands remain searchable with their reason.
- Recent commands rank above obscure commands after repeated use.
- The palette never changes the active document until a result is committed.

### P1: specialised work without specialised chrome

#### US-09: Annotate a PDF

As Moana, I want to highlight and comment on a PDF and save the annotations without understanding sidecars.

Acceptance criteria:

- Annotation tools appear in the PDF context.
- A clear save state covers document plus annotations.
- Export/flatten is separate from Save.
- Missing write capability is explained inline without blocking viewing.

#### US-10: Build a mixed-media project

As Jules, I want to combine titles, pictures, documents, and clips on a `.unibrow` timeline.

Acceptance criteria:

- Add/import uses user language such as Photo, Title, Clip, and Document.
- Native sources remain linked until export.
- Missing source files are visible in the timeline and Problems panel.
- Project Save remains available even when export cannot proceed.

#### US-11: Go live safely

As Jules, I want an unmistakable studio state and a deliberate path from Preview to LIVE to Record.

Acceptance criteria:

- Preview, LIVE, and Recording are distinct text/icon/colour states.
- LIVE state is visible in the persistent studio header and status bar.
- Enabling LIVE does not start recording.
- Source failures appear inline and do not stop healthy sources.

#### US-12: Ask AI with explicit grants

As Alex, I want to ask a local or chosen AI about the open tab while controlling what it may read or change.

Acceptance criteria:

- AI is a right sidebar that pushes content.
- Current Read, Write, and Ask grants are always inspectable.
- No document data leaves the machine before a user action and valid grant.
- Proposed writes show a diff and use normal undo.

#### US-13: Install support for an unknown file

As Moana, I want the app to explain that it cannot open a file and offer safe support options.

Acceptance criteria:

- The file remains represented in its tab.
- Suggestions say what capability they add, publisher, licence, permissions, and source.
- Install is not a modal and does not replace the current tab.
- Open as text/hex remains available.

#### US-14: Customise workspaces and monitors

As Bear, I want toolbars and panels arranged for my workflow, including a second monitor.

Acceptance criteria:

- Drag handles clearly distinguish movable bars from command buttons.
- Bars dock to defined regions or detach into owned windows.
- Layouts persist per named workspace.
- If a monitor disappears, detached bars return to a visible safe area without losing saved intent.

#### US-15: Browse and edit images without codec vocabulary

As Moana, I want to save an image from the web and make common visual adjustments.

Acceptance criteria:

- Download opens in a new tab by default.
- Image actions use Crop, Rotate, Resize, Adjust, and Export.
- Original format details are secondary.
- Save preserves the working file; Export presents delivery choices.

### P2: creation and extension depth

#### US-16: Author a consistent mod

As Priya, I want to register commands and capabilities without injecting arbitrary primary chrome.

Acceptance criteria:

- Mods contribute commands, handlers, settings sections, and optional docked views through documented contracts.
- Primary command placement remains user/system controlled.
- Mod diagnostics and reload actions live under Developer settings and the palette.
- Permission and failure states use standard components.

#### US-17: Switch named workspaces

As Jules, I want Browse, Write, Create, and Live workspaces with familiar shared chrome.

Acceptance criteria:

- Workspace switch changes panel/toolbar arrangement, not tab identity or file ownership.
- Switching is reversible and does not close documents.
- Default workspace follows content unless explicitly pinned.
- Custom workspaces are editable and exportable as local files.

## Cross-story quality bar

- No modal dialogs in normal workflows.
- Nothing except an invoked palette or context menu temporarily covers content.
- Minimum target: 32 px pointer, 44 px touch mode.
- Full keyboard operation and visible focus.
- 100%, 125%, 150%, and 200% scaling remain usable.
- Light, dark, and high-contrast themes communicate state without colour alone.
- Windows 11 is primary; Linux follows platform conventions without losing layout parity.
