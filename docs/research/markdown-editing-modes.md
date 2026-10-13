# Markdown editing modes — product research

Survey of how common Markdown and rich-text products expose **source**, **live preview**, **reading**, and **WYSIWYG** editing; how they switch modes; what happens to **round-trip fidelity** (user-chosen syntax: `*` vs `_`, setext headings, fence style, front matter, raw HTML); and how **unsupported** constructs are handled.

**Audience:** contributors implementing Unibrowser document editing ([ADR 0035](../adr/0035-one-document-model-wysiwyg.md), [document-editor-modes.md](../design/document-editor-modes.md)).

**Method:** public help pages, open-source READMEs, and issue trackers (October 2026). This is research, not a normative spec.

## Mode vocabulary

| Pattern | Typical modes | What the user edits |
| --- | --- | --- |
| **Obsidian-style** | Source, Live Preview, Reading | Still a text file; preview is render overlay or separate view |
| **Typora / Mark Text** | Single surface (optional source toggle in Typora) | WYSIWYG-like; syntax hidden or revealed at caret |
| **Split hybrid** | Source + preview pane (StackEdit, EasyMDE, VS Code) | Plain text in one pane; preview read-only or weakly linked |
| **Dual engine** | Markdown tab vs WYSIWYG tab (Toast UI Editor) | Two editors; conversion on switch |
| **Block WYSIWYG** | One canvas (Notion, CKEditor 5) | Structured document; export to Markdown is lossy optional |

Unibrowser’s target (Bear): **three user-facing modes** on one **shared rich-text document model**, with Markdown and Word as **serialisation plugins**, not separate editors ([ADR 0035](../adr/0035-one-document-model-wysiwyg.md)).

## Comparison table

