# Timeline encode intents (H.264 export)

When you export **H.264** from a `.unibrow` project, the shell builds a `TimelineEncodeIntent` from `project.json` and passes it to ffmpeg via `encode_h264_timeline_intents`.

## Honoured (approximate)

| Intent | Source in `project.json` | Behaviour |
| --- | --- | --- |
| Clip duration | `timeline.clips[].duration_sec` | Each video clip is trimmed with `ffmpeg -t` before concat/mix. |
| Fade between videos | `timeline.transitions[]` where `kind == "fade"` and the next clip is also video | **Only when there are exactly two video clips.** Uses `xfade=fade` between trimmed segments. On failure, falls back to plain concat and records a note. |
| Duck under video | `timeline.clips[].duck_under_video` on **audio** clips | External audio tracks are mixed over the video at about **35%** volume with `adelay` from `start_sec`. Does not duck the video’s own embedded audio (see ignored). |

Export message and stub notes list what was applied.

## Ignored or limited

| Item | Reason |
| --- | --- |
| Non video/audio assets | Skipped with a note (images, MIDI, etc.). |
| Fade with 3+ video clips | Concat only; fade honoured only for a two-clip timeline. |
| Fade when next clip is not video | Transition ignored with a note. |
| Video embedded audio + duck | Mix uses timeline **audio** assets only; embedded video audio is not levelled by `duck_under_video`. |
| Clip `start_sec` on video | Video segments are chained in timeline order; gaps are not rendered as black frames. |
| Transitions other than `fade` | Not implemented. |
| MIDI / SMF on timeline | Not rendered to audio in this path (see [midi-audio-render.md](midi-audio-render.md)). |

## Related

- [unibrow format](unibrow-format-migration.md)
- ffmpeg sidecar: [ffmpeg-sidecar.md](ffmpeg-sidecar.md)
