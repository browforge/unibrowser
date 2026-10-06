# Unibrowser visual language

This document is normative for all Unibrowser UI work. It has the same design
weight as [principles.md](../principles.md). When an implementation or mockup
conflicts with this document, fix it or record a deliberate exception.

## Direction: Quiet Utility / Workshop Modernism

Unibrowser is a professional utility for concentrated, long-term use. It is
clean because it is organised, not because it is empty. It is dense without
being cluttered, handsome without decoration, and modern without following
fashion.

The reference is a well-run workshop or working kitchen: a work surface, tools
where they are needed, labelled drawers, compartments, and durable controls.
The interface should produce trust and relief:

> Everything is where it should be. Take your time.

> Thank God, I do not have to think about the UI.

The UI is used, not seen. It gives the user the information and control needed
for the task while consuming as little attention as possible. It is not a
showroom, an empty house, an engagement surface, or friendliness theatre.

Bear's practical test:

- Utility for 1000-hour focus, but not an "ADHD garage".
- A professional Japanese kaizen artisan workshop or working French kitchen:
  clean and organised because that is useful.
- "I can relax, no one is swindling me" and "thank God I do not have to think
  about the UI", not showroom opulence.
- Used, not seen, but handsome and not an eyesore. Almost no aesthetic.
- Not 1990s software, MS Access, a form designer, or default HTML.
- Not performative minimalism with nothing visible, pencil-thin rules, and
  microscopic type.
- Mise en place, not a showroom or empty house.
- Simple, not austere. Dense, not cluttered. Handsome, not ornamental.
  Professional, not corporate. Modern, not fashionable.

## Reference concepts

Bear's two concept sketches, "single document with AI" and "multimodal
workshop", are direction references, not pixel specs.

What this language takes from them:

- A dark slate shell frames the work without making every document dark.
- Text, PDF, and office documents may sit on a warm off-white working surface.
- A slim activity rail opens supporting panels only: Files, Search, Project,
  Assets, Settings, and genuine contributed panels.
- One compact title-bar row carries tabs, location/command access, global
  document actions, and platform window controls.
- The active file type is the only command context authority. There is no
  permanent Browse/Text/Create/Live switcher.
- AI is a docked right work region with AI Assistant, Tools, and Assets tabs.
- Dense multimodal work uses clear dock headers, joined panes, numeric
  inspector fields, restrained sliders, a timeline, and a slim status bar.
- Density comes from visible organisation and stable placement, not from tiny
  text or arbitrary separators.

What this language deliberately does not take:

- Rounded chat bubbles. Conversation is structured as a transcript with a
  subtle speaker edge or surface change; bubbles are reserved for direct
  quoted user input only, if needed.
- Glossy toggles, luminous switches, gradients, and floating glow.
- The concept images' excess rounding on chat/composer controls.
- Icon-only navigation without persistent labels at normal desktop width.
- A primary action in every panel. Save is the one shell-level primary action;
  local panels use ordinary controls unless the decision genuinely requires
  emphasis.
- Permanently visible panels without a task relationship. The multimodal
  arrangement is a named workspace, not the default for a single document.

## Testable visual rules

### Workbench surfaces

The application uses five surface roles. A component must use the role matching
its physical relationship to the work, not whichever value looks attractive.

| Role | Light | Dark | Purpose |
| --- | --- | --- | --- |
| Base surface | `#F1F0EC` | `#1B1D1E` | Window background and main workbench |
| Recessed well | `#E7E5DF` | `#141617` | Editor gutters, canvas surrounds, timeline tracks, input wells |
| Panel surface | `#FAF9F6` | `#232526` | Docked sidebars, menus, tool regions |
| Raised control | `#FFFFFF` | `#2D3031` | Buttons, inputs, selected tool controls |
| Selected | `#DCE9F2` | `#253E4C` | Current tab, row, tool, or bounded selection |
| Disabled | same geometry | same geometry | Reduced text/icon contrast, never reduced layout or hidden affordance |

- The shell frame uses dark slate in light and dark themes. Warm document
  surfaces remain task-local. A light-theme setting may make utility panels
  light, but the default identity keeps the rail and top frame dark.
- Adjacent docked regions use one 1 px divider or a clear surface change.
- Spacing alone separates related groups within one surface.
- A border is used for an actual boundary, input affordance, selected object,
  table structure where column reading needs it, or overlay edge.
- Do not border every row, group, or section. One boundary must not be doubled
  by two adjacent borders.
- Cards are reserved for independently movable or repeatable objects such as a
  mod result or recovery item. Main page sections are not cards.

### Radius

