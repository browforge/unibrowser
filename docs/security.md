# Security and trust model (blitz)

Unibrowser is a **local-first** shell. Treat network and script content as untrusted unless you explicitly trust the source.

## Trust tiers (practical)

| Tier | Examples | Shell behaviour |
| --- | --- | --- |
| **Local files** | `file://` under your home, project trees | Normal handlers; path sandbox blocks `..` escapes |
| **Internal pages** | `unibrowser://settings`, `unibrowser://downloads` | Bundled UI only; no remote fetch |
| **Remote web** | `https://` tabs | Isolated content webview; **Remote web content** chip in chrome |
| **HTML previews** | DOCX/EPUB spine HTML | Rendered in the shell DOM; **remote HTML** banner; no script execution in preview |
| **User scripts** | `.rhai`, `.js` script viewer | Rhai allowlist; optional document commands behind a checkbox |

## UI signals

- **Trust banners** (`trust-banner`) on DOCX/EPUB previews and script sources.
- **Remote web** label on the mods line when the active tab is HTTP(S).
- **Click to play** on video/audio: no autoplay until you press the overlay (blocks drive-by media).

## Downloads

- `javascript:` and `data:` download URLs are rejected.
- Download history records success/failure; retry reuses `handle_download_url`.

## Not a sandbox (yet)

- Native mods (`cdylib`) and Rhai run with host access per current ADRs.
- VSIX extension host is stubbed; no arbitrary JS extension execution.

## Related

- [ADR 0002](adr/0002-mod-system-and-sandboxing.md)
- [downloads.md](downloads.md)
- [browser-shell.md](browser-shell.md)
