# Performance notes

## Lazy-loaded heavy modules

| Module | When loaded |
| --- | --- |
| **Monaco** | First text/diff editor tab (`import()` via `@monaco-editor/loader`) |
| **pdf.js** | First PDF tab (`loadPdfJs()` in `lazyModules.ts`) |
| **three.js** | First 3D/board views that need it (`loadThree()`) |
| **mammoth** | First DOCX preview (`import("mammoth")` in `renderDocx`) |
| **dxf-parser** | First DXF tab (dynamic import in viewer) |

The Vite bundle still contains these packages; lazy `import()` defers parse/compile cost until the viewer opens.

## Cold start measurement

Run on your machine (Linux/macOS):

```bash
./scripts/measure-cold-start.sh
```

The script prints wall time for:

1. `cargo test -p unibrowser-engine` (Rust core smoke)
2. `npm run build` in `crates/unibrowser-desktop` (UI production build)

Use the output in release notes; numbers vary by CPU and cache state. For apples-to-apples comparisons, run twice and use the second run, or clear `target/` and `node_modules/.vite` when measuring true cold builds.

## Tips

- Prefer opening files over browsing huge directories when testing listing performance (`UB_LIST_STRESS=1` for stress tests).
- Large PDFs: see `UNIBROWSER_PROFILE_OPEN=1` in [blitz-status.md](blitz-status.md).
