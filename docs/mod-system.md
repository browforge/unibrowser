# Mod system overview

Unibrowser uses **Factorio-style native mods** as the source of truth for formats and deep behaviour. **VS Code `.vsix` packages** are a **compatibility import path** for ecosystem reuse — not a replacement ([ADR 0014](adr/0014-vsix-compatibility-layer.md)).

## Native mods (first-class)

- Manifest: `info.json` per [ADR 0006](adr/0006-mod-manifest-and-versioning.md).
- Register commands, handlers, document backends via `unibrowser-mod-api`.
- Execution tiers: [ADR 0002](adr/0002-mod-system-and-sandboxing.md) (T0 bundled → T3 sidecar).
- Shipped in the box for everyday users (`base-text`, `base-web`, `base-files`, …).

```
mods/my-format/
  info.json
  src/ … Rust or future WASM …
```

## VSIX compat (optional, power users)

- **Goal:** Reuse existing `.vsix` where contribution points map cleanly (commands, grammars, themes, …).
- **Limit:** Most extensions depend on `vscode.*` APIs; only a **subset** is honoured or shimmed; see mapping table in [ADR 0014](adr/0014-vsix-compatibility-layer.md).
- **UX:** No VSIX prompts in the default install path ([ADR 0013](adr/0013-dual-audience-foss-and-everyday-users.md)).
- **Phases:** Metadata-only (V1) → Monaco assets (V2) → minimal extension host (V3) → optional Open VSX (V4).

```
user-extensions/
  publisher.extension-1.2.3/   # unpacked from .vsix
    package.json
    …
```

Native `info.json` mods may **depend on** assets produced from a VSIX (e.g. grammar-only pack) via a future `vsix_import` manifest field — design TBD.

## Choosing native vs VSIX

| Need | Prefer |
|------|--------|
| New file format / timeline / backend | Native `info.json` mod |
| Syntax highlighting for a language | VSIX (V2) or native grammar bundle |
| Cross-app command (font size, save) | Native handler + registry |
| Reuse a popular VS Code command-only extension | VSIX V1+ with stub host |
| GPL office/CAD kernel | Native T3 sidecar, not VSIX |

## Loading order

1. Virtual `base` mod  
2. Topological sort of `info.json` dependencies  
3. (Future) VSIX contributions merged into registry **after** native handlers, with explicit precedence rules for keybindings

## See also

- [architecture.md](architecture.md)
- [api-gap-upstream.md](api-gap-upstream.md)
- [roadmap.md](roadmap.md)
