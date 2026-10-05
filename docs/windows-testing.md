# Windows manual testing checklist

Bear's primary target is **Windows**. Linux cloud agents only cross-check engine crates; full desktop builds run on **`windows-latest`** CI and on your PC.

## Prerequisites

1. [Rust](https://rustup.rs/) stable (MSVC toolchain: `rustup default stable-msvc`).
2. [Node.js](https://nodejs.org/) 20 LTS (`node --version`).
3. [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (usually preinstalled on Windows 11).
4. Optional for media demos: **ffmpeg** and **ffprobe** on `PATH` (or set `UNIBROWSER_FFMPEG_PATH`).
5. Optional for PDF timeline export: **pdftoppm** (Poppler) on `PATH`.
6. Optional for VSIX Node host: `node` on `PATH` and **Settings → Advanced → Node extension host (experimental)** enabled.

## Build and run

```powershell
cd crates\unibrowser-desktop
npm ci
npm run build
cd ..\..
cargo build -p unibrowser-media --bin unibrowser-media-sidecar
cargo run -p unibrowser-desktop
```

Or from `crates\unibrowser-desktop`:

```powershell
npm run tauri dev
```

## Smoke steps

| Step | What to verify |
| --- | --- |
| Cold start | App opens; home tab; no panic dialog |
| `file://` tab | Open a local `.md` or `.png` from omnibox or drag-drop |
| `https://` tab | Load a simple HTTPS page; loading overlay clears |
| **Download** | On an https tab, trigger a small file download → should emit in-app flow (open tab or save per Settings), not only Edge's default folder UI |
| Downloads folder | `unibrowser://downloads` → **Open Downloads folder** opens your user Downloads (known folder) |
| ffmpeg sidecar | With ffmpeg on PATH, open a short `.mp4`; waveform/thumbnail paths work; check diagnostics for media probe |
| Media sidecar binary | After `cargo build`, `target\debug\unibrowser-media-sidecar.exe` exists next to the desktop binary when bundled |
| Node extension host | Enable pref; install a sample VSIX with `main`; run a contributed command; diagnostics show Node pid |
| Multi-window | New window; independent tabs; close and reopen |
| F12 | DevTools on https tab |

## Diagnostics

Open `unibrowser://diagnostics` and confirm:

- `web_download_hook_wired: true` on Windows
- Download hook detail mentions **DownloadStarting**

## When something fails

- Note Windows version, build (`git rev-parse HEAD`), and copy details from the error toast.
- Compare with [qa-desktop.md](qa-desktop.md) and [browser-shell.md](browser-shell.md).
- CI reference: [ci.md](ci.md) (`windows` job).
