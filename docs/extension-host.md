# Extension host (VSIX execution)

VSIX packages install metadata today; JavaScript execution is still a stub ([ADR 0018](adr/0018-extension-host-stub.md), [ADR 0014](adr/0014-vsix-compatibility-layer.md)).

## What works (V1 + stub path)

| Layer | Behaviour |
| --- | --- |
| Install | `.vsix` → `vsix-extensions/` under config; `install.json` + extracted `extension/` tree |
| `contributes.commands` | Registered in `CommandRegistry`; titles appear in **palette** and **Command Manager → Extensions** tab |
| `contributes.keybindings` | Merged into `keybindings.json` after defaults (user overrides win) |
| `activationEvents` | Parsed from `package.json`; **`onCommand:&lt;id&gt;`** is evaluated when that command runs (stub activation, no JS) |
| Run | `execute_command` / palette / chords call **`unibrowser-extension-host` stub** — returns a message, no `main` JS |
| Diagnostics | `unibrowser://diagnostics` lists activated extensions and host stub version |

## Remaining gaps (full host)

- No `vscode` API surface (`executeCommand`, workspace, window, languages).
- No full activation scheduler (`onLanguage:`, workspaceContains, `*` startup, etc.) — only **`onCommand:`** matching at stub execute time.
- No extension `main` module load (Node, worker, or WASM T2).
- No webview or language-server contributions.
- Command enablement still assumes document handlers for non-VSIX commands; VSIX ids bypass document handlers.
- Sandboxed storage, secrets, and marketplace trust/signing beyond install-time metadata.

## Crate map

| Crate | Role |
| --- | --- |
| `unibrowser-vsix` | Zip install, `contributions.json`, registry registration |
| `unibrowser-extension-host` | `HostStatus`, stub execute message |
| `unibrowser-engine` | `CommandRegistry`, palette search |
| Desktop `state.rs` | `refresh_extension_catalog`, `vsix_host_commands` set |

## Testing a VSIX locally

1. Build a minimal `extension/package.json` with `contributes.commands` and optional `activationEvents`: `["onCommand:demo.hello"]`.
2. Pack as `.vsix`, install from Mod portal.
3. Open Command Manager → **Extensions** tab or palette; run the command — expect a stub toast, not JS.

## Related

- [extension-host crate](../crates/unibrowser-extension-host/src/lib.rs)
- [Plugin SDK](plugin-sdk/README.md) (native mods, not VSIX JS)
