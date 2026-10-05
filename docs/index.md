# Unibrowser documentation

Markdown docs for contributors and maintainers. The product README stays the entry point for **running** the app; this index links design records, guides, and QA.

## New contributors (FOSS)

1. Read [getting-started.md](getting-started.md) for clone, build, test, and where to send changes.
2. Skim [vision.md](vision.md) and [architecture.md](architecture.md) so engine / mod / shell boundaries stay clear.
3. Follow [docs/CONTRIBUTING.md](CONTRIBUTING.md) (build, checks, proving mods, PR expectations) and [CONTRIBUTING.md](../CONTRIBUTING.md) (community and asset policy).
4. Run `./scripts/smoke-desktop-qa.sh` before you open a PR.

## Architecture and product

| Doc | Purpose |
| --- | --- |
| [vision.md](vision.md) | Goals: unified file + web shell, mod-driven formats |
| [architecture.md](architecture.md) | Crate layout, data flow |
| [roadmap.md](roadmap.md) | Phase plan |
| [mod-system.md](mod-system.md) | Bundled mods and portal |
| [mod-api-stable.md](mod-api-stable.md) | Frozen mod API 0.1 |
| [plugin-sdk.md](plugin-sdk.md) | Hello-world `.ubmod` build and install |
| [plugin-sdk/README.md](plugin-sdk/README.md) | Native mod SDK ABI and user `mods/` dir |
| [commands-cross-format.md](commands-cross-format.md) | Font-size, find, and page zoom per viewer |
| [commands-undo-redo.md](commands-undo-redo.md) | Undo/redo and PDF export matrix |
| [packaging.md](packaging.md) | Planned MSI/NSIS/AppImage/deb artefacts |
| [chrome.md](chrome.md) | Tabs, omnibox, status bar chrome |
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
| [ux-a11y.md](ux-a11y.md) | Keyboard map and focus notes |
| [report-visuals.md](report-visuals.md) | Wednesday report screenshots and capture script |
| [ocr-sidecar.md](ocr-sidecar.md) | Planned Tesseract OCR sidecar (UI stub today) |

## Architecture decision records (ADRs)

ADRs are numbered in `docs/adr/`. Read the ones that touch your area before large changes.

| ADR | Title |
| --- | --- |
| [0001](adr/0001-shell-and-rendering.md) | Shell and rendering |
| [0002](adr/0002-mod-system-and-sandboxing.md) | Mod system and sandboxing |
| [0003](adr/0003-command-handler-enablement.md) | Command handler enablement |
| [0004](adr/0004-s-key-command-palette.md) | Command palette |
| [0005](adr/0005-context-modes-and-toolbars.md) | Context modes and toolbars |
| [0006](adr/0006-mod-manifest-and-versioning.md) | Mod manifest and versioning |
| [0007](adr/0007-licensing.md) | Licensing |
| [0008](adr/0008-api-gap-upstream-requests.md) | API gap upstream requests |
| [0009](adr/0009-browser-as-unified-shell.md) | Browser as unified shell |
| [0010](adr/0010-ui-commands-and-command-manager.md) | UI commands and Command Manager |
| [0011](adr/0011-lazy-conversion-native-until-encode.md) | Lazy conversion native until encode |
| [0012](adr/0012-context-switching-ux-benchmarks.md) | Context switching UX benchmarks |
| [0013](adr/0013-dual-audience-foss-and-everyday-users.md) | Dual audience FOSS and everyday users |
| [0014](adr/0014-vsix-compatibility-layer.md) | VSIX compatibility layer |
| [0015](adr/0015-native-project-format.md) | Native project format (`.unibrow`) |
| [0016](adr/0016-foss-ux-benchmarks.md) | FOSS UX benchmarks |
| [0017](adr/0017-scripting-engine-rhai.md) | Scripting engine (Rhai) |
| [0018](adr/0018-extension-host-stub.md) | Extension host stub (not a full VSIX JS host) |
| [0019](adr/0019-wasm-t2-sandbox.md) | WASM T2 sandbox (design + feature stub) |

## Changelog

Release notes for the 36h blitz live in [CHANGELOG.md](../CHANGELOG.md) at the repo root.
