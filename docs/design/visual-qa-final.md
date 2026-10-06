# Visual QA final matrix

Date: 2026-10-06

## Method

The application was rebuilt before capture. Each application image is a fresh
1280 x 800 capture. Each comparison is 2560 x 800: the application is on the
left and the reference is on the right, with both halves forced to 1280 x 800.
All application captures and all side-by-sides in
`docs/screenshots/vqa-final/` were opened and inspected.

Grades:

- **Pass**: structure, hierarchy, and visual treatment correspond to the
  reference.
- **Minor drift**: the intended structure is present, but spacing, density,
  typography, colour, or fixture content differs.
- **Fail**: a required region, state, or relationship is absent or materially
  different.

The image/SVG and rich-document dark comparisons use
`command-system-gallery-dark.png` because there is no dedicated dark screen
mockup. Rail comparisons use the nearest document-screen reference. The
chrome-collapsed comparison uses `chrome-before-after.png`, which is a
specification board rather than a full application screen. Grades for those
screens therefore emphasise the named region.

## Matrix

| Screen | Theme | Chrome | Command Manager | Rail | Main work region | Right dock | Bottom / status | Overall |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Document + Problems + AI | Light | Minor drift | Minor drift | Pass | Minor drift | Minor drift | Minor drift | Minor drift |
| Document + Problems + AI | Dark | Minor drift | Minor drift | Pass | Minor drift | Minor drift | Minor drift | Minor drift |
| PDF annotation | Light | Minor drift | **Fail** | Pass | **Fail** | **Fail** | Minor drift | **Fail** |
| PDF annotation | Dark | Minor drift | **Fail** | Pass | **Fail** | **Fail** | Minor drift | **Fail** |
| Image / SVG | Light | Minor drift | **Fail** | Pass | **Fail** | **Fail** | Minor drift | **Fail** |
| Image / SVG | Dark | Minor drift | **Fail** | Pass | **Fail** | **Fail** | Minor drift | **Fail** |
| Rich document | Light | Minor drift | Minor drift | Pass | **Fail** | Minor drift | **Fail** | **Fail** |
| Rich document | Dark | Minor drift | Minor drift | Pass | **Fail** | Minor drift | **Fail** | **Fail** |
| New tab | Light | **Fail** | **Fail** | Minor drift | **Fail** | **Fail** | Minor drift | **Fail** |
| New tab | Dark | **Fail** | **Fail** | Minor drift | **Fail** | **Fail** | Minor drift | **Fail** |
| Rail collapsed | Light | Minor drift | Minor drift | Pass | Minor drift | Minor drift | **Fail** | Minor drift |
| Rail collapsed | Dark | Minor drift | Minor drift | Pass | Minor drift | Minor drift | **Fail** | Minor drift |
| Rail expanded | Light | Minor drift | Minor drift | **Fail** | Minor drift | Minor drift | **Fail** | **Fail** |
| Rail expanded | Dark | Minor drift | Minor drift | **Fail** | Minor drift | Minor drift | **Fail** | **Fail** |
| Chrome collapsed | Light | Minor drift | **Fail** | Pass | Minor drift | Minor drift | Minor drift | **Fail** |
| Chrome collapsed | Dark | Minor drift | **Fail** | Pass | Minor drift | Minor drift | Minor drift | **Fail** |

## Region findings

### Document with Problems and AI

- **Chrome:** the two-row dark-slate frame, breadcrumb, Save action, and SVG
  window controls correspond to the mockup. The application tab and breadcrumb
  are denser, and the active-tab geometry still differs slightly.
- **Command Manager:** populated Code controls are visible and bounded. The
  mockup has a more deliberate family rhythm and page-wide distribution.
- **Rail:** the collapsed icon rail has the expected width, dark treatment, and
  bottom controls.
- **Main:** real Monaco content, gutter, line numbers, token colour, diagnostics,
  and one continuous surface are present. Fixture text and editor proportions
  differ from the reference.
- **AI:** the joined right dock, tabs, permission chip, transcript edge, composer,
  and Send action are present. It is narrower and more compressed.
- **Problems / status:** all three rows are visible and bounded above the status
  bar. The application adds Output and Terminal tabs and filter controls, so the
  region is busier than the mockup.

### PDF annotation

- **Chrome:** title, breadcrumb, Save, controls, and the dark frame correspond.
- **Command Manager:** PDF Annotation is truthful and populated, but Select,
  Note, Area, page, view, and Export PDF controls from the mockup are absent.
- **Pages:** the left thumbnail region and active-page treatment are present.
  The one-page fixture cannot demonstrate the mockup's multi-page navigation.
- **Page surface:** the warm surface, white page, shadow, yellow highlight, and
  red area annotation are present. The demo PDF has almost no document content,
  the page is shallow, and the page scale does not correspond to the reference.
- **Annotations:** type, page, label, author, and time are present. Selected-tool
  properties, border properties, reply, and resolve controls are missing.
