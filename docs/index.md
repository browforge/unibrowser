# Unibrowser documentation

Markdown docs for contributors and maintainers. The product README stays the entry point for **running** the app; this index links design records, guides, and QA.

**Philosophy (read first):** [principles.md](principles.md) is Bear's standing tiebreaker for every design decision. Use its Decision checklist; cite it in new ADRs ([adr/0000-template.md](adr/0000-template.md)).

## New contributors (FOSS)

1. Read [getting-started.md](getting-started.md) for clone, build, test, and where to send changes.
2. Read [principles.md](principles.md), then skim [vision.md](vision.md) and [architecture.md](architecture.md) so engine / mod / shell boundaries stay clear.
3. Follow [docs/CONTRIBUTING.md](CONTRIBUTING.md) (build, checks, proving mods, PR expectations) and [CONTRIBUTING.md](../CONTRIBUTING.md) (community and asset policy).
4. Run `./scripts/smoke-desktop-qa.sh` before you open a PR. Optionally `./scripts/lint-doc-links.sh` on doc edits.

## Architecture and product

| Doc | Purpose |
| --- | --- |
| [principles.md](principles.md) | Core philosophy and decision checklist (tiebreaker) |
| [vision.md](vision.md) | Goals: unified file + web shell, mod-driven formats |
| [architecture.md](architecture.md) | Crate layout, data flow |
| [roadmap.md](roadmap.md) | Phase plan |
| [mod-system.md](mod-system.md) | Bundled mods and portal |
| [mod-api-stable.md](mod-api-stable.md) | Frozen mod API 0.1 |
| [plugin-sdk.md](plugin-sdk.md) | Hello-world `.ubmod` build and install |
| [plugin-sdk/README.md](plugin-sdk/README.md) | Native mod SDK ABI and user `mods/` dir |
| [commands-cross-format.md](commands-cross-format.md) | Font-size, find, and page zoom per viewer |
| [proofing-cross-format.md](proofing-cross-format.md) | Offline spell/grammar squiggles per surface |
| [commands-undo-redo.md](commands-undo-redo.md) | Undo/redo and PDF export matrix |
| [packaging.md](packaging.md) | Planned MSI/NSIS/AppImage/deb artefacts |
| [file-associations.md](file-associations.md) | No OS default handlers yet; FOSS packaging notes |
| [chrome.md](chrome.md) | Tabs, omnibox, status bar chrome |
| [workspace-folders.md](workspace-folders.md) | Workspace root tree + git dirty hints in Files |
| [privacy.md](privacy.md) | No telemetry; local data |
| [network.md](network.md) | Offline mode; what can call out |
| [safe-mode.md](safe-mode.md) | Disable user mods for troubleshooting |
| [updates.md](updates.md) | Manual FOSS update check (releases URL) |
| [fullscreen-content.md](fullscreen-content.md) | F11 chrome-hidden content focus |
| [session-restore.md](session-restore.md) | Tabs/windows on launch preference |
| [split-view.md](split-view.md) | Experimental side-by-side tabs |
| [security.md](security.md) | Trust model and UI signals |
| [perf.md](perf.md) | Lazy modules and cold-start probe |
| [ci.md](ci.md) | Local check script and CI on Origin |
| [ui-commands.md](ui-commands.md) | Command Manager and toolbars |
| [keybindings.md](keybindings.md) | Chords and overrides |
| [multi-window.md](multi-window.md) | Per-window state vs shared prefs |
| [downloads.md](downloads.md) | Download disposition |
| [browser-shell.md](browser-shell.md) | WebView downloads (Linux vs WebView2 gap) |
| [extension-host.md](extension-host.md) | VSIX stub vs full JS host |
| [ffmpeg-sidecar.md](ffmpeg-sidecar.md) | Media sidecar pool |
| [api-gap-upstream.md](api-gap-upstream.md) | Upstream API gaps |

## QA and report visuals

| Doc | Purpose |
| --- | --- |
| [blitz-status.md](blitz-status.md) | Living Wed report: shipped, gaps, runbook, known bugs |
| [qa-desktop.md](qa-desktop.md) | Manual desktop checklist |
| [ux-a11y.md](ux-a11y.md) | Keyboard map, motion, hit targets, focus mode |
| [wasm-t2-status.md](wasm-t2-status.md) | T2 WASM loader plan and proving stub |
| [pdf-annotate.md](pdf-annotate.md) | PDF highlight sidecar (stub) |
| [report-visuals.md](report-visuals.md) | Wednesday report screenshots and capture script |
| [ocr-sidecar.md](ocr-sidecar.md) | Planned Tesseract OCR sidecar (UI stub today) |

