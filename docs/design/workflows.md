# End-to-end workflows

These journeys translate [user stories](user-stories.md) into buildable flows. Each names current-build friction observed in the October prototype screenshots or existing behaviour.

## 1. First launch

**Goal:** open useful work immediately; explain the unified shell once.

1. App opens a New tab page with **New** (empty in-memory `.unibrow` project), **New from template**, **Open file**, and **Open folder** on the paper card ([ADR 0036](../adr/0036-new-project-empty-by-default-and-templates.md)).
2. Recent files/projects and Downloads appear below only when populated.
3. A compact welcome line explains that nothing is written to disk until the user edits or saves.
4. User may choose **Take a 60-second tour** or **Dismiss**.
5. Dismissal persists. Tour steps point to real controls without blocking them.
6. Default workspace is **Browse** and Command Manager follows context.

Current friction:

- Raw mod inventory leads the entire interface.
- Home, Downloads, Download manager, New project, and Settings form an unexplained button row.
- First screen has a large empty/dark region and developer-oriented chrome.

## 2. Open a downloaded PDF and annotate

**Goal:** move from download to useful markup without format vocabulary.

1. Download completes; a low-priority toast offers **Open** and **Show in folder**.
2. Default behaviour opens the PDF in a new tab.
3. PDF Command Manager tab activates: Select, Find, Zoom, Highlight, Comment, Export.
4. User selects text or drags a region, then chooses **Highlight**.
5. Annotation appears immediately; tab shows dirty state.
6. Ctrl+S saves document state/annotation sidecar atomically.
7. **Export** offers a flattened PDF when needed, separate from Save.

Current friction:

- Universal commands are mixed with layout/debug controls.
- The viewer can be visually absent in headless captures.
- Export and Save are not presented as a clear working-copy versus delivery distinction.

## 3. Browse, save an image, then tweak it

**Goal:** treat the web and local image as one continuous task.

1. User opens a web tab and chooses **Save image** from the context menu.
2. Download shelf/status reports progress without stealing focus.
3. Image opens in a new tab when complete.
4. Image context activates: Crop, Rotate, Resize, Adjust, Copy, Export.
5. User crops and rotates; changes enter the normal undo stack.
6. Ctrl+S saves the working image; **Export** exposes format/quality choices under **More options**.

Current friction:

- "Demo download" appears in production chrome.
- Download manager and Downloads are duplicated at top level.
- Codec/format capabilities leak into command availability without a coherent image task flow.

## 4. Fix spelling across a document

**Goal:** identify, navigate, and resolve issues without blocking Save.

1. Squiggles appear as the user types.
2. Clicking a squiggle opens a small anchored context menu with Replace, Ignore once, Learn word.
3. **Problems** opens as a docked bottom panel using Ctrl+Shift+M or status count.
4. User filters to Spelling and steps through results.
5. Selecting a row moves the editor and highlights the range.
6. Replace all matching is available from the detail pane.
7. Save remains enabled throughout.

Current friction:

- Problems uses a tall, dark block with filters consuming a separate row.
- Error text is clipped into the status bar.
- An unrelated AI panel can overlap the toolbar.

## 5. Create a `.unibrow` video with title cards

**Goal:** make a mixed-media project without understanding proxy/raster internals.

1. User chooses **New from template** on the New tab page (or **New** for an empty project).
2. Template gallery lists built-in starters (for example **Simple video**); user configures name and optional location, then confirms plugin relationships.
3. Applied template opens in memory; **Simple video** hints the **Create** workspace layout: sources, preview, inspector, timeline.
4. User drags photos/documents/clips into Sources or chooses **Add**.
5. **Title** creates a title card at the playhead.
6. User edits title text in Inspector and drags duration on timeline.
7. Ctrl+S saves `.unibrow` regardless of missing export tools.
8. **Export** opens a non-modal delivery panel with resolution and destination.
9. Missing ffmpeg/PDF renderer appears inline with install/configure guidance.

Current friction:

- Project controls compete with browser and developer controls.
- "New project" is duplicated.
- Timeline/page switching lacks a declared workspace hierarchy.

## 6. Go live

**Goal:** distinguish preparation, transmission, and recording at a glance.

1. In a `.unibrow` project, user switches to **Live** workspace.
2. Source tiles show Ready/Warning/Offline with text plus icon.
3. Preview is active by default; no output is sent.
4. User chooses **Go LIVE** in the studio header.
5. Button becomes persistent **LIVE** badge with elapsed time and **End live** action.
6. **Record** remains a separate control and state.
7. Source failures appear in Sources and Problems without ending healthy output.
8. Ending live leaves the project open and saved.

Current friction:

- LIVE controls are buried inside document content.
- Current screenshot does not visibly prove the LIVE badge.
- Status hierarchy does not distinguish viewer state from broadcast state strongly enough.

## 7. Install a mod for an unknown file

**Goal:** recover from unsupported content without leaving or covering the file.

