# Terminal panel (stub)

## UI

The bottom panel next to the status bar has two tabs:

| Tab | State |
| --- | --- |
| **Output** | Active. Rhai palette demo and script host lines append here (`unibrowser-script-output` event). |
| **Terminal** | Disabled (grey). Label shows **Terminal** with a “planned” tooltip. |

Close the panel with **×**; Rhai output re-opens it on the Output tab.

## Why no PTY yet?

A real integrated terminal needs a pseudo-terminal, scrollback, signal handling, and safe cwd/env per window. That is more than a blitz stub. Prefer your system terminal for now.

## Future direction

- Optional tab: connect to `portable-pty` or platform PTY behind a Tauri command stream.
- Respect workspace root as default cwd when workspace folders are pinned.
