# CMD-fix evidence (command system + Quiet Utility chrome)

Regenerate: `./scripts/capture-cmd-fix.sh` (requires `xvfb-run`, ImageMagick `import`, `WEBKIT_DISABLE_DMABUF_RENDERER=1`, `WEBKIT_DISABLE_COMPOSITING_MODE=1`).

## What changed vs `cmd-batch/`

1. **Command Manager** — light-theme stroke/colour on CM controls; capture gate requires ≥6 visible, contrasting `.ui-command` entries.
2. **Monaco** — built-in `vs` / `vs-dark` themes use hex Quiet Utility colours from Rust; theme is applied before opening launch paths; `doc-capture-mode` uses a DOM text mirror so WebKitGTK snapshots show readable editor text (canvas paint is still validated in diagnostics).
3. **Chrome** — bookmark bar hidden for open documents; title breadcrumb for local paths; empty Output panel collapsed; markdown toolbar hidden during capture.
4. **Captures** — `text-family-plain-txt.png`, `text-family-markdown.png`, `text-family-rich-docx.png`, plus dark/pdf/image CM variants.

## Assertions at capture

- Light demos: `data-theme=light`, `--ub-base=#f1f0ec`.
- CM: `validateCommandManagerForCapture()` (≥6 visible, ≥6 contrast).
- Text tabs: mirror or Monaco diagnostics logged as `[doc-capture] diagnostics …` on stderr.
- Rust: PNG editor-band mean luma ≥ 0.32 for `.txt`/`.md` launch paths.