| Product | Mode switching | Round-trip / style preservation | Unsupported or lossy constructs | Sources |
| --- | --- | --- | --- | --- |
| **Obsidian** | **Reading** vs **Editing** (view switcher, `Ctrl/Cmd+E`); inside Editing: **Live Preview** vs **Source** (status bar, command palette, optional hotkey). Default editing mode in Settings. Side-by-side with `Ctrl/Cmd+click` view switcher. | **Source mode** keeps bytes as typed. **Live Preview** is a text editor with inline render; extra blank lines in source may still appear in LP (design trade-off). **Reading view** uses a different render path than LP; HTML-in-backticks and some edge cases can diverge ([forum](https://forum.obsidian.md/t/live-preview-and-reading-mode-are-very-different/87552)). | Plugin Markdown, embeds, callouts; LP does not hide all syntax (e.g. some list indentation faked). Community plugins add toolbars, not core fidelity rules. | [Views and editing mode](https://obsidian.md/help/edit-and-read), [help source](https://github.com/obsidianmd/obsidian-help/blob/master/en/Editing%20and%20formatting/Views%20and%20editing%20mode.md) |
| **Typora** | Hybrid single pane; **Source code mode** for raw Markdown ([shortcut keys](https://support.typora.io/Shortcut-Keys/)). Optional floating toolbar (1.14+). | Internal AST → save/export; **does not guarantee** preserving `~~~` vs ` ``` `, indented code vs fences, or copy/paste source ([issue #1769](https://github.com/typora/typora-issues/issues/1769)). Syntax preferences can **restrict** list markers (`*` vs `-`) in strict settings ([Markdown Guide](https://www.markdownguide.org/tools/typora/)). Recommends `*` over `_` in docs ([Markdown Reference](https://support.typora.io/Markdown-Reference/)). | Pandoc-only syntax not in Typora’s subset; complex emphasis+code interactions ([#3847](https://github.com/typora/typora-issues/issues/3847)). | [Typora](https://www.typora.io/), [What's New 1.14](https://support.typora.io/What's-New-1.14/) |
| **Mark Text** | WYSIWYG-first (Muya); **floating format bar** on selection; no Obsidian-style source/reading split | Edits through rendered view; export is canonical Markdown from engine, not byte-preserving. | Math, diagrams, GFM extensions per [EDITING.md](https://github.com/marktext/marktext/blob/develop/docs/EDITING.md). | [Mark Text](https://marktext.me/), [EDITING.md](https://github.com/marktext/marktext/blob/develop/docs/EDITING.md) |
| **Zettlr** | **Preview** (WYSIWYG) vs **Raw** (WYSIWYM); status bar “Rendering” toggle; per-element render checkboxes in Settings; `Ctrl/Cmd+Alt+M` proposed for toggle ([PR #6254](https://github.com/Zettlr/Zettlr/pull/6254)) | Raw mode shows syntax; Preview renders in place. Export preview ≠ editor preview for spacing ([Appearance](https://docs.zettlr.com/en/editor/appearance/)). | Fine-grained “which elements render” toggles. | [Appearance](https://docs.zettlr.com/en/editor/appearance/), [Status bar](https://docs.zettlr.com/en/editor/statusbar/), [Settings](https://docs.zettlr.com/en/reference/settings/) |
| **Notion** | No Markdown file modes; **block** canvas; `/` block menu; floating bar for inline marks | Stored as blocks/API JSON; Markdown import/export is conversion, not a live `.md` round trip. | Database views, embeds, synced blocks; “turn into” changes block type. | [Slash commands](https://www.notion.com/help/guides/using-slash-commands), [Shortcuts](https://www.notion.com/help/keyboard-shortcuts) |
| **Milkdown** | Preset-dependent (minimal chrome to Notion-like); plugin menu optional | **ProseMirror doc → remark.stringify**: canonical output (`_`→`*`, indented code vs fences unless `remarkStringifyOptionsCtx`); empty lines need `remarkPreserveEmptyLinePlugin` ([PR #1765](https://github.com/Milkdown/milkdown/pull/1765), [#994](https://github.com/Milkdown/milkdown/issues/994)) | Anything outside schema nodes/marks. | [Milkdown](https://milkdown.dev/), [serializer](https://github.com/Milkdown/milkdown/blob/main/packages/transformer/src/serializer/state.ts) |
| **TipTap / ProseMirror** | Host-defined: **BubbleMenu**, slash commands, separate Markdown extension | Default path is **HTML or JSON** document, not Markdown source fidelity. Markdown via community remark pipelines loses delimiter choice on serialize ([remark-prosemirror](https://github.com/handlewithcarecollective/remark-prosemirror)). | Custom nodes via schema; unknown HTML often stripped unless GHS-style plugins. | [BubbleMenu](https://tiptap.dev/docs/editor/extensions/functionality/bubble-menu), [ProseMirror transforms](https://prosemirror.net/docs/guide/#transform) |
| **Lexical** | No default modes; app supplies toolbar/plugins | `@lexical/markdown` import/export transforms; not designed for untouched-block byte identity. | Custom nodes; markdown transformers are explicit. | [Lexical intro](https://lexical.dev/docs/intro), [@lexical/markdown](https://lexical.dev/docs/packages/lexical-markdown) |
| **Toast UI Editor** | `changeMode('markdown' \| 'wysiwyg')`; Markdown mode has `previewStyle` tab/vertical ([API](https://nhn.github.io/tui.editor/latest/)) | **Known conversion bugs**: `**bold**` → `<strong>` on round trip ([#2708](https://github.com/nhn/tui.editor/issues/2708)); blank lines shrink on repeated switch ([#3093](https://github.com/nhn/tui.editor/issues/3093)); table widgets lost ([#3261](https://github.com/nhn/tui.editor/pull/3261)). | `customHTML`, `widgetRules`, `frontMatter` options; still lossy on complex HTML/tables. | [Toast UI Editor](https://github.com/nhn/tui.editor), [API](https://nhn.github.io/tui.editor/latest/ToastUIEditorCore/) |
| **CKEditor 5 (Markdown)** | WYSIWYG with optional **Markdown data processor** (`getData`/`setData` as GFM) | Docs: **HTML remains recommended**; Markdown strips unsupported rich features ([Markdown feature](https://ckeditor.com/docs/ckeditor5/latest/features/markdown.html)). Round-trip bugs: formatting inside code ([#9177](https://github.com/ckeditor/ckeditor5/issues/9177)), table colspan corruption ([#16794](https://github.com/ckeditor/ckeditor5/issues/16794)), comments/HTML ([#16834](https://github.com/ckeditor/ckeditor5/issues/16834)). | List styles, page breaks, advanced tables; migrator to remark/rehype changes autolink behaviour ([PR #18645](https://github.com/ckeditor/ckeditor5/pull/18645)). | [Markdown output](https://ckeditor.com/docs/ckeditor5/latest/features/markdown.html) |
| **StackEdit / EasyMDE** | Toolbar + **preview toggle** / side preview | Source pane preserves text until user edits; preview is render-only. | Limited to editor feature set. | [StackEdit](https://stackedit.io/), [EasyMDE](https://github.com/Ionaru/easy-markdown-editor) |
| **VS Code** | Editor + optional **preview**; no WYSIWYG body edit | File bytes = source of truth. | Extensions may add custom preview only. | [Markdown in VS Code](https://code.visualstudio.com/docs/languages/markdown) |

## Patterns

### 1. Three modes beat format-specific editors

Products users praise for “just writing” converge on:

1. **Source** — trust and diff-friendly bytes.
2. **Live / WYSIWYG** — semantic editing without caring about `#`.
3. **Reading** — calm, non-editable review (export-like typography).

Obsidian separates **view** (read vs edit) from **editing mode** (LP vs source). Zettlr collapses to two render toggles but same idea. Typora and Mark Text default to WYSIWYG-like with escape hatches to source.

### 2. Fidelity splits into “semantic” vs “cosmetic”

Almost no WYSIWYG Markdown editor promises **byte-identical** save without a **source spare** or **opaque regions**:

- **Semantic round-trip:** headings, lists, emphasis mean the same after edit.
- **Cosmetic round-trip:** same `*` vs `_`, same fence character, same setext vs ATX, same number of blank lines.

Tools that parse → AST → stringify (Milkdown, CKEditor Markdown, Toast UI, Typora export) **normalize cosmetics** unless they keep parallel source spans (the direction in [ADR 0035](../adr/0035-one-document-model-wysiwyg.md)).

### 3. Dual-engine switches are fragile

Toast UI and similar **Markdown ↔ HTML** bridges accumulate open issues on mode switch. A **single document model** with Markdown as an export/import serializer avoids synchronising two editors.

### 4. Unsupported content needs a deliberate story

| Strategy | Example | Risk |
| --- | --- | --- |
| **Opaque / raw block** | HTML comment nodes, `{=html}` in Pandoc workflows | User sees “unknown” chip; still editable in source |
| **Strip on export** | CKEditor Markdown processor | Silent data loss |
| **Emulate + disclose** | ADR 0034 underline in Markdown | User trusts emulation badge |
| **Read-only guard** | Obsidian when plugin missing | Honest but blocks edit |

Unibrowser normative choice: **opaque blocks** + **byte-identical untouched regions** ([ADR 0035](../adr/0035-one-document-model-wysiwyg.md)).

### 5. Toolbars follow the model, not the mode

Notion: inline marks on selection, block types via `/`. ADR 0034: **same semantic block picker and compact toolbar** in every mode; only the **content surface** changes ([document-editor-modes.md](../design/document-editor-modes.md)).

## Implications for Unibrowser

1. **WYSIWYG is default** for new users; source and split preview remain first-class ([ADR 0035](../adr/0035-one-document-model-wysiwyg.md)).
2. **One ProseMirror-style model** for `.md` and `.docx`; mods supply serialisers per [ADR 0030](../adr/0030-core-as-skeleton.md).
3. **Round-trip rule:** untouched blocks copy original source bytes; edited blocks serialize through the mod; unknown syntax stays opaque.
4. **Do not** ship a Toast-style dual editor that rewrites `**` to `<strong>` on toggle.
5. **Reading mode** is a presentation projection of the same model (typography, no carets), not a separate Markdown renderer with divergent rules (lesson from Obsidian Reading vs LP).

## Related docs

- [text-editor-toolbars.md](text-editor-toolbars.md) — toolbar density research
- [ADR 0034: Compact text toolbar](../adr/0034-compact-text-toolbar-and-semantic-styles.md)
- [ADR 0035: One document model](../adr/0035-one-document-model-wysiwyg.md)
- [document-editor-modes.md](../design/document-editor-modes.md)
