# Desktop manual QA

Automated gate:

```bash
./scripts/smoke-desktop-qa.sh
```

The script runs `cargo test --workspace` and `npm run build`, then prints the checklist for a Wednesday-style report.

## Spellcheck

Monaco in the desktop shell does not ship with a spellchecker extension. Enabling real spellcheck needs either a Monaco contribution (dictionary + markers) or moving plain prose to a native control. **Deferred** until extension-host or a small bundled dictionary mod is justified.
