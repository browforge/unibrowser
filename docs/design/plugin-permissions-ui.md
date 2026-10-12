# Plugin permissions UI (Quiet Utility)

Design companion to [ADR 0029](../adr/0029-plugin-permissions-and-network-security.md). Surfaces follow [ui-spec.md](ui-spec.md) and [visual-language.md](visual-language.md): warm-neutral workbench, compact density, restrained accent, text labels on permission actions, no upsell or guilt copy.

## 1. Network consent (first use)

**Surface type:** Non-modal **banner** anchored below the title bar (same band as offline notices). Not a dialog; does not steal focus from the document. Enter opens default action; Esc dismisses to “Don't allow” without closing the tab.

**Wireframe (1280×800, light)**

```
┌─ Title bar ─────────────────────────────────────────────────────────────┐
│ [≡]  document.md — Unibrowser                              [ − □ × ]      │
├───────────────────────────────────────────────────────────────────────────┤
│ Network request                                              [ Don't allow ]│
│ Material Icons Import wants to contact api.iconify.design                 │
│                                                                           │
│ Why: Download icon metadata for the shape you picked.                     │
│ May send: Icon name and search text (not your document).                  │
│                                                                           │
│ [ Allow once ]  [ Always allow ]                                            │
└───────────────────────────────────────────────────────────────────────────┘
│ … document / Command Manager unchanged …                                  │
```

**Layout**

- Banner min-height 88 px; max 120 px with wrapped disclosure (max 3 bullets).
- Left: 16 px Lucide `globe` in neutral icon colour (not warning red).
- Title row: plugin **display name** (semibold 13 px) + “wants to contact” + **origin** monospace 12 px.
- Body: “Why:” + manifest reason (13 px). “May send:” + disclosure bullets.
- Actions right-aligned on ≥900 px; stack on narrow widths with primary = **Allow once**, secondary = **Always allow**, quiet = **Don't allow**.

**Copy (defaults)**

