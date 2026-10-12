# ADR 0029: Plugin permissions and network security

- **Status:** Accepted
- **Date:** 2026-10-12
- **Amends:** [ADR 0002](0002-mod-system-and-sandboxing.md), [ADR 0019](0019-wasm-t2-sandbox.md), [ADR 0027](0027-command-system-capabilities-and-extensibility.md), [ADR 0028](0028-platform-content-kinds-and-plugin-scope.md) (permission tables)

## Context

[ADR 0028](0028-platform-content-kinds-and-plugin-scope.md) allows plugins to contribute almost anything the host mediates: commands, panels, providers, services, and format logic. Many legitimate features need sensitive capabilities: an icon pack mod that downloads Material Design Icons for import, an assistant mod that sends document excerpts to a user-chosen model endpoint, or a catalogue mod that searches Open VSX.

Unibrowser is **local-first** ([principles.md](../principles.md), [privacy.md](../privacy.md)): nothing leaves the machine unless the user clearly opts in. **Save is sacred** ([ADR 0023](0023-saving-is-sacred.md)): permission denial must grey controls and explain why, never block Save, autosave, or editing.

Today, permission ideas are scattered: T2 WASM network denial in [ADR 0019](0019-wasm-t2-sandbox.md), content/UI permission names in [ADR 0028](0028-platform-content-kinds-and-plugin-scope.md), AI grants in [ADR 0026](0026-ai-assistant-as-mod.md), coarse portal network toggles in [network.md](../network.md), and a **blocking** VSIX Node trust dialog in [extension-host.md](../extension-host.md). This ADR unifies a **capability-based** model with explicit user consent, revocable grants, and honest tier boundaries (sandboxed WASM vs full-trust native).

## Industry comparison

What other platforms do well, and what fails in practice.

