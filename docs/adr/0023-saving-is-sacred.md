# ADR 0023: Saving is sacred

- **Status:** Accepted
- **Date:** 2026-10-06

## Context

Bear’s product principles: FOSS users and everyday users want the same thing — easy, predictable, local-first software. Nothing may hold the user’s work hostage behind validation, diagnostics, or “fix this first” gates. Cloud accounts and hosted autosave are out of scope.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — user owns files; autosave is sidecar only |
| Never blocks save or work | Yes — core of this ADR |
| Local-first | Yes — rotating slots on disk, no cloud |
| Predictable across formats | Yes — same save posture for text, projects, settings |
| Simple durable mechanism | Yes — temp + rename, timer, slot files |
| Mod not core | N/A — fundamental host IO policy |

## Decision

### 1. Nothing blocks Save

- **Save**, **Save As**, and **autosave** always persist the user’s current state as-is, non-destructively.
- Diagnostics (spelling, grammar, LSP, format lints), invalid or incomplete project state (empty timeline, missing video track, broken SVG, code with errors), and export-time quality checks **must not** prevent saving.
- **Export / render** may refuse or produce a stub when prerequisites are missing, but only with a clear inline explanation and an always-available path to **save the project** (never a blocking modal).
- User files are written with **atomic** IO (temp + rename) where the host controls the path.

### 2. Local autosave, no cloud

- Settings → **Autosave**: Off / On, interval presets (1, 2, 5, 10, 15, 30 minutes; default 5), and **keep N** rotating slots.
- Plain local timer in the shell; slots live in a sidecar directory beside the file (`.{name}.ubautosave/`) or under the app config dir for untitled buffers.
- Autosave **never** overwrites the user’s file unless they explicitly choose that (manual Save).
- No focus steal, no popup on autosave; status bar shows last autosave time.
- Crash recovery reuses the newest slot via existing session-restore / recovery flows (non-modal).

## Consequences

- `write_manifest` uses permissive save validation; strict checks remain for **open/import**.
- DOCX save reports warnings but writes supported body content instead of hard-failing.
- Document autosave writes rotating JSON slots, not `TEXT_SAVE` over the original path.
- Philosophy and UX guardrails are summarised in [principles.md](../principles.md).

## References

- [ADR 0021](0021-diagnostics-host-lsp.md), [ADR 0022](0022-proofing-as-mods.md) (diagnostics never gate save)
- [session-restore.md](../session-restore.md)
