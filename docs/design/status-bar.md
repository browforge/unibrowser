# Context-aware status bar

The status bar reports the active task. It does not repeat the address field or
the Command Manager context.

## Fixed slots

The bar has two stable slots:

- **Left**: primary task state, ordered from outcome to position. Examples:
  save state, language, caret; PDF page; workshop timecode and format.
- **Right**: secondary state and utilities. Examples: encoding, problems,
  zoom, audio format, network scope.

Items retain their slot across contexts. Context changes replace the item set,
not the placement model. Path is omitted whenever the address field or title
breadcrumb already identifies the location.

AI and notifications are utilities, not permanent status prose. AI appears as
a compact button only when it has a state requiring attention. Notifications
are absent when empty and become a compact count button when items exist.

## Item component model

`statusBar.ts` defines:

```ts
type StatusSlot = "left" | "right";

interface StatusItem {
  id: string;
  slot: StatusSlot;
  text: string;
  title?: string;
  priority?: number;
  compactText?: string;
  kind?: "text" | "state" | "warning";
}
```

Items are plain text components with optional tooltip and semantic kind. They
must not introduce a raised button, card, glass surface, or decorative icon.
Every visible item must meet WCAG AA contrast of 4.5:1.

## Context defaults

| Context | Left slot | Right slot |
| --- | --- | --- |
| Text document | save state, language, caret | problems, encoding, active encode |
| PDF | page | zoom, annotation count |
| Workshop | timecode, output format | sample rate, channels |
| Audio/video/MIDI | media kind, format/state | mute/audio state |
| Settings | Changes save immediately | none |
| Mod portal | Open VSX network state | local permission scope |
| File browser | selection state | none |
| Image, SVG, DXF and readers | viewer identity | zoom when available |
| Web | none | zoom |
| New tab | none | none |

Generic hints do not displace context state. Errors and active work may add a
higher-priority warning or progress item.

## Mod contribution API

Mods register a contribution through `registerStatusContribution()`:

```ts
registerStatusContribution({
  id: "example.selection",
  contexts: ["example-viewer"],
  items: (model) => [{
    id: "example-count",
    slot: "left",
    text: "3 selected",
    priority: 80,
  }],
});
```

Registration returns a disposal function. Contributions must use a stable,
namespaced id, declare contexts, provide truthful current state, and return no
item when there is nothing useful to report. Contributions cannot reorder
another component, claim the notification utility, or render arbitrary HTML.

## Overflow

Items sort by descending priority within each slot. At narrow widths:

1. low-priority supporting items (priority 60 or below) hide first;
2. retained items truncate with their full value in `title`;
3. left retains primary task state; right retains active warnings/progress;
4. utility buttons remain compact;
5. the bar never wraps or increases shell height.

Large empty gaps between related items, overlap, clipping, or grey-on-grey text
are visual QA failures.