| Token | Value | Use |
| --- | --- | --- |
| `radius-control` | 4 px | Buttons, inputs, compact selectors, tooltips |
| `radius-panel` | 6 px | Menus, palettes, toasts, independent cards |
| `radius-none` | 0 px | Docked panel joins, tables, editor/canvas regions |

- Controls and panels must not exceed 6 px.
- Tabs may use 4 px on their two exposed corners only.
- Pills are allowed only when the shape itself communicates a compact state,
  such as LIVE, a permission grant, or an on/off switch.
- Nested rounded containers are prohibited.

### Borders and dividers

- Default boundary: 1 px `border`.
- Strong boundary: 1 px `border-strong` for active resize edges, focused
  structure, or the edge between canvas and dock.
- High contrast: 2 px system colour boundaries.
- Section spacing is 16 or 24 px. Do not add a line between sections if the
  spacing and heading already establish the relationship.
- Lists use row background on hover/selection. They do not use a line between
  every row unless the content is genuinely tabular.
- Inputs always have a visible edge at rest. A text label must never secretly
  behave as a button.

### Shadows and elevation

- Docked regions have no shadow.
- Raised controls rely on surface and border, not shadow.
- Menus and palettes: `0 8px 20px rgba(25, 22, 16, 0.16)` plus a 1 px border.
- Toasts: `0 4px 12px rgba(25, 22, 16, 0.14)` plus a 1 px border.
- Detached platform windows use the operating system shadow.
- No inner glow, ambient card shadow, coloured shadow, glass, blur, or stacked
  elevation.

### Typography

UI font: **Source Sans 3**, SIL Open Font License 1.1.

Mono font: **Source Code Pro**, SIL Open Font License 1.1.

Source Sans 3 is workmanlike rather than geometric, has open apertures, clear
punctuation and numerals, and remains legible in dense controls. It is
well-hinted on Windows ClearType and Linux FreeType. Source Code Pro is from the
same family, gives code and technical values clear character distinction, and
avoids a mixed visual voice. Both are redistributable and available without an
account. Platform fallbacks are Segoe UI and DejaVu Sans.

```css
--font-ui: "Source Sans 3", "Segoe UI", "DejaVu Sans", sans-serif;
--font-mono: "Source Code Pro", "Cascadia Mono", "DejaVu Sans Mono", monospace;
```

| Token | Size / line | Weight | Use |
| --- | --- | --- | --- |
| `type-12` | 12 / 16 | 400 | Status and compact metadata, minimum UI size |
| `type-13` | 13 / 18 | 400 | Dense controls and secondary copy |
| `type-14` | 14 / 20 | 400 | Default UI and body |
| `type-14-strong` | 14 / 20 | 600 | Panel and group headings |
| `type-16` | 16 / 22 | 600 | Section headings |
| `type-20` | 20 / 26 | 600 | Page headings |
| `type-26` | 26 / 32 | 600 | New tab title only |

- No functional UI text below 12 px at 100% scale.
- Default controls use 13 or 14 px. Long descriptions use 14 px.
- Use 400 and 600 weights. Weight 700 is reserved for exceptional numeric
  emphasis, not navigation.
- Do not use all caps except short operational states such as LIVE and REC.
- Text contrast must meet WCAG AA: 4.5:1 for normal text, 3:1 for text at least
  18 pt or 14 pt bold, icons, focus, and control boundaries.
- `text-3` is for metadata only and must still meet 4.5:1 on its surface. There
  is no grey-on-grey exception.

### Colour

Colour has authority. One restrained blue accent marks selection, focus, and
the primary action. Semantic colours retain one meaning throughout the app.

#### Light

| Token | Value |
| --- | --- |
| `base` | `#F1F0EC` |
| `well` | `#E7E5DF` |
| `panel` | `#FAF9F6` |
| `control` | `#FFFFFF` |
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
| `info` | `#2D647D` |

#### Dark

| Token | Value |
| --- | --- |
| `base` | `#1B1D1E` |
| `well` | `#141617` |
| `panel` | `#232526` |
| `control` | `#2D3031` |
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
| `info` | `#79B1C9` |

#### High contrast

Use system colours only: `Canvas`, `CanvasText`, `ButtonFace`, `ButtonText`,
`Highlight`, `HighlightText`, `GrayText`, and `LinkText`.

- No transparency, shadows, or colour-only state.
- Boundaries and focus indicators are 2 px.
- Selection is `Highlight` with `HighlightText`.
- Disabled controls keep their outline and use `GrayText`.

#### Semantic meanings

- Blue accent: selection, focus, navigation, one primary action.
- Green: completed or healthy. It never means selected.
- Amber: caution or degraded state where work may continue.
- Red: error, destructive action, LIVE, or recording. It never means ordinary
  selection or decoration.
