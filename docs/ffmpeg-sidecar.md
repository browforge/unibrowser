# ffmpeg / ffprobe T3 sidecar

Video and audio engines use **ffmpeg** as an **LGPL sidecar** ([ADR 0007](adr/0007-licensing.md), [architecture.md](architecture.md)). The Apache-licensed core **does not** link ffmpeg.

## Usage in this repo

- Crate: `unibrowser-media`
- `probe_ffmpeg_install()` — is `ffmpeg` on `PATH`?
- `probe_media_file(path)` — runs `ffprobe` when available
- Override binary: `UNIBROWSER_FFMPEG_PATH`, `UNIBROWSER_FFPROBE_PATH`

## Packaging (future)

Ship an **LGPL** ffmpeg build in the installer or document system package install. Community mods that encode must invoke the sidecar over IPC/CLI, not `dlopen`.