| Platform | Declaration | Consent moment | Granularity | Revocation / expiry | Notable failures |
| --- | --- | --- | --- | --- | --- |
| **Android** | `normal` vs `dangerous` permissions in manifest; dangerous = runtime ([overview](https://developer.android.com/guide/topics/permissions/overview)) | Ask when the user performs the action that needs access ([requesting](https://developer.android.com/training/permissions/requesting)) | Permission groups; background location split to Settings on API 30+ ([background location](https://developer.android.com/develop/sensors-and-location/location/permissions/background)) | **Only this time** for location/mic/camera on API 30+ ([Android 11 permissions](https://developer.android.com/about/versions/11/privacy/permissions)); **auto-reset** unused grants after months ([blog](https://android-developers.googleblog.com/2021/09/making-permissions-auto-reset-available.html)) | **Permission fatigue** from repeated prompts; users tap Allow without reading; overbroad legacy install-time grants on old API levels |
| **iOS / App Store** | Privacy manifest (`PrivacyInfo.xcprivacy`): collected data types, required-reason APIs, tracking domains ([privacy manifests](https://developer.apple.com/documentation/bundleresources/privacy-manifest-files)) | Per-resource prompts (camera, photos, …) at use; **App Tracking Transparency** for cross-app tracking ([App Privacy Details](https://developer.apple.com/app-store/app-privacy-details/)) | Store **Privacy Nutrition Labels** declare collection/tracking before install | Tracking domains blocked when ATT denied ([TN3182](https://developer.apple.com/documentation/technotes/tn3182-adding-privacy-tracking-keys-to-your-privacy-manifest)) | Label accuracy depends on publisher honesty; manifest + label lag behind SDK sprawl; users confuse “tracking” vs “functional” network |
| **Chrome extensions (MV3)** | `permissions`, `host_permissions`, `optional_permissions`, `optional_host_permissions` ([declare](https://developer.chrome.com/docs/extensions/develop/concepts/declare-permissions)); **`activeTab`** temporary host access after user gesture ([activeTab](https://developer.chrome.com/docs/extensions/develop/concepts/activeTab)) | Optional permissions via `chrome.permissions.request` at runtime ([permissions API](https://developer.chrome.com/docs/extensions/reference/api/permissions)) | Per-origin host patterns; subset requests from wildcards | User can remove extension or trim site access in browser settings | **`<all_urls>` / `*://*/*`** invites abuse and long review ([review process](https://developer.chrome.com/docs/webstore/review-process)); **excessive permissions** rejections ([CWS policy](https://developer.chrome.com/docs/webstore/program-policies/permissions)); install-time host warnings train click-through |
| **VS Code** | Extensions run **in-process** with full Node APIs; no capability sandbox ([Workspace Trust](https://code.visualstudio.com/docs/editing/workspaces/workspace-trust)) | **Workspace Trust** / Restricted Mode disables non-declaring extensions; **publisher trust** prompt on first install from third party ([extension runtime security](https://code.visualstudio.com/docs/configure/extensions/extension-runtime-security)) | Trust is workspace + publisher, not per API | User manages trusted publishers and workspace trust | Malicious extension can ignore Restricted Mode; trust is binary, not per-domain network; users enable “unsupported in untrusted” overrides |
| **Adobe UXP** | `requiredPermissions`: `network.domains[]`, `localFileSystem`, `launchProcess`, clipboard ([network](https://developer.adobe.com/uxp/guides/how-to/recipes/network/), [manifest v5](https://developer.adobe.com/photoshop/uxp/2022/guides/uxp-guide/uxp-misc/manifest-v5/), [filesystem](https://developer.adobe.com/uxp/guides/how-to/recipes/filesystem-operations/)) | Undeclared network fails; `launchProcess` and sensitive FS levels use **consent at use** ([external process](https://developer.adobe.com/uxp/guides/how-to/recipes/external-process/)) | Explicit HTTPS domain allow-list; FS tiers `plugin` / `request` / `fullAccess` | Reinstall/update may re-prompt for elevated FS | Wildcard domains (`https://*.adobe.io`) still broad; users may not read manifest at install |
| **OFX / VEGAS-style native plugins** | Host loads **native DLL** in-process; API is suites and actions, not a security boundary ([OFX core](https://openfx.readthedocs.io/en/main/Reference/ofxCoreAPI.html)) | User trust = installing plugin binary | Full process memory, FS, network via OS | Uninstall plugin | **Full trust by default**; one bad plugin compromises the host; no standard permission manifest across hosts |

**Lessons for Unibrowser:** combine UXP-style **manifest allow-lists** and Android-style **ask-at-use** with MV3-style **optional/runtime** grants; avoid VS Code’s “full runtime, hope trust saves you” for community mods; avoid OFX full-trust for marketplace defaults; mitigate fatigue with **per-domain** network grants, plain-language payloads, and **non-modal** grant paths tied to the control that needs access ([command-system.md](../design/command-system.md)).

## Decision

### 1. Capability manifest (declare everything)

Plugins declare capabilities in `info.json` under a versioned `permissions` object (semver key `permission_schema`, initial `1`). The host refuses load if the payload uses a capability not declared (fail at load for WASM; audit + block at call site for native until signing policy tightens).

| Capability key | Meaning | Default |
| --- | --- | --- |
| `network.domains` | HTTPS (and optional WSS) origins; no wildcards wider than one subdomain label unless `network.allow_subdomain_wildcard: true` on that entry | **Empty = no network** |
| `filesystem.scopes` | `plugin_data`, `user_pick`, `workspace_roots`, `downloads` (each optional path class) | `plugin_data` only for T2 |
| `clipboard` | `none`, `read`, `write`, `read_write` | `none` |
| `process.launch` | Allowed schemes (`https`, `mailto`, …) and/or extensions for `openPath` / sidecar spawn | `[]` |
| `content.read` | Content kinds (registry ids) the mod may read via tree/selection APIs | As needed |
| `content.write` | Content kinds the mod may edit via host transactions | As needed |
| `secrets` | Named secret slots stored in OS keychain (see §4) | `[]` |
| `background` | Scheduled or idle work (`alarms`, `sync` hooks) | `false` |
| `commands.*`, `ui.*`, `content.*`, `service.*`, `provider.*` | As in [ADR 0028](0028-platform-content-kinds-and-plugin-scope.md) §7 | Least privilege |

Document read/write permissions are **per content kind**, not per file extension. Host format does not replace kind checks.

**Execution tier interaction**

| Tier | Enforcement |
| --- | --- |
| **T2 WASM** | All capabilities enforced by host imports; undeclared import = instantiate failure ([ADR 0019](0019-wasm-t2-sandbox.md)) |
| **T1 native (signed)** | Host mediates network, secrets, process launch, and FS outside `plugin_data`; vtable calls audited |
| **T1 native (unsigned / dev)** | Treated as **full trust** with install-time warning (§6) |
| **T3 sidecar** | Separate manifest + IPC capability tokens; same user grant store |
| **VSIX Node lane** | Maps to `network.domains` + `content.read`/`write` for declared scopes; migrate off blocking trust dialog toward this model ([extension-host.md](../extension-host.md)) |

### 2. Network: off by default, per plugin, per domain

- No plugin network I/O unless `network.domains` is non-empty **and** the user has granted that plugin access to the specific origin (scheme + host + port).
- **First use** of a domain: show the **network consent** surface ([plugin-permissions-ui.md](../design/plugin-permissions-ui.md)) with:
  - Plugin name and publisher
  - Plain-language **reason** from manifest `permissions.network.reason` (required when any domain is declared)
  - **Exactly what may leave the device**: bullet list from manifest `permissions.network.disclosure` (required for assistant/export mods; optional for opaque blob fetches)
  - Destination shown as the concrete origin (no hidden redirects without a new prompt)
- User choices: **Allow once**, **Always allow**, **Don't allow**.
- **Allow once** applies to the current session (until quit) for that plugin+origin pair; **Always** persists in the grant store.
- **Offline mode** ([network.md](../network.md)) is a **global kill switch**: all plugin network (including Open VSX) is denied with reason “Offline mode is on”; web navigation the user started explicitly remains allowed per existing policy.
- Portal **Allow network** toggle remains a coarse gate before catalogue search; per-domain grants still apply to each fetch.

### 3. Grant store, revocation, version reset

- Grants persist in platform config as `permission-grants.json` (human-readable, local only).
- Structure: plugin id → capability → grant records (origin, scope, choice, granted_at, manifest_version).
- **Settings → Plugins → Permissions**: list every plugin with expandable capabilities; **Revoke** per origin/scope; link to activity log.
- When a plugin **version** increases and the manifest adds **new** domains, filesystem scopes, content kinds, `process.launch` entries, or `secrets` keys, all **persistent** grants for that plugin are **reset**; session “allow once” is cleared. The user sees a one-line banner in Mod portal: “Permissions reset — review what changed.”
- Removing a permission from manifest does not auto-grant new access on downgrade; old grants for removed origins are dropped on load.

### 4. Secrets and credentials

- API keys, tokens, and passwords for plugins use host API `secrets.put/get/delete` only.
- Storage: **OS keychain** (Credential Manager / Keychain / libsecret); service name `unibrowser.plugin.<plugin_id>.<slot>`.
- Never write secrets to `preferences.json`, mod folders, or logs.
- Activity log records **that** a secret slot was used, not its value.

### 5. Network activity log

- Append-only local log `network-activity.log.jsonl` (rotating, cap 10 000 entries): `timestamp`, `plugin_id`, `origin`, `method`, `path_or_summary`, `bytes_out`, `bytes_in`, `grant_type` (`once` | `always`), `correlation_id` optional.
- Settings UI: filter by plugin, date, domain; **Clear log** (user-initiated).
- No upload; export via Save As JSON for support.

### 6. Mod Portal and install trust

- Portal cards and detail expansion show **permission chips** derived from manifest (Network: N domains, Read: kinds, …). High-risk: `fullAccess` FS, `process.launch`, native **full trust** tier.
- **WASM (T2)** mods: default install copy — “Runs sandboxed; network and files are gated.”
- **Native DLL (T1) without sandbox**: **full-trust tier** — secondary warning panel before Enable: “This mod runs as native code with the same power as installed applications. Only enable if you trust the publisher.” No dark patterns; Decline leaves the mod disabled; Save unaffected.

### 7. Command Manager and commands (no blocking modals)

- If a control needs an ungranted capability, resolution returns `locked` or `unsupported` with reason text ([ADR 0027](0027-command-system-capabilities-and-extensibility.md)); control stays **greyed in place**.
- Tooltip / palette: reason + **Grant** link (keyboard reachable) opening the consent surface or Permissions page prefilled for that plugin.
- **Never** a blocking modal on Save, autosave, or background editing. **Never** cover Save geometry.
- Denied network: command stays disabled; user can continue editing and save locally.

### 8. AI and projections

- Assistant mods use the same `network.domains` + `content.read` kinds + disclosure bullets; align grant UX with [ADR 0026](0026-ai-assistant-as-mod.md) read/write/ask semantics inside this store (AI “Ask” is not a substitute for network consent).

## Principles check

| Check | Answer |
| --- | --- |
| Trusts the user | Yes — explicit grants, revocable, honest full-trust warning for native |
| Never blocks save or work | Yes — denial greys features only |
| Local-first | Yes — grants and logs on disk; network opt-in per domain |
| Predictable and consistent | Yes — same grant pattern for icon fetch, Open VSX, and AI |
| Simple durable mechanism | Yes — JSON grants + JSONL log + keychain |
| Mod not core | Yes — core enforces; plugins declare |

## Consequences

**Positive**

- One model covers native mods, WASM, sidecars, and (eventually) VSIX network.
- Users can audit what left the machine without reading packet captures.
- Version reset closes the “silent new endpoint” hole.

**Tradeoffs**

- Per-domain prompts can still fatigue; mitigated by Always allow and clear disclosure requirements.
- Native T1 enforcement depends on host mediation discipline; WASM is the safer default for community.
- Manifest authoring burden: plugins must maintain `disclosure` strings for sensitive mods.

## References

- [plugin-permissions-ui.md](../design/plugin-permissions-ui.md) — copy and wireframes
- [ui-implementation-plan.md](../design/ui-implementation-plan.md) — PERM-01…
- [mod-system.md](../mod-system.md), [plugin-sdk/README.md](../plugin-sdk/README.md)
- [security.md](../security.md), [network.md](../network.md)
