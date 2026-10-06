# Unibrowser principles

FOSS users and grandma want the same thing: easy, predictable, consistent software that does what they want, when they want it.

**Standing instruction (Bear):** this file is the project's core philosophy and the **tiebreaker for every design decision**. When ADRs, UX, or implementation disagree, principles win unless an ADR explicitly records a conscious trade-off (with a Principles check).

## Decision checklist

Before shipping or extending a feature, answer **yes** unless you document why not in the ADR or PR:

1. **Does it trust the user?** (No nanny gates, dark patterns, or hidden defaults that override explicit choice.)
2. **Does it never block saving or work?** (Save / Save As / autosave always persist current state; diagnostics and validation inform, they do not refuse save.)
3. **Is it local-first with no account or cloud required?** (Network only when the user clearly opts in or the action is obviously remote, e.g. opening a URL.)
4. **Is it predictable and consistent across formats?** (Same command id and mental model where the problem is the same; grey + reason when a surface cannot comply.)
5. **Is it the simple, durable mechanism** (RSS / IRC / kei-truck) **over the clever hosted one?** (Files, timers, sidecars; not SaaS-shaped glue.)
6. **Is it a mod rather than core**, unless it is fundamental infrastructure? (Engine stays small; spellcheck, grammar, format servers, and checkers are mods.)

## Local-first

- No accounts, no cloud requirement for core editing.
- Prefer simple durable mechanisms (local files, rotating autosave slots, RSS/IRC-style reliability) over clever hosted services.
- Modern UX niceties without enshittification: squiggles and diagnostics inform; they do not gate Save.

## Saving is sacred

See [ADR 0023](adr/0023-saving-is-sacred.md).

- Save / Save As / autosave always succeed for the current editor state.
- Export may explain what is missing; saving the project remains available.

## Mods on a small engine

See [ADR 0021](adr/0021-diagnostics-host-lsp.md) and [ADR 0022](adr/0022-proofing-as-mods.md).

- Generic diagnostics host + removable mods (spelling, grammar LSP, format checkers).
- Bundled defaults (e.g. en-NZ spelling, Harper) ship for out-of-the-box squiggles but stay uninstallable mods.

## Privacy

Nothing leaves the machine by default. See [privacy.md](privacy.md).
