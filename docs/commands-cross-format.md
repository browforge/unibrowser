# Cross-format commands

One stable command id per user action; bundled mods register handlers per `backend_id`. The desktop host intercepts HTTP tabs (user-style font, page zoom) and `.unibrow` project tabs (timeline title cards).

Legend:

| State | Meaning |
| --- | --- |
| **Live** | Enabled; execute updates viewer/backend state |
| **Grey** | Visible in Command Manager; `disabled_reason` explains why |

## Coverage matrix (headline commands)

| Format / backend | `unibrowser.font.size.increase` | `unibrowser.find.inDocument` | `unibrowser.page.zoom.fit` | `unibrowser.export.printPdf` |
| --- | --- | --- | --- | --- |
| Plain text / code (`base-text`, `base-js`) | Live | Live | Grey (use font size) | Live (Markdown HTML path for `.md`) |
| Live web (`base-web` + host) | Live (per-origin user-style) | Live (webview find) | Live (reset origin zoom to 100%) | Grey |
| SVG (`base-svg`) | Live when `<text>` / `<tspan>` | Live when text elements | Live | Grey |
| DOCX (`base-docx`) | Live (reader preview pt) | Live | Grey | Live |
| EPUB (`base-epub`) | Live (reader font pt) | Live | Live (reader scale) | Grey |
| DXF (`base-dxf`) | Live when TEXT/MTEXT | Live when TEXT/MTEXT | Live | Grey |
| PNG / images (`base-png`) | Live when `tEXt` key `unibrowser_text` | Live when annotation + selection | Live | Grey |
| PDF (`base-pdf`) | Grey (no reflow) | Grey | Live (fit scale 100%) | Grey |
| CSV (`base-csv`) | Grey | Grey | Grey | Grey |
| `.unibrow` project (host) | Live (title clip `title_font_pt` only) | Grey | Grey | Grey |
| Hex (`base-hex`) | Live (not in Bear headline set) | Grey | Grey | Grey |

Tests: `crates/unibrowser-mod-host/tests/cross_format_commands.rs` plus per-mod unit tests under `mods/base-*`.

## `unibrowser.font.size.increase` / `.decrease`

| Context | Live when | Effect |
| --- | --- | --- |
| **Text / Monaco** | Text or code tab | `font_size_pt`; Monaco refresh |
| **SVG** | SVG contains text elements | Rewrites `font-size` on `<text>` / `<tspan>`; saves with file |
| **DOCX** | `.docx` tab | Mammoth HTML preview size from `reader_font_pt` |
| **EPUB** | `.epub` tab | Spine `srcdoc` body font from `reader_font_pt` |
| **HTTP** | Web tab | Per-origin user-style (`#unibrowser-user-style`) |
| **DXF** | Drawing has TEXT/MTEXT | Scales effective text height for wireframe preview |
| **PNG** | `unibrowser_text` PNG metadata (`font_pt=…;label=…`) | Text annotation layer font pt |
| **`.unibrow`** | Project tab with title clips | `title_font_pt` on timeline (encode via resvg) |
| **PDF** | Grey | Raster page; use zoom or select PDF text layer (limited) |
| **PNG (plain)** | Grey | No `unibrowser_text` annotation |
| **CSV** | Grey | Table view |

## `unibrowser.find.inDocument` (Ctrl+F)

| Context | Live when | Effect |
| --- | --- | --- |
| Text, JS, DOCX, EPUB, web | Handler registered | Shell find bar / web find / iframe `window.find` |
| SVG, DXF | Text entities present | Document find surface |
| PNG | Text annotation selected | Find bar (label text) |
| PDF, CSV, binary | Grey | Reason on handler |

## `unibrowser.page.zoom.fit` (Ctrl+0)

| Context | Live when | Effect |
| --- | --- | --- |
| Web | HTTP tab | Reset per-origin page zoom to 100% |
| PDF | `.pdf` | `render_scale` = 1.0 |
| PNG / SVG | Image/vector tab | Fit preview to viewport |
| DXF | `.dxf` | Reset CAD preview zoom |
| EPUB | `.epub` | Reader scale 100% |
| Text / DOCX font views | Grey | Use font-size commands |

## `unibrowser.export.printPdf`

| Context | Live when | Effect |
| --- | --- | --- |
| Text (Markdown) | `.md` / `.markdown` | Print HTML preview to PDF |
| DOCX | `.docx` | Print mammoth HTML preview |
| Other | Grey | No printable HTML surface |

## Related

| Command | Notes |
| --- | --- |
| `unibrowser.page.zoom.increase` / `.decrease` | Web CSS zoom, PDF scale, image `preview_zoom` |
| `unibrowser.text.save` | Text and SVG when writable |

## Docs

- [ui-commands.md](ui-commands.md)
- [keybindings.md](keybindings.md)
- [mod-api-stable.md](mod-api-stable.md)
