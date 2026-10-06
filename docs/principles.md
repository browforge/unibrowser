# Unibrowser principles

FOSS users and grandma want the same thing: easy, predictable, consistent software that does what they want, when they want it.

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
