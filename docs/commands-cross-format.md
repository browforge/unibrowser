# Cross-format commands (font size demo)

One command id drives the blitz **Increase font size** demo across viewers: `unibrowser.font.size.increase` (and `unibrowser.font.size.decrease`). Handlers live on bundled mods; the desktop host adds special cases for HTTP tabs (user-style injection).

Legend for Command Manager and palette:

| State | Meaning |
| --- | --- |
| **Live** | Enabled for the active tab; execute updates viewer state |
| **Greyed** | Slot visible; `disabled_reason` explains why (wrong tab or unsupported format) |

## `unibrowser.font.size.increase` / `.decrease`

| Context | Live when | Effect |
| --- | --- | --- |
| **Text / Monaco** (`base-text`) | Plain-text or code tab | `font_size_pt` on backend; Monaco `fontSize` on refresh |
| **SVG** (`base-svg`) | `.svg` file tab | Preview scale via `reader_font_pt` in viewer; backend `font_size_pt` |
| **DOCX** (`base-docx`) | `.docx` tab | Mammoth HTML preview `font-size` from `reader_font_pt` |
| **EPUB** (`epub` stub backend) | `.epub` tab | Spine chapter `srcdoc` body font-size from `reader_font_pt` |
| **HTTP / HTTPS** (`base-web`) | Web tab in app shell | Per-origin user-style injection (`adjust_page_font_for_url`); not persisted in page DOM |
| **PDF** (`base-pdf`) | Greyed | Raster/vector page; no reflow font (handler registered, enablement off) |
| **PNG / images** (`base-png`) | Greyed | Bitmap preview only |
| **Video / audio** | Greyed | No text surface |
| **DXF proving mod** | Greyed | CAD tools stubbed |
| **Files / home** | Greyed | No document session |
| **`.unibrow` project** | Open project tab | **Title clips only:** adjusts `title_font_pt` on timeline title cards (used at H.264 encode via resvg); no effect when the timeline has no title clips |

## `unibrowser.find.inDocument` (Ctrl+F)

| Context | Live when | Effect |
| --- | --- | --- |
| **Text / Monaco** | Text tab | Find bar; Monaco find or markdown preview search |
| **DOCX** | `.docx` tab | Find bar; Mammoth HTML highlight search |
| **EPUB** | `.epub` tab | Find bar; `window.find` in chapter iframe |
| **PDF / image / video / files** | Greyed | No in-document find handler |

## `unibrowser.page.zoom.increase` / `.decrease`

| Context | Live when | Effect |
| --- | --- | --- |
| **HTTP / HTTPS** | Web tab | CSS `zoom` per origin (`web-zoom-prefs.json`), not font size |
| **PDF** | `.pdf` tab | `render_scale` on raster preview |
| **Images** | PNG/JPEG/GIF/WebP | `preview_zoom` on image viewer |
| **Text / DOCX font** | Greyed | Use font-size commands instead |

## Related commands

| Command | Notes |
| --- | --- |
| `unibrowser.text.save` | Live on text and SVG when path is writable |
| `mod.hello` | Live when a user **cdylib** mod is loaded (`dynamic-mods`); stub message if palette runs without load |
| VSIX `demo.*` | Live on Extensions tab context per stub host |

## Docs

- UI model: [ui-commands.md](ui-commands.md)
- Keybindings: [keybindings.md](keybindings.md)
- Mod API ids: [mod-api-stable.md](mod-api-stable.md)
