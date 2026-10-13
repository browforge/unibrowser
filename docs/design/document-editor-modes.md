# Document editor modes (UX)

How users switch between **WYSIWYG**, **Source**, **Split**, and **Reading** for Markdown and rich documents. Normative model and serialiser rules: [ADR 0035](../adr/0035-one-document-model-wysiwyg.md). Toolbar semantics: [ADR 0034](../adr/0034-compact-text-toolbar-and-semantic-styles.md), [text-toolbar.md](text-toolbar.md).

Research: [markdown-editing-modes.md](../research/markdown-editing-modes.md).

## Design intent

Users open a **document**, not a “Markdown app” or “Word app”. The file extension chooses the **save format**; the **editing experience** stays familiar: formatted body text on a calm surface, slim toolbars, and optional raw text when they want it.

WYSIWYG is the **default for new profiles** so casual writers never need to learn `#` syntax. Source and Split remain visible in one click for developers and diff-minded users. Reading mode is for review and presentation, not a separate product.

## Visual shell (quiet-utility / render style)

Align with [visual-language.md](visual-language.md): dark slate **workbench** around a **warm off-white paper card** for text and office documents (same family as rich-document mockups and `docx-paper` surfaces).

```
┌─ Title bar + tabs ────────────────────────────────────────────────────┐
├─ Compact text toolbar (32 px) ─────────────────────────────────────────┤
├─ Command Manager (40 px Compact) ────────────────────────────────────┤
├─ Mode strip (28 px) ─── [ WYSIWYG | Source | Split | Reading ] ───────┤  ← this doc
├─ workspace ───────────────────────────────────────────────────────────┤
│  ┌─ optional rail ─┐  ┌─ document card (max-width, soft shadow) ───┐ │
│  │                 │  │  … content …                              │ │
│  │                 │  └───────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────┘
├─ Status bar: mode indicator · host format · save · cursor ────────────┤
```

- **Paper card:** centred column, comfortable line length (~65–75 ch), padding 48–64 px vertical; background `--surface-paper` (warm off-white light / elevated dark).
- **Mode strip:** single row of **segmented text toggles** (not icons-only) sitting **between** CM and content so it is discoverable without stealing toolbar space from formatting.
- **Split:** card divides horizontally (default 50/50) or vertically on narrow width; draggable gutter; source on left, preview on right.

### Mode toggle placement

| Control | Location | Behaviour |
| --- | --- | --- |
| Primary mode | **Mode strip** segmented control | WYSIWYG · Source · Split · Reading |
| Quick read | Status bar **clickable mode chip** (like Obsidian) | Cycles edit ↔ Reading; long-press opens same menu as strip |
| Keyboard | `Ctrl+Alt+V` cycle edit modes; `Ctrl+E` toggle Reading (aligned with Obsidian muscle memory where harmless) | Configurable in Settings |
| Command palette | “Document: Switch to … mode” | Four explicit entries |

**Reading** greys the mode strip except “Edit” escape hatch. **Split** remembers last split ratio per workspace.

### Relationship to Obsidian / Zettlr

