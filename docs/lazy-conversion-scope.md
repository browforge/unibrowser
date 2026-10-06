# Lazy conversion scope (shipped)

Per [ADR 0011](adr/0011-lazy-conversion-native-until-encode.md), **native references stay in `project.json` until encode/export**. The blitz ships this for `.unibrow` timeline **H.264 export**:

| Clip / asset | Stored native | Rasterised at export |
| --- | --- | --- |
| MP4 video | File path | ffmpeg trim/copy |
| PDF page | PDF + page index | **pdftoppm** |
| SVG still | `.svg` path | **resvg** |
| Title card | Text + `title_font_pt` | resvg SVG |
| MIDI | `.mid` | **fluidsynth** → WAV mix |
| Audio file | Path | ffmpeg mix |

Preview uses proxies (pdf.js, thumbs, block UI); export uses the table above. Gaps (e.g. headless PDF without pdftoppm) are listed in [timeline-encode-intents.md](timeline-encode-intents.md).
