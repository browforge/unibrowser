# Third-party components

Short attributions for components users commonly ask about. Full Rust crate licences: generate with `./scripts/generate-notice.sh` (requires `cargo install cargo-license`).

## Bundled in the desktop UI (npm)

| Component | Licence | Use in Unibrowser |
| --- | --- | --- |
| [Monaco Editor](https://github.com/microsoft/monaco-editor) | MIT | Text/code editor (`monaco-editor`, `@monaco-editor/loader`) |
| [pdf.js](https://github.com/mozilla/pdf.js) | Apache-2.0 | PDF viewer (`pdfjs-dist`) |
| [mammoth.js](https://github.com/mwilliamson/mammoth.js) | BSD-2-Clause | DOCX → HTML preview (lazy import) |
| [marked](https://github.com/markedjs/marked) | MIT | Markdown preview in Monaco split view |
| [Mermaid](https://github.com/mermaid-js/mermaid) | MIT | Diagram rendering in Markdown preview (lazy `import()`; bundles transitive libs such as [cytoscape](https://github.com/cytoscape/cytoscape.js) MIT and [KaTeX](https://github.com/KaTeX/KaTeX) MIT) |
| [Vite](https://github.com/vitejs/vite) / [Rolldown](https://github.com/rolldown/rolldown) | MIT | Front-end build (dev/build only) |

Monaco and pdf.js retain their own copyright notices in `node_modules` and in production bundles.

## Desktop shell

| Component | Licence | Notes |
| --- | --- | --- |
| [Tauri 2](https://github.com/tauri-apps/tauri) | MIT **or** Apache-2.0 (per crate) | Native window, IPC, plugins |
| [WebKitGTK](https://webkitgtk.org/) / platform WebView | LGPL-2.1+ (Linux) | Embedded web content; system library on Linux |

## Optional / sidecar (not linked into Apache core)

| Component | Licence | Notes |
| --- | --- | --- |
| [ffmpeg](https://ffmpeg.org/) / [ffprobe](https://ffmpeg.org/ffprobe.html) | LGPL-2.1+ (default build) | Timeline H.264 export, thumbnails, audio waveform PCM when installed on PATH; see [ffmpeg-sidecar.md](../ffmpeg-sidecar.md) |
| [resvg](https://github.com/RazrFalcon/resvg) / [usvg](https://github.com/RazrFalcon/usvg) / [tiny-skia](https://github.com/RazrFalcon/tiny-skia) | MIT / MIT / BSD-3-Clause | Lazy SVG and title-card raster at timeline H.264 encode (`unibrowser-media`) |
| [FluidSynth](https://www.fluidsynth.org/) | LGPL-2.1+ | Optional MIDI → WAV at timeline export (sidecar on PATH) |
| [pdftoppm](https://poppler.freedesktop.org/) (Poppler) | GPL-2.0 | Optional PDF page raster at timeline export (sidecar on PATH) |
| [wkhtmltopdf](https://wkhtmltopdf.org/) | LGPL-3.0 | Optional PDF export fallback on Linux |
| Chromium (system) | BSD-style + others | Optional headless print-to-PDF |

Users install ffmpeg/Chromium themselves; Unibrowser invokes them as separate processes ([ADR 0007](../adr/0007-ffmpeg-sidecar.md)).

## Rust crates (workspace)

Workspace crates (`unibrowser-engine`, `unibrowser-desktop`, bundled `mods/*`, etc.) are Apache-2.0 unless a crate `Cargo.toml` states otherwise. Transitive dependencies are mostly MIT / Apache-2.0 / BSD; use `cargo license` for a shipping manifest.

## VSIX / Open VSX extensions

Third-party VSIX packages are **user-installed** and carry their own licences. Unibrowser stores metadata only until a full extension host ships ([extension-host.md](../extension-host.md)).
