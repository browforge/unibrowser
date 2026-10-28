# Bundled Hunspell data (`en.aff`, `en-GB.dic`)

These files are copied from the **LibreOffice dictionaries** repository (British English
MySpell/Hunspell list maintained for LibreOffice and related projects).

- **Source:** https://github.com/LibreOffice/dictionaries/tree/master/en  
- **Files:** `en_GB.aff` → shipped as `en.aff`; `en_GB.dic` → shipped as `en-GB.dic`  
- **Upstream license:** GNU **LGPL** (see the header comments in `en.aff` and the project
  `README` / `LICENSE` files in that repository). Hunspell itself is under MPL / LGPL / GPL
  as noted upstream.

Unibrowser (Apache License 2.0) distributes these word lists as **separate data files**
used at runtime by the `spell-en-nz` diagnostics mod. They are not modified beyond
repackaging for `include_str!` bundling. This attribution satisfies upstream notice
requirements; retain this file when updating the dictionary snapshot.

**Update procedure:** Replace `en.aff` and `en-GB.dic` from a tagged LibreOffice
dictionaries release; run `cargo test -p spell-en-nz` and desktop proofing tests. Do not
add capture-fixture or demo vocabulary to the word list.
