# Timeline encode intents (H.264 export)

When you export **H.264** from a `.unibrow` project, the shell builds a `TimelineEncodeIntent` from `project.json` and passes it to `encode_h264_timeline_intents` (ffmpeg sidecar + lazy raster lanes).

Export resolution defaults to **1920×1080** (`export_width` / `export_height` on the intent). Preview in the shell keeps using existing raster proxies; vectors stay references in `project.json` until encode ([ADR 0011](adr/0011-lazy-conversion-native-until-encode.md)).

## Visual bus (timeline “video” chain)

| Clip kind | Stored in project | At H.264 encode |
| --- | --- | --- |
| MP4 video | `assets/…` path | Trim/copy via ffmpeg |
| PDF page | Vector PDF + `pdf_page` (1-based) | **pdftoppm** → PNG → short MP4 loop at export resolution; preview uses pdf.js raster proxy |
| SVG still | Vector `.svg` | **resvg** (MIT) → PNG → MP4 loop |
| Title card | `title_text`, `title_font_pt` | SVG title card via resvg → PNG → MP4; font size follows clip field (also bumped by `unibrowser.font.size.increase` on an open `.unibrow` tab) |

Clips on the visual bus are ordered by `start_sec`, trimmed, optionally faded, then concatenated (or cross-faded when exactly two clips have a fade transition).

## Audio mix

| Clip kind | At encode |
| --- | --- |
| Audio file | Mixed over the video bus with `adelay` from `start_sec` |
| MIDI (SMF) | **fluidsynth** + soundfont → WAV, then mixed; skipped with an honest export note if tools or soundfont are missing |

Soundfont resolution order: Settings **FluidSynth soundfont** path → `UNIBROWSER_SOUNDFONT_PATH` → common Linux `FluidR3_GM.sf2` locations.

## Honoured (approximate)

| Intent | Source in `project.json` | Behaviour |
| --- | --- | --- |
| Clip duration | `timeline.clips[].duration_sec` | Each visual segment is trimmed with `ffmpeg -t` before concat/mix. |
| Fade between visuals | `timeline.transitions[]` where `kind == "fade"` and the next clip is on the visual bus | **Only when there are exactly two visual clips.** Uses `xfade=fade`. On failure, plain concat + note. |
| Duck under video | `timeline.clips[].duck_under_video` on **audio/MIDI** clips | Mixed at about **35%** volume with `adelay` from `start_sec`. |
| Clip fade in / out | `timeline.clips[].fade_in_sec` / `fade_out_sec` on visual clips | Per-clip `ffmpeg` `fade` filter when non-zero. |

Export message and `EncodeIntentReport.ignored` list skipped MIDI lanes, lazy-tool gaps, and other limits.

## Ignored or limited

| Item | Reason |
| --- | --- |
| `Other` asset types | Skipped with a note |
| Fade with 3+ visual clips | Concat only; fade honoured only for a two-clip timeline |
| Fade when next clip is not on the visual bus | Transition ignored with a note |
| Video embedded audio + duck | Mix uses timeline **audio/MIDI** assets only |
| Clip `start_sec` on video | Visual segments are chained in order; gaps are not black frames |
| Transitions other than `fade` | Not implemented |
| PDF without **pdftoppm** | Export fails for that PDF segment (pdf.js headless fallback not wired here) |

## Related

- [unibrow format](unibrow-format-migration.md)
- [midi-audio-render.md](midi-audio-render.md)
- ffmpeg sidecar: [ffmpeg-sidecar.md](ffmpeg-sidecar.md)
