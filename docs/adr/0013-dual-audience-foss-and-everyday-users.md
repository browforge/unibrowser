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

Power features (Command Manager pin, palette, custom layouts, native mod install, optional **`.vsix` import**) are **available**, not **required** for daily use. VSIX / Open VSX never appear in the grandma onboarding path ([ADR 0014](0014-vsix-compatibility-layer.md)).

### Everyday audience — principles

| Principle | Implication |
|-----------|-------------|
| **It just works** | Bundled mods open common types out of the box; no “install a mod first” for PDF, images, text, web. |
| **No format jargon in primary UI** | Grandma does not choose raster vs vector or PNG vs WebP — she opens a photo; the shell picks a backend. Technical detail in status/tooltips or advanced panels only. |
| **Formats under the hood** | Conversion and proxies are invisible until export or an explicit advanced action ([ADR 0011](0011-lazy-conversion-native-until-encode.md)). |
| **Chrome OS–style shell** | Tabs, omnibox, familiar browser chrome ([ADR 0009](0009-browser-as-unified-shell.md)). |

### Shared bridge: progressive disclosure

- **Default path** — open tab, edit/view, save — minimal choices, sensible defaults.
- **Depth path** — mods, keybindings, toolbars, pin, palette, export pipelines — same app, no mode switch to “expert edition”.
- **Honest limits** — unsupported format = clear tab/message, not a crash or jargon dump.

## Consequences

- Product and UX reviews ask: **does this help grandma without blocking hackers?** and **does this respect user space?**
- Primary UI copy avoids implementation terms; docs and ADRs may use precise language.
- Bundled `base-*` mods ship with the app; community mods extend, not replace, baseline usability.
- Features that require accounts, analytics, or non-optional network calls need explicit ADR justification.

## Related

- [vision.md](../vision.md) — dual audience section (canonical user-facing summary)
- [ADR 0011](0011-lazy-conversion-native-until-encode.md) — technical formats hidden until preview/export
