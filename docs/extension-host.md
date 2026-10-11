# Extension host (VSIX execution)

VSIX packages install metadata and (optionally) run extension JavaScript through an **opt-in Node sidecar** ([ADR 0018](adr/0018-extension-host-stub.md), [ADR 0014](adr/0014-vsix-compatibility-layer.md)). VSIX is a **compat lane** alongside native platform plugins ([ADR 0028](adr/0028-platform-content-kinds-and-plugin-scope.md)): contributions map into the same fixed Command Manager, palette, and (when implemented) panel slots; kind-specific tools should declare `content_kinds` like native mods.

## What works

| Layer | Behaviour |
| --- | --- |
| Install | `.vsix` → `vsix-extensions/` under config; `install.json` + extracted `extension/` tree |
| Compatibility | Static scan of `extension/main` for `vscode.*` usages vs shim registry → **Works / Partial / Unsupported** badge in mod portal |
| **Color themes** | Valid `contributes.themes` JSON converted to Monaco (`tokenColors` + `colors`); picker in **Settings → Appearance → Editor theme** |
| **Grammars** | `contributes.grammars` → Monaco via **vscode-textmate** + Oniguruma WASM when the editor language id matches (see below) |
| `contributes.commands` | Registered in `CommandRegistry`; palette + **Command Manager → Extensions** |
| `contributes.keybindings` | Merged into `keybindings.json` after defaults (user overrides win) |
| `activationEvents` | `onCommand:<id>` recorded when the command runs (Rust activation log) |
| **Node host** | Settings → Advanced → **Node extension host (experimental)** (default **off**). When on and `node` is on PATH, `execute_command` loads `extension/main` via `examples/vsix-node-host/host.mjs` |
| Diagnostics | `unibrowser://diagnostics` lists activation records, host version, Node pid/alive/restart count/last error |
| Trust | First enable per extension id shows a blocking trust dialog; trusted ids stored in preferences |
| Disable | Turning an extension off sends `deactivate` to the Node host for that id |

### TextMate grammars (applied)

Enabled VSIX grammars load through `monacoTextmate.ts` (`vscode-textmate`, `vscode-oniguruma`, `monaco-editor-textmate`). Install notes show **TextMate applied in Monaco when language matches**. Smoke: install [examples/vsix-openvsx-fixture](../examples/vsix-openvsx-fixture/) and open `sample.fixturelang` (language id `fixturelang`).

### Editor themes

| Pref id | Meaning |
| --- | --- |
| `auto` | Follow app light/dark (`vs` / `vs-dark`) |
| `vs` / `vs-dark` | Built-in Monaco themes |
| `vsix:{extensionId}:{slug}` | Theme from an installed, enabled VSIX |

### Enablement (palette / Command Manager)

| Condition | VSIX contributed commands |
| --- | --- |
| Pref **off** | Grey — enable Node extension host in Settings → Advanced |
| Pref **on**, not trusted / user cancelled trust | Enable blocked until trusted |
| Pref **on**, no `node` on PATH | Grey — Node.js not found on PATH |
| Pref **on**, no `package.json` `main` | Grey — extension has no `main` entry |
| Pref **on**, `node` + `main` + trusted | **Enabled** — runs JS handler |

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

**Not implemented:** LSP, webviews, secrets, full workspace FS API, terminal, debug, SCM, tests, authentication, applying TextMate grammars from the VSIX, etc.

## Node sidecar protocol

- **Script:** `examples/vsix-node-host/host.mjs` (override with `UNIBROWSER_VSIX_NODE_HOST`).
- **Transport:** one JSON object per line on stdin; one response line on stdout (multi-round `execute` until `complete`).
- **Ops:** `ping`, `activate`, `deactivate`, `execute`, `host_results` (builtin or workspace-edit round-trip), `format_document` (run registered formatting providers).

The desktop shell keeps a single long-lived Node process per session (`unibrowser-extension-host::node_host`). If the child exits unexpectedly, the pool respawns with exponential backoff (cap 30s) on the next request.

## Lifecycle and security

| Topic | Behaviour |
| --- | --- |
| **cwd** | On `activate`, the host sets `process.chdir(install_dir)` so relative imports resolve inside the extension install tree |
| **Environment** | Child is spawned with `env_clear()` and only `PATH` set (no app secrets or API keys) |
| **stderr** | Discarded to avoid leaking host paths in UI |
| **Shutdown** | Session end can call `shutdown_node_host()` to drop the child |

## Sample & smoke fixtures

| Path | Role |
| --- | --- |
| [examples/vsix-sample-extension](../examples/vsix-sample-extension/) | Hand-written sample commands + formatter (`sayHello`, `probeEditor`, `formatViaEdit`) |
| [examples/vsix-openvsx-fixture](../examples/vsix-openvsx-fixture/) | **Apache-2.0** synthetic “Open VSX style” extension (theme + formatter + grammar note); `openvsx_fixture_install_smoke` in `unibrowser-vsix` |

Tests (`unibrowser-extension-host`, skip if `node` missing): sample command, probe/builtin bus, format provider, workspace edit host call.

## Crate map

| Crate | Role |
| --- | --- |
| `unibrowser-vsix` | Install, compatibility, theme discovery, Monaco theme resolve, keybindings metadata |
| `unibrowser-extension-host` | Node host, shim registry, `monaco_theme` conversion, text edits |
| Desktop `editor_themes.rs` | Tauri commands for theme list + resolve |
| Desktop `monacoEditorTheme.ts` | `defineTheme` + apply preference |
| Desktop `vsix_host.rs` | Editor snapshot, edits, format document, builtin bridge |

## Related

- [extension-host crate](../crates/unibrowser-extension-host/src/lib.rs)
- [Plugin SDK](plugin-sdk/README.md) (native mods, not VSIX JS)
