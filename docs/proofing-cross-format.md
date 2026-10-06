# Cross-format proofing (squiggles)

One offline engine (`unibrowser-proofing`) powers spelling and grammar squiggles. See [ADR 0021](adr/0021-shared-proofing-engine.md).

Legend:

| State | Meaning |
| --- | --- |
| **Live** | Debounced `proofing_check` + squiggles |
| **Partial** | Engine ready; adapter not wired in UI yet |
| **Grey** | Surface skipped (security or `spellcheck=false`) |

## Coverage matrix

| Surface | Spelling | Grammar | Notes |
| --- | --- | --- | --- |
| Monaco text / code | **Live** | **Live** (comments/strings in code) | `proofing.ts` markers |
| Markdown preview | **Partial** | **Partial** | Source editor checked; preview HTML later |
| DOCX preview | **Partial** | **Partial** | Experimental browser spellcheck still available |
| SVG text | **Partial** | **Partial** | Adapter planned |
| `.unibrow` title / video text | **Partial** | **Partial** | Timeline fields planned |
| Web forms | **Partial** | **Partial** | `web_fields` classifier + inject script planned |
| Password / card / OTP fields | **Grey** | **Grey** | Never checked |

## Commands

| Command | Default binding | Effect |
| --- | --- | --- |
| `unibrowser.proofing.toggle` | (palette) | Enable/disable squiggles globally |
| `unibrowser.proofing.nextProblem` | F8 | Next diagnostic in Monaco |
| `unibrowser.proofing.previousProblem` | Shift+F8 | Previous diagnostic |

## Dictionaries

| Scope | Path |
| --- | --- |
| Global added | `{config}/proofing-global-added.txt` |
| Global ignored | `{config}/proofing-global-ignored.txt` |
| Workspace added | `{workspace}/.unibrowser/proofing-added.txt` |
| Workspace ignored | `{workspace}/.unibrowser/proofing-ignored.txt` |

Workspace entries override global for the same word.

## Tests

- `crates/unibrowser-proofing/tests/engine_tests.rs`
- `crates/unibrowser-proofing/tests/code_adapter_tests.rs`
- `crates/unibrowser-proofing/tests/web_adapter_tests.rs`