- Every semantic state includes text, icon, shape, or position in addition to
  colour.

### Icons

Use **Lucide**, ISC licence, as the general UI icon set.

Lucide has a consistent 24 px grid, broad utility coverage, restrained shapes,
and recognisable actions. Its outline language remains quiet over long use and
renders consistently on Windows and Linux. It is not used as decoration.

- Use 16 px icons in 28 to 30 px controls, 18 px in panel headers, and 20 px
  only for empty states or prominent tools.
- Use a 1.75 px stroke at 16/18 px and 2 px at 20 px.
- Do not mix outline and filled packs. Custom file/status symbols must share the
  same grid and optical weight.
- Unfamiliar, destructive, permission, and LIVE actions require a text label.
- Icons never carry severity or state alone.
- Add Lucide's ISC attribution when the package or copied SVGs enter product
  code.

### Density

- Base spacing unit: 4 px.
- Unified title bar: 40 px.
- Command Manager: 40 px Compact (default) or 64 px Large.
- Total chrome above content: 80 px Compact, 104 px Large, or 40 px with
  Command Manager collapsed.
- Standard controls: 30 px; compact controls: 26 px.
- Dock header: 36 px; status bar: 24 px.
- List rows: 30 to 34 px; selected rows may expand to 44 to 56 px for detail.
- Sidebars: 256 px left, 320 px right by default.
- Activity rail: 40 px collapsed, 188 px expanded, or auto-hidden; 16 px Lucide
  icons and 12 px labels in expanded mode.
- Content padding: 12 to 16 px for tools, 20 to 24 px for settings pages.
- A 1280x800 document view must show the document, its primary tools, at least
  eight problem rows when the panel is open, and the AI composer when docked.
- Touch mode may increase targets to 44 px without inflating desktop mode.

## Reject list

| Reject | Reason |
| --- | --- |
| Pills for ordinary controls | Makes every action look like a promotional chip and weakens state meaning |
| Radius above 6 px | Softens boundaries and produces toy/startup character |
| Cards inside cards | Hides information structure behind containers |
| Glass, blur, gradients, blobs | Decorative atmosphere competes with work and weakens trust |
| Oversized icons or type | Consumes work area and performs importance |
| Huge whitespace | Replaces organisation with emptiness |
| Hairline text and low contrast | Looks quiet by becoming hard to read |
| Grey text on grey surfaces | Fails trust, legibility, and often WCAG AA |
| Floating panels for persistent tools | Covers work and obscures spatial relationships |
| Shadows on docked regions | Falsely claims elevation and wastes contrast |
| Colourful icon collections | Makes tools slower to scan and colour less semantic |
| Text that secretly acts as a button | Hides affordance |
| Engagement badges, streaks, or prompts | Optimises service attention over the user's task |
| Corporate character illustration | Adds friendliness theatre without task value |
| Win95 bevels and 3 px grey boxes | Retro imitation rather than durable utility |
| Default browser controls | Reads as unfinished and varies by platform |
| A divider between everything | Produces form-builder noise instead of hierarchy |
| Tiny legacy icons | Fail modern scaling and long-session recognition |
| Animation without state meaning | Consumes attention without removing uncertainty |

## Screenshot drift checklist

A reviewer must be able to answer yes to every applicable item:

1. Does the screenshot read as one workbench with docked regions, not a grid of
   cards?
2. Are all control and panel radii 4 to 6 px or less?
3. Does every visible border define a real boundary, input, selection, or
   tabular relationship?
4. Are docked panels shadowless and overlays the only elevated surfaces?
5. Is ordinary text at least 12 px and clearly legible, including secondary
   text?
6. Is there one restrained accent, with green/amber/red used only for semantic
   states?
7. Are buttons visibly buttons, inputs visibly inputs, and labels not secretly
   interactive?
8. Are related controls compact and organised without premium whitespace?
9. Are icons from one quiet stroke family at 16 to 18 px, with labels where
   meaning is not universal?
10. Does selection remain understandable in greyscale and without colour?
11. Is every persistent element useful for the task shown?
12. Is there any pill-everything, glass, gradient, nested-card, giant-heading,
    decorative-illustration, or engagement UI? If yes, the design has drifted.
13. At 1280x800, is enough real work visible to continue without first closing
    UI?
14. Would a user plausibly say "of course that is where it is" rather than
    notice the styling?
15. Is there exactly one visible file-type context authority, with the rail
    limited to side-panel destinations and saved layouts absent from permanent
    chrome?
16. Does expanded chrome above content stay within 80 px in Compact density?

## Review requirement

Every UI ticket from UI-07 onward must include a screenshot review against this
checklist. Any deliberate failure needs a written reason and a follow-up ticket.
