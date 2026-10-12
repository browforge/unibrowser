# ADR 0034: Compact text toolbar and semantic block styles

- **Status:** Accepted
- **Date:** 2026-10-12
- **Relates to:** [ADR 0010](0010-ui-commands-and-command-manager.md), [ADR 0028](0028-platform-content-kinds-and-plugin-scope.md), [ADR 0027](0027-command-system-capabilities-and-extensibility.md), [command-system.md](../design/command-system.md)

## Context

The Text context Command Manager was modelled on a **workstation ribbon**: labelled groups (History, Text Formatting, Find, …) in one 64–80 px row. That fits PDF and CAD families with fewer, domain-specific controls. For Markdown and everyday rich text it is too tall and too literal about **font size**, which does not map to portable Markdown semantics.

Bear’s direction:

1. **Common text editing** (bold, italic, lists, links, headings) belongs on a **compact icon toolbar**: one slim row, icon-only controls, tooltips, Lucide at 16 px.
2. **Font size is the wrong abstraction** for Markdown. The primary block control is a **semantic style picker** (Paragraph, Heading 1–6, Block quote, Code block). The same idea applies across hosts: when a format has no native semantic, the host **emulates** via configured styles (Word “Heading 1”, HTML default CSS, DOCX named styles).
3. The Command Manager **coexists** with the compact strip: in Text context it **collapses** to a shorter strip for non-inline families (History, Find, Proofing, Export, mod tools), while inline formatting lives on the compact toolbar.

Research: [text-editor-toolbars.md](../research/text-editor-toolbars.md).

## Decision

### 1. Two-tier chrome in Text context

| Tier | Height | Contents |
| --- | --- | --- |
| **Compact text toolbar** | 32 px (fixed) | Icon-only inline + block-style picker; tooltips; optional overflow chevron |
| **Command Manager strip** | 40 px Compact (existing) | History, Find, Proofing, Export, and mod families **not** duplicated on the compact bar |

Total default Text chrome above content: **72 px** (32 + 40), excluding title bar and tabs. Large density (64 px CM) remains available for Office/Image/CAD contexts; Text context defaults to **CM Compact + compact text toolbar**.

Ctrl+F1 collapses **both** tiers together (same as today for CM; compact text toolbar follows CM visibility).

### 2. Semantic block styles (content-kind concept)

Introduce a platform semantic **`unibrowser.semantic.block_style`** (logical control family), distinct from **`unibrowser.semantic.text_run`** inline marks.

| Semantic style | Meaning |
| --- | --- |
| `paragraph` | Body text block |
| `heading_1` … `heading_6` | Document outline levels |
| `blockquote` | Quoted block |
| `code_block` | Fenced or indented code block |
| `list_bullet` / `list_ordered` / `list_task` | List containers (toggle from picker or separate list split) |

The **block-style picker** is a single dropdown (current style label + chevron), not six separate ribbon buttons. Keyboard: Ctrl+Alt+1–6 for headings, Ctrl+Alt+0 for paragraph (aligned with Google Docs conventions where practical).

**Editor display size** (Monaco zoom, preview font scaling) remains **`unibrowser.editor.display_size`**: supported on all text hosts, never written into Markdown body text. Labels must say “Editor size” or “Zoom text”, not “Font size”.

Remove **document font size** stepper from the default Text compact strip and from the Text Formatting family’s primary slot; rich document (DOCX) mods may still expose size under Office context or a “Character” overflow menu.

### 3. Emulation model

Capability states remain per [ADR 0027](0027-command-system-capabilities-and-extensibility.md): `supported`, `emulated`, `unsupported`.

**Emulation** means: the user selects a **semantic style**; the host applies the nearest **native representation** for the host format and records an **emulation note** when the mapping is lossy or approximate. UI shows the existing **emulation marker** on the control (`.cm-emulation-mark`) and in tooltips/screen reader text.

Rules:

1. **Semantic intent is stable** across hosts (user asks for “Heading 2”, not “18 pt bold”).
2. **Persistence is host-specific**; the picker always shows the same choices; capability resolver picks `supported` vs `emulated`.
3. **No silent downgrade**: if the host can only approximate (e.g. underline in Markdown via `<u>`), state is `emulated` with a note.
4. **Plain text** exposes the full picker greyed where unsupported; paragraph/body remains the only active block style.

Family-level defaults and per-control overrides follow [command-system.md](../design/command-system.md) § Provider precedence.

### 4. Style mapping table

