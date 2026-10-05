# OCR T3 sidecar (planned)

Image text extraction will follow the same pattern as [ffmpeg-sidecar.md](ffmpeg-sidecar.md): an **optional GPL/LGPL-compatible sidecar** (likely **Tesseract**) invoked over CLI or a small JSON line protocol. The Apache-licensed core will not link OCR libraries directly.

## Current status (blitz)

- Image viewer shows a disabled **Extract text (OCR)** control with a tooltip pointing here.
- No sidecar binary is bundled yet; no `UNIBROWSER_OCR_*` env vars are wired.

## Planned shape

| Piece | Role |
| --- | --- |
| `unibrowser-ocr-sidecar` (future) | Accept `{ "path": "...", "lang": "eng" }`, return plain text + optional bounding boxes |
| Desktop command | `ocr_image_path` → calls sidecar when `UNIBROWSER_OCR_SIDECAR_BIN` is set |
| UI | Enable button when probe succeeds; show text in a read-only panel with copy |

## Packaging notes

- Ship Tesseract data files (`tessdata`) beside the sidecar or document system package install.
- Respect `path_security` on input paths; cap output size like document read limits.

## References

- Media sidecar sketch: `crates/unibrowser-media` / `unibrowser-media-sidecar`
- Licensing: [ADR 0007](adr/0007-licensing.md)
