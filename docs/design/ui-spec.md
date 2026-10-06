# UI specification

Normative visual/component spec for implementation. Values are CSS logical pixels at 100% scaling.

## Visual direction

**Quiet Utility / Workshop Modernism.** Familiar browser structure, compact professional tools, warm-neutral workbench surfaces, and one restrained accent. It should feel durable beside Windows and Linux applications without copying Fluent cards, acrylic, or oversized spacing.

Hierarchy comes from typography, spacing, and surface tone before borders. Cards are used only for independent objects (mod result, recovery item), not every section.

[Visual language](visual-language.md) is normative. Its drift checklist is part of screenshot acceptance.

## Tokens

### Type

Font stack:

```css
--font-ui: "Source Sans 3", "Segoe UI", "DejaVu Sans", sans-serif;
--font-mono: "Source Code Pro", "Cascadia Mono", "DejaVu Sans Mono", monospace;
```

| Token | Size / line | Weight | Use |
| --- | --- | --- | --- |
| `type-12` | 12 / 16 | 400 | status, compact metadata; minimum UI size |
| `type-13` | 13 / 18 | 400 | dense controls, secondary copy |
| `type-14` | 14 / 20 | 400 | default UI/body |
| `type-14-strong` | 14 / 20 | 600 | panel headings |
| `type-16` | 16 / 22 | 600 | section heading |
| `type-20` | 20 / 26 | 600 | page heading |
| `type-26` | 26 / 32 | 600 | New tab title only |

Do not use all caps except short status labels such as LIVE.
Normal text must meet WCAG AA 4.5:1. Large text, icons, focus, and control boundaries must meet 3:1. `text-3` is still readable metadata, not a low-contrast exemption.

### Spacing

Base unit 4 px.

| Token | Value | Use |
| --- | --- | --- |
| `space-0` | 0 | flush |
| `space-1` | 4 | icon/text micro gap |
| `space-2` | 8 | related controls |
| `space-3` | 12 | component padding |
| `space-4` | 16 | group separation |
| `space-5` | 20 | panel padding |
| `space-6` | 24 | section separation |
| `space-8` | 32 | page sections |
| `space-10` | 40 | sparse empty-state separation |

### Size

- Unified custom title bar: 40 px.
- Command Manager Compact: 40 px (default).
- Command Manager Large: 64 px.
- Total expanded chrome above content: **80 px Compact**.
- Total expanded chrome above content: **104 px Large**.
- Collapsed Command Manager: 0 px; total chrome above content **40 px**.
- Activity rail: 40 px collapsed, 188 px expanded, 0 px auto-hidden.
- Status bar: 24 px.
- Standard button: 30 px, compact 26 px.
- Icon button: 30x30 px.
- Touch mode target: 44x44 px.
- Sidebar header: 36 px.
- Settings row: min 48 px.

### Radius

- `radius-4`: 4 px for buttons, inputs, tabs (exposed corners only), tooltips.
- `radius-6`: 6 px for menus, palettes, toasts, independent cards.
- `radius-none`: 0 px for docked joins, tables, editor/canvas regions.
- No pill buttons except status chips/toggles whose shape communicates state.
- No control or panel radius above 6 px.

### Elevation

- Docked regions: no shadow; use surface difference or 1 px boundary.
- Menus/palette: `0 8px 20px rgba(25,22,16,.16)`, plus 1 px border.
- Toast: `0 4px 12px rgba(25,22,16,.14)`, plus 1 px border.
- Detached toolbar window uses platform window shadow.
- High contrast: shadows removed; 2 px boundaries.

### Motion

- Hover/focus colour: 80 ms.
- Panel resize/open: 140 ms ease-out, only if it clarifies movement.
- Palette: 100 ms opacity + 4 px translate.
- No animated lifts, shimmer, bouncing, or decorative page entrance.
- Reduced motion: duration 0 except determinate progress.

## Colour

Colour names describe jobs, not pigments.

### Light

| Token | Value |
| --- | --- |
| `canvas` / `base` | `#F1F0EC` |
| `well` | `#E7E5DF` |
| `surface-1` / `panel` | `#FAF9F6` |
| `surface-2` / `control` | `#FFFFFF` |
| `selected` | `#DCE9F2` |
| `text-1` | `#202223` |
| `text-2` | `#4B5052` |
| `text-3` | `#62686A` |
| `border` | `#C8C6BF` |
| `border-strong` | `#8A8D8C` |
| `accent` | `#2D647D` |
| `accent-hover` | `#214F65` |
| `accent-strong` | `#173F52` |
| `accent-on` | `#FFFFFF` |
| `focus` | `#176B94` |
| `success` | `#2F6B48` |
| `warning` | `#8A5A12` |
| `danger` | `#A33B32` |
| `live` / `recording` | `#A33B32` |

### Dark

