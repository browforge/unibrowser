# Workflow walkthrough QA

1280 light, production release binary, isolated `XDG_CONFIG_HOME`, 100% UI scale.
Script: `scripts/capture-workflow-walkthrough.sh`.
PNGs: `docs/screenshots/workflow-walkthrough/`.

Each step records what is in the PNG, then grades the user story. Fail means a dead end, missing feedback, the wrong Command Manager family, confusing copy, or anything that blocks Save.

## 1. Open a file from New tab (US-01)

| Step | Evidence | Literal description | Grade |
| --- | --- | --- | --- |
| New tab | `01-new-tab.png` | Tab New tab. Omnibox `unibrowser://home`. CM family Start: Recent, File, Folder, Project. Card title Start where you left off. Lead: Open a local file or project. Nothing is uploaded unless you choose a network action. Actions: Open file, Open folder, Start a new project. Opened recently lists sample.md and sample.pdf. Downloads and Open Downloads sit under that. Save is in the title bar. | Pass |
| Open from recent | `01-opened-recent.png` | New tab stays. A second tab sample.pdf is selected. CM family is PDF Annotation: Highlight, Underline, Strike, Select, Hand. The Field report page, thumbnail rail, and annotation dock are visible. Save stays in the title bar. | Pass |

US-01: a New tab recent row opens the file in the same tab strip with the PDF handler. The app does not ask about format internals. Open file is also present; that path uses the system picker and was not used for this capture.

## 2. Annotate a PDF and save (US-09, US-05)

| Step | Evidence | Literal description | Grade |
| --- | --- | --- | --- |
| Open | `02-pdf-open.png` | sample.pdf. CM PDF Annotation. Page 1 of 3. Existing yellow highlight on the thumbnail note. Annotation dock shows Opening summary, 2 annotations. Save is in the title bar. Status: Saved, Page 1 of 3, 2 annotations. | Pass |
| Highlight | `02-pdf-annotated.png` | Highlight tooltip is visible over the first CM control. The page still shows the same yellow mark. Annotation count stays 2. No new rect, no dirty tab mark. | Fail |
| Save | `02-pdf-saved.png` | Same page and count after Ctrl+S. Save remains in the title bar. Status still Saved. | Pass |

US-09 Fail: arming Highlight and dragging across the first paragraph did not add a mark or change the count. Existing sidecar annotations prove viewing works. Save was never blocked.

US-05 Pass for Save presence. The new-mark path never became dirty, so save feedback for a fresh annotation was not exercised.

## 3. Edit markdown (US-03, US-06)

| Step | Evidence | Literal description | Grade |
| --- | --- | --- | --- |
| Open | `03-md-open.png` | walk.md. CM family Text: Undo, Redo, Font, 12 pt. Editor shows Demo markdown. Save in the title bar. Status Saved, markdown, Ln 1 Col 1. | Pass |
| Edit | `03-md-edited.png` | Line 11 is Walkthrough note. Status is Unsaved. CM stays Text. Save stays in the title bar. | Pass |
| Save | `03-md-saved.png` | Line 11 remains. Status returns to Saved, Ln 11 Col 19. A small Save chip also appears at the lower right. | Pass |

US-03 / US-06: Text family stays put. Save stays available. Dirty feedback is truthful after sync (`d4dbb9f`). The extra Save chip after Ctrl+S is surplus feedback, not a block.

## 4. Workshop timeline and export (US-10)

| Step | Evidence | Literal description | Grade |
| --- | --- | --- | --- |
| Open | `04-workshop-open.png` | sample.unibrow. CM family Project: Add asset, Add title, Split, Delete clip, overflow chevron. Sources, stage (Unibrowser title), inspector, transport, and timeline (0s to 10s) are distinct. Save in the title bar. | Pass |
| Add title | `04-workshop-title.png` | Add title tooltip on the third CM control. Sources gain New title. Timeline Title track shows TTL Title 3.0s and TTL New title 5.0s. Inspector still shows the original title clip. Save remains. | Pass |
| Export | `04-workshop-export.png` | Same Project prefix. No Export control on the row. No delivery panel. Overflow was not a usable export path in this walk. | Fail |

US-10: adding a title works and Save stays available. Export at 1280 lives only in overflow and this walk did not open a delivery panel.

## 5. Install a mod from the portal (US-13)

| Step | Evidence | Literal description | Grade |
| --- | --- | --- | --- |
| Open | `05-portal-open.png` | Tabs sample.step and Mod portal. CM family Mods: Search, Install, Updates, Permissions, Open VSX network. Discover shows the Open VSX field (network off) and Suggested result CAD file support with licence, local-files note, Contribution details, and a flat Install. Save in the title bar. | Pass |
| Install | `05-portal-install.png` | Same card and Install control. No in-card progress, no Installed state, no Open with [mod] on sample.step. An earlier take of this click showed a toast: Install unibrowser-base-cad, .ubmod catalogue stub, sideload when the package is ready. | Fail |

US-13 Fail: Install does not install a handler. The file tab stays as it is. Product now writes an in-card status when the click lands (`d4dbb9f`). That is feedback, not a completed install.

## 6. Change autosave in settings (US-05)

| Step | Evidence | Literal description | Grade |
| --- | --- | --- | --- |
| Open | `06-settings-open.png` | settings tab. CM family Settings. General: On startup Open a new tab, Downloads Open in new tab, Workspace Follow current, Auto-save on with Every 5 minutes, Open advanced settings. Status: Changes save immediately. Save in the title bar. | Pass |
| Change interval | `06-settings-autosave.png` | The interval select is still Every 5 minutes. The switch stays on. | Fail |

The Auto-save row is present and the status bar says changes save immediately. This walk did not land a new interval. The same select works in the interaction-state sheets.

## 7. Switch themes

| Step | Evidence | Literal description | Grade |
| --- | --- | --- | --- |
| Settings light | `07-theme-light.png` | Same General page as flow 6, light surfaces. | Pass |
| Appearance | `07-theme-appearance.png` | A floating Unibrowser commands window (Dock in main window, Markdown proving, Apply subscript proving) covers the Appearance row and the top of General. Theme is not reachable in this frame. | Fail |
| Dark | `07-theme-dark.png` | Still General on light surfaces. Theme select was not changed. | Fail |

Theme lives under Appearance (Light, Dark, Match system). A prior take in this session reached that page with Theme: Light. The floating command window can cover the category rail and block the change. The shell did not switch to dark in the committed PNGs.

## Fail list

| Fail | Story | What happened |
| --- | --- | --- |
| Highlight drag added no mark | US-09 | Tooltip armed. Count stayed 2. Ticket QA-01. |
| Workshop Export not reached | US-10 | Export is overflow-only at 1280. No delivery panel. Ticket QA-02. |
| Catalogue Install is a stub | US-13 | Toast / in-card stub only. No Open with [mod]. In-card copy shipped in `d4dbb9f`. Rest is ticket QA-03. |
| Autosave interval unchanged | US-05 | Control present. This walk did not change Every 5 minutes. Ticket QA-04. |
| Theme change blocked | Appearance | Floating Unibrowser commands window covers Appearance. Dark not applied. Ticket QA-05. |
| Markdown said Saved while dirty | US-05 | Fixed in `d4dbb9f`. Status now Unsaved after edit. |

Save was available on every document surface. No walk blocked Save.
