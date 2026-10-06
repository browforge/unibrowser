# VQA Monaco evidence

Regenerate with:

```sh
./scripts/capture-vqa-monaco.sh
```

These PNGs use the real Monaco DOM and token renderer. Capture does not insert
a text mirror. The capture exits nonzero unless the encoded PNG contains the
expected theme background plus visible text pixels in the editor rectangle.

See `docs/design/visual-qa-2026-10-06.md` for root-cause evidence and pixel
counts.
