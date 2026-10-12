# Capability surface map (SKEL-04)

Host routing uses `unibrowser.capability.*` ids. Default providers are declared in each mod `info.json` under `capability_provides`.

| Capability | Default mod | Desktop routing |
| --- | --- | --- |
| `unibrowser.capability.open_text` | `base-text` | `capability_router::open_via_provider_mod` → `documentViewer` / Monaco (`textEditor.ts`) |
| `unibrowser.capability.render_pdf` | `base-pdf` | `documentViewer.ts` PDF shell |
| `unibrowser.capability.render_image` | `base-png` | `imageRasterViewer.ts` |
| `unibrowser.capability.render_svg` | `base-svg` | `documentViewer.ts` SVG path |
| `unibrowser.capability.open_docx` | `base-docx` | `documentViewer.ts` DOCX preview |
| `unibrowser.capability.open_dxf` | `base-dxf` | `documentViewer.ts` DXF canvas |
| `unibrowser.capability.open_video` | `base-video` | `documentViewer.ts` media lane |
| `unibrowser.capability.open_audio` | `base-audio` | `documentViewer.ts` media lane |
| `unibrowser.capability.open_epub` | `base-epub` | `epub_preview.rs` + viewer |
| `unibrowser.capability.open_csv` | `base-csv` | `csv_preview.rs` |
| `unibrowser.capability.open_script` | `base-js` | `textEditor.ts` with script hints |
| `unibrowser.capability.project_surface` | `project_backend` | `.unibrow` workshop (still core container IO) |
| `unibrowser.capability.browse_files` | `base-files` | `filesPanel.ts` directory tabs |

Plain-text open is the first format routed only through the provider registry in `state::open_file_document_with_mode`. Other formats use the same router with legacy fallback until follow-up slices land.
