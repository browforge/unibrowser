# Saving is sacred — audit checklist (ADR 0023)

Use this when adding save paths, validators, or export gates.

| Rule | Status in blitz |
| --- | --- |
| Save / Save As never blocked by spellcheck, grammar, or diagnostics | **Pass** — proofing is advisory only |
| Save / Save As never blocked by missing optional mod APIs | **Pass** |
| `.unibrow` `project.json` save is permissive (warn, do not refuse) | **Pass** |
| DOCX complex save refuses destructive overwrite; HTML sidecar + lite save path | **Pass** (warn strings) |
| Autosave writes rotating sidecar slots only | **Pass** — `.{name}.ubautosave/slot-*` |
| Autosave never overwrites the user's primary file | **Pass** |
| Untitled buffers: picker only on explicit Save; path under config autosave | **Pass** |
| Close tab: best-effort flush to autosave slot (force close) | **Pass** (text/SVG/JS backends) |
| Restore from autosave slot | **Shipped** — palette **Restore from autosave…** + Settings restore latest |
| Close dirty tab | **Shipped** — no modal; autosave flush; **Closed — Undo** toast reopens tab with slot content |
| Export / encode failures do not block document save | **Pass** |

Tests: `crates/unibrowser-fs`, `crates/unibrowser-desktop/src-tauri/src/autosave.rs`, project save tests in `unibrowser-project`.
