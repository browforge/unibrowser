# UI specification

Normative visual/component spec for implementation. Values are CSS logical pixels at 100% scaling.

## Visual direction

**Quiet workstation.** Familiar browser structure, compact professional tools, warm-neutral surfaces, and one restrained accent. It should feel native beside Windows 11 apps without copying Fluent cards, acrylic, or oversized spacing.

Hierarchy comes from typography, spacing, and surface tone before borders. Cards are used only for independent objects (mod result, recovery item), not every section.

## Tokens

### Type

Font stack:

```css
--font-ui: "Segoe UI Variable", "Segoe UI", system-ui, sans-serif;
--font-mono: "Cascadia Code", "Cascadia Mono", ui-monospace, monospace;
```

| Token | Size / line | Weight | Use |
| --- | --- | --- | --- |
| `type-11` | 11 / 16 | 400 | metadata, status |
| `type-12` | 12 / 18 | 400 | dense controls |
| `type-13` | 13 / 20 | 400 | default UI/body |
| `type-14` | 14 / 21 | 600 | panel headings |
| `type-16` | 16 / 24 | 600 | section heading |
| `type-20` | 20 / 28 | 650 | page heading |
| `type-28` | 28 / 36 | 650 | New tab title only |

Do not use all caps except short status labels such as LIVE.

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

- Title/menu: 32 px.
- Tab strip: 34 px.
- Navigation: 40 px.
- Command Manager tabs: 28 px.
- Command Manager body: 44 px Compact, 52 px Comfortable.
- Status bar: 24 px.
- Standard button: 30 px, compact 26 px.
- Icon button: 30x30 px.
- Touch mode target: 44x44 px.
- Sidebar header: 40 px.
- Settings row: min 48 px.

### Radius

- `radius-2`: 2 px for focus-adjacent inner elements.
- `radius-4`: 4 px for buttons, inputs, tooltips.
- `radius-6`: 6 px for tabs (top only), menus, toasts.
- `radius-8`: 8 px for cards and empty-state containers.
- No pill buttons except status chips/toggles whose shape communicates state.

### Elevation

- Docked regions: no shadow; use surface difference or 1 px boundary.
- Menus/palette: `0 8px 24px rgba(0,0,0,.18)`, plus 1 px border.
- Toast: `0 4px 16px rgba(0,0,0,.16)`.
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
| `canvas` | `#F4F5F7` |
| `surface-1` | `#FFFFFF` |
| `surface-2` | `#ECEFF3` |
| `surface-3` | `#E2E6EB` |
| `text-1` | `#171A1F` |
| `text-2` | `#4D5662` |
| `text-3` | `#707A87` |
| `border` | `#C9CFD7` |
| `border-strong` | `#949EAA` |
| `accent` | `#1769AA` |
| `accent-hover` | `#0F5A94` |
| `accent-soft` | `#DCEEFF` |
| `focus` | `#0067C0` |
| `success` | `#167846` |
| `warning` | `#9A5B00` |
| `danger` | `#B42318` |
| `live` | `#C21E32` |
| `recording` | `#A8172A` |

### Dark

| Token | Value |
| --- | --- |
| `canvas` | `#17191D` |
| `surface-1` | `#202329` |
| `surface-2` | `#282C33` |
| `surface-3` | `#323740` |
| `text-1` | `#F3F5F7` |
| `text-2` | `#C0C7D0` |
| `text-3` | `#929CAA` |
| `border` | `#434A55` |
| `border-strong` | `#687382` |
| `accent` | `#69B7F2` |
| `accent-hover` | `#8DC9F6` |
| `accent-soft` | `#183A54` |
| `focus` | `#79C0FF` |
| `success` | `#55C58A` |
| `warning` | `#E2A64E` |
| `danger` | `#FF786D` |
| `live` | `#FF5266` |
| `recording` | `#FF5266` |

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

Use **Lucide** icons, ISC licence.

Reasons:

- Open licence compatible with Apache-2.0 distribution.
- Consistent 24 px grid and stroke language.
- Broad familiar action coverage.
- Easy SVG/tree-shaking and Windows/Linux parity.

Rules:

- Standard optical size 16 px in 30 px controls; 18 px in sidebar/page actions.
- Stroke 1.75 at 16 px; avoid mixing filled icon packs.
- Text accompanies unfamiliar, destructive, LIVE, and permission actions.
- Never use an icon as the only error/severity channel.
- Product/file-type icons may be custom but must share a 16/20/24 grid.
- Add Lucide ISC attribution to `NOTICE.md` / third-party docs when implemented.

## Components

### Tab strip

- 34 px high, `surface-2`.
- Tab min 112 px, preferred 180 px, max 240 px.
- Active tab `surface-1`; inactive transparent; active has 2 px accent top edge.
- Icon 16, label ellipsised, state glyph, close 20 hit/16 icon.
- Dirty dot replaces close until hover/focus; accessible label still says unsaved.
- Pinned tabs 40 px icon-only only when user pins them.
- Drag insertion marker 2 px accent.

### Omnibox

- 32 px high inside 40 px row.
- Default shows breadcrumb for local path; focus switches to editable full path.
- URL security/origin identity sits at leading edge.
- Results drop below omnibox but never cover toolbar rows; max 8 results.
- Technical schemes such as `unibrowser://` show friendly names until focused.

### Command Manager

Tabs:

- 28 px, no pill background.
- Active tab has 2 px accent underline and `text-1`.
- Inactive `text-2`; hover `surface-2`.
- **Follow/Pin** at right with explicit label.

Buttons:

- 30 px minimum; icon + label in Compact where space permits.
- Group separators 1 px, 20 px high, 8 px margins.
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
