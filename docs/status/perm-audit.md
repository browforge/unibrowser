# PERM-01..10 audit (2026-10-20)

Scope: plugin permissions and network consent per ADR 0029, `docs/design/plugin-permissions-ui.md`, and Phase I in `docs/design/ui-implementation-plan.md`.

Baseline audited: `1323f64` on `main`. Design intent: network off by default; plugins reach only declared sites; user approves and can revoke; every gated request is logged.

## Summary

| Ticket | Status | Evidence | What is missing |
| --- | --- | --- | --- |
| PERM-01 | done | `perm01_mod_manifest_has_permissions_field_and_validation`, `perm01_loader_validates_permissions_json`, `perm01_invalid_wildcard_domain_fails_load`, `perm01_invalid_permissions_skips_mod_folder`; `permissions.rs` + loader/native validation | Portal chips; proving fixture with two domains in UI |
| PERM-02 | done (backend) | `perm02_permission_grants_persistence_path`, `perm02_revoke_always_grant_without_restart`, `permission_grants.rs` + `permission-grants.json` path | Settings UI revoke; portal banner on version reset |
| PERM-03 | partial | `perm03_wasm_host_linker_has_no_network_import`, `perm03_no_central_mod_network_shim_module`, `perm03_rejects_without_opt_in`, `perm03_offline_mode_blocks_catalog`, `permAudit.test.mjs` Open VSX toggle | Host-mediated mod `fetch`/WebSocket API with per-domain + grant checks; undeclared host blocked at gate; portal/catalog not on same grant+log path as mods |
| PERM-04 | missing | `permAudit.test.mjs` PERM-04 (skipped); no `permissionConsentBanner.ts` | First-use banner, queue, Allow once / Always / Don't allow wired to grant store |
| PERM-05 | missing | `permAudit.test.mjs` PERM-05; settings has Mods + AI grants only | Settings → Plugins → Permissions cards, per-origin revoke, capability summary |
| PERM-06 | missing | `perm06_no_network_activity_log_path`; `perm06_gated_fetch_appends_activity_log_row` (ignored) | `network-activity.log.jsonl` writer, rotation, user-visible table/filters/export |
| PERM-07 | missing | `perm07_no_plugin_secrets_host_commands` | `secrets.put/get/delete` via OS keychain; no plain-text secret prefs |
| PERM-08 | missing | `permAudit.test.mjs` portal `permission_summary` only | Manifest-derived chips (Network count, Sandboxed, Full trust), domain list in detail |
| PERM-09 | missing | `permAudit.test.mjs` PERM-09 (skipped); `ResolvedUiCommand` has `reason` only | `grant_action` on disabled DTO, Grant… opens consent without blocking Save |
| PERM-10 | partial | `perm10_vsix_trust_still_separate_from_grant_store`, `permAudit.test.mjs` VSIX trust bar | Portal T2 sandbox copy + T1 full-trust panel; align VSIX Node trust with grant store |

## Network safety probes

| Check | Result | Evidence |
| --- | --- | --- |
| WASM guest network import | blocked (no import) | `perm03_wasm_host_linker_has_no_network_import` |
| WASM/JS plugin fetch without declaration | not gated end-to-end | `perm03_wasm_cannot_fetch_without_declaration_and_grant` (ignored); no mod fetch API; VSIX Node sidecar has full Node networking |
| Undeclared host | not implemented | `perm03_undeclared_host_blocked_at_fetch_gate` (ignored) |
| Revoke without restart | implemented (grant store) | `perm02_revoke_always_grant_without_restart` |
| Request log user-visible | not implemented | `perm06_no_network_activity_log_path`; PERM-06 UI skipped |
| Consent prompt UI | not implemented | PERM-04 skipped |
| Open VSX (coarse gate) | works | `perm03_rejects_without_opt_in`, `perm03_offline_mode_blocks_catalog` |

Native T1 mods remain full-trust at the OS level until PERM-03 call-site mediation and PERM-10 install UX land. AI assistant uses a separate grant store (`unibrowser-ai-assistant`), not ADR 0029 network grants.

## Recommended build order

1. **PERM-03 + PERM-06** — central mod network shim (deny by default), JSONL activity log on every decision.
2. **PERM-04 + PERM-09** — consent banner and Command Manager Grant… wired to the same pending-request queue.
3. **PERM-05** — Settings Permissions page with revoke (backend revoke already immediate).
4. **PERM-08 + PERM-10** — portal chips and install trust copy; migrate VSIX trust toward grant store.
5. **PERM-07** — keychain secrets API (can trail network if mods have no secret slots yet).

## How to re-run evidence

```bash
cargo test -p unibrowser-mod-api perm01
cargo test -p unibrowser-mod-host perm01 perm03
cargo test -p unibrowser-desktop perm02 perm03 perm06 perm07 perm10
cd crates/unibrowser-desktop && node --experimental-strip-types --test tests/permAudit.test.mjs
```

Full workspace: `npm run build` in `crates/unibrowser-desktop`, then `cargo test --workspace`.
