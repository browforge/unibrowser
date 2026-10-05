# Safe mode (disable user mods)

Use safe mode when a **user-installed** native mod or dev cdylib misbehaves. Bundled `base-*` format mods still load so text, PDF, and web tabs keep working.

## Launch flag (one session)

```bash
UNIBROWSER_SAFE_MODE=1 npm run tauri dev
```

Or for a packaged binary:

```bash
UNIBROWSER_SAFE_MODE=1 ./Unibrowser
```

The flag applies to the current process only. User mods under `~/.config/unibrowser/mods` (Linux) and `UNIBROWSER_LOAD_CDYLIB_MOD=1` dev loads are skipped.

## Settings (persistent)

1. Open `unibrowser://settings`.
2. Expand **Advanced**.
3. Enable **Disable all user mods (safe mode)**.
4. **Restart** Unibrowser.

The preference is stored in `preferences.json` as `disable_user_mods`.

## Verify

Open `unibrowser://diagnostics`. The environment section shows whether safe mode is on and whether the launch flag or settings preference applied.

## Per-mod enable (without safe mode)

When safe mode is **off**, the mod portal lists **Native mods** and **VSIX** rows with **Enable** / **Disable** per package. That toggles `state.json` (native) or VSIX prefs without turning off all user mods. The shell reloads the command registry when possible; if commands still look stale, restart once.

Safe mode (launch flag or Settings) blocks **all** user mod loads; per-mod toggles are disabled until you turn safe mode off.

## What safe mode does not do

- It does not disable bundled mods, VSIX metadata, or Rhai palette demos wired in the shell.
- It does not block https or file access.
- It is not a sandbox; see [mod-system.md](mod-system.md) and [ADR 0002](adr/0002-mod-system-and-sandboxing.md).
