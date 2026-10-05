# Terminal panel

## UI

Bottom dock (shared with **Output**):

| Tab | Behaviour |
| --- | --- |
| **Output** | Rhai / script host lines (`unibrowser-script-output`) |
| **Terminal** | **xterm.js** + `portable-pty` shell |

- **Toggle:** palette **Toggle terminal panel** or **Ctrl+`** (default).
- **Resize:** drag the top edge of the bottom panel (grip).
- **Shell:** PowerShell (`-NoLogo`) on Windows; `$SHELL` or `/bin/bash` on Linux/macOS.
- Default cwd: app workspace directory when the session starts.

## Backend

- Crate: `unibrowser-terminal` (PTY tests in crate).
- Tauri: `terminal_spawn`, `terminal_write`, `terminal_resize`; `terminal-output` events.

## Limits

- One PTY session per app instance (blitz).
- No split panes, no SSH profiles, no ConPTY-specific features beyond `portable-pty`.
