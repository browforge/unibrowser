# ffmpeg / ffprobe T3 sidecar

Video and audio engines use **ffmpeg** as an **LGPL sidecar** ([ADR 0007](adr/0007-licensing.md), [architecture.md](architecture.md)). The Apache-licensed core **does not** link ffmpeg.

## Usage in this repo

- Crate: `unibrowser-media`
- `probe_ffmpeg_install()` — is `ffmpeg` on `PATH`?
- `probe_media_file(path)` — runs `ffprobe` when available (direct subprocess by default)
- Override binary: `UNIBROWSER_FFMPEG_PATH`, `UNIBROWSER_FFPROBE_PATH`
- IPC sketch: set `UNIBROWSER_MEDIA_USE_SIDECAR=1` and optionally `UNIBROWSER_MEDIA_SIDECAR_BIN` to the `unibrowser-media-sidecar` binary (JSON request/response per line on stdin/stdout)

## Sidecar binary

- Crate `unibrowser-media`, binary `unibrowser-media-sidecar` (JSON line protocol; loops on stdin until EOF).
- `SidecarSession` in `sidecar.rs` keeps one process alive for multiple requests (integration test: `sidecar_ping_roundtrip`).
- Env: `UNIBROWSER_MEDIA_USE_SIDECAR=1`, optional `UNIBROWSER_MEDIA_SIDECAR_BIN` (defaults to sibling of the running executable).

## Tauri bundling (sketch)

Ship the sidecar next to the desktop app (LGPL, not linked):

1. Build: `cargo build -p unibrowser-media --release` (produces `unibrowser-media-sidecar`).
2. In `crates/unibrowser-desktop/src-tauri/tauri.conf.json`, add under `bundle`:

```json
"externalBin": ["../../../../target/release/unibrowser-media-sidecar"]
```

Adjust the path for your CI output dir, or copy/rename the binary into `src-tauri/binaries/` per [Tauri external binaries](https://v2.tauri.app/develop/resources/) and list the stub name there. Set `UNIBROWSER_MEDIA_SIDECAR_BIN` at runtime if the install layout differs.

CI (`.github/workflows/ci.yml`) builds the sidecar with:

`cargo build -p unibrowser-media --bin unibrowser-media-sidecar`

Desktop calls `unibrowser_media::warm_sidecar_pool()` on startup when `UNIBROWSER_MEDIA_USE_SIDECAR=1`; probes reuse `SidecarSession` via `probe_file_pooled`.

## Packaging (future)

Ship an **LGPL** ffmpeg build in the installer or document system package install. Community mods that encode must invoke the sidecar over IPC/CLI, not `dlopen`.
