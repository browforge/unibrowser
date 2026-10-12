# Text editor toolbar research

Survey of how popular Markdown and rich-text products expose formatting: toolbar density, default controls, block/heading pickers, overflow, and whether the surface is source Markdown, hybrid live preview, or full WYSIWYG.

**Audience:** contributors choosing the Unibrowser compact text toolbar ([ADR 0034](../adr/0034-compact-text-toolbar-and-semantic-styles.md), [text-toolbar.md](../design/text-toolbar.md)).

**Method:** public docs, open-source READMEs, and vendor help pages (October 2026). Where a product has no fixed toolbar (keyboard-first), that is noted explicitly.

## Comparison table

| Product | Toolbar layout & density | Default controls | Block / heading control | Overflow | Markdown vs WYSIWYG | Sources |
| --- | --- | --- | --- | --- | --- | --- |
| **Typora** | Historically chromeless; optional **floating** toolbar (1.14+), not a full-width ribbon | Bold, italic, underline, strike, code, link, image via menus/context; floating bar when enabled | Heading levels via `#` syntax, menu, or shortcuts; no persistent font-size ribbon | N/A (minimal chrome) | Hybrid live preview; syntax hidden while typing | [Typora 1.14](https://support.typora.io/What's-New-1.14/), [shortcut keys](https://support.typora.io/Shortcut-Keys/), [product page](https://www.typora.io/), [toolbar discussion](https://github.com/typora/typora-issues/issues/2412) |
| **Obsidian** | **No core toolbar**; community plugins add top ribbon, selection bubble, or bottom bar | Core: slash-less Markdown + hotkeys; plugins add bold, lists, H1–H6, etc. | Plugins expose **Text style** dropdown (H1–H6, paragraph) on selection bar | Plugin-defined; Awesome Format Bar moves whole groups to overflow menu | Source Markdown in editor; preview is separate pane | [Text Formatting Toolbar plugin](https://github.com/jsmorabito/obsidian-text-formatting-toolbar), [Awesome Format Bar](https://community.obsidian.md/plugins/awesome-format-bar), [Power Editor](https://github.com/obsidian-power-plugins/obsidian-power-editor) |
| **Notion** | **No fixed doc toolbar**; **floating bar on text selection**; `/` block menu | Selection bar: bold, italic, underline, strike, link, colour; `/` for blocks | `/h1`–`/h3`, `/turn…` to change block type | Block menu search narrows options | Block WYSIWYG; stored as structured blocks, not `.md` files | [Slash commands](https://www.notion.com/help/guides/using-slash-commands), [keyboard shortcuts](https://www.notion.com/help/keyboard-shortcuts) |
| **iA Writer** | **No formatting toolbar** (by design) | Focus on writing; styles via `#` Markdown only | Headings via `#` … `######` | N/A | Markdown source with typographic preview themes | [iA Writer features](https://ia.net/writer) |
| **Mark Text** | **Floating inline format bar** on selection (compact icon row) | Bold, italic, underline, strike, inline code, math, link, image, clear format | Block types via Markdown typing; no dedicated heading dropdown on bar | Pop-up only; no document ribbon | WYSIWYG-style live render (Muya) | [EDITING.md](https://github.com/marktext/marktext/blob/develop/docs/EDITING.md), [Mark Text site](https://marktext.me/) |
| **StackEdit** | **Fixed top toolbar** above editor (moderate density) | Bold, italic, heading, quote, lists, link, image, table, etc. | Heading control in toolbar (Markdown prefix insertion) | Horizontal scroll or grouped buttons on narrow widths | Split or toggle: Markdown source + optional preview | [StackEdit](https://stackedit.io/) |
| **HackMD** | **Top toolbar** + optional side panels | Standard MD buttons (bold, italic, headers, lists, code, link) | Header button inserts `#` levels | Toolbar wraps or hides lesser-used items on small screens | Hybrid editor with live preview mode | [HackMD](https://hackmd.io/) |
| **GitHub (comments & `.md`)** | **Single slim row** above `textarea` (`markdown-toolbar-element`) | Bold, italic, header, quote, code, link, image, lists, task list, mention | `<md-header>` cycles heading prefixes | Fixed set; no ribbon overflow pattern in component | Markdown source in `textarea`; rendered preview separate | [markdown-toolbar-element](https://github.com/github/markdown-toolbar-element), [formatting syntax](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax) |
| **VS Code (Markdown)** | **No rich-text toolbar** in editor; language features in command palette | Bold etc. via extensions or keybindings; preview is read-only HTML | `#` in source; outline view for headings | N/A | Source-first; optional preview pane | [VS Code Markdown](https://code.visualstudio.com/docs/languages/markdown) |
| **Google Docs** | **Full toolbar** (~40 px row) + menus | Font family/size **and** paragraph tools | **Styles** dropdown: Normal, Title, Subtitle, Heading 1–6 | Toolbar collapses into **More** (⋯) on narrow windows | Full WYSIWYG; styles stored semantically | [Paragraph styles](https://support.google.com/docs/answer/1663349), [shortcuts](https://support.google.com/docs/answer/179738) |
| **Microsoft Word** | **Home ribbon** + **Styles gallery** (first row on ribbon) | Font box + **Quick Styles** gallery | **Paragraph styles** (Heading 1–9, Normal, etc.); gallery overflow arrow | Styles gallery **More**; ribbon **Collapse** | WYSIWYG; named styles in file | [Styles gallery](https://support.microsoft.com/en-us/word/add-and-remove-styles-from-the-quick-styles-gallery), [Press Store styles](https://www.microsoftpressstore.com/articles/article.aspx?p=2952389&seqNum=5) |
| **TinyMCE** | Configurable **icon toolbar** (often one row) | Bold, italic, lists, link, image; plugins add rows | `blocks` / `format` dropdown for headings & paragraphs | `toolbar_mode: 'sliding'` or `'wrap'` | HTML WYSIWYG; outputs HTML | [Toolbar options](https://www.tiny.cloud/docs/tinymce/latest/toolbar-configuration-options/) |
| **CKEditor 5** | Configurable toolbar; default demo is **one dense row** | Undo, heading dropdown, font family/size, colours, inline styles, lists | **`heading` dropdown** (paragraph + H1–H6 configurable) | **`shouldNotGroupWhenFull`** or nested dropdowns (`label` + `items`) | HTML WYSIWYG | [Toolbar setup](https://ckeditor.com/docs/ckeditor5/latest/getting-started/setup/toolbar.html), [Headings](https://ckeditor.com/docs/ckeditor5/latest/features/headings.html) |
| **Quill (Snow)** | **Fixed top bar** (~42 px), icons + dropdowns | Inline formats, lists, indent, **header dropdown**, font/size/colour | `{ header: [1,2,3,4,5,6,false] }` — `false` clears to body | Toolbar array is fixed; custom HTML toolbar possible | Delta/HTML WYSIWYG | [Toolbar module](https://quilljs.com/docs/modules/toolbar/) |
| **TipTap / ProseMirror** | **Host-defined**; common pattern is **BubbleMenu** on selection | Project adds bold, italic, link buttons to floating menu | Block types via commands (often slash menu or separate dropdown) | Floating UI positions menu; no classic overflow | WYSIWYG on ProseMirror doc | [BubbleMenu](https://tiptap.dev/docs/editor/extensions/functionality/bubble-menu), [Floating toolbar example](https://tiptap.dev/docs/ui-components/utils-components/floating-element) |
| **Milkdown** | **Plugin-based**; presets mirror VS Code–like minimal chrome or Notion-like | `@milkdown/plugin-menu` optional menu bar | Heading via commands / slash | Depends on preset | WYSIWYG Markdown (CST) | [Milkdown docs](https://milkdown.dev/) |
| **Toast UI Editor** | **Top toolbar** in WYSIWYG mode; **Markdown mode** hides most chrome | Bold, italic, strike, heading, quote, lists, table, link, image | Heading dropdown in WYSIWYG toolbar | 2-row toolbar on small widths in some skins | **Dual mode**: Markdown tab vs WYSIWYG tab | [Toast UI Editor](https://github.com/nhn/tui.editor) |
| **EasyMDE** | **Toolbar row** attached to CodeMirror (icons + tooltips) | Bold, italic, headings, quote, link, image, preview toggle | Heading buttons insert `#` prefixes | Toolbar items configurable; can hide buttons | Markdown source + side/bottom preview | [EasyMDE](https://github.com/Ionaru/easy-markdown-editor) |
| **Lexical** | **No default toolbar**; `@lexical/react` ToolbarPlugin is app code | Examples show compact icon buttons | Heading via custom dropdown plugins | App-defined | React WYSIWYG framework | [Lexical playground](https://lexical.dev/docs/intro) |

## Patterns

### 1. Markdown-first products split “chrome” from “formatting”

Writers like Typora, iA Writer, and VS Code keep the window quiet and rely on syntax + shortcuts. When they add formatting UI, it is almost always **selection-local** (Mark Text, Typora 1.14 float bar, Obsidian plugins), not a labelled ribbon that competes with the title bar.

### 2. Block type is a dropdown; font size is not

Google Docs and Word expose **paragraph styles** (Normal, Heading 1–6). CKEditor and Quill put **heading** in a dropdown and treat “paragraph” as clearing block format (`false` in Quill). GitHub’s toolbar inserts `#` prefixes instead of changing font metrics. **Font size in the toolbar implies a presentation model Markdown files do not have.**

### 3. Hybrid editors use two tiers

- **Tier A:** slim fixed or floating **icon** strip for inline marks (bold, italic, link).
- **Tier B:** block semantics via picker, slash command, or typing `#` .

Notion goes further: slash menu for blocks, floating bar only for inline marks.

### 4. Overflow is “drop whole groups”, not wrap

CKEditor groups into dropdowns when full; Quill uses a fixed array; Unibrowser’s Command Manager already specifies **complete-group overflow** ([command-system.md](../design/command-system.md)). GitHub’s component uses a single row with no wrap.

### 5. WYSIWYG embeddables default to dense icon rows

TinyMCE, CKEditor, and Quill Snow ship **one primary toolbar row** with icon toggles and a few dropdowns (heading, list, colour). Labels appear on hover (tooltips), not under every button.

### 6. Emulation is normal

Markdown hosts emulate underline/colour with HTML; Word “Heading 1” is a named style, not a literal HTML `<h1>`; browsers apply default CSS to `<h1>`. Products rarely hide controls when the file cannot store a feature; they **grey out**, **approximate**, or show a **note** (aligned with Unibrowser `emulated` capability).

## Lessons for Unibrowser

1. **Promote common text marks to a dedicated 32 px icon toolbar** with tooltips; keep History, Find, Proofing, and Export on the Command Manager strip (compact 40 px) per [ADR 0034](../adr/0034-compact-text-toolbar-and-semantic-styles.md).
2. **Replace document “font size” with a semantic block-style picker** (Paragraph, H1–H6, Quote, Code block) for Markdown and rich text; keep **editor zoom / display size** separate and labelled.
3. **Prefer selection bubble for secondary marks** only after the fixed strip exists; do not duplicate the same bold button in three places.
4. **Overflow:** move Proofing and Export groups first; never split Bold/Italic/Link from the compact strip.
5. **Plugins** contribute commands to either the compact strip (`toolbar_slot: text_compact`) or the CM strip (`command_manager`), not ad hoc HTML toolbars ([ADR 0028](../adr/0028-platform-content-kinds-and-plugin-scope.md)).
6. **Show emulated styles** with the existing `.cm-emulation-mark` and tooltip note; placement stays stable ([ADR 0027](../adr/0027-command-system-capabilities-and-extensibility.md)).

## Related docs

- [ADR 0034: Compact text toolbar and semantic styles](../adr/0034-compact-text-toolbar-and-semantic-styles.md)
- [text-toolbar.md](../design/text-toolbar.md)
- [command-system.md](../design/command-system.md)
- [UI implementation plan](../design/ui-implementation-plan.md) (TXT tickets)
