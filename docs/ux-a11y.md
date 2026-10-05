# UX accessibility notes (desktop shell)

Target: keyboard-first use alongside mouse; WCAG-minded focus and labels. Full audit is out of scope for the blitz.

## Reduced motion

| Source | Behaviour |
| --- | --- |
| Settings → **Reduce motion** = **System** | Honors `prefers-reduced-motion: reduce` from the OS; updates live when the OS setting changes |
| **On** | Forces reduced motion via `data-reduce-motion="true"` on `<html>` |
| **Off** | Sets `data-reduce-motion="false"` so CSS animations/transitions stay enabled even if the OS prefers reduce |

CSS shortens animations, transitions, and scroll smoothing when reduced motion is active ([`styles.css`](../crates/unibrowser-desktop/src/styles.css)).

## Larger hit targets

Settings → Accessibility → **Larger hit targets** sets `data-large-targets="true"` on `<html>` and increases minimum sizes on tabs, nav buttons, Command Manager chips, and context tabs. Independent of **Focus mode** (Appearance).

## Focus mode (Appearance)

**Focus mode** compacts chrome for power users: smaller tabs, tighter Command Manager, hides the mods line. Does not change content webview zoom.

## Command Manager tabs

| Key | Action |
| --- | --- |
| Tab / Shift+Tab | Move focus into/out of the tab row |
| Arrow Left / Right (or Up / Down) | Roving focus between context tabs (`Text`, `Code`, `Extensions`, …) |
| Home / End | First / last tab |
| Enter / Space | Activate focused tab (loads its command chips) |

Command chips in the active tab panel are regular buttons with `aria-label` from the command title.

## Command palette

| Key | Action |
| --- | --- |
| Chord (default Ctrl+Shift+P) | Open palette |
| Escape | Close palette (does not run a command) |
| Type | Filter results |
| Enter | Run highlighted command |

Palette is `role="dialog"` with `aria-modal="true"`.

## Chrome toolbar

Nav buttons use `aria-label` from the same strings as tooltips (en-NZ `t()` via `applyChromeI18n()`).

## Focus rings

`:focus-visible` outlines use accent blue; high-contrast mode uses a thicker green outline ([Settings accessibility](qa-desktop.md)).

## Gaps

- Monaco editor: no native spellcheck; see experimental mirror in Advanced settings (also DOCX preview + EPUB notes when enabled).
- Embedded webview content: inherits platform WebView focus; not fully styled by shell CSS.
- Floating Command Manager bars: drag handle is visual only; keyboard reposition not implemented.

## Related

- [qa-desktop.md](qa-desktop.md)
- [ADR 0016 FOSS UX benchmarks](adr/0016-foss-ux-benchmarks.md)
