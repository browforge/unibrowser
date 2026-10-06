# VQA remediation captures (2026-10-06)

Current evidence for VQA-03 through VQA-12. Generated with `scripts/capture-vqa-*.sh`
after `./scripts/ensure-release-desktop.sh`.

Compare against `docs/design/mockups/quiet-utility/*.png` and
`docs/design/visual-language.md`.

VQA-15 (final matrix) is intentionally deferred to the GPT review pass.

## Command Manager and Problems correction

`cm-problems-{markdown,typescript,pdf}-{light,dark}.png` are the current
evidence for populated Command Manager controls and a three-row Problems table.
Regenerate with `./scripts/capture-vqa-cm-problems.sh`. The capture fails unless
at least six controls have non-zero control, label, and SVG boxes at 3:1
contrast, and unless all three Problems rows are visible inside the table.
