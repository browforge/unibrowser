# Extension host (VSIX execution)

VSIX packages install metadata and (optionally) run extension JavaScript through an **opt-in Node sidecar** ([ADR 0018](adr/0018-extension-host-stub.md), [ADR 0014](adr/0014-vsix-compatibility-layer.md)).

## What works

| Layer | Behaviour |
| --- | --- |
| Install | `.vsix` → `vsix-extensions/` under config; `install.json` + extracted `extension/` tree |
| Compatibility | Static scan of `extension/main` for `vscode.*` usages vs shim registry → **Works / Partial / Unsupported** badge in mod portal |
| **Color themes** | Valid `contributes.themes` JSON converted to Monaco (`tokenColors` + `colors`); picker in **Settings → Appearance → Editor theme** |
| **Grammars** | `contributes.grammars` listed as **recognised, not applied** (see below) |
| `contributes.commands` | Registered in `CommandRegistry`; palette + **Command Manager → Extensions** |
| **Node host** | Opt-in Advanced pref; `execute_command` loads `main` via `examples/vsix-node-host/host.mjs` |
| Trust / lifecycle | Per-extension trust prompt; Node restart backoff; `deactivate` on disable |

### Why grammars stay “not applied”

Monaco in the desktop shell already bundles built-in Monarch grammars for common languages. VS Code TextMate grammars need a **TextMate → Monarch/onig** pipeline (typically `vscode-textmate` + Oniguruma WASM). That is a sizeable dependency and startup cost for a blitz slice, so we only **record** grammar contributions for compatibility reporting. Colour themes reuse existing Monaco `defineTheme` and are cheap to convert.

### Editor themes

| Pref id | Meaning |
| --- | --- |
| `auto` | Follow app light/dark (`vs` / `vs-dark`) |
| `vs` / `vs-dark` | Built-in Monaco themes |
| `vsix:{extensionId}:{slug}` | Theme from an installed, enabled VSIX |

## Compatibility matrix (shimmed `vscode.*` APIs)

Static install scan compares extension `main` against this set. Anything else referenced counts toward **Partial** / **Unsupported**.

| API | Notes |
| --- | --- |
| `commands.registerCommand` | Handler table in Node host |
| `commands.executeCommand` | Unibrowser builtin command bus |
| `window.showInformationMessage` | Toast (info) |
| `window.showWarningMessage` | Toast (warning) |
| `window.showErrorMessage` | Toast (error) |
| `window.activeTextEditor` | Snapshot per execute; `document.getText()` / `languageId` |
| `TextEditor.edit` | Replace / insert / delete → workspace edits + undo |
| `workspace.getConfiguration` | `vsix-extension-config.json` per extension id |
| `workspace.applyEdit` | Same edit path as `TextEditor.edit` |
| `languages.registerDocumentFormattingEditProvider` | **`Format document`** command |
| `Position`, `Range`, `Selection`, `TextEdit`, `WorkspaceEdit` | Edit builders |
| `TextEdit.insert` / `replace` / `delete` | Treated as shimmed when scanned |

## Node sidecar protocol

Ops: `ping`, `activate`, `deactivate`, `execute`, `host_results`, `format_document`.

## Sample & smoke fixtures

| Path | Role |
| --- | --- |
| [examples/vsix-sample-extension](../examples/vsix-sample-extension/) | Hand-written sample commands + formatter |
| [examples/vsix-openvsx-fixture](../examples/vsix-openvsx-fixture/) | **Apache-2.0** synthetic “Open VSX style” extension (theme + formatter + grammar note); `openvsx_fixture_install_smoke` in `unibrowser-vsix` |

## Crate map

| Crate | Role |
| --- | --- |
| `unibrowser-vsix` | Install, compatibility, theme discovery, Monaco theme resolve |
| `unibrowser-extension-host` | Node host, shim registry, `monaco_theme` conversion, text edits |
| Desktop `editor_themes.rs` | Tauri commands for theme list + resolve |
| Desktop `monacoEditorTheme.ts` | `defineTheme` + apply preference |

## Related

- [extension-host crate](../crates/unibrowser-extension-host/src/lib.rs)
- [Plugin SDK](plugin-sdk/README.md) (native mods, not VSIX JS)
