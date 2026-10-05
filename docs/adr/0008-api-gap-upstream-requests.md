# ADR 0008: API gaps and upstream requests

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

When mods need host capability that does not exist, ad hoc FFI leaks abstraction. The API should evolve deliberately with versioning.

## Decision

1. Mods declare required **capabilities** in manifest (`capabilities`: `["font-size", "export-png"]`) for documentation; runtime uses trait methods on backends.
2. If implementation needs a new host API, authors open a **GitHub issue** with label `api-request` using template:
   - Mod id and use case
   - Proposed trait or context key
   - Backward compatibility story
3. Maintainers respond with: **accept** (target API version), **defer**, or **wontfix** (mod must use sidecar).
4. Breaking changes bump **major** `unibrowser-mod-api` semver; host keeps one previous major loadable for one release when feasible.

Document gaps in [api-gap-upstream.md](../api-gap-upstream.md).

## Consequences

- Feature pressure visible in issue tracker, not scattered in Discord.
- Capability negotiation fields reserved in `HostContext` for future wire formats (WASM/T3).
