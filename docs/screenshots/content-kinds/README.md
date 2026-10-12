# Content-kind capability screenshots (1280×800, light)

Capture the **Grayscale preview** toggle from the `image-grayscale-preview` mod in the **Adjust** family (`unibrowser.family.image-adjust`). Regenerate after a release desktop build and Linux deps from [getting-started.md](../../getting-started.md):

```bash
./scripts/capture-content-kinds.sh
```

Uses isolated capture config (`scripts/capture-isolated-config.sh`), `WEBKIT_DISABLE_DMABUF_RENDERER=1`, and `UNIBROWSER_SAFE_MODE=1`.

| File | What the frame shows |
| --- | --- |
| `grayscale-standalone-png-1280-light.png` | `examples/demo-pack/sample.png` open; Command Manager on the **Image** tab with **Adjust** visible. **Grayscale preview** sits after **Colour** in that family. Capability chip **supported** (no emulated note on the control). |
| `grayscale-embedded-docx-1280-light.png` | `examples/demo-pack/sample-rich.docx` with the embedded image node selected (`UNIBROWSER_SELECT_EMBEDDED_IMAGE=1`). Same **Adjust** row; **Grayscale preview** shows **emulated** with reason text: “Preview only; DOCX stores rich text, not pixel filters.” |
| `grayscale-embedded-markdown-1280-light.png` | `examples/demo-pack/sample.md` with the embedded image reference selected (`UNIBROWSER_SELECT_EMBEDDED_IMAGE=1`). **Grayscale preview** **emulated** with: “Preview only; Markdown stores references, not raster edits.” |
