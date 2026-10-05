# Continuous integration

## Local check (matches intended CI)

```bash
./scripts/check-workspace.sh
```

Runs `cargo test --workspace` and `npm run build` in `crates/unibrowser-desktop`.

## Does CI actually run?

**Plain answer:** for this **Origin-hosted** repo (`origin.cursor.com/git/bearjcc/...`), we do **not** have evidence that `.github/workflows/ci.yml` executes on push. There is no `.origin` pipeline config in-tree, and Origin does not automatically run GitHub Actions workflows just because the YAML file exists.

| Expectation | Reality today |
| --- | --- |
| Green checks on every `main` push | **Not guaranteed** — verify manually or after you wire Origin/GitHub Actions |
| `windows` job protects Bear's target | **Only when** a forge runs the workflow on `windows-latest` |
| Bundle artefacts in CI | Defined in workflow (`linux-bundles`, `windows-installers` artifacts); **unverified** until CI runs |

Use `./scripts/check-workspace.sh` before pushing. When you connect GitHub Actions or an Origin pipeline that invokes the same steps, update this section with a link to live runs.

## Workflow file (contract)

[`.github/workflows/ci.yml`](../.github/workflows/ci.yml)

| Job | What it does |
| --- | --- |
| `linux` | `npm ci` + `npm run build`, `cargo test --workspace`, debug builds, **`tauri build`** (deb + AppImage), upload `linux-bundles` |
| `windows-check` | Ubuntu cross-compile: `cargo check --exclude unibrowser-desktop --target x86_64-pc-windows-msvc` |
| `windows` | `windows-latest`: tests + debug build + **`tauri build`** (NSIS + MSI), upload `windows-installers` |

Manual Windows QA: [windows-testing.md](windows-testing.md). Packaging paths: [packaging.md](packaging.md).

Linux packages: GTK 3, WebKit2GTK 4.1, and related Tauri deps (see workflow `apt-get` steps).

## Related

- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [qa-desktop.md](qa-desktop.md)
