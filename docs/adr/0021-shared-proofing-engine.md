# ADR 0021: Shared spelling and grammar proofing engine

- **Status:** Accepted
- **Date:** 2026-10-06

## Context

Bear wants one offline proofing engine everywhere (same “one command id / one engine” story as cross-format font). Users edit plain text, Markdown, DOCX preview, SVG text, `.unibrow` titles, code comments, and web form fields. Nothing may leave the machine; password and payment fields must never be checked. UX is squiggles only (no popups over content), with palette navigation between problems.

## Decision

1. **Core crate** `crates/unibrowser-proofing` exposes `ProofingEngine::check(text, CheckContext) -> Vec<Diagnostic>` with `{ start, end, kind, message, suggestions }`.
2. **Spelling** uses Hunspell-compatible dictionaries via pure-Rust [`spellbook`](https://github.com/helix-editor/spellbook) (MPL-2.0). Ship bundled FOSS `en.aff` + locale `.dic` files; default language **en-NZ** (British dictionary + NZ locale preference in UI).
3. **Grammar** uses [`harper-core`](https://github.com/Automattic/harper) (Apache-2.0) locally. Optional **LanguageTool** (or other) engines may ship as separate mods later; never as a cloud default.
4. **Engine swapping (Factorio rule):** `ProofingEngine` is constructed from bundled engines today; future mods register alternate `SpellingEngine` / `GrammarEngine` traits without forking surfaces.
5. **Dictionaries:** plain-text word lists in the user config dir (global) and workspace `.unibrowser/proofing-words.txt` (workspace). Precedence: workspace add/ignore **over** global **over** bundled Hunspell. Context menu actions map to add/ignore scopes; Settings lists and removes entries.
6. **Surfaces** are thin adapters (TypeScript or host commands) that debounce input, call `proofing_check`, and render Monaco markers or DOM overlays. Code surfaces pass `check_ranges` from a comment/string lexer (Monaco tokens later; tree-sitter optional).
7. **Web** fields: injected script classifies `input` / `textarea` / `contenteditable`; skips `spellcheck=false`, `type=password`, `autocomplete=cc-*` / `one-time-code`. Classification logic is shared with Rust (`web_fields` module) for tests.
8. **Commands:** `unibrowser.proofing.toggle`, `.nextProblem`, `.previousProblem` (global + per-surface enablement like other cross-format commands).

## Non-goals (this slice)

- LanguageTool server mod (documented hook only).
- Hunspell `.dic` / cSpell import UI (nice-to-have).
- Full tree-sitter coverage for all languages.
- Replacing browser experimental spellcheck mirror until proofing is stable (mirror remains behind Advanced).

## Consequences

- First `harper-core` build increases compile time and dependency weight; acceptable for offline grammar.
- Diagnostics use UTF-8 byte offsets in the engine API; Monaco adapters convert to line/column.
- Coverage matrix lives in [proofing-cross-format.md](../proofing-cross-format.md); vision matrix tracks honest surface status.

## References

- [commands-cross-format.md](../commands-cross-format.md) (pattern for coverage tables)
- [privacy.md](../privacy.md) (local-only processing)
