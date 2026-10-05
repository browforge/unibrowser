# MIDI → audio render (stub)

MIDI files open in a **proving viewer** (SMF header + note-on preview). Full offline render is not implemented yet.

## Tooling probe

`probe_midi_audio_render` (palette **Render MIDI to audio (stub)** and the MIDI viewer button) checks:

| Tool | Role |
| --- | --- |
| **ffmpeg** on `PATH` | Future encode/mix step |
| **fluidsynth** on `PATH` | Future SMF → WAV synthesis |

Both must be present for the command and viewer button to enable. Otherwise they stay grey and show an install hint.

Override binaries with `UNIBROWSER_FFMPEG_PATH` / standard `PATH` lookup (see [ffmpeg-sidecar.md](ffmpeg-sidecar.md)).

## What runs today

- Enabled: returns a stub message naming the open MIDI path.
- Disabled: explains which tool is missing.

**Timeline H.264 export** (when a `.mid` clip is on the timeline): `fluidsynth` → WAV → ffmpeg mix over the video bus when binary and soundfont are available; otherwise export proceeds and the result message lists the skip reason. See [timeline-encode-intents.md](timeline-encode-intents.md).

The palette **Render MIDI to WAV** viewer stub remains unwired.

## Related

- Timeline H.264 export: [timeline-encode-intents.md](timeline-encode-intents.md)
