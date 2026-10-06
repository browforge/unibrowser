# Contributing

How to build Unibrowser, run checks, add a proving mod, and open a PR. The repo root [CONTRIBUTING.md](../CONTRIBUTING.md) covers community norms and `.unibrow` asset policy; this page is the developer runbook.

## Principles (tiebreaker)

Read [principles.md](principles.md) before UX, ADR, or architecture changes. Use the **Decision checklist** there when choosing between options. New ADRs must include a **Principles check** ([adr/0000-template.md](adr/0000-template.md)). In PRs, call out which principle you are serving when the change is not obvious.

## Build and run

```bash
cargo test --workspace
cd crates/unibrowser-desktop && npm ci && npm run build
npm run tauri dev
```

Linux desktop builds need GTK 3 and WebKitGTK dev packages (see `.github/workflows/ci.yml`). Engine-only changes should keep `cargo check --workspace --exclude unibrowser-desktop --target x86_64-pc-windows-msvc` green when you touch shared crates.

CLI: `cargo run -p unibrowser-cli -- --version`

Full gate (matches CI intent):

```bash
./scripts/check-workspace.sh
./scripts/smoke-desktop-qa.sh
./scripts/blitz-metrics.sh
./scripts/lint-doc-links.sh
```

`lint-doc-links.sh` checks relative links from `docs/index.md`; pass `--all` to scan every file under `docs/`.

## Architecture (where to edit)

| Layer | Path | Role |
| --- | --- | --- |
| Mod API | `crates/unibrowser-mod-api` | Command ids, handler traits, capabilities |
| Engine | `crates/unibrowser-engine` | Registry, palette, Command Manager resolution |
| Desktop shell | `crates/unibrowser-desktop` | Tauri commands, webview, TS viewers |
| Bundled mods | `mods/base-*`, `mods/toy-*` | Per-format backends and command handlers |
| Projects | `crates/unibrowser-project` | `.unibrow` manifest and timeline |

New user-facing commands: register `CommandMeta` in the engine or a mod, add a `HandlerRegistration`, wire a Tauri `execute_command` intercept when the shell must open a picker or webview, and add palette/CM strings via `t()` in the desktop UI when needed.

## Add a proving mod

1. Copy the pattern from `mods/base-dxf` or `examples/hello-world.ubmod` and [proving-mod-dxf.md](proving-mod-dxf.md).
2. Register the mod in `unibrowser-mod-host` / `AppState::bundled_mod_registers()` and give it a `backend_id`.
3. Implement `DocumentBackend`, `register()` with greyed stubs for unimplemented tools, and unit tests beside the mod.
4. Document limitations in a short `docs/proving-mod-*.md` page and link from [docs/index.md](index.md) if the mod is user-visible.

Do not bypass `path_security` or download disposition helpers in the shell.

## Pull requests

- One logical change per commit; messages in complete sentences.
- Behaviour changes: add or extend Rust unit tests; note manual QA in the PR if UI-only.
- Keep `cargo test --workspace` and `npm run build` green before push.
- No large binaries in git; use tiny fixtures or documented local fetch for media.
- Security-sensitive paths: describe threat model briefly if you change trust banners, downloads, or webview policy.

## Docs and changelog

User-facing blitz notes: [blitz-status.md](blitz-status.md) and [CHANGELOG.md](../CHANGELOG.md). ADRs live under `docs/adr/` (start from [0000-template.md](adr/0000-template.md)); link new decisions there when the change is architectural. Flag principle tensions in [blitz-status.md](blitz-status.md) **Principles watchlist** when shipping something that still violates the checklist.
