# ADR 0016: FOSS UX benchmarks (recent redesigns)

- **Status:** Accepted (design input; not implementation)
- **Date:** 2026-10-05

## Context

Bear’s direction: study **revamped FOSS UI/UX from roughly the last five years** (≈2021–2026) as evidence that **community-driven desktops and apps can ship excellent, modern interfaces** — often with clearer defaults, honest power-user customisation, and less “accountant UI” than some proprietary suites. Unibrowser should learn from **what shipped**, not from stereotypes about “old open source look”.

This ADR complements proprietary craft benchmarks in [ADR 0012](0012-context-switching-ux-benchmarks.md) (SOLIDWORKS, GIMP, Affinity, Blender, Resolve). Those teach **context switching and toolbars**; the cases below teach **defaults, theming, density, and discoverability** in FOSS that millions of users actually run.

**Scope:** documented, verifiable releases and official posts only. Product opinions below are **lessons for Unibrowser**, not reviews of entire distributions.

## Case studies (verified)

### Audacity — workflow polish (3.4) then Qt rebuild (4.0)

| When | What changed | Source |
|------|----------------|--------|
| **2023-11-02** | **Audacity 3.4** — consolidated **export** window (all options + native file browser), musical **beats/measures** grid, tempo/time signature, non-destructive **time stretch**; **new defaults** (e.g. Time Signature toolbar shown, solo button multi-track mode). | [Audacity 3.4 blog](https://audacityteam.org/blog/audacity-3-4/), [GitHub release notes](https://github.com/audacity/audacity/releases/tag/Audacity-3.4.0) |
| **2026** (4.0) | **Audacity 4** — interface rebuilt on **Qt**; toolbars/panels **dock, float, hide**; saved **Workspaces** (Modern, Classic, Music); **light / dark / high-contrast** themes, accent and clip colours; **Home** screen with recent project thumbnails; separate edit **modes removed** in favour of **context-sensitive** tools (e.g. clip gain, sample draw when zoomed). | [Audacity 4 product page](https://www.audacityteam.org/audacity-4/), [GitHub Audacity 4.0.0 release](https://github.com/audacity/audacity/releases/tag/Audacity-4.0.0) |

**Lessons for Unibrowser**

- **Export vs project** — one clear export surface (3.4) aligns with [ADR 0015](0015-native-project-format.md) / [ADR 0011](0011-lazy-conversion-native-until-encode.md): don’t bury delivery behind save.
- **Workspaces** — named layouts (4.0) parallel **Command Manager + `toolbars.json`** presets (Affinity-style studios, Audacity workspaces).
- **Context-sensitive tools** — fewer global “modes”, more behaviour from selection/zoom — analogue to **grey-out** and `poll()` rather than hiding the manager frame.
- **Theming** — accent + light/dark/HC as first-class settings, not afterthought.

### Pop!_OS / COSMIC — new Rust desktop (2024–2025)

System76 replaced a **GNOME extension–based** Pop!_OS with **COSMIC**, a **Rust, Wayland** desktop environment, after maintaining tiling on top of GNOME became unsustainable ([LWN on COSMIC stable](https://lwn.net/Articles/1050268/)).

| When | What changed | Source |
|------|----------------|--------|
| **2024** | **COSMIC alpha** (first public alpha **August 2024** per LWN); roadmap posts on **design matching**, display settings, workspaces across monitors. | [COSMIC: The Road to Alpha](https://system76.com/blog/post/cosmic-the-road-to-alpha/) (2024-01-11), [More Alpha, More Fun](https://blog.system76.com/blog/post/cosmic-more-alpha-more-fun) (2024-03-13), [LWN](https://lwn.net/Articles/1050268/) |
| **2025-09** | **COSMIC beta** / Pop!_OS 24.04 beta — COSMIC **Files** (gallery, previews, search), **Launcher** (apps, files, calculator, web search), tiling/stacking, panel layouts, theming. | [9to5Linux on COSMIC beta](https://9to5linux.com/system76-releases-cosmic-beta-desktop-environment-and-pop_os-24-04-lts-beta) |
| **2025-12-11** | **Pop!_OS 24.04 LTS** with **COSMIC Epoch 1** stable — modular DE, **panel + dock** placement, applets, accent/theming, workspace behaviour, COSMIC-native apps (Files, Terminal, Store, …). | [Pop!_OS letter from founder](https://system76.com/blog/post/pop-os-letter-from-our-founder), [LWN](https://lwn.net/Articles/1050268/) |

**Lessons for Unibrowser**

- **Stable shell, swappable parts** — COSMIC’s compositor + applets + apps mirrors **stable chrome / swapping manager tabs** ([ADR 0012](0012-context-switching-ux-benchmarks.md)).
- **Delay release for coherent defaults** — System76 slipped alpha to ship **MVP COSMIC apps** together ([More Alpha, More Fun](https://blog.system76.com/blog/post/cosmic-more-alpha-more-fun)); Unibrowser should not ship half-themed format mods without a consistent chrome story.
- **Cross-app theming** — `xdg-desktop-portal` accent/dark propagation ([More Alpha, More Fun](https://blog.system76.com/blog/post/cosmic-more-alpha-more-fun)) — reminder that **Monaco / WebView / native Rust** must share one token set where possible ([ADR 0001](0001-shell-and-rendering.md)).
- **Discoverability** — unified **Launcher** search (apps + files + utilities) — cousin of **command palette** ([ADR 0004](0004-s-key-command-palette.md)).

### Linux Mint / Cinnamon — incremental “everyday desktop” polish

Mint optimises for **familiar desktop users** (Windows-like affordances) with steady Cinnamon releases rather than a single Big Bang rewrite.

| When | What changed | Source |
|------|----------------|--------|
| **2022** | **Corner bar** applet (screen-edge hit target, peek desktop, drag-to-desktop); **Nemo** selection highlight cleanup (name only, not icon wash). | [Mint blog Oct 2022](https://blog.linuxmint.com/?p=4424) |
| **2023** | **Cinnamon “Styles”** — curated **mixed / dark / light** modes and **variants** (compatible theme bundles); removed monochrome icon compromise. | [Mint blog Apr 2023](https://blog.linuxmint.com/?m=202304) |
| **2024–2025** | **Cinnamon 6.4** default theme — **rounder, darker, more contrast**; **native Clutter dialogs** (session, polkit, display, …) for Wayland consistency; cleaner **Alt+Tab**, OSDs, notifications (ongoing in 6.4 cycle). | [Mint 22.1 Xia what’s new](https://www.linuxmint.com/rel_xia_cinnamon_whatsnew.php), [Mint blog Sep 2024](https://blog.linuxmint.com/?m=202409) |
| **2024** | **Mint 22 Wilma** — Nemo **actions layout editor** (context menu organisation); workspace switcher tweaks; keybinding search. | [Mint 22 what’s new](https://linuxmint.com/rel_wilma_whatsnew.php) |

**Lessons for Unibrowser**

- **Curated defaults beat theme dumps** — “Styles” = pick a **variant that works** without MIME-level theme homework — parallel to **grandma path** + Mod Portal ([ADR 0013](0013-dual-audience-foss-and-everyday-users.md)).
- **Density without noise** — darker/contrast theme and separated dialog buttons improve scanability without adding new features ([Xia what’s new](https://www.linuxmint.com/rel_xia_cinnamon_whatsnew.php)).
- **Power without modal overload** — corner bar + configurable actions = **optional** affordances; **keybinding search** in settings matches documenting `keybindings.json` for hackers.
- **Consistency across toolkit boundaries** — moving dialogs off Gtk to match shell — Unibrowser analogue: **one visual language** across Tauri chrome and embedded web surfaces.

## Decision — apply to Unibrowser chrome

| Theme | FOSS evidence | Unibrowser direction |
|-------|----------------|----------------------|
| **Density** | Mint 6.4 contrast/rounding; Audacity 4 consistent rotary controls | Command Manager: readable spacing scale; avoid ribbon-style vertical bloat; timeline may be denser than text tab |
| **Defaults** | Audacity 3.4 toolbar defaults; Mint Styles variants; COSMIC shipped with complete app set | Ship **working bundled `base-*`** + one **default workspace**; granny never edits JSON first |
| **Discoverability** | COSMIC Launcher; Audacity 4 Home; Mint corner bar (learn-once) | **Palette** + plain “Install support”; optional **recent projects** on home ([ADR 0015](0015-native-project-format.md)) |
| **Customisation** | Audacity workspaces; COSMIC panel/dock/applets; Mint Nemo actions editor | **`toolbars.json`**, float bars, pin context — **FOSS path**; same UI grandma uses for Mod Portal |
| **Theming** | Audacity 4 themes; COSMIC accent; Cinnamon Styles | Light/dark + **high contrast**; accent tokens shared shell ↔ Monaco ([ADR 0001](0001-shell-and-rendering.md)) |
| **Onboarding** | COSMIC delayed alpha for coherent apps; Audacity export in one window | Progressive disclosure ([ADR 0013](0013-dual-audience-foss-and-everyday-users.md)); **skippable** first-run only; no account wall; never block nth launch |

### Bear UX product rules (pet peeves)

Normative rules for Unibrowser chrome and growth features — complement [ADR 0013](0013-dual-audience-foss-and-everyday-users.md).

1. **Never get in the way** — Onboarding, tips, and “what’s new” may exist; they must **not block** work on the nth install or after factory reset. Always skippable; never forced every launch.
2. **Mod suggestions** — Install/recommend prompts are fine when they **do not obscure** the file, tab, or canvas the user was already using.
3. **Everything in its proper place** — Match FOSS lessons (single export window, native dialogs that look like the shell) and Unibrowser’s command/chrome model; avoid duplicate settings surfaces.
4. **User taste is always right** — If someone floats toolbars to another monitor or builds an odd layout, **persist and honour it** without lectures or automatic revert.

UX reviews score new UI against these four before shipping.

### What we do not copy blindly

- **Store / hub lock-in** — Audacity integrates **Muse Hub** for loops/effects ([3.4 blog](https://audacityteam.org/blog/audacity-3-4/)); Unibrowser keeps **Open VSX + sideload** ([ADR 0014](0014-vsix-compatibility-layer.md)), no required marketplace account.
- **Whole-desktop scope** — COSMIC is a DE; Unibrowser is a **unified browser shell** — borrow **launcher/search and theming**, not applets for battery/VPN.
- **Windows mimicry as goal** — Mint’s corner bar is intentional familiarity; Unibrowser’s primary metaphor is **browser + tabs** ([ADR 0009](0009-browser-as-unified-shell.md)).

## Consequences

- UX reviews add a **FOSS benchmark** check alongside SOLIDWORKS/GIMP criteria ([ADR 0012](0012-context-switching-ux-benchmarks.md)).
- Roadmap theming/workspace work should cite **workspace + theme** patterns from Audacity 4 and **Styles** from Cinnamon, implemented via existing chrome/command manager — not a second preferences app.
- Format/licensing research stays in the project research brief; **this ADR owns recent FOSS UX references**.

## Open questions

- Ship **named workspaces** (video vs office vs browse) as JSON presets in v1 or wait for project format ([ADR 0015](0015-native-project-format.md))?
- **Home / start surface** — empty tab vs recent `.unibrow` vs file picker only?
- **HC theme** — WCAG-driven tokens before or after first public desktop build?

## Related

- [ADR 0012](0012-context-switching-ux-benchmarks.md) — proprietary context-switching benchmarks
- [ADR 0013](0013-dual-audience-foss-and-everyday-users.md) — progressive disclosure
- [ui-commands.md](../ui-commands.md), [vision.md](../vision.md)
