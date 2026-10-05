# `unibrowser-mod-api` stable surface (0.1.x freeze check)

Compiled mods (cdylib / future WASM) must target this API. Breaking changes bump `MOD_API_VERSION` and `unibrowser_api` in `info.json` per [ADR 0006](adr/0006-mod-manifest-and-versioning.md).

## Version

- Crate: `unibrowser-mod-api` **0.1.0**
- Constant: `MOD_API_VERSION` (`"0.1.0"`)

## Stable modules (T1)

| Module | Purpose |
|--------|---------|
| `navigation` | `BrowserLocation`, `TabId`, `DownloadDisposition`, `display_label` |
| `types` | `Mode`, `SelectionDomain`, `SelectionSnapshot`, `Capabilities`, `CommandMeta`, `Enablement`, command id constants |
| `context` | `CommandContext`, `HostContext` |
| `handlers` | Shared handler ids (`FONT_INCREASE_ID`, `TEXT_SAVE`, …) |
| `manifest` | `ModManifest` JSON shape |

## Not stable yet

- Engine `DocumentBackend` trait (lives in `unibrowser-engine`, not this crate)
- Dynamic `cdylib` loader ABI (draft: [docs/plugin-sdk](plugin-sdk/README.md), `examples/ub-mod-cdylib-stub`; Linux load via `UNIBROWSER_LOAD_CDYLIB_MOD=1`)
- VSIX / extension-host types

## Mod author checklist

1. Pin `unibrowser_api` in `info.json` to the host’s `MOD_API_VERSION`.
2. Depend only on types in the table above from Rust mods; register commands via host loader when wired.
3. Ship `library` = `libub_mod_<name>.so` (platform-specific) beside `info.json`.

Example native stub: `examples/ub-mod-cdylib-stub` (`cargo build -p ub-mod-cdylib-stub`).
