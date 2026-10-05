# Continuous integration

## Local check (matches CI)

```bash
./scripts/check-workspace.sh
```

Runs `cargo test --workspace` and `npm run build` in `crates/unibrowser-desktop` (same gates as GitHub Actions).

## GitHub Actions

Workflow: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)

| Job | What it does |
| --- | --- |
| `linux` | `npm ci` + `npm run build`, `cargo test --workspace`, debug builds for desktop + media sidecar + egui spike |
| `windows-check` | Ubuntu cross-compile: `cargo check --workspace --exclude unibrowser-desktop --target x86_64-pc-windows-msvc` (desktop needs a real Windows host for `ring`/WebView2) |
| `windows` | `windows-latest`: same npm build + `cargo test --workspace` + debug `unibrowser-desktop` (primary Bear target) |

Manual Windows QA beyond CI: [windows-testing.md](windows-testing.md).

Linux packages: GTK 3, WebKit2GTK 4.1, and related Tauri deps (see workflow `apt-get` step).

## Origin (Cursor cloud)

When this repository is connected to **Origin** as the source of truth:

1. Push to `main` or open a PR from `cursor/**` branches.
2. Origin runs the same workflow definitions hosted on the forge (equivalent to the GitHub Actions file above unless your team adds Origin-specific steps).
3. Use `./scripts/check-workspace.sh` on an agent or laptop before pushing to catch failures early.

If CI is not wired yet on Origin, the workflow file is the contract: add a pipeline that invokes `scripts/check-workspace.sh` on Ubuntu with Rust stable and Node 20.

## Related

- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [qa-desktop.md](qa-desktop.md)
