# Extension host (VSIX execution)

VSIX packages install metadata and (optionally) run extension JavaScript through an **opt-in Node sidecar** ([ADR 0018](adr/0018-extension-host-stub.md), [ADR 0014](adr/0014-vsix-compatibility-layer.md)).

## What works

| Layer | Behaviour |
| --- | --- |
| Install | `.vsix` → `vsix-extensions/` under config; `install.json` + extracted `extension/` tree |
| Compatibility | Static scan of `extension/main` for `vscode.*` usages vs shim registry → **Works / Partial / Unsupported** badge in mod portal |
| Themes / grammars | `contributes.themes` and `contributes.grammars` listed as **recognised, not applied** (no Monaco theme/grammar wiring yet) |
| `contributes.commands` | Registered in `CommandRegistry`; palette + **Command Manager → Extensions** |
| `contributes.keybindings` | Merged into `keybindings.json` after defaults |
| `activationEvents` | `onCommand:<id>` recorded when the command runs (Rust activation log) |
| **Node host** | Settings → Advanced → **Node extension host (experimental)** (default **off**). When on and `node` is on PATH, `execute_command` loads `extension/main` via `examples/vsix-node-host/host.mjs` |
| Diagnostics | `unibrowser://diagnostics` lists activation records, host version, Node pid/alive/restart count/last error |
| Trust | First enable per extension id shows a blocking trust dialog; trusted ids stored in preferences |
| Disable | Turning an extension off sends `deactivate` to the Node host for that id |

### Enablement (palette / Command Manager)

| Condition | VSIX contributed commands |
| --- | --- |
| Pref **off** | Grey — enable Node extension host in Settings → Advanced |
| Pref **on**, not trusted / user cancelled trust | Enable blocked until trusted |
| Pref **on**, no `node` on PATH | Grey — Node.js not found on PATH |
| Pref **on**, no `package.json` `main` | Grey — extension has no `main` entry |
| Pref **on**, `node` + `main` + trusted | **Enabled** — runs JS handler |

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

## `vscode` shim

Passed to `activate(vscode)` when the extension `main` module loads. Handlers receive a fresh `vscode` view per `execute` (editor snapshot and configuration are not stale closures).

| API | Support |
| --- | --- |
| `vscode.commands.registerCommand(id, fn)` | Stores handler; `dispose()` removes it |
| `vscode.commands.executeCommand(id)` | Dispatches **Unibrowser builtin** commands via the desktop command bus |
| `vscode.window.showInformationMessage` / `showWarningMessage` / `showErrorMessage` | Chrome toasts |
| `vscode.window.activeTextEditor` | Read-only document + `TextEditor.edit` (replace/insert/delete → workspace edits with undo) |
| `vscode.workspace.getConfiguration(section)` | Per-extension JSON under config |
| `vscode.workspace.applyEdit(WorkspaceEdit)` | Queues workspace edits (same apply path as `edit`) |
| `vscode.languages.registerDocumentFormattingEditProvider` | Registered providers; palette **`Format document`** (`unibrowser.editor.formatDocument`) calls `format_document` on the Node host |
| `Position`, `Range`, `Selection`, `TextEdit`, `WorkspaceEdit` | Minimal builders for edits |

**Not implemented:** LSP, webviews, secrets, file system workspace API, terminal, debug, SCM, tests, authentication, applying TextMate themes/grammars from the VSIX, etc.

## Sample extension

- Source: [examples/vsix-sample-extension](../examples/vsix-sample-extension/)
- Pack: `./examples/vsix-sample-extension/pack-vsix.sh`
- Commands: `sayHello`, `probeEditor`, `formatViaEdit` (prefix via `editor.edit`)
- Formatting: markdown provider adds a trailing newline; exercised by **`Format document`** and `node_host_format_document_provider`
- Tests (`unibrowser-extension-host`, skip if `node` missing): sample command, probe/builtin bus, format provider, workspace edit host call

## Crate map

| Crate | Role |
| --- | --- |
| `unibrowser-vsix` | Zip install, compatibility scan, theme/grammar notes, registry registration |
| `unibrowser-extension-host` | Activation log, Node sidecar, shim registry, text edit apply |
| `unibrowser-engine` | `CommandRegistry`, palette search |
| Desktop `vsix_host.rs` | Editor snapshot, edits, format document, builtin bridge |

## Related

- [extension-host crate](../crates/unibrowser-extension-host/src/lib.rs)
- [Plugin SDK](plugin-sdk/README.md) (native mods, not VSIX JS)
