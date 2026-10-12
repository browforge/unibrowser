# Packaging

Unibrowser bundles via **Tauri 2** (`tauri.conf.json` targets: `deb`, `appimage`, `nsis`, `msi`).

## CI / automation status

This repository is hosted on **Origin** (`origin.cursor.com`). The workflow file [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) is the **intended** contract, but **automated runs are not confirmed** on Origin for this project (no linked GitHub Actions mirror observed). Treat bundle steps below as what CI *should* run once a forge executes the workflow; local `./scripts/check-workspace.sh` remains the reliable gate until then.

## Linux (VM / `ubuntu-latest`)

**Prerequisites:** run `sudo ./scripts/setup-linux-deps.sh` (GTK/WebKit, xvfb, ImageMagick, fuse, libfuse2, patchelf, and the wasm32 target), or install the same packages as [ci.md](ci.md) `linux` job manually.

```bash
cd crates/unibrowser-desktop
npm ci
cargo build -p unibrowser-media --bin unibrowser-media-sidecar --release
npm run tauri build
```

**Outputs (workspace target dir):**

| Artefact | Typical path |
| --- | --- |
| `.deb` | `target/release/bundle/deb/unibrowser_*_amd64.deb` |
| AppImage | `target/release/bundle/appimage/unibrowser_*_amd64.AppImage` |

**Cloud VM smoke (2026-10-05):** `npm run tauri build` succeeded on Ubuntu (GTK/WebKit deps from CI list; `patchelf` present; system `fuse` package conflict did not block bundler).

| Artefact | Size | Headless smoke |
| --- | --- | --- |
| `Unibrowser_0.1.0_amd64.deb` | ~17.3 MiB (bundler) | `dpkg-deb -I` shows package metadata |
| `Unibrowser_0.1.0_amd64.AppImage` | ~91.1 MiB (bundler) | `./Unibrowser_0.1.0_amd64.AppImage --appimage-version` → `Version: effce44` |

Full GUI launch was not exercised in the headless VM; use Bear's desktop for an interactive smoke.

## Windows (Bear machine / `windows-latest`)

```powershell
cd crates\unibrowser-desktop
npm ci
cargo build -p unibrowser-media --bin unibrowser-media-sidecar --release
npm run tauri build
```

| Artefact | Typical path |
| --- | --- |
| NSIS | `target\release\bundle\nsis\Unibrowser_*-setup.exe` |
| MSI | `target\release\bundle\msi\Unibrowser_*_x64_en-US.msi` |

WebView2: bundle config uses `downloadBootstrapper` so the installer can fetch the runtime when missing ([browser-shell.md](browser-shell.md)).

## Versioning

`Cargo.toml` / `tauri.conf.json` `version` and [CHANGELOG.md](../CHANGELOG.md).

## File associations

Still not registered in installers; see [file-associations.md](file-associations.md).
