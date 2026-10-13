# Compact text toolbar (design)

Specification for the **in-document formatting row** in Text context. Normative behaviour: [ADR 0034](../adr/0034-compact-text-toolbar-and-semantic-styles.md). Research: [text-editor-toolbars.md](../research/text-editor-toolbars.md).

## Visual north stars (Bear concept renders)

Two concept renders define the product direction. **Render 1** is the binding target for the text toolbar and markdown/docx editing chrome. **Render 2** is the north star for **dock panel density** (3D, timeline, layers, colour grading), not for the text toolbar row.

### Render 1 — Markdown editor (binding for this toolbar)

- **Frame:** Dark navy application shell (`#0f1a24` class of values). The editable document sits on a **light paper card** (`#faf9f6`) with a soft shadow inside the frame.
- **Document tab:** Small tab on the card top edge with file name and close **×** (not the main window tab strip).
- **In-card toolbar (this doc):** One slim row on the **top edge of the paper card** (not the global Command Manager strip):
  1. **Format dropdown** at the left — label is the **document format name** (`Markdown ▾`, `Word ▾`, `Plain text ▾`), not a font size and not a heading level. The menu lists block/heading styles (paragraph, H1–H6, block quote, code block). **No font-size control** on this row.
  2. **Inline marks:** Bold, Italic, Inline code — Lucide-style **thin stroke**, monochrome ink on paper, **no tiles, borders, or filled button chrome**.
  3. **Divider** (thin vertical rule, generous horizontal spacing).
  4. **Lists:** Bullet list, Numbered list — separate icon buttons (not a split chevron on the bar).
  5. **Divider**
  6. **Insert:** Link, Image — same icon treatment.
  7. **Divider**
  8. **Overflow:** `⋮` for secondary marks (strikethrough, underline, clear formatting, etc.).
- **Typography in the card body:** Serif-forward body (`Source Serif 4` / Palatino class). Headings in **dark navy**. Block quotes: **pale blue fill**, **left accent bar** in accent blue.
- **Card status line (footer of paper card):** `{format} · {live word count} · {Saved HH:MM | Not saved}` with zoom **`100%`** right-aligned on the card (editor zoom lives here or in overflow — not on the formatting row). Values are computed from the open document and tab dirty/autosave state, not concept-art literals.
- **App bar above the card (not this doc, but same screenshot):** Breadcrumb (`Notes / file.md ▾`), ghost **+ New**, Undo/Redo icons, filled blue **Save**, **⋮**.
- **Left rail:** Icon + small label stacked (Files, Search, Project, 3D, Video, Audio, Code, Paint, Assets, Settings).
- **Right dock:** Tabs **AI Assistant**, **Tools**, **Assets**.

Implementation flag: `body.text-document-card` mounts `#text-compact-toolbar-host` into `.document-card-toolbar-mount` and hides the Command Manager strip for this layout.

### Render 2 — Pro suite (dock panels only)

- **Theme:** Dark.
- **Panel chrome:** Small-caps section headers, compact vertical rhythm.
- **Controls:** Sliders with numeric readouts, toggles in **accent blue**.
- **Use:** Inspector / timeline / layers / colour grading — **do not** copy Render 2 button chrome onto the Render 1 paper toolbar.

## Placement (Render 1)

```
┌─ App title bar (breadcrumb, Save, window controls) ─────────────────────────┐
├─ Left rail │  ┌─ doc tab × ────────────────────────────────────────────┐ │
│            │  ├─ Markdown ▾ │ B I code │ list # │ link image │ ⋮ ────────┤ │  ← compact toolbar
│            │  ├─ document body (serif, paper) ──────────────────────────┤ │
│            │  └─ Markdown · words · Saved ······················· 100% ─┘ │
│            │                                    │ AI │ Tools │ Assets │ dock
└────────────┴────────────────────────────────────┴─────────────────────────┘
```

- Shown when `CommandContext` is **Text** / **Office** with `text_compact_toolbar` families (Markdown, plain text, rich document).
- Hidden for Code, Image, PDF, CAD, Browse.
- Collapsed global chrome (Ctrl+F1) still hides peripheral chrome; the in-card row tracks text editing visibility.

## Default icon set and order (Render 1)

Icons: **Lucide 16 px**, stroke **1.75**, hit target **28–30 px**, **icon-only** except the format dropdown label. Tooltips: label + shortcut + capability note.

| # | Control id | Icon | Notes |
| --- | --- | --- | --- |
| 1 | `unibrowser.control.semantic.block_style` | (label only) | Trigger text = format name: **Markdown**, **Word**, **Plain text** |
| — | *divider* | | |
| 2 | `unibrowser.control.text.bold` | `bold` | Toggle |
| 3 | `unibrowser.control.text.italic` | `italic` | Toggle |
| 4 | `unibrowser.control.text.code_inline` | `code` | Toggle |
| — | *divider* | | |
| 5 | `unibrowser.control.text.list` | `list` | Bullet |
| 6 | `unibrowser.control.text.list_numbered` | `list-ordered` | Numbered |
| — | *divider* | | |
| 7 | `unibrowser.control.text.link` | `link` | |
| 8 | `unibrowser.control.text.insert_image` | `image` | |
| — | *divider* | | |
| 9 | `unibrowser.control.text_compact.overflow` | `ellipsis` | Secondary marks |

**Not on the in-card row:** Undo, Redo, Find, Save, font family, **font size / pt**, editor zoom stepper (card footer or Command Manager when CM visible). Strikethrough and clear formatting default to **overflow**.

## Block-style picker menu

Same semantic ids as ADR 0034 (paragraph, heading_1 … heading_6, blockquote, code_block). Mixed selection uses **Mixed** in menu state; trigger label stays the **format name**, not the active heading.

## Overflow

Same width breakpoints as ADR 0034, but **never** show font size on this row. At narrow widths, drop Image, then Link, then Numbered list, before dropping Bold/Italic.

## Plugins

Unchanged: `surfaces: ["text_compact_toolbar"]` and `contributes.toolbars[]` with `placement: "text_compact"`. In-card styling uses `.document-card-toolbar-mount` tokens when `text-document-card` is active.

## Evidence

- `docs/screenshots/txt-toolbar/bear-render-1-markdown-1280.png`
- `docs/screenshots/txt-toolbar/bear-render-1-docx-1280.png`
- Capture demo: `text-card-render1` + `content-kind-md-preview` (markdown) / rich docx sample (office).

### Walkthrough (1280×800 captures vs Render 1)

**Markdown (`bear-render-1-markdown-1280.png`):** Dark navy work area frames a centred paper card (`#faf9f6`). A small in-card tab reads `sample.md` with ×. The in-card row shows **Markdown** + chevron, dividers, **B / I / code**, bullet and numbered list, **link / image**, then **⋮** — ink on paper, no tiles. Preview body uses serif type; a block quote appears as pale blue with a left accent bar. Card footer shows live word count and save state plus `100%` zoom. Title bar breadcrumb, blue **Save**, expanded left rail (icon + label), right dock **AI Assistant / Tools / Assets**. Global CM strip is hidden (`cm-collapsed` + `text-document-card`).

**Word (`bear-render-1-docx-1280.png`):** Same frame and card chrome; tab `sample-rich.docx`. Format trigger label **Word**; same icon order on the in-card row. Docx reader body on paper; footer uses live word count and save state.

## Related

- [command-system.md](command-system.md)
- [ui-commands.md](../ui-commands.md)
- [ui-implementation-plan.md](ui-implementation-plan.md) — TXT tickets
