# UI fonts and icons (desktop shell)

## Source Sans 3

Copyright 2010–2023 Adobe. Licensed under the [SIL Open Font License 1.1](https://scripts.sil.org/OFL).

Bundled file: `crates/unibrowser-desktop/public/fonts/source-sans-3-400.ttf`

## Source Code Pro

Copyright 2010–2023 Adobe. Licensed under the [SIL Open Font License 1.1](https://scripts.sil.org/OFL).

Bundled files: `source-code-pro-400.ttf`, `source-code-pro-600.ttf` under `crates/unibrowser-desktop/public/fonts/`.

When Adobe OFL files cannot be fetched at build time, the repository may temporarily ship metric-compatible substitutes; run `./scripts/fetch-ui-fonts.sh` (network required) to replace them with official Source Code Pro binaries from Fontsource.

## Lucide icons

SVG path data in `crates/unibrowser-desktop/src/ui/lucideIcon.ts` is derived from [Lucide](https://lucide.dev/) (ISC licence). Icons are inlined in the app bundle; no CDN.
