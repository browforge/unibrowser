# ADR 0013: Dual audience — FOSS power users and everyday users

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

Unibrowser must satisfy two audiences at once without forking the product:

1. **FOSS community** — expects free (beer + speech), hackability, mods, and software that **does not colonise user space**.
2. **Chrome OS / everyday users** (“grandma test”) — expects **it just works**: open files and sites, no codec or format vocabulary required.

These are not separate SKUs; one shell must layer **simple defaults** over **optional depth**.

## Decision

### FOSS audience — principles

| Principle | Implication |
|-----------|-------------|
| **Free as in beer and speech** | Apache-2.0 core; mods can use other licences if labelled ([ADR 0007](0007-licensing.md)). |
| **Moddable (Factorio-style)** | Small engine; formats and behaviour via mods and contribution points ([ADR 0002](0002-mod-system-and-sandboxing.md), [ADR 0006](0006-mod-manifest-and-versioning.md)). |
| **Do not interfere with user space** | User files stay on disk where they put them; config in standard paths (`keybindings.json`, `toolbars.json`); no telemetry or dark patterns by default; no locking users into proprietary cloud for core workflows. |
| **User owns data and mods** | Plain-text config; mod manifests inspectable; engine does not phone home for open/save. |
| **Engine stays out of the way** | Host provides navigation, commands, chrome — not ads, upsells, or nudging that overrides user intent. |

Power features (Command Manager pin, palette, custom layouts, **sideload**, Open VSX browse) are **available**, not **required** for daily use.

**Mod Portal** ([ADR 0014](0014-vsix-compatibility-layer.md)) is the shared bridge: grandma sees **“Install support for this file”** when needed; hackers see the same UI with catalogs and sideload. Onboarding does not mention VSIX, Open VSX, or MIME types.

### Everyday audience — principles

| Principle | Implication |
|-----------|-------------|
| **It just works** | Bundled mods open common types out of the box; if something is missing, **Mod Portal** offers installable support in plain language — not “you need a VSIX”. |
| **No format jargon in primary UI** | Grandma does not choose raster vs vector or PNG vs WebP — she opens a photo; the shell picks a backend. Technical detail in status/tooltips or advanced panels only. |
| **Formats under the hood** | Conversion and proxies are invisible until export or an explicit advanced action ([ADR 0011](0011-lazy-conversion-native-until-encode.md)). |
| **Chrome OS–style shell** | Tabs, omnibox, familiar browser chrome ([ADR 0009](0009-browser-as-unified-shell.md)). |

### Shared bridge: progressive disclosure

- **Default path** — open tab, edit/view, save — minimal choices, sensible defaults.
- **Depth path** — mods, keybindings, toolbars, pin, palette, export pipelines — same app, no mode switch to “expert edition”.
- **Honest limits** — unsupported format = clear tab/message, not a crash or jargon dump.

### Bear UX product rules (pet peeves)

These apply to **both** audiences. FOSS UX context: [ADR 0016](0016-foss-ux-benchmarks.md).

| Rule | Requirement |
|------|-------------|
| **Never get in the way** | First-run **onboarding is allowed** only if it is **skippable**, remembers “don’t show again”, and does **not** reappear on every launch after reinstall, profile reset, or factory defaults. No modal stacks that block open/save/browse. |
| **Mod suggestions with consent** | **Mod Portal** prompts (missing handler, recommendations) are OK when they **do not cover or replace** the content the user was already viewing or working on — use non-blocking placement (inline tab message, side panel, toast), not a full-screen takeover of the document surface. |
| **Everything in its proper place** | Commands, status, and chrome follow [ADR 0012](0012-context-switching-ux-benchmarks.md): stable shell vs swapping surfaces; export vs project ([ADR 0015](0015-native-project-format.md)); errors near what failed. |
| **User taste is always right** | Layout choices the product did not predict are **supported without nagging** — e.g. floating Command Manager bars on a **second monitor**, extreme toolbar density, pin/follow preferences. Persist silently via `toolbars.json` / window state; no “are you sure?” or reset-to-our-defaults unless the user asks. |

## Consequences

- Product and UX reviews ask: **does this help grandma without blocking hackers?**, **does this respect user space?**, and **does it violate a Bear UX rule above?**
- Primary UI copy avoids implementation terms; docs and ADRs may use precise language.
- Bundled `base-*` mods ship with the app; community mods extend, not replace, baseline usability.
- Features that require accounts, analytics, or non-optional network calls need explicit ADR justification.

## Related

- [vision.md](../vision.md) — dual audience section (canonical user-facing summary)
- [ADR 0011](0011-lazy-conversion-native-until-encode.md) — technical formats hidden until preview/export
