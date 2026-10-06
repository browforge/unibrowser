# Live mod authoring (stub)

Live **sources** and **sinks** are Factorio mods on top of `crates/unibrowser-live`. The desktop host loads manifests that contribute `liveSources` / `liveSinks` (see `mods/live-proving/info.json`).

## Proving mod

`mods/live-proving` ships:

| Id | Kind | Purpose |
| --- | --- | --- |
| `live-proving/test-pattern` | `live_media` | Moving colour bars (fake camera) |
| `live-proving/sine-temperature` | `data_stream` | `temperature` channel, sine wave °C |

Run the engine proof:

```bash
cargo test -p unibrowser-live live_mode_writes_expected_duration -- --nocapture
```

Requires `ffmpeg` and `ffprobe` on PATH (skips otherwise).

## Authoring a real live mod (not implemented here)

1. Depend on `unibrowser-live` and implement `MediaSource` or `DataSource` (or sink trait when exposed).
2. Declare ids in `info.json` under `contributes.liveSources` or `liveSinks`.
3. Request capabilities in the manifest (`camera`, `microphone`, `screen_capture`, `network_in`, …); host shows consent UI and live indicator.
4. Respect [network.md](network.md) and [principles.md](principles.md): no background connections.

See [ADR 0024](adr/0024-live-sources-compositor-outputs.md) for the full engine design.
