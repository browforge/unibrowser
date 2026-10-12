# Compact text toolbar (design)

Specification for the **32 px icon toolbar** in Text document context. Normative behaviour: [ADR 0034](../adr/0034-compact-text-toolbar-and-semantic-styles.md). Research: [text-editor-toolbars.md](../research/text-editor-toolbars.md).

## Placement

```
┌─ Title bar + tabs (existing) ─────────────────────────────────────────┐
├─ Compact text toolbar (32 px) ────────────────────────────────────────┤  ← this doc
├─ Command Manager strip (40 px Compact) ───────────────────────────────┤
├─ Document content ────────────────────────────────────────────────────┤
└─ Status bar ──────────────────────────────────────────────────────────┘
```

- Shown when `CommandContext` is **Text** and the active document exposes `unibrowser.content.rich_text` or `unibrowser.content.text_run` editing (Markdown, plain text, rich document).
- Hidden for Code, Image, PDF, CAD, and Browse contexts (their families stay on the CM strip only).
- Visibility tracks Command Manager: hidden when user collapses chrome (Ctrl+F1).

## Default icon set and order

Icons: **Lucide 16 px**, stroke 1.75, icon-only buttons 28×28 px hit target inside 32 px row. Tooltips: label + shortcut + capability note (emulated/unsupported).

| # | Control id | Icon (Lucide) | Type | Notes |
| --- | --- | --- | --- | --- |
| 1 | `unibrowser.semantic.block_style` | `pilcrow` + label | Dropdown | Shows current style name (e.g. “Heading 2”); see § Block-style picker |
| — | *divider* | | | |
| 2 | `unibrowser.format.bold` | `bold` | Toggle | |
| 3 | `unibrowser.format.italic` | `italic` | Toggle | |
| 4 | `unibrowser.format.strikethrough` | `strikethrough` | Toggle | |
| 5 | `unibrowser.format.code_inline` | `code` | Toggle | Inline code, not code block |
| — | *divider* | | | |
| 6 | `unibrowser.format.link` | `link` | Button | Opens link editor when selection non-empty |
| 7 | `unibrowser.list.bullet` | `list` | Toggle / split | Main: bullet; chevron: numbered, task |
| — | *divider* | | | |
| 8 | `unibrowser.format.clear` | `remove-formatting` | Button | Clears inline marks on selection |
| 9 | `text_compact.overflow` | `ellipsis` | Menu | Low-priority items (see § Overflow) |

**Not on the compact bar (Command Manager strip instead):** Undo, Redo, Find, Replace, Proofing, Export, editor display size, mod-specific insert tools (tables, images) unless mod opts into compact zone.

**Removed from default Text chrome:** document font family, document font size stepper (`14 pt`).

### Secondary marks (overflow menu by default)

Available under overflow chevron unless user pins them in settings (future UI-18):

- Underline (`underline`)
- Subscript / superscript (`subscript`, `superscript`)
- Text colour (`palette`)
- Highlight / background (`highlighter`)
- Block quote toggle (`text-quote`) — duplicate of picker entry for keyboard users
- Code block (`square-code`)
- Horizontal rule (`minus`)

Emulated controls keep the **dotted emulation mark** on the icon corner ([command-system gallery](mockups/quiet-utility/command-system-gallery.html)).

## Block-style picker

Dropdown width: min 160 px, max 240 px. Each row: style name + optional preview glyph (H1 larger than H3).

