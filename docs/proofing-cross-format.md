# Cross-format proofing (squiggles)

Offline spelling and grammar are **mods** on the generic diagnostics host (`unibrowser-diagnostics`). See [ADR 0021](adr/0021-diagnostics-host-lsp.md) and [ADR 0022](adr/0022-proofing-as-mods.md).

## Coverage

| Surface | Spelling | Grammar / lint | Adapter |
|--------|----------|----------------|---------|
| **Live** | Debounced `proofing_check` + squiggles | Harper via `grammar-harper` + `harper-ls` when installed | Host + mods |
| Monaco text / code | **Live** | Comments/strings in code only | `proofing.ts` markers |
| Markdown preview | Partial | Partial | Same engine, preview DOM TBD |
| DOCX preview | Partial | Partial | Selection / preview hook |
| SVG `<text>` | Partial | — | Export / edit path |
| `.unibrow` title | Partial | — | `unibrow-checker` mod (timeline locator) |
| DXF TEXT/MTEXT | Partial | Layer-0 lint stub | `dxf-checker` mod |
| Web forms | Partial | — | Injected field classifier (`web_fields`) |

## Commands

| Command | Default binding | Role |
|---------|-----------------|------|
| `unibrowser.proofing.toggle` | (palette) | Enable/disable squiggles globally |
| `unibrowser.proofing.nextProblem` | F8 | Next diagnostic in Monaco |
| `unibrowser.proofing.previousProblem` | Shift+F8 | Previous diagnostic |

## Dictionary files

| Scope | Path |
|-------|------|
| Global added | `{config}/proofing-global-added.txt` |
| Global ignored | `{config}/proofing-global-ignored.txt` |
| Workspace added | `{workspace}/.unibrowser/proofing-added.txt` |
| Workspace ignored | `{workspace}/.unibrowser/proofing-ignored.txt` |

Precedence: workspace **over** global **over** bundled Hunspell (`spell-en-nz` mod).

## Tests

- `crates/unibrowser-diagnostics/tests/lsp_client_tests.rs` (stub LSP server)
- `crates/unibrowser-diagnostics/tests/dictionary_tests.rs`
- `crates/unibrowser-diagnostics/tests/locator_mod_tests.rs` (DXF + timeline locators)
- `crates/unibrowser-diagnostics/tests/web_adapter_tests.rs` (password skip)
- Legacy: `crates/unibrowser-proofing/tests/*` (transitional)