| Token | Value |
| --- | --- |
| `canvas` / `base` | `#1B1D1E` |
| `well` | `#141617` |
| `surface-1` / `panel` | `#232526` |
| `surface-2` / `control` | `#2D3031` |
| `selected` | `#253E4C` |
| `text-1` | `#F0EFEB` |
| `text-2` | `#C9CBC8` |
| `text-3` | `#AEB2AF` |
| `border` | `#4D5151` |
| `border-strong` | `#777C7B` |
| `accent` | `#79B1C9` |
| `accent-hover` | `#9CC7D9` |
| `accent-strong` | `#B7D7E5` |
| `accent-on` | `#142027` |
| `focus` | `#83C7E6` |
| `success` | `#74BE91` |
| `warning` | `#E1B466` |
| `danger` | `#EC8178` |
| `live` / `recording` | `#EC8178` |

### High contrast

Use system colours where available:

```css
Canvas; CanvasText; ButtonFace; ButtonText; Highlight; HighlightText; GrayText; LinkText;
```

Rules:

- 2 px boundaries.
- No transparency.
- Disabled state uses `GrayText` plus disabled icon/label, never opacity under 60%.
- Focus is 2 px `Highlight` plus 1 px Canvas separation.

## Icons

Use **Lucide** icons, ISC licence. See [visual-language.md](visual-language.md) for the reject rules and attribution requirement.

Reasons:

- Open licence compatible with Apache-2.0 distribution.
- Consistent 24 px grid and stroke language.
- Broad familiar action coverage.
- Easy SVG/tree-shaking and Windows/Linux parity.

Rules:

- Standard optical size 16 px in 30 px controls; 18 px in sidebar/page actions.
- Stroke 1.75 at 16/18 px and 2 at 20 px; avoid mixing filled icon packs.
- Text accompanies unfamiliar, destructive, LIVE, and permission actions.
- Never use an icon as the only error/severity channel.
- Product/file-type icons may be custom but must share a 16/20/24 grid.
- Add Lucide ISC attribution to `NOTICE.md` / third-party docs when implemented.

## Components

### Shell frame and activity rail

- The default shell frame is dark slate in both themes; task surfaces use the
  light/dark workbench tokens appropriate to their content.
- Unified title bar order: menu, tabs, conditional browser navigation,
  location/command field, Undo, Redo, Save, layout overflow, drag region,
  platform window controls.
- There is no permanent Browse/Text/Create/Live switcher.
- Activity rail begins below the title bar and ends above the status bar.
- Collapsed rail items are 40x40 px with a 16 px Lucide icon. Expanded mode
  adds a 12 px label in a 188 px pushing region.
- Core destinations are Files (`files`), Search (`search`), Project
  (`panels-top-left`), Assets (`package-open`), and Settings (`settings`).
  Use Lucide 16 px, 1.75 px stroke. Contributions may add an item only when it
  opens a distinct side panel.
- CAD, Video, Audio, Code, and Paint are file contexts, not rail destinations.
- Active destination uses a 3 px accent leading edge, selected surface, icon,
  and label weight. Colour is not the only cue.
- Rail modes are Expanded, Collapsed, Auto-hide, and Hidden under
  View > Appearance. Auto-hide reveals on pointer edge, Alt+Shift+A, or a panel
  command and pushes content while open.
- At widths below 800 px the rail moves to View and the palette.

### Unified title-bar tabs

- Share the 40 px title-bar row; no independent full-width tab-strip row.
- Tab min 112 px, preferred 180 px, max 240 px.
- Active tab `surface-1`; inactive transparent; active has 2 px accent top edge.
- Icon 16, label ellipsised, state glyph, close 20 hit/16 icon.
- Dirty dot replaces close until hover/focus; accessible label still says unsaved.
- Pinned tabs 40 px icon-only only when user pins them.
- Drag insertion marker 2 px accent.

### Location / command field

- 30 px high inside the 40 px title-bar row.
- Default shows breadcrumb for local path; focus switches to editable full path.
- URL security/origin identity sits at leading edge.
- Results drop below the title bar but never cover the Command Manager; max 8 results.
- Technical schemes such as `unibrowser://` show friendly names until focused.
- Back/Forward/Reload occupy the same leading group only for folder and web
  tabs. They are absent, not disabled placeholders, for document tabs.

### Command Manager

- Two densities:
  - **Compact (default):** one 40 px row. Lucide 16 px icon plus short 12-13 px
    label inline.
  - **Large:** one 64 px row. Lucide 20 px icon above a 12 px label, with the
    group name on the row baseline. Total chrome is 104 px.
- Compact is the default because commands remain visible and labelled while
  returning 24 px more document height than Large. Large is an explicit
  workstation preference for users who value faster visual targeting over
  maximum canvas height.
- File-type context label at left with 2 px accent underline and `text-1`.
- Clicking it opens available contexts/customisation without creating another
  permanent row.
- **Following: {type} / Pin** at right with explicit labels.
- It follows the active file type automatically unless pinned.

Groups and buttons:

