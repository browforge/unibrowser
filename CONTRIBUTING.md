# Contributing to Unibrowser

Thank you for helping build a unified file and web browser shell. This project is **Apache-2.0** (see `LICENSE`); bundled mods and sidecars may use other licences as documented under `docs/`.

## Before you start

1. Read `README.md` and `docs/architecture.md` for the engine / mod / shell split.
2. Skim relevant ADRs in `docs/adr/` when your change touches navigation, downloads, projects, or UI commands.
3. Run the automated gate locally:

```bash
cargo test --workspace
cd crates/unibrowser-desktop && npm ci && npm run build
./scripts/smoke-desktop-qa.sh
```

On Linux you need GTK/WebKit dev packages (see `.github/workflows/ci.yml`). Windows desktop builds are not required for every change; `cargo check --workspace --exclude unibrowser-desktop --target x86_64-pc-windows-msvc` must stay green for engine and mod edits.

## How to contribute

- **Small fixes:** Open a PR with a clear description and tests when behaviour changes.
- **Features:** Prefer extending existing mods, engines, or shell surfaces before adding parallel systems. New UI should follow Command Manager and `t()` i18n patterns in the desktop shell.
- **Security:** Paths go through `path_security`; downloads use `apply_download_disposition`. Do not bypass these for convenience.

## Code style

- **Rust:** Match surrounding modules; keep commands thin and logic in libraries (`unibrowser-project`, `unibrowser-engine`, etc.).
- **TypeScript:** Shell UI lives in `crates/unibrowser-desktop/src/`; use existing viewer and settings patterns rather than new global modals for prefs.
- **Commits:** One logical change per commit; complete sentences in messages.

## Tests

- Add unit tests next to new Rust logic (`smf`, `downloads`, `unibrowser-project`, etc.).
- Manual checks for UI: `docs/qa-desktop.md`.

## Community

Be respectful and constructive. If you are unsure whether an idea fits the vision in `docs/vision.md`, open an issue or draft PR early for feedback.
