# ADR 0022: Proofing as mods

- **Status:** Accepted
- **Date:** 2026-10-06

## Context

Offline squiggles (spelling, grammar, and format-specific lints) should behave like other Unibrowser capabilities: **removable mods** on top of a generic diagnostics host (ADR 0021). Grandma should get en-NZ spelling and Harper grammar out of the box without a separate “proofing app”.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — bundled mods are removable; squiggles only |
| Never blocks save or work | Yes — proofing mods do not participate in save validation |
| Local-first | Yes — Hunspell + optional `harper-ls`; no LanguageTool cloud |
| Predictable across formats | Partial — text strong; DOCX/web/SVG adapters still landing |
| Simple durable mechanism | Yes — register checker or LSP via manifest |
| Mod not core | Yes — explicit Factorio split from `unibrowser-diagnostics` |

## Decision

1. **Factorio rule:** `unibrowser-diagnostics` is the engine; **proofing is mods** that register checkers or LSP servers.
2. **Bundled preinstall (removable):**
   - `mods/spell-en-nz` — Hunspell via `spellbook`, en-NZ default (British `.dic` + locale preference)
   - `mods/grammar-harper` — `harper-ls` on stdio (`info.json` `contributes.languageServer`)
3. **Other language packs** ship as separate dictionary mods (e.g. future `spell-en-us`), not hard-coded in the host.
4. **Structured / code formats** use existing language servers (XML, JSON, YAML, Rust, …) launched through the host LSP client when a mod or VSIX contributes `languageServer` metadata.
5. **VSIX / Open VSX** extensions that bundle or download an LSP (vscode-languageclient pattern, e.g. LTeX, cSpell where feasible): run the server through our client; record Works/Partial/Unsupported on the compatibility badge.
6. **Example native mods** (proving locators):
   - `mods/dxf-checker` — TEXT/MTEXT spelling hints + layer-0 text lint → `DxfEntity` markers on the wireframe
   - `mods/unibrow-checker` — title-card spelling + black-frame stub → `TimelineRange` markers
7. **Rules unchanged from earlier proofing slice:**
   - Web adapter skips `type=password`, `autocomplete=cc-*` / `one-time-code`, and `spellcheck=false`
   - Code surfaces check comments and string literals by default
   - Squiggles only; no covering popups
   - Dictionary precedence: workspace add/ignore **over** global **over** bundled Hunspell

## Non-goals

- Hunspell `.dic` import UI, cSpell wordlist import (nice-to-have)
- LanguageTool cloud or server-by-default
- Replacing browser experimental spellcheck mirror until adapters are stable

## Consequences

- Desktop `proofing_check` builds a `DiagnosticsHost`, loads the **spell-en-nz** checker, and applies dictionary snapshots from config / `.unibrowser/`.
- Harper grammar activates when `harper-ls` is on `PATH` and the grammar mod is enabled (LSP path; no `harper-core` in the hot path).
- Legacy crate `unibrowser-proofing` may remain for transitional tests but is not the architectural centre.

## References

- [ADR 0021](0021-diagnostics-host-lsp.md)
- [proofing-cross-format.md](../proofing-cross-format.md)
