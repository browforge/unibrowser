# Packaging (stubs for CI later)

Unibrowser ships from source during the blitz. This page records intended release artefacts so CI can grow without re-deciding formats.

## Windows

| Artefact | Tooling (planned) | Notes |
| --- | --- | --- |
| MSI | WiX v4 or `cargo-wix` against the Tauri `bundle` output | Per-machine install under `Program Files`; optional Start Menu shortcut |
| NSIS | Tauri `bundle.targets = ["nsis"]` | Smaller download; same `unibrowser-desktop` binary + WebView2 bootstrapper check |

**Stub steps (local smoke, not wired in CI yet):**

```powershell
cd crates/unibrowser-desktop
npm ci
npm run tauri build
# Inspect crates/unibrowser-desktop/src-tauri/target/release/bundle/nsis/ and msi/ when targets are enabled in tauri.conf.json
```

Document WebView2 runtime requirement in the installer welcome text ([browser-shell.md](browser-shell.md)).

## Linux

| Artefact | Tooling (planned) | Notes |
| --- | --- | --- |
| AppImage | `tauri-bundler` AppImage target + `linuxdeploy` | Portable; ships GTK/WebKit deps where possible |
| `.deb` | `dpkg-deb` from bundle `deb` target | Depends on `libwebkit2gtk-4.1-0`, `libayatana-appindicator3-1` (or distro equivalents) |

**Stub steps:**

```bash
cd crates/unibrowser-desktop
npm ci
npm run tauri build
# bundle/deb and bundle/appimage under src-tauri/target/release/bundle/
```

Headless PDF export on Linux expects `chromium` or `wkhtmltopdf` on the **user** PATH ([commands-undo-redo.md](commands-undo-redo.md)); installers do not need to bundle a browser for that.

## macOS (out of scope for blitz CI)

DMG via Tauri `dmg` target when Apple signing is available.

## Versioning

Release version follows `Cargo.toml` / `tauri.conf.json` `version` field and [CHANGELOG.md](../CHANGELOG.md).

## File associations

Unibrowser does not register OS default handlers in this blitz. See [file-associations.md](file-associations.md) for current behaviour and planned opt-in association steps for FOSS installers.
