# ffmpeg / ffprobe T3 sidecar

Video and audio engines use **ffmpeg** as an **LGPL sidecar** ([ADR 0007](adr/0007-licensing.md), [architecture.md](architecture.md)). The Apache-licensed core **does not** link ffmpeg.

## Usage in this repo

- Crate: `unibrowser-media`
- `probe_ffmpeg_install()` — is `ffmpeg` on `PATH`?
- `probe_media_file(path)` — runs `ffprobe` when available (direct subprocess by default)
- Override binary: `UNIBROWSER_FFMPEG_PATH`, `UNIBROWSER_FFPROBE_PATH`
- IPC sketch: set `UNIBROWSER_MEDIA_USE_SIDECAR=1` and optionally `UNIBROWSER_MEDIA_SIDECAR_BIN` to the `unibrowser-media-sidecar` binary (JSON request/response per line on stdin/stdout)

## Packaging (future)

Ship an **LGPL** ffmpeg build in the installer or document system package install. Community mods that encode must invoke the sidecar over IPC/CLI, not `dlopen`.