- Commands are arranged in named groups separated by one 1 px divider.
  Text examples: History, Type, Find, Proofing, Export. CAD/3D examples:
  Select, View, Measure, Section, Export.
- Compact buttons are 30 px high with icon + short label inline.
- Large buttons use a 20 px icon with the label below; the group name remains
  visible beneath the button cluster.
- A split button has a separately bordered chevron target for variants. The
  main target repeats the last/default action.
- Groups never wrap or create a third row. At constrained widths the least-used
  complete group moves into the overflow chevron first. Individual commands do
  not detach from their group.
- Group priority is context-specific and deterministic. The context label and
  Follow/Pin never enter overflow.
- Disabled uses `text-3`, unchanged geometry, no hover claim.
- Disabled tooltip:
  - Title: command label.
  - Reason sentence.
  - Optional remedy/link.
  - Shortcut.

Example:

> **Increase font size**  
> Unavailable because this image has no editable text layer.  
> Ctrl+=

Customisation:

- Edit mode adds 6-dot drag handle to groups.
- Drop markers appear only during drag.
- Import/export layouts live in workspace management, not the normal row.
- Collapse to tabs only removes the whole Command Manager row and leaves the
  40 px title bar. Ctrl+F1 toggles it.

### Command palette

- Width 680 px at 1280; max 760; top 18% of **content region**.
- Search field 44 px.
- Results max height 440 px.
- Sections: Commands, Open tabs/files, Settings, Help.
- Row 40 px: icon, primary label, context/path, shortcut.
- Disabled row remains focusable for explanation; Enter does not execute.
- Preview/help area optional on wide screens.
- Palette is the only substantial overlay permitted over content.

### Docked panel

- Header 40 px: title, optional count, flexible spacer, More, Close.
- Tabbed panel uses 32 px internal tabs.
- 1 px boundary against content.
- Resize affordance 6 px visual, 10 px hit.
- Empty state has one sentence and one action.

### Toast

- Bottom-right above status bar, width 320-420 px.
- One-line title, optional second line, max two actions.
- Auto-dismiss 5 s only for informational/success.
- Errors persist until dismissed or resolved.
- Stack maximum three; further notifications enter drawer.
- Never overlap LIVE studio header or active inline bar.

### Inline bar

- Full width of content region, 36-56 px.
- Icon, concise message, actions, dismiss.
- Severity uses 3 px leading stripe plus icon/text.
- Recovery and unknown-handler bars persist per tab.

### Problems list

- Compact table/list with columns: severity, message, source, location.
- 32 px row, selected 40-56 px with actions/detail.
- Toolbar integrated into panel header: severity menu, source filter, search.
- Virtualised over 200 rows.
- Keyboard: Up/Down, Enter navigate, F8/Shift+F8 next/previous.
- Severity is icon + word in accessible name.

### Sidebar

- Default 320 px right / 260 px left.
- Header and internal tabs follow Docked panel.
- Content scrolls independently.
- Opening pushes content.
- AI sidebar:
  - Internal tabs: AI Assistant, Tools, Assets.
  - Provider/model row.
  - Grant chip/button, e.g. `Read: this file` / `Edits: ask`.
  - Conversation.
  - Composer fixed at bottom.
  - Tool calls collapsed under each response, not a permanent exhaustive list.

### Settings row

- Label and description left.
- Control right at >=900 px; below label at narrower widths.
- Changes apply immediately if reversible.
- Restart-required state appears inline.
- Group headings every 24-32 px; avoid cards around every setting.
- Search result shows category breadcrumb.

### Mod card

- List card, min height 92 px.
- Icon 40; name, one-line capability, publisher/source.
- Metadata chips: licence, compatibility, local/network permission.
- Primary action right: Install/Update/Enable.
- Expandable details: manifest, versions, permissions, files.
- Install progress replaces action and is determinate where possible.
- Warning intensity matches actual privilege/risk.

### Buttons and inputs

- Primary action: one per local decision area, accent fill.
- Secondary: surface fill/border.
- Quiet: transparent.
- Danger: outline/quiet by default; filled only for immediate irreversible risk.
- Focus ring never removed.
- Press state precedes operation completion; progress/success remain separate states.

## Copy rules

- Use user goals: "Open support for this file", not "Resolve MIME handler".
- Use sentence case.
- Say what is safe after failure.
- No "Oops", blame, jokes, or generic "Something went wrong".
- `LIVE` is uppercase because it is a broadcast state; ordinary controls are not.
- Avoid "mod" in grandma-path prompts when "support" is clearer; Mod Portal may use the technical term.

## Principles check

- **Intentionality:** every persistent region has one job; dev clutter is removed.
- **Spacing:** compact hierarchy, not premium whitespace.
- **Colour:** accent marks selection/action; red reserved for LIVE/danger.
- **Interaction:** disabled reasons, truthful progress, visible focus.
- **Interruptions:** dock/inline/toast ladder; no modals.
- **Saving:** Save remains visible and independent of validation/export.
- **User control:** density, accent, workspaces, panels, detached bars persist.