1. User opens an unsupported file.
2. Tab displays metadata/hex-safe preview where possible.
3. Inline header says "Unibrowser cannot open this file yet."
4. Actions: **Find support**, **Open as text**, **Open as hex**.
5. **Find support** opens the Mod sidebar or a new Mod Portal tab beside the file.
6. Suggested cards state capability, publisher, licence, source, permissions, compatibility.
7. User installs; progress stays within the card.
8. On success, current file tab offers **Open with [mod]**.

Current friction:

- Mods are visible as an undifferentiated inventory in global chrome.
- Mod Portal, Mods button, and unknown-file support do not yet form one clear journey.

## 8. Customise toolbars across two monitors

**Goal:** support expert layouts without exposing drag machinery to everyone.

1. User opens **View > Customise workspace** or palette command.
2. Command Manager enters edit mode with labelled drag handles and drop regions.
3. User drags a toolbar by its handle to the second monitor.
4. It becomes an owned detached window at the drop position.
5. User renames the layout and saves it as **Engineering**.
6. App persists monitor-relative placement.
7. If monitor two is absent, toolbar returns docked to the nearest valid region and offers **Restore previous position** when monitor returns.

Current friction:

- Pin, export/import layout, and command buttons are mixed together.
- Drag affordances are not visually separated from action buttons.
- Layout controls occupy permanent primary chrome.

## 9. Recover after a crash

**Goal:** resume without a modal or destructive choice.

1. App restores tabs and opens the newest safe state.
2. A recovery bar appears below the relevant tab/omnibox: "Recovered changes from 03:14."
3. Actions: **Keep recovered**, **Compare**, **Open saved copy**, **Dismiss**.
4. User may continue editing before choosing.
5. Saving writes current state; recovery remains available until safely superseded or dismissed.

Current friction:

- Recovery capability exists but is not integrated into the new hierarchy.
- Technical backup identifiers can leak into implementation-oriented surfaces.

## 10. Ask AI about the open tab

**Goal:** use a chosen model with explicit, visible access.

1. User invokes **AI assistant** from View, palette, or an optional toolbar slot.
2. Right sidebar opens and pushes content left.
3. Header shows provider/model and grant summary, e.g. **Read this file; Ask before edits**.
4. User asks a question.
5. If Read is not granted, an inline card explains scope and offers **Allow once** / **Manage grants**.
6. Answer streams in the sidebar without moving focus from the document.
7. A proposed edit shows a diff card with Apply, Apply all, Reject.
8. Applied changes use the document undo stack.

Current friction:

- AI panel floats over primary toolbar and content.
- The tool list overwhelms chat and exposes "No active document" repeatedly.
- Grants are plain text rather than a concise, interactive permission summary.

## 11. Search and run a command

**Goal:** act without knowing where a command lives.

1. User presses Ctrl+Shift+P.
2. Palette opens centred in the document region, not across the app header.
3. User types "increase font".
4. Results show command, shortcut, context, and availability.
5. Disabled result remains selectable for its reason but Enter does not execute it.
6. Enabled result runs; palette closes; focus returns to prior surface.

Current friction:

- Current palette is a narrow raw input with no result hierarchy.
- It overlaps the floating AI panel.
- Global tool rows remain visible behind it, increasing noise.

## 12. Find across formats

**Goal:** use one Find model for text, PDF, DOCX, web, and project content.

1. Ctrl+F opens an anchored find bar at the top of the document surface.
2. Query and next/previous use the same placement and shortcuts everywhere.
3. Scope defaults to current surface; project-capable formats offer a scope selector.
4. Results report "3 of 12"; navigation maintains context.
5. Esc closes Find and restores focus.

Current friction:

- Find implementations exist per viewer but lack a documented common frame.
- Toolbars make it hard to distinguish document-local controls from shell controls.

## 13. Change workspace without losing place

**Goal:** move among Browse, Write, Create, and Live while preserving documents.

1. User selects workspace from a compact switcher near Command Manager tabs or via palette.
2. Docked panels and toolbars change; tabs and active selection do not.
3. Workspace label briefly confirms the change in the status bar.
4. User can pin the workspace or return to **Follow content**.

Current friction:

- Existing persona/context concepts are scattered across Settings and Command Manager.
- Current build exposes context with labels such as "Following: browse" but does not make the relationship legible.

## 14. Adjust settings and developer diagnostics

**Goal:** keep ordinary settings calm while retaining deep diagnostics.

1. Settings opens in a normal tab.
2. Left navigation groups General, Appearance, Files & downloads, Editing, Accessibility, Privacy & AI, Mods, Keybindings.
3. Search filters setting rows.
4. **Developer** is collapsed at the bottom and off by default.
5. Loaded mods, capture tests, demo state, shell logs, and debug commands live there.
6. Changes apply immediately when reversible; destructive reset actions offer inline undo/restore.

Current friction:

- Loaded mods and demo controls occupy primary chrome.
- Settings is treated like a URL but currently lacks a coherent two-column layout in screenshots.
