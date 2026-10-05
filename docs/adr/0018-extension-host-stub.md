# ADR 0018: Extension host stub (VSIX execution)

## Status

Accepted (stub phase)

## Context

[ADR 0014](0014-vsix-compatibility-layer.md) V1 installs `.vsix` packages and imports `contributes.commands` / `keybindings` metadata. Unibrowser registers command titles in the palette but cannot run extension JavaScript without an **extension host**.

## Decision

1. **Crate `unibrowser-extension-host`** exposes `HostStatus` and version only — no VS Code API shim yet.
2. **Palette** marks VSIX-imported command ids as disabled with reason `Needs extension host (ADR 0018)`.
3. **Keybindings** from enabled VSIX merge into `KeybindingMap` after defaults; user `keybindings.json` overrides win.
4. **Future host** (not this slice): isolated worker or WASM (T2), message bridge for `executeCommand`, activation events, and `package.json` `main` entry.

## Consequences

- Install/disable/uninstall of VSIX is safe without JS execution.
- Chords bound to VSIX commands resolve via `dispatch_key_chord` but `execute_command` still has no handler until host lands.

## Related

- [0014 VSIX compatibility](0014-vsix-compatibility-layer.md)
- `crates/unibrowser-extension-host`