| Key | Text |
| --- | --- |
| Title pattern | `{pluginTitle} wants to contact {origin}` |
| Why label | `Why:` |
| May send label | `May send:` |
| Allow once | `Allow once` |
| Always allow | `Always allow` |
| Don't allow | `Don't allow` |
| Offline blocked | `Offline mode is on. Network for mods is turned off. You can still save and edit.` |
| Denied follow-up (tooltip) | `Network not allowed for {origin}. Grant access from the command or Settings → Plugins → Permissions.` |

**Assistant variant** (when disclosure includes document text)

- May send bullets must name content kinds, e.g. “Selected text from the active document (up to the limit shown in the assistant panel).”
- Extra quiet line under bullets: `Nothing is sent until you choose Allow.`

**Interaction**

- Choosing **Allow once** / **Always allow** runs the pending request; banner closes.
- **Don't allow** / Esc: pending request fails; calling command stays grey with grant affordance.
- Second concurrent request queues one banner; do not stack multiple banners.

## 2. Settings → Plugins → Permissions

**Path:** Settings → **Plugins** → **Permissions** (sibling to Installed mods / Offline catalogue).

**Wireframe**

```
┌ Settings ─────────────────────────────────────────────────────────────────┐
│ Plugins                                                                 ▼ │
│   Installed mods                                                          │
│   Permissions          ← active                                             │
│   Offline catalogue                                                       │
├───────────────────────────────────────────────────────────────────────────┤
│ Search plugins…                                                           │
│                                                                           │
│ ┌ Material Icons Import ──────────────────────────────── Enabled ─────┐ │
│ │ Network                                                                │ │
│ │   api.iconify.design          Always allow          [ Revoke ]         │ │
│ │ Filesystem                                                             │ │
│ │   Plugin data folder only     (sandbox)                              │ │
│ │ Document access                                                        │ │
│ │   Read: Image nodes                                                  │ │
│ └────────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ ┌ AI Assistant (proving) ───────────────────────────────── Disabled ────┐ │
│ │ Network — no grants yet                                                │ │
│ │ [ Open activity log ]                                                  │ │
│ └────────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│ Offline mode: Off   Mod portal network: Allowed   [ Open activity log ]   │
└───────────────────────────────────────────────────────────────────────────┘
```

**Layout**

- Settings row pattern: label + description left, control right ([ui-spec.md](ui-spec.md)).
- One **list card** per plugin (min 92 px); no grid of tiles.
- Each card: plugin icon 40 px, title, enabled/disabled chip, expandable sections (Network, Filesystem, Clipboard, Processes, Document access, Background, Secrets slots as names only).
- Network rows: origin, grant type (`Once this session` | `Always` | `Not allowed`), **Revoke** (quiet button).
- Footer row: global **Offline mode** state (read-only link to Settings → Network), portal network toggle summary, **Open activity log**.

**Copy**

| Element | Text |
| --- | --- |
| Page title | `Permissions` |
| Page description | `What each mod can do on your device. Revoke access any time.` |
| Empty network | `Network — no grants yet` |
| Revoke confirm (inline, no modal) | `Revoked. {pluginTitle} cannot reach {origin} until you allow it again.` (toast, 5 s) |
| Version reset banner | `Permissions reset for {pluginTitle}. This update added new access. Review below.` |
| Full-trust native note | `Runs as native code (full trust). Enable only if you trust the publisher.` |

## 3. Network activity log

**Path:** From Permissions footer, or Settings → Plugins → Permissions → **Open activity log**. Optional status-bar link when a mod fetch failed (non-modal).

**Wireframe**

```
┌ Network activity ─────────────────────────────────────────────────────────┐
│ Showing last 30 days on this device. Nothing here is uploaded.             │
│ Filter: [ All plugins ▼ ]  [ All domains ▼ ]   Search…                  │
├───────────────────────────────────────────────────────────────────────────┤
│ 12 Oct 2026, 09:14   Material Icons Import   api.iconify.design         │
│                      GET /search?q=home      1.2 KB sent · 48 KB received │
│                      Grant: Always allow                                   │
├───────────────────────────────────────────────────────────────────────────┤
│ 12 Oct 2026, 09:02   Open VSX catalogue      open-vsx.org                 │
│                      GET /api/-/search       query text · 12 KB received  │
│                      Grant: Allow once (session)                         │
├───────────────────────────────────────────────────────────────────────────┤
│ [ Save log… ]                                    [ Clear log ]            │
└───────────────────────────────────────────────────────────────────────────┘
```

**Layout**

- Table-like rows, 56 px min height; zebra optional (subtle surface alternate).
- Timestamp local; plugin title; origin; method + path summary; byte counts rounded; grant type.
- **Save log…** → Save As JSON dialog; **Clear log** → quiet confirm inline (“Clear recorded network activity?” with **Clear** / **Cancel**).

**Copy**

| Element | Text |
| --- | --- |
| Header | `Showing last 30 days on this device. Nothing here is uploaded.` |
| Empty state | `No mod network activity recorded yet.` |
| Clear confirm | `Clear recorded network activity?` |
| Secret use row | `Used stored credential (value not shown)` |

## 4. Command Manager grant affordance

When a control is greyed for missing network (or other) permission:

- Tooltip: `{commandTitle} — {reason}` then new line `Press Grant to allow {pluginTitle} to reach {origin}.`
- Context menu / overflow: **Grant…** (same as Grant link).
- Activating Grant opens the network consent banner with the pending action attached; no modal.

**Copy for locked state**

- Reason: `Network access to {origin} is not allowed.`
- Remedy chip in palette: `Grant network access`

## 5. Mod Portal pre-install chips

On mod card metadata row ([ui-spec.md](ui-spec.md) mod card):

| Manifest signal | Chip label |
| --- | --- |
| `network.domains` non-empty | `Network: {n} domains` |
| `filesystem.scopes` includes `workspace_roots` | `Files: workspace` |
| `process.launch` non-empty | `Can open other apps` |
| T2 WASM | `Sandboxed` |
| T1 native unsigned | `Full trust` |

Expanded details list each domain and content kind read/write in plain language (manifest `title` + auto labels from registry).

## Visual tokens

- Permission banner: `surface-raised`, 1 px `border-subtle`, no drop shadow.
- Risk semantic colour only for **Full trust** and **Revoke** hover, not for routine network ask.
- Focus ring on all three banner buttons; **Allow once** is primary fill, **Always allow** secondary, **Don't allow** quiet.

## Evidence (when implemented)

- Screenshots: consent banner (light/dark), Permissions page with two plugins, activity log with two rows, greyed CM control + tooltip (1280×800).
- Attach to PERM tickets in [ui-implementation-plan.md](ui-implementation-plan.md).