- Obsidian separates **view** (book vs pen) from **LP vs Source**; we expose **four named modes** in one strip to reduce hunting in nested menus ([Obsidian help](https://obsidian.md/help/edit-and-read)).
- Zettlr’s status bar “Rendering” toggle maps to our **WYSIWYG vs Source** within edit modes ([Zettlr status bar](https://docs.zettlr.com/en/editor/statusbar/)).

## Per-mode behaviour

### WYSIWYG (default)

- Caret in formatted text; block handles for drag (future); slash menu optional phase-2.
- Typora-like: syntax for the **current block** may appear when the caret enters it (optional setting “Show Markdown while editing”).
- **Opaque** blocks: muted card, label “Raw Markdown (preserved)”, click opens mini source editor or jumps Source mode at span.
- Images: inline preview with alt text tool on selection.

### Source

- Monospace buffer showing **serialised projection** with `source_map` fidelity (unchanged regions match file bytes).
- Gutter: proofing, optional heading outline.
- Compact toolbar **still works**: toggling bold rewrites model and marks span dirty; buffer updates without reformatting untouched lines.
- Status: “Markdown source” / “DOCX projection unavailable — use WYSIWYG” for binary hosts.

### Split

- Left: Source as above. Right: preview render of **model** (same engine as WYSIWYG, read-only).
- Scroll sync: approximate; selection in source highlights block in preview.
- Toolbar commands apply to **focused pane**; default focus = source.

### Reading

- Same typography as WYSIWYG; no caret, no toolbar edits (toolbar hidden or disabled).
- Links active; comments/annotations future.
- Export and Print actions remain in CM strip.

## Toolbar and heading picker (all edit modes)

Per [text-toolbar.md](text-toolbar.md):

| Element | WYSIWYG | Source | Split | Reading |
| --- | --- | --- | --- | --- |
| Block-style picker | Active | Active (maps range → block) | Active in focused pane | Hidden |
| Bold / italic / … | Active | Active | Active in focused pane | Hidden |
| Lists / link | Active | Active | Active in focused pane | Hidden |
| CM: Undo, Find, Proofing | Active | Active | Active | Undo hidden; Find optional |

Capability greying follows **host format** (Markdown vs DOCX vs plain), **not** mode.

## Persistence

| State | Stored where |
| --- | --- |
| Default mode for new tabs | User settings (`document.default_mode = wysiwyg`) |
| Last mode per file path | Workspace layout JSON (per tab) |
| Split ratio | Workspace layout JSON |
| “Show Markdown while editing” | User settings |

Mode is **not** written into the document file.

## Edge cases

### Mode switch

| Case | UX |
| --- | --- |
| Dirty source buffer | Inline banner: “Source has uncommitted changes” with **Apply** / **Discard** before WYSIWYG |
| Parse error in source | Stay in Source; underline error range; WYSIWYG shows “Fix source to continue” on attempted switch |
| Large file (> host limit) | Offer Source only with banner; WYSIWYG disabled with reason |

### Round-trip and trust

| Case | UX |
| --- | --- |
| User never edits | Save produces **identical** file bytes (Markdown) |
| User edits one paragraph | Only that paragraph may change delimiter style; status bar optional “1 block reformatted” on first save after edit |
| Opaque block | Never auto-expanded; command “Parse as Markdown” explicit with preview |
| Emulated underline (MD) | Emulation dot on toolbar per ADR 0034 |

### Host format

| Case | UX |
| --- | --- |
| `.docx` | No Source of full OOXML; Source = plain-text preview optional future; modes: WYSIWYG + Reading |
| `.txt` | WYSIWYG = plain paragraph; heading picker greyed |
| Read-only session (history replay failed) | All edit modes disabled; Reading allowed |

### Chrome density

- Ctrl+F1 collapses CM **and** compact toolbar; **mode strip stays** (one line) so users do not lose orientation.
- At 1280×800, mode strip uses abbreviated labels: **Visual · Source · Split · Read**.

### Proofing and AI

- Proofing runs on **model text** in all edit modes.
- AI proposed edits apply to model; diff preview shows semantic blocks, not raw Markdown unless user expands.

## Accessibility

- Mode strip: `role="tablist"`, `aria-selected` on active mode.
- Reading mode: `aria-readonly="true"` on document region.
- Announce mode changes via live region (“Visual editing”, “Source editing”).

## Evidence (when implemented)

Screenshots to add under `docs/screenshots/`:

- `document-modes-wysiwyg-light.png` — paper card, mode strip, compact toolbar
- `document-modes-split-dark.png` — split with sync selection
- `document-modes-opaque-block.png` — raw preserved block

## Implementation tickets

See **Phase L** in [ui-implementation-plan.md](ui-implementation-plan.md) (DOC-01–DOC-10).

## Related

- [ADR 0035](../adr/0035-one-document-model-wysiwyg.md)
- [markdown-editing-modes.md](../research/markdown-editing-modes.md)
- [history-panel.md](history-panel.md)
