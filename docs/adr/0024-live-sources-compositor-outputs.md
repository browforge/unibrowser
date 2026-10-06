# ADR 0024: Live sources, live compositor, live outputs

- **Status:** Accepted (engine design + proving skeleton)
- **Date:** 2026-10-06

## Context

Bear wants **learn once, apply anywhere**: the same crop, resize, overlay, script, and chart tools used while editing a `.unibrow` timeline must work **live** on a frame clock. Preview already rasterises for the monitor; that path can also feed record and stream **sinks**. Inputs include cameras, audio, screen capture, network streams, IoT data, and input devices. Nothing in this ADR ships full RTMP/MQTT mods; the **engine** owns abstractions, compositor policy, and shared encode lanes.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — capture, screen, and network live only after explicit user action; visible **live** indicator |
| Never blocks save or work | Yes — live pipeline runs off UI thread; drop frames instead of blocking chrome |
| Local-first | Yes — no cloud ingest; stream keys in local prefs; offline mode respected for catalog mods |
| Predictable across formats | Partial — same layer graph as timeline; HTML/SVG/video paths reuse export materialisers |
| Simple durable mechanism | Yes — scenes are `.unibrow` compositions; recordings are files user owns |
| Mod not core | Yes — devices and protocol **sources/sinks** are mods; engine is traits + clock + compositor |

## Decision

### 1. Source abstraction

Today’s clip/asset becomes a **`Source`** with two timing modes:

| Mode | Behaviour |
| --- | --- |
| **File** | Seekable, timeline-bound, duration known |
| **Live** | Clock-driven, unseekable, may **drop** frames under load |

**Media sources** produce `MediaFrame` (RGBA + PTS). **Data sources** produce typed `DataSample` (number, text, event). Live kinds are **mods**: V4L2 / Media Foundation camera, mic, screen/window, RTMP/SRT/WebRTC/HLS/RTSP in, MQTT/HTTP/serial/WebSocket, keyboard/mouse/gamepad/MIDI overlay feeds, chat, SDR/HAM (later).

Crate: `crates/unibrowser-live` (`MediaSource`, `DataSource`, `SourceTiming`).

### 2. Data vs media; bindings

Data streams **bind** to layer properties (text template, chart scalar, opacity, script inputs). Timeline **keyframes** are a recorded data stream replayed in file mode. Binding types: `PropertyBinding` + `DataBinding` in `unibrowser-live`.

### 3. Live compositor

`.unibrow` timeline renders in **live mode** on a `FrameClock` (target fps). Same layer graph: transforms, crop, effects.

| Layer kind | Live materialise path |
| --- | --- |
| Video | ffmpeg / hw decode (mod-provided) |
| HTML/JS | Offscreen webview or snapshot tick |
| SVG | `resvg` (same as encode) |
| Text/chart | Engine overlay + bindings |

**GPU (wgpu)** path for real-time preview; **CPU** compositor fallback (proving slice). **Latency:** target one frame period; **policy:** `DropFramePolicy` — never block the UI; skip catch-up when behind.

### 4. Scenes

A **scene** is a `LiveScene` (layers + bindings). Multiple scenes per project with hotkey/script **scene switch** and transition hooks. **Event overlays** (donation alert) subscribe to data `Event` samples.

### 5. Outputs (sinks) as mods

| Sink | Role |
| --- | --- |
| Preview window | Monitor path (existing webview/surface) |
| Record file | Same H.264 intents as offline export |
| Stream out | RTMP / SRT / WHIP (mod) |
| Virtual camera / NDI | Mod |

**Multiple sinks** attach to one compositor fan-out. Encode pipeline shared with `unibrowser-media` / timeline export ([ADR 0007], [ADR 0011]).

### 6. Scripting

Sandboxed **per-frame** script layer (JS via existing scripting engine or WASM) reads data streams and sets layer properties. Scripts cannot enable capture or network without host capability tokens.

### 7. Permissions and privacy

- Capture devices, screen, and inbound network: **user-initiated** only, with persistent consent per device/mod.
- **Visible live indicator** when any live source or outbound stream is active.
- **No network by default** for live mods; aligns with [network.md](../network.md) and offline mode.
- Broadcast keys stored in **local** prefs (encrypted-at-rest optional later).

### 8. Worked examples (product targets)

**(a) Backyard weather channel** — IP cam RTSP mods + MQTT thermometer + forecast image file sources + live graph bound to temperature; RTMP sink mod rebroadcasts.

**(b) Game streamer** — game capture mod + input-display overlay (keyboard data) + chat feed mod + HTML/JS speedrun timer layer + donation **event** overlay animation.

### 9. Proving skeleton (this slice)

| Piece | Location |
| --- | --- |
| Traits + clock + compositor + file sink | `crates/unibrowser-live` |
| Fake live video (test pattern) | `mods/live-proving` |
| Fake data (sine temperature) | `mods/live-proving` |
| E2E: N frames → MP4 + ffprobe duration | `crates/unibrowser-live/tests/live_proving_e2e.rs` |

No real device or network mods in-tree.

## Consequences

- Desktop shell gains live UI, indicators, and mod catalog entries later; engine API is stable-first.
- Timeline export and live record share encoder configuration to avoid “two truths”.
- Drop-frame policy must be reflected in diagnostics (dropped frame counter).

## References

- [ADR 0011](0011-lazy-conversion-native-until-encode.md), [ADR 0007](0007-licensing.md)
- [live-mod-authoring.md](../live-mod-authoring.md)
- `crates/unibrowser-live`, `mods/live-proving`
