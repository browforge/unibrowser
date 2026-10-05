# ADR 0007: Licensing

- **Status:** Accepted
- **Date:** 2026-10-05

## Context

The core must stay permissive for commercial CAD/CAM plugins. AGPL/GPL engines (MuPDF, ONLYOFFICE CE, Inkscape, libdxfrw) must not infect the host binary.

## Decision

- **Unibrowser engine, mod API, and first-party base mods:** **Apache-2.0** (see root `LICENSE`).
- **Optional alternative** for contributors who want file-level copyleft on core only: MPL-2.0 (not dual-licensed in skeleton; Apache chosen for patent grant and Tauri alignment).
- **Third-party mods** may use any licence; UI must show licence label for AGPL/GPL plugins.
- **No GPL/AGPL code linked into** `unibrowser-engine` or `unibrowser-mod-api`.
- **DCO** recommended for contributions (`.github` template can be added in Phase 1).

## Consequences

- ffmpeg in core must be **LGPL** build; pdf.js over MuPDF for default PDF view.
- GPL tools run as **T3 sidecar** mods only.