- **Right side:** the unrelated AI dock remains open, while the reference gives
  that width to the PDF and annotation workspace.
- **Status:** annotation save state, count, page, and zoom are present; file size
  is absent.

### Image / SVG

- **Chrome / rail:** shell identity and collapsed rail correspond to the design
  language.
- **Command Manager:** the Image context is truthful, but the visible family is
  too sparse and does not expose the object and transform hierarchy shown in the
  capability reference.
- **Main:** the SVG preview paints, but it occupies a small black preview block.
  A large raw-source panel and a basic Selection form dominate the work region.
  This does not correspond to a centred image canvas or the structured object
  workspace.
- **Right side:** the AI dock is unrelated to image inspection and leaves no
  proper properties dock.

### Rich document

- **Chrome / rail / AI:** the standard shell regions remain coherent.
- **Command Manager:** Office is truthful and populated, though grouping and
  mixed-state presentation remain less complete than the capability gallery.
- **Main:** rich text is readable, including bold and italic emphasis, but it is
  pinned to a flat host edge with conversion notes and HTML-style controls.
  There is no centred paper surface, document margins, or page hierarchy.
- **Bottom:** the reference document screen includes a populated Problems
  region; this capture leaves a large empty lower strip.

### New tab

- **Chrome:** an `Ubuntu` directory tab remains beside New tab, the location is
  `unibrowser://home`, and legacy Home / Downloads / Download manager / New
  project / Settings controls appear. This does not match the compact Start
  chrome.
- **Command Manager:** Browse and text-format controls are shown instead of the
  Start family with Open File, Open Folder, Project, and Recent.
- **Main:** the heading, explanatory copy, primary actions, structured recent
  list, and downloads progress are absent. Only four bare filenames appear.
- **Right side:** AI remains docked although the reference new tab is a quiet,
  full-width start surface.
- **Rail / status:** the rail is recognisable, but the selected destination and
  bottom status do not correspond to the Start screen.

### Rail collapsed and expanded

- **Collapsed:** the 40 px icon-only rail corresponds to the specification.
- **Expanded:** labels appear, but the rail is about 80 px rather than the
  specified 188 px panel. It does not provide enough width for a useful panel
  or hierarchy and materially compresses labels.
- **Other regions:** editor and AI remain stable in both rail states. The
  comparison reference includes Problems, which is absent in these rail-only
  captures.

### Chrome collapsed

- **Title row:** the 40 px title row remains compact and coherent.
- **Command Manager:** the second row is still visible. The requested
  `cm-collapsed` state therefore does not produce the 40 px total chrome
  specified by `chrome-before-after.png`.
- **Work region:** the editor, rail, AI dock, and status remain stable, so the
  failure is isolated to collapse state application.

## Ranked defects

1. **S0 - VQA-16: New tab renders an incomplete home projection.** It exposes a
   residual directory tab and legacy destination row, lacks the Start command
   family, structured recent/download content, and primary actions, and keeps
   AI open.
2. **S1 - VQA-17: Chrome collapse demo state does not collapse the Command
   Manager.** Both themes retain the second 40 px row, so total chrome remains
   80 px.
3. **S1 - VQA-18: PDF Command Manager lacks core PDF controls.** Add Select,
   Note, Area, page, view/zoom, and Export PDF controls with stable compact
   overflow.
4. **S1 - VQA-19: PDF fixture and workspace do not prove realistic fit-width
   viewing.** Replace the one-line, one-page fixture with a multi-page document
   containing representative text; preserve full-height paper geometry and
   page navigation.
5. **S1 - VQA-20: SVG workspace is split between a tiny preview and raw source.**
   Promote the preview to the main work surface and move object properties to a
   deliberate inspector; raw source should be an explicit mode.
6. **S1 - VQA-21: Rich document lacks a paper/page presentation.** Remove
   conversion/debug chrome from the common path and render content on a centred
   page with readable margins and a warm surround.
7. **S1 - VQA-22: Expanded rail width and content are incomplete.** Implement
   the specified 188 px expanded state and give labels enough width to form a
   useful navigation panel.
8. **S2 - VQA-23: PDF annotation properties and actions are absent.** Add the
   selected annotation's tool/border properties plus reply and resolve actions.
9. **S2 - VQA-24: Scenario-specific docks are not controlled.** PDF, image, and
   New tab captures retain the AI dock even where references allocate the area
   to the primary task.
10. **S3 - VQA-25: Final shell spacing still drifts from the references.**
    Normalise active-tab height, Command Manager group rhythm, status density,
    and title-region vertical alignment across light and dark themes.

## Verdict

The document + Problems + AI screen and collapsed rail are functionally close,
with minor visual drift. PDF has the correct workspace skeleton but still fails
the command and realistic-document regions. Image/SVG, rich document, New tab,
expanded rail, and collapsed chrome retain structural failures. Final visual
acceptance is blocked by VQA-16 through VQA-22.
