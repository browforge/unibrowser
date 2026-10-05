# Branding and window chrome

## Product name

The desktop shell uses **Unibrowser** as the product name (not a registered trademark claim in this repo).

| Surface | Value |
| --- | --- |
| Tauri `productName` | `Unibrowser` |
| Default window title | `Unibrowser` |
| Active tab | `Unibrowser — {tab label}` (web UI updates title via Tauri `setTitle`) |
| HTML `<title>` | Same as window title |
| About page | `Unibrowser {version}` plus optional git hash |

Config: `crates/unibrowser-desktop/src-tauri/tauri.conf.json`.

## Icons (bundle)

Packaging icons live under `crates/unibrowser-desktop/src-tauri/icons/`:

- `32x32.png`, `128x128.png`, `128x128@2x.png`
- `icon.icns` (macOS)
- `icon.ico` (Windows)

Listed in `tauri.conf.json` → `bundle.icon`. Replace those files to change the installed app icon; do not commit third-party trademarked artwork without permission.

## About / diagnostics

`unibrowser://about` does not embed a separate logo asset in the blitz build; version and licence text only. Optional marketing assets belong in distributor packaging, not in core source, unless you add them explicitly under `icons/` with clear licence.