| Menu item | Semantic id | Default shortcut |
| --- | --- | --- |
| Paragraph | `paragraph` | Ctrl+Alt+0 |
| Heading 1 | `heading_1` | Ctrl+Alt+1 |
| Heading 2 | `heading_2` | Ctrl+Alt+2 |
| Heading 3 | `heading_3` | Ctrl+Alt+3 |
| Heading 4 | `heading_4` | Ctrl+Alt+4 |
| Heading 5 | `heading_5` | Ctrl+Alt+5 |
| Heading 6 | `heading_6` | Ctrl+Alt+6 |
| Block quote | `blockquote` | Ctrl+Shift+9 (proposed) |
| Code block | `code_block` | Ctrl+Shift+` (proposed) |

Mixed block selection shows **Mixed** in the trigger label (`aria-valuetext`).

Capability: plain text → only Paragraph enabled; headings show unsupported reason from ADR 0034 mapping table.

## Overflow

1. **Never wrap** the compact row to a second line.
2. At widths below 720 px (between dividers), move **Clear formatting** then **Link** into the overflow menu.
3. At widths below 560 px, move **Strikethrough** and **Inline code** into overflow.
4. **Block-style picker**, **Bold**, **Italic**, and **List** split stay visible until 400 px; below that, list chevron merges into overflow (bullet only on bar).
5. Mod-contributed icons use the same overflow rules by **priority** (`primary` → `normal` → `low`); complete mod groups move as a unit.

Command Manager overflow (unchanged): Proofing and Export families leave the strip before History; compact bar does not duplicate those commands.

## How plugins add items

1. Manifest `contributes.controls[]` with `surfaces: ["text_compact_toolbar"]` and `content_kinds: ["unibrowser.content.rich_text", …]`.
2. Optional `insert.after`: `unibrowser.format.link` | `unibrowser.list.bullet` | `text_compact.overflow`.
3. Host validates IDs, capability provider, and tooltip contract before load ([CMD-07](ui-implementation-plan.md#cmd-07-mod-contribution-manifest-schema)).
4. VSIX compat lane maps known contributions into the same slots ([extension-host.md](../extension-host.md)).

Mods that need a **labelled** control (e.g. “Insert citation”) should use **Command Manager** or palette, not widen the compact bar.

## Keyboard shortcuts

| Action | Shortcut |
| --- | --- |
| Bold | Ctrl+B |
| Italic | Ctrl+I |
| Strikethrough | Ctrl+Shift+X (proposed; align with palette) |
| Inline code | Ctrl+` |
| Link | Ctrl+K |
| Bullet list | Ctrl+Shift+8 |
| Numbered list | Ctrl+Shift+7 |
| Block quote | Ctrl+Shift+9 |
| Clear formatting | Ctrl+\\ |
| Focus compact toolbar | Alt+F10 (roving toolbar; WAI-ARIA toolbar pattern) |
| Next/previous control | Arrow keys inside toolbar |

Shortcuts resolve through the host keybinding map; tooltips read live bindings ([Mark Text lesson](https://github.com/marktext/marktext/pull/4862): hints must follow user overrides).

## ASCII mock (1280 px viewport)

Title bar omitted. Lucide icons shown as `[icon]`. CM strip abbreviated.

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ [¶ Heading 2 ▾] │ [B] [I] [S] [`] │ [link] [list▾] │ [clear] [⋯]                                          32px │
├──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Text ›  [Undo][Redo] │ Find [⌕][↔] │ Proofing [ABC✓] │ Export [↓]              Following ○  Pin ☐  [CM ⋯] 40px │
├──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                                                  │
│  ## Release notes                                                                                                │
│                                                                                                                  │
│  Ship the **compact toolbar** with semantic headings. Users pick Heading 2 from the picker, not 14 pt.            │
│                                                                                                                  │
│                                                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Selection on “compact” shows `[B]` pressed; underline in overflow would show emulation dot if Markdown policy emulates via HTML.

## Evidence (when implemented)

- Light/dark/hc screenshots at 1280×800: Markdown, plain text (greyed headings), rich document.
- `.cm.json` sibling for compact bar bounds (mirror `shell-density` captures).
- axe pass on toolbar roving focus and `aria-disabled` unsupported buttons.

## Related

- [command-system.md](command-system.md)
- [ui-commands.md](../ui-commands.md)
- [ui-implementation-plan.md](ui-implementation-plan.md) — TXT tickets
