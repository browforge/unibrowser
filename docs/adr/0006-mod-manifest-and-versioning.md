# ADR 0006: Mod manifest and versioning

- **Status:** Accepted (amended 2026-10-05 — manifest is metadata only)
- **Date:** 2026-10-05

## Context

Factorio-style **`info.json`**: id, version, dependencies. The manifest is **not** the mod — it points at a **compiled payload** ([ADR 0002](0002-mod-system-and-sandboxing.md)). Mods need predictable load order and API compatibility.

## Decision

Each mod **package** contains `info.json` **plus** a loadable binary (or WASM module):

```json
{
  "name": "base-text",
  "version": "0.1.0",
  "title": "Plain text",
  "author": "Unibrowser Team",
  "unibrowser_api": "0.1.0",
  "dependencies": {
    "base": ">=0.1.0"
  },
  "optional_dependencies": {},
  "description": "Plain text documents",
  "library": "libub_mod_base_text.so"
}
```

Rules:

- **`name`**: unique mod id (lowercase, hyphenated).
- **`unibrowser_api`**: semver of `unibrowser-mod-api` the mod was built against; host checks compatibility.
- **`dependencies`**: semver requirements on other mod names; topological sort for load order; missing or cyclic deps fail load.
- **Virtual `base` mod**: always present; version `0.1.0` in skeleton.
- **`library`**: filename of the **native** mod binary (`cdylib`) for this platform; **required** for community T1 packages (optional in skeleton while static-linked).
- **`wasm_module`** (future): alternative entry for T2 instead of or in addition to `library`.
- **`license`**: SPDX id; required for install UI; proprietary licences allowed for T1 vendor mods.

**Not in manifest:** business logic, scripts, or “behaviour JSON”. Those live in the compiled artefact (or VSIX compat lane, which is separate).

**T0 dev exception:** in-repo `base-*` may static-register without shipping a separate `.so` in the tree; release builds should still align with manifest + binary packaging over time.

## Consequences

- Community mods ship as `.ubmod` (or folder): `info.json` + `libub_mod_*.{so,dll,dylib}` and/or `.wasm` (format TBD).
- Engine bumps `UNIBROWSER_MOD_API_VERSION` on breaking **ABI** / trait changes.
- Closed-source and paid mods are **allowed** at T1; engine and bundled `base-*` stay FOSS.

See [api-gap-upstream.md](../api-gap-upstream.md).
