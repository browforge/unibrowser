# Vision matrix (Bear decisions)

Honest status for core product decisions. **Shipped** = usable in the desktop shell today; **Partial** = visible behaviour with important gaps; **Missing** = not implemented or stub only.

| Decision | Status | Evidence |
| --- | --- | --- |
| SOLIDWORKS Command Manager: grey out tools instead of hiding | **Shipped** | `Enablement::Disabled` in handlers; palette/CM render disabled chips with reasons (`crates/unibrowser-engine/src/command.rs`, `crates/unibrowser-desktop/src/commandManager.ts`) |
| Context tabs auto-follow document + separate Pin | **Shipped** | `context_pin`, `select_tab_manual`, pin control (`crates/unibrowser-engine/src/ui_command.rs`, `commandManager.ts`, `pin_command_manager_context` in `commands.rs`) |
| Ctrl+Shift+P palette + `keybindings.json` | **Shipped** | `DEFAULT_PALETTE_CHORD`, `KeybindingMap::load_merged` (`keybinding.rs`, `config/keybindings.json`, palette in shell) |
| Drag-anywhere toolbars; float to another monitor | **Partial** | Screen coords in `toolbars.json` (`ToolbarCoordSpace::Screen`, `commandManager.ts`); **detached OS windows** via `presentation: detached_window` + `toolbar_windows.rs` / right-click **Detach to own window** |
| Affinity-style context switching | **Partial** | Auto context + manual tab preview (`transient_tab`, ADR 0012); not full Affinity persona presets |
| Everything is a mod | **Partial** | Bundled `mods/base-*` + manifest loader (`unibrowser-mod-host`); shell stubs (EPUB, CSV) still in `src-tauri` |
| Compiled mod payloads (cdylib / WASM) + manifest-only JSON | **Partial** | Linux cdylib (`dynamic-mods`, `examples/ub-mod-cdylib-stub`); **WASM** via wasmi + `examples/ub-mod-wasm` (`crates/unibrowser-mod-host/src/wasm.rs`, `crates/unibrowser-mod-sdk`) |
| Open VSX + `.ubmod` + sideload | **Partial** | Mod portal, Open VSX search stub, `.ubmod` build script (`mod_portal`, `open_vsx`, `docs/plugin-sdk.md`); VSIX JS host still spike |
| Lazy conversion (encode-time raster) | **Partial** | Timeline H.264 intents (`docs/timeline-encode-intents.md`, ADR 0011); not all formats |
| `.unibrow` cross-format project | **Shipped** | `unibrowser-project`, board/timeline UI, share zip |
| One cross-format font command | **Shipped** | `unibrowser.font.size.increase` / decrease across text, SVG, DOCX, EPUB, HTTP (`docs/commands-cross-format.md`) |
| Skippable onboarding; no repeat unless asked | **Shipped** | Tour + checklist (`onboardingTour.ts`, `preferences.rs`); Settings **Show welcome** → `reset_onboarding_prefs_cmd` |
| Suggestions never cover content | **Shipped** | Non-blocking tour card + checklist in Settings (ADR 0013); extension tip dismissible; bookmark bar uses content bounds (`docs/onboarding.md`) |

## Recent vision slice (this commit)

- WASM T2 loader (wasmi) with guest SDK and `examples/ub-mod-wasm` (command + `.ubwasm` handler test).
- Detached toolbar Tauri windows with persisted `presentation` in `toolbars.json`.

## Top gaps after this slice

1. VSIX / extension host depth (full `vscode` API, lifecycle).
2. Native mod load on Windows/macOS (Linux user `mods/` today).
3. Full WIT sandbox (fuel, caps, signing policy on WASM).
4. Affinity-grade workspace personas beyond CM JSON.
