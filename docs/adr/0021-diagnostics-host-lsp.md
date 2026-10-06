# ADR 0021: Diagnostics host + LSP

- **Status:** Accepted (supersedes shared proofing-as-core framing)
- **Date:** 2026-10-06

## Context

Bear wants one **diagnostics** story across text, CAD, video, PDF, and web surfaces: a problem list, severities, quick-fixes, Next/Previous problem, and squiggles or markers without modal popups. Spell and grammar are important but must not be baked into the engine as the only checkers. Language servers and native checkers should plug in like Factorio mods. Nothing leaves the machine by default.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — problems are hints; user chooses what to fix |
| Never blocks save or work | Yes — diagnostics never gate Save (ADR 0023) |
| Local-first | Yes — LSP children and Hunspell run on device; no cloud checker |
| Predictable across formats | Partial — one host model; not every surface wired yet |
| Simple durable mechanism | Yes — problem list + locators; rendering in viewers |
| Mod not core | Yes — spell/grammar/format checkers are mods (ADR 0022) |

## Decision

1. **Engine crate** `crates/unibrowser-diagnostics` (`unibrowser-diagnostics`) is a generic **diagnostics host**. It owns:
   - `Problem` list (severity, message, suggestions, code actions)
   - **Locators** (typed ranges, not only LSP line/column): `TextRange`, `DxfEntity`, `TimelineRange`, `ImageRegion`, `PdfRegion`, `DomNode`
   - **Dictionary store** (global + workspace plain-text add/ignore lists)
   - **Native** `NativeDiagnosticsChecker` trait for in-process mods
   - **LSP client** (stdio JSON-RPC): `initialize`, `textDocument/didOpen`, `didChange`, `textDocument/publishDiagnostics`; `codeAction` / completion / hover are nice-to-have later
   - Child language servers launched from mod manifests; **env-scrubbed** spawn (`env_clear`, `PATH` only), same posture as the Node extension host
   - Host API: register checkers, `check_text_document`, format-specific hooks (`check_dxf`, `check_unibrow`), next/previous problem cursor
2. **Text ↔ LSP:** adapters map UTF-8 byte ranges in the host to Monaco; LSP `publishDiagnostics` maps to `TextRange` line/column fields.
3. **Non-text:** checkers emit native locators (or LSP extension notifications carrying a Locator later). DXF and `.unibrow` example mods prove wireframe and timeline markers.
4. **Rendering** stays in surfaces (Monaco markers, timeline ticks, DXF overlay): the host does not paint UI.
5. **Commands** remain `unibrowser.proofing.toggle`, `.nextProblem`, `.previousProblem` (palette + F8); the shell routes them through the host problem list.

## Non-goals (this slice)

- Full Problems panel UI (host data model only)
- Debounced incremental scheduling in the desktop shell (host ready; adapters debounce)
- Every VSIX language-server pack wired end-to-end (compatibility badge + manifest pattern documented in ADR 0022)

## Consequences

- `unibrowser-proofing` is legacy; new work lands in `unibrowser-diagnostics` + mods.
- Grammar via `harper-core` in-process is replaced by the **grammar-harper** LSP mod when `harper-ls` is available.
- Coverage tables: [proofing-cross-format.md](../proofing-cross-format.md), [vision-matrix.md](../vision-matrix.md).

## References

- [ADR 0022](0022-proofing-as-mods.md) (spell/grammar as mods)
- [ADR 0014](0014-vsix-compatibility-layer.md) (VSIX language client pattern)
- [privacy.md](../privacy.md)