Logical semantic → how each host persists or emulates. “Native” = `supported`; otherwise `emulated` with note shown.

| Semantic | Markdown (`.md`) | HTML (fragment / contenteditable) | DOCX | ODT | RTF | Plain text (`.txt`) |
| --- | --- | --- | --- | --- | --- | --- |
| `paragraph` | Blank line / normal paragraph | `<p>` or unwrapped block | Normal / Body Text style | Text Body | Normal | N/A (always body) |
| `heading_1`–`heading_6` | `#` … `######` line prefix | `<h1>`–`<h6>` | Named style Heading 1–6 (OOXML) | Heading styles | `\pard` + heading style | **Unsupported** (reason: no structure) |
| `blockquote` | `>` prefix | `<blockquote>` | Quote style | Quotations | Quote style | **Unsupported** |
| `code_block` | Fenced ``` or indent | `<pre><code>` | Code / Preformatted (mod policy) | Preformatted text | Monospace paragraph | **Unsupported** |
| `list_bullet` | `-` / `*` | `<ul><li>` | List Paragraph + num | Bullet list | Bullet | **Unsupported** |
| `list_ordered` | `1.` | `<ol><li>` | Numbering | Numbered list | Numbering | **Unsupported** |
| `list_task` | `- [ ]` GFM | `<input type="checkbox">` (emulated) | Checkbox content control (if mod) | **Unsupported** or emulated | **Unsupported** | **Unsupported** |
| **Bold** (inline) | `**` / `__` | `<strong>` | Character bold | Bold | `\b` | **Unsupported** |
| **Italic** | `*` / `_` | `<em>` | Italic | Italic | `\i` | **Unsupported** |
| **Underline** | `<u>` inline HTML (emulated) | `<u>` | Underline | Underline | `\ul` | **Unsupported** |
| **Strikethrough** | `~~` | `<s>` | Strike | Strike | Strike | **Unsupported** |
| **Sub/superscript** | `<sub>` / `<sup>` or font scaling (emulated) | `<sub>` / `<sup>` | Native | Native | Native | **Unsupported** |
| **Text colour** | `<span style>` (emulated, policy) | CSS colour | Run colour | Character colour | Colour table | **Unsupported** |
| **Font family / pt size** | **Unsupported** (no portable semantic) | CSS (supported in HTML host) | Native character formatting | Native | Native | **Unsupported** |

HTML “emulation” of headings is **supported** when the host is HTML; browser default CSS applies until author CSS overrides — same pattern as Word using named styles instead of literal metrics.

### 5. Plugin contributions

Mods declare controls with `surfaces: ["text_compact_toolbar", "command_manager"]` (schema extension in CMD-07). Rules:

- Core owns **slot order** and overflow policy on the compact bar ([text-toolbar.md](../design/text-toolbar.md)).
- Mods may insert **after** core block picker into declared extension zones, or contribute **Command Manager-only** tools (e.g. citation insert).
- Mods must not inject raw HTML toolbars ([ADR 0028](0028-platform-content-kinds-and-plugin-scope.md) §5).

### 6. Markdown-specific policy

Default Markdown mod (`markdown-baseline`) keeps inline HTML emulation **on** for underline, colour, sub/sup when noted in capability responses. Block styles use **native** `#` headings and fenced code where possible.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Full picker visible; unsupported greyed with reason, not hidden |
| Never blocks save or work | Toolbar state does not gate Save |
| Local-first | No cloud style catalogue required |
| Predictable across formats | Same semantic names; emulation disclosed |
| Simple mechanism | Two fixed slots + resolver, not per-format ribbons |
| Mod not core | Format mapping lives in `base-*` mods; core owns slots and picker shell |

## Consequences

**Positive**

- Markdown editing matches user expectations from GitHub, Mark Text, and Docs (styles not point sizes).
- Less vertical chrome; more room for content at 1280 px width.
- Clear story for ADR 0028 content kinds: block vs inline semantics.

**Negative**

- Text context gains a second toolbar row (mitigated: 32 px icons only).
- CMD-06 rendering must know `text_compact_toolbar` surface.
- Existing screenshots and CM JSON fixtures that show font size in Text Formatting need refresh (UI tickets).

## References

- [text-editor-toolbars.md](../research/text-editor-toolbars.md)
- [text-toolbar.md](../design/text-toolbar.md)
- [command-system.md](../design/command-system.md)
- [ui-implementation-plan.md](../design/ui-implementation-plan.md) (TXT-01–TXT-06)
