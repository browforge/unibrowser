# Visual QA final matrix

Date: 2026-10-07

## Method

The release application was rebuilt before capture. Each application image is
1280 x 800. Each labelled comparison places the application on the left and the
reference on the right at 1280 x 800 per half. Every PNG in
`docs/screenshots/vqa-final/` was opened and inspected.

Grades:

- **Pass**: the named structure and behaviour correspond to the reference.
- **Minor drift**: the structure is present, with spacing, density, typography,
  colour, or fixture differences.
- **Fail**: a required state is absent or reports false information.

The image/SVG and rich-document dark comparisons use the closest dark command
system reference. Rail comparisons use the closest document reference.
Chrome-collapsed uses the specification board, so its grade emphasises chrome.

## Matrix

| Screen | Theme | Chrome | Command Manager | Rail | Main work region | Right dock | Bottom / status | Overall |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Document + Problems + AI | Light | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Document + Problems + AI | Dark | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| PDF annotation | Light | Minor drift | **Fail** | Pass | Minor drift | Minor drift | Minor drift | Minor drift |
| PDF annotation | Dark | Minor drift | **Fail** | Pass | Minor drift | Minor drift | Minor drift | Minor drift |
| Image / SVG | Light | Minor drift | Minor drift | Pass | Minor drift | Minor drift | Minor drift | Minor drift |
| Image / SVG | Dark | Minor drift | Minor drift | Pass | Minor drift | Minor drift | Minor drift | Minor drift |
| Rich document | Light | Minor drift | Minor drift | Pass | Minor drift | Pass | Minor drift | Minor drift |
| Rich document | Dark | Minor drift | Minor drift | Pass | Minor drift | Pass | Minor drift | Minor drift |
| New tab | Light | Minor drift | Minor drift | Pass | Minor drift | Pass | Minor drift | Minor drift |
| New tab | Dark | Minor drift | Minor drift | Pass | Minor drift | Pass | Minor drift | Minor drift |
| Rail collapsed | Light | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Rail collapsed | Dark | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Rail expanded | Light | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Rail expanded | Dark | Minor drift | Minor drift | Pass | Pass | Minor drift | Minor drift | Minor drift |
| Chrome collapsed | Light | Pass | Pass | Pass | Pass | Minor drift | Minor drift | Pass |
| Chrome collapsed | Dark | Pass | Pass | Pass | Pass | Minor drift | Minor drift | Pass |

## Region findings

### Document with Problems and AI

- **Chrome:** both shell rows, active tab, breadcrumb, Save action, and window
  controls are stable. The tab and breadcrumb remain denser than the reference.
- **Command Manager:** Code controls are visible without overlap. Group rhythm
  remains tighter than the reference.
- **Rail:** the 40 px icon rail and lower controls correspond.
- **Main:** real Monaco code, line numbers, token colour, diagnostics, and
  scrollable editor content are visible with the Command Manager mounted.
- **AI:** the dock, tabs, permission state, transcript, composer, and Send
  action are present. The dock is more compressed than the reference.
- **Problems / status:** the panel is populated and bounded. Extra tabs and
  filtering controls make this region busier than the reference.

### PDF annotation

- **Chrome / rail:** the shell identity and collapsed rail correspond.
- **Command Manager:** Select, Note, Area, page, view, and Export PDF are
  visible. The page field incorrectly reports `1 / 1` for a three-page PDF.
- **Pages:** three thumbnails and full-height fit-width paper geometry are
  present.
- **Main:** representative text, highlights, and area annotation are visible.
- **Annotations:** tool, border, reply, and resolve controls are present.
- **Right side:** AI is closed, preserving space for the PDF task.
- **Status:** page and zoom state are visible but remain denser than reference.

### Image / SVG

- **Chrome / Command Manager / rail:** the Image context is truthful and the
  shell remains coherent.
- **Main:** preview is the dominant surface. Raw source is hidden unless Source
  mode is selected.
- **Inspector:** structured properties occupy the right inspector rail. AI is
  closed.
- **Fixture:** the 120 x 80 SVG provides weak visual evidence for scaling and
  detailed inspection.

### Rich document

- **Chrome / Command Manager / rail:** standard shell regions remain coherent.
- **Main:** rich text appears on a centred paper page with margins and a warm
  surround. Conversion and debug controls are absent from the common path.
- **Right side:** AI is closed.
- **Lower region:** the paper ends well above the available workspace, leaving
  an abrupt unused surround below it.

### New tab

- **Chrome:** one New tab is present. No previous Home or directory tab remains.
- **Command Manager:** Start actions correspond to the page.
- **Main:** Start heading, Open file, Open folder, New project, structured
  Recent, and Downloads regions are present.
- **Right side:** AI is closed.
- **Data:** current recent records have no relative timestamps and Downloads is
  empty, making the page less complete than the reference.

### Rail collapsed and expanded

- **Collapsed:** the rail is the specified 40 px icon-only state.
- **Expanded:** the rail is 188 px, labels are readable, and the Files panel
  contains usable content.
- **Other regions:** editor layout remains stable in both states.

### Chrome collapsed

- **Chrome:** only the approximately 40 px title row remains. There is no
  Command Manager row.
- **Main:** real Monaco code remains painted and correctly sized after collapse.
- **Other regions:** rail, AI, and status remain stable.

## Remaining tickets

1. **S1: PDF page control is false.** It reports `1 / 1` while thumbnails and
   the document contain three pages.
2. **S2: New-tab data is less complete than the reference.** Recent rows lack
   relative timestamps and the current Downloads region is empty.
3. **S2: Rich-document paper height is weak.** The page ends early and leaves
   an abrupt lower surround.
4. **S2: SVG evidence is weak.** Replace the 120 x 80 fixture with a larger,
   representative asset.
5. **S3: Shell density still drifts.** Command groups, status content, and some
   title-region spacing remain tighter than the references.
6. **S3: Problems content is generic.** The fixture produces 16 spelling rows
   rather than three representative diagnostics.

## Verdict

No structural VQA blocker remains. Monaco paints with the Command Manager
visible, captures use the real product presentation, Home replacement is
consistent across file types, and all required scenario layouts are present.
The PDF page count is the only remaining false-state failure.
