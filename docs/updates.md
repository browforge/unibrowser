# Updates (FOSS)

Unibrowser does **not** ship a background update checker, auto-updater, or silent download of new binaries.

## Check for updates (manual)

In **Settings**, **Check for updates** opens the project **releases** page in your system browser. That is the only network action tied to updates: you choose when to look.

Packagers and forks can point releases elsewhere at build or run time:

```bash
export UNIBROWSER_RELEASES_URL="https://github.com/your-org/your-fork/releases"
```

## How to stay current

| Audience | Suggested flow |
| --- | --- |
| Developers | `git pull` on `main`, `cargo test --workspace`, rebuild the desktop crate |
| Linux users | Rebuild from source or install a new `.deb` / AppImage when your distro or CI publishes one ([packaging.md](packaging.md)) |
| Windows / macOS | Re-run the installer or build from source when releases exist |

Version and optional git hash appear on `unibrowser://about`.

## Privacy

No update telemetry is sent when you click **Check for updates**; only your browser loads the releases URL you configured. See [privacy.md](privacy.md).
