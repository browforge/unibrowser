# REC capture audit notes

## Document card format label (spell-test.txt)

Plain `.txt` files now use **Plain text** in the document card footer (`documentCardStatus.ts`) and open in Monaco **source** mode (`syncDocumentEditorForTextTab`), matching the status bar `plaintext` language.

## Rich-text toolbar on plain text

**Design question (not changed in this pass):** The compact document card still mounts the shared command-manager strip (paragraph/block picker) for text tabs. For plain `.txt` in source mode, that strip is largely markdown-oriented. Product should decide whether to hide the block picker entirely for non-markdown paths or show a reduced plain-text toolbar (save, find, proofing only). Until that decision, captures rely on the corrected footer label and source editor surface rather than removing chrome that other text tabs may still need.

## Spell squiggles and Problems count

Monaco marker squiggles use inline `ub-proofing-*` decorations plus capture-mode CSS so WebKitGTK snapshots show a wavy underline under flagged words. `proofing_check` hits populate the Problems store; `syncProblemsStatusBar()` keeps the status bar count in sync even when the bottom panel is closed.

REC pixel gates sample the `teh` word box and diff line-4 row using layout regions recorded at capture time; pass/fail is on pixel color only, not on sidecar diagnostic counts.