## Architecture decision records (ADRs)

ADRs are numbered in `docs/adr/`. Read the ones that touch your area before large changes. New ADRs: copy [0000-template.md](adr/0000-template.md) (includes **Principles check** aligned with [principles.md](principles.md)).

| [0000](adr/0000-template.md) | ADR template (+ Principles check) | Use for new ADRs |
| ADR | Title | Implemented (blitz) |
| --- | --- | --- |
| [0001](adr/0001-shell-and-rendering.md) | Shell and rendering | `crates/unibrowser-desktop`, [architecture.md](architecture.md) |
| [0002](adr/0002-mod-system-and-sandboxing.md) | Mod system and sandboxing | `crates/unibrowser-mod-host`, [mod-system.md](mod-system.md) |
| [0003](adr/0003-command-handler-enablement.md) | Command handler enablement | `crates/unibrowser-engine`, `mods/base-*` |
| [0004](adr/0004-s-key-command-palette.md) | Command palette | `crates/unibrowser-desktop/src/palette.ts`, [ui-commands.md](ui-commands.md) |
| [0005](adr/0005-context-modes-and-toolbars.md) | Context modes and toolbars | `crates/unibrowser-engine`, [toolbars.md](toolbars.md) |
| [0006](adr/0006-mod-manifest-and-versioning.md) | Mod manifest and versioning | `crates/unibrowser-mod-api`, bundled `mods/` |
| [0007](adr/0007-licensing.md) | Licensing | [NOTICE.md](../NOTICE.md), [licenses/](licenses/) |
| [0008](adr/0008-api-gap-upstream-requests.md) | API gap upstream requests | [api-gap-upstream.md](api-gap-upstream.md) |
| [0009](adr/0009-browser-as-unified-shell.md) | Browser as unified shell | `crates/unibrowser-desktop`, [chrome.md](chrome.md) |
| [0010](adr/0010-ui-commands-and-command-manager.md) | UI commands and Command Manager | `crates/unibrowser-desktop/src/commandManager.ts`, [ui-commands.md](ui-commands.md) |
| [0011](adr/0011-lazy-conversion-native-until-encode.md) | Lazy conversion native until encode | `mods/base-pdf`, [perf.md](perf.md) |
| [0012](adr/0012-context-switching-ux-benchmarks.md) | Context switching UX benchmarks | Command Manager pin UX, [chrome.md](chrome.md) |
| [0013](adr/0013-dual-audience-foss-and-everyday-users.md) | Dual audience FOSS and everyday users | [onboarding.md](onboarding.md), grandma backup in Settings |
| [0014](adr/0014-vsix-compatibility-layer.md) | VSIX compatibility layer | `crates/unibrowser-desktop/src-tauri/src/vsix_*`, [extension-host.md](extension-host.md) |
| [0015](adr/0015-native-project-format.md) | Native project format (`.unibrow`) | `crates/unibrowser-project`, `mods/base-unibrow` |
| [0016](adr/0016-foss-ux-benchmarks.md) | FOSS UX benchmarks | [ux-a11y.md](ux-a11y.md), [getting-started.md](getting-started.md) |
| [0017](adr/0017-scripting-engine-rhai.md) | Scripting engine (Rhai) | `crates/unibrowser-rhai`, palette Rhai demo |
| [0018](adr/0018-extension-host-stub.md) | Extension host stub (not a full VSIX JS host) | [extension-host.md](extension-host.md), diagnostics VSIX list |
| [0019](adr/0019-wasm-t2-sandbox.md) | WASM T2 sandbox (design + feature stub) | ADR + feature flag stub in mod-host (no production WASM mods) |
| [0020](adr/0020-linux-download-open-as-tab.md) | Linux web download → fetch → file tab | `webview.rs` + `handle_download_url`; [browser-shell.md](browser-shell.md) |
| [0021](adr/0021-diagnostics-host-lsp.md) | Diagnostics host + LSP client | `crates/unibrowser-diagnostics` |
| [0022](adr/0022-proofing-as-mods.md) | Proofing as mods (spell, grammar, format checkers) | `mods/spell-en-nz`, `mods/grammar-harper`, [proofing-cross-format.md](proofing-cross-format.md) |
| [0023](adr/0023-saving-is-sacred.md) | Saving is sacred; local autosave | [principles.md](principles.md), `crates/unibrowser-fs` |
| [principles.md](principles.md) | Product principles (local-first, sacred save) | ADRs 0021–0023 |

## Changelog

Release notes for the 36h blitz live in [CHANGELOG.md](../CHANGELOG.md) at the repo root.
