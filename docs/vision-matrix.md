# Vision matrix (Bear decisions)

Honest status for core product decisions. **Shipped** = usable in the desktop shell today; **Partial** = visible behaviour with important gaps; **Missing** = not implemented or stub only.

| Decision | Status | Evidence |
| --- | --- | --- |
| SOLIDWORKS Command Manager: grey out tools instead of hiding | **Shipped** | `Enablement::Disabled` in handlers; palette/CM render disabled chips with reasons (`crates/unibrowser-engine/src/command.rs`, `commandManager.ts`) |
| Context tabs auto-follow document + separate Pin | **Shipped** | `context_pin`, `select_tab_manual`, pin control (`ui_command.rs`, `commandManager.ts`, `commands.rs`) |
| Ctrl+Shift+P palette + `keybindings.json` | **Shipped** | `KeybindingMap::load_merged` (`keybinding.rs`, palette in shell) |
| Drag-anywhere toolbars; float to another monitor | **Shipped** | Screen coords + `presentation: detached_window` Tauri windows (`toolbars.md`, `toolbar_windows.rs`, `commandManager.ts`) |
| Affinity-style context switching | **Shipped** | Auto context + pin + **Studio personas** (Settings → workspace: Text/Office/Image/Web) (`workspace_personas.rs`, `workspaceLayoutPanel.ts`) |
| Everything is a mod | **Partial** | Bundled `mods/base-*` + manifest loader; MIDI/settings stubs still in `src-tauri` |
| Compiled mod payloads (cdylib / WASM) + manifest-only JSON | **Partial** | WASM wasmi + SDK + `ub mod new wasm`; Linux cdylib (`dynamic-mods`); Win/macOS native load still Linux-only |
| Open VSX + `.ubmod` + sideload | **Partial** | Mod portal, Open VSX search, sideload; VSIX JS host still spike |
| Lazy conversion (encode-time raster) | **Shipped** | Timeline H.264 lane documented in [lazy-conversion-scope.md](lazy-conversion-scope.md), [timeline-encode-intents.md](timeline-encode-intents.md) |
| `.unibrow` cross-format project | **Shipped** | `unibrowser-project`, board/timeline UI, share zip |
| One cross-format font command | **Partial** | Same id on text, web, SVG text, DOCX, EPUB, DXF TEXT/MTEXT, PNG `unibrowser_text`, `.unibrow` titles; grey on plain PNG/PDF/CSV ([commands-cross-format.md](commands-cross-format.md), `cross_format_commands` tests) |
| One offline diagnostics host + proofing mods (squiggles) | **Partial** | `unibrowser-diagnostics` + `spell-en-nz` / `grammar-harper`; Monaco live; DXF/unibrow locator mods; DOCX/web/SVG adapters partial ([proofing-cross-format.md](proofing-cross-format.md), ADR 0021–0022) |
| Saving is sacred + local autosave | **Partial** | Permissive `write_manifest`; rotating `.ubautosave` slots; status bar time; export stubs do not block save ([principles.md](principles.md), ADR 0023) |
| Live sources + compositor + outputs | **Partial** | Engine traits, frame clock, CPU compositor, file sink; proving mods only ([ADR 0024](adr/0024-live-sources-compositor-outputs.md), `unibrowser-live`) |
| Skippable onboarding; no repeat unless asked | **Shipped** | Tour + checklist; dismiss forever; Settings **Show welcome** only when asked |
| Principles watchlist (close tab, VSIX trust, offline catalog, save pickers) | **Shipped** | [blitz-status.md](blitz-status.md) Resolved table; [network.md](network.md) |
| Suggestions never cover content | **Shipped** | Unknown-file **header banner** (not modal); onboarding card; mod portal optional (`unknown-file-banner`, ADR 0013) |
| Unknown file grandma path | **Shipped** | Banner + Open as text/hex + mod suggestions (`open_active_file_as_*`, `mods/base-hex`) |

## Recent slice

- Studio personas, lazy-conversion scope doc, unknown-file banner, `base-hex`, `mod-authoring.md`, `unibrowser mod new wasm`.

## Top gaps

1. VSIX / extension host depth.
2. Native mod load on Windows/macOS.
3. WASM sandbox hardening (fuel, caps).
4. Move remaining shell stubs (MIDI, etc.) into bundled mods.
