# ADR 0006: Mod manifest and versioning

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

Factorio `info.json` style: id, version, dependencies. Mods need predictable loading order and API compatibility.

## Decision

Each mod directory contains `info.json`:

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
  "description": "Plain text documents"
}
```

Rules:

- **`name`**: unique mod id (lowercase, hyphenated).
- **`unibrowser_api`**: semver of `unibrowser-mod-api` the mod was built against; host checks compatibility.
- **`dependencies`**: semver requirements on other mod names; topological sort for load order; missing or cyclic deps fail load.
- **Virtual `base` mod**: always present; version `0.1.0` in skeleton.

Native library optional: `library` field (`libub_mod_base_text.so`) for dynamic load; bundled mods may also register via built-in table for dev ergonomics.

## Consequences

- Community mods ship as folder or `.ubmod` zip (format TBD).
- Engine bumps `UNIBROWSER_MOD_API_VERSION` on breaking trait changes.

See [api-gap-upstream.md](../api-gap-upstream.md).
