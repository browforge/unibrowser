# ADR 0026: AI assistant as a mod

- **Status:** Accepted (proving skeleton)
- **Date:** 2026-10-06

## Context

Bear wants an **OpenAI-compatible** chat sidebar (any base URL: Ollama, llama.cpp, LM Studio). No default cloud endpoint; API key stored locally. The assistant must use the same **command registry** and **text/raster projections** (ADR 0025) so it works on web, DOCX, SVG, images, timeline, and code — **learn once** for humans and models.

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — nothing sent until user invokes; grants per scope |
| Never blocks save or work | Yes — writes use undo stack; never gate save |
| Local-first | Yes — local endpoints only unless user configures URL |
| Predictable across formats | Yes — projections + palette commands as tools |
| Simple durable mechanism | Yes — HTTP chat + permission table |
| Mod not core | Yes — shipped as removable mod; host exposes projections only |

## Decision

1. **Mod** `ai-assistant-proving` / crate `unibrowser-ai-assistant`: sidebar UX later; proving crate now.
2. **Client**: OpenAI-compatible `POST /v1/chat/completions`; **no default base URL**; key in local prefs file.
3. **Tools**: registered command ids (palette parity).
4. **Permissions** (per grant): **Read** / **Write** / **Ask-before-acting**. Scopes: global, workspace, file type, site/origin. Visible indicator when AI can read active tab.
5. **Write** path: normal undo stack; **Ask** path: `PendingAction` until user approves.
6. **Never** expose password / `autocomplete=cc-*` fields in projections.
7. **Proving tests**: mock server; read projection with Read grant; Write denied without grant; Ask creates pending approval.

## Consequences

- Full sidebar UI and VSIX bridge are follow-ups.
- Cloud URLs are user opt-in only.

## References

- [ADR 0025](0025-mod-apis-and-dependencies.md), [principles.md](../principles.md)
- `crates/unibrowser-ai-assistant`
