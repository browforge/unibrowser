# Extension host (VSIX execution)

VSIX packages install metadata and (optionally) run extension JavaScript through an **opt-in Node sidecar** ([ADR 0018](adr/0018-extension-host-stub.md), [ADR 0014](adr/0014-vsix-compatibility-layer.md)).

## What works

| Layer | Behaviour |
| --- | --- |
| Install | `.vsix` → `vsix-extensions/` under config; `install.json` + extracted `extension/` tree |
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
- **Ops:** `ping`, `activate` (`install_dir`, `extension_dir`, `main`, `extension_id`), `deactivate` (`extension_id`), `execute` (`extension_id`, `command_id`, optional `editor`, `configuration`, `execute_token`), `host_results` (builtin command round-trip from extension).

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
| `vscode.window.showInformationMessage(text)` | Toast (info) |
| `vscode.window.showWarningMessage(text)` | Toast (warning severity) |
| `vscode.window.showErrorMessage(text)` | Toast (error) |
| `vscode.window.activeTextEditor` | Read-only: `document.getText()`, `document.languageId`, `document.uri` from active Monaco / `base-text` tab |
| `vscode.workspace.getConfiguration(section)` | JSON file per extension: `{config}/vsix-extension-config.json` keyed by extension id |
| `vscode.commands.executeCommand(id, ...args)` | Dispatches **Unibrowser builtin** commands (e.g. `unibrowser.font.size.increase`) via the desktop command bus; re-entrancy skips nested VSIX dispatch |

**Not implemented:** workspace/fs APIs beyond configuration, languages, webviews, secrets, progress, terminals, debug, SCM, tests, authentication, etc.

Extensions that call missing APIs will throw at runtime; keep samples to the shim above.

## Sample extension

- Source: [examples/vsix-sample-extension](../examples/vsix-sample-extension/)
- Pack: `./examples/vsix-sample-extension/pack-vsix.sh`
- Commands:
  - `unibrowser.helloSample.sayHello` — information toast via the shim.
  - `unibrowser.helloSample.probeEditor` — reads `activeTextEditor`, `helloSample.previewChars` from configuration, calls `unibrowser.font.size.increase`, then info toast.
- Tests (`unibrowser-extension-host`, skip if `node` missing):
  - `node_host_runs_sample_extension_command`
  - `node_host_probe_editor_and_builtin_command`

## Crate map

| Crate | Role |
| --- | --- |
| `unibrowser-vsix` | Zip install, `contributions.json`, `read_extension_main`, registry registration |
| `unibrowser-extension-host` | Activation log, Node sidecar client, shim protocol |
| `unibrowser-engine` | `CommandRegistry`, palette search |
| Desktop `vsix_host.rs` | Pref + PATH gating, editor snapshot, builtin `executeCommand` bridge |
| Desktop `vsix_extension_config.rs` | Per-extension configuration JSON |

## Remaining gaps (full host)

- No worker/WASM host tier; Node only for this spike.
- No `onLanguage:` / workspace / `*` activation scheduler beyond `onCommand:` logging.
- No extension sandbox beyond cwd + minimal env (still full Node APIs).
- No webview, LSP, or marketplace signing beyond install metadata and per-extension trust prompt.
- Command enablement for non-VSIX commands unchanged.

## Related

- [extension-host crate](../crates/unibrowser-extension-host/src/lib.rs)
- [Plugin SDK](plugin-sdk/README.md) (native mods, not VSIX JS)
