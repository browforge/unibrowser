# Extension host (VSIX execution)

VSIX packages install metadata and (optionally) run extension JavaScript through an **opt-in Node sidecar** ([ADR 0018](adr/0018-extension-host-stub.md), [ADR 0014](adr/0014-vsix-compatibility-layer.md)).

## What works

| Layer | Behaviour |
| --- | --- |
| Install | `.vsix` → `vsix-extensions/` under config; `install.json` + extracted `extension/` tree |
| `contributes.commands` | Registered in `CommandRegistry`; palette + **Command Manager → Extensions** |
| `contributes.keybindings` | Merged into `keybindings.json` after defaults |
| `activationEvents` | `onCommand:<id>` recorded when the command runs (Rust activation log) |
| **Node host (spike)** | Settings → Advanced → **Node extension host (experimental)** (default **off**). When on and `node` is on PATH, `execute_command` loads `extension/main` via `examples/vsix-node-host/host.mjs` |
| Diagnostics | `unibrowser://diagnostics` lists activation records and host status |

### Enablement (palette / Command Manager)

| Condition | VSIX contributed commands |
| --- | --- |
| Pref **off** | Grey — enable Node extension host in Settings → Advanced |
| Pref **on**, no `node` on PATH | Grey — Node.js not found on PATH |
| Pref **on**, no `package.json` `main` | Grey — extension has no `main` entry |
| Pref **on**, `node` + `main` | **Enabled** — runs JS handler |

## Node sidecar protocol

- **Script:** `examples/vsix-node-host/host.mjs` (override with `UNIBROWSER_VSIX_NODE_HOST`).
- **Transport:** one JSON object per line on stdin; one response line on stdout.
- **Ops:** `ping`, `activate` (`extension_dir`, `main`, `extension_id`), `execute` (`extension_id`, `command_id`).

The desktop shell keeps a single long-lived Node process per session (see `unibrowser-extension-host::node_host`).

## `vscode` shim (spike surface)

Passed to `activate(vscode)` when the extension `main` module loads.

| API | Support |
| --- | --- |
| `vscode.commands.registerCommand(id, fn)` | Stores handler; `dispose()` removes it |
| `vscode.window.showInformationMessage(text)` | Queues a toast; desktop shows it via normal command result / chrome toast |

**Not implemented:** `executeCommand`, workspace/fs APIs, languages, webviews, secrets, configuration, subscriptions beyond dispose, errors/warnings, progress, terminals, debug, SCM, tests, authentication, etc.

Extensions that call missing APIs will throw at runtime; keep samples to the shim above.

## Sample extension

- Source: [examples/vsix-sample-extension](../examples/vsix-sample-extension/)
- Pack: `./examples/vsix-sample-extension/pack-vsix.sh`
- Command: `unibrowser.helloSample.sayHello` — information toast via the shim.
- Test: `cargo test -p unibrowser-extension-host node_host_runs_sample_extension_command` (skips if `node` missing).

## Crate map

| Crate | Role |
| --- | --- |
| `unibrowser-vsix` | Zip install, `contributions.json`, `read_extension_main`, registry registration |
| `unibrowser-extension-host` | Activation log, Node sidecar client, stub helpers |
| `unibrowser-engine` | `CommandRegistry`, palette search |
| Desktop `vsix_host.rs` | Pref + PATH gating for palette enablement |

## Remaining gaps (full host)

- No worker/WASM host tier; Node only for this spike.
- No `onLanguage:` / workspace / `*` activation scheduler beyond `onCommand:` logging.
- No extension sandbox beyond loading declared `main` from the install dir.
- No webview, LSP, or marketplace trust beyond install metadata.
- Command enablement for non-VSIX commands unchanged.

## Related

- [extension-host crate](../crates/unibrowser-extension-host/src/lib.rs)
- [Plugin SDK](plugin-sdk/README.md) (native mods, not VSIX JS)
