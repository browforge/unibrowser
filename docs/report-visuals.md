# Wednesday report visuals

Use this when you need screenshots or short clips for a weekly status post. Pair with the capability table in [README.md](../README.md#what-works-today-wed-report-draft).

## Automated gate first

```bash
./scripts/smoke-desktop-qa.sh
```

Only capture visuals after tests and `npm run build` pass.

## Recommended shots (minimum set)

Capture at **1920×1080** or your laptop native resolution; use light or dark theme consistently across the set.

| # | Scene | How to open | What to show |
| --- | --- | --- | --- |
| 1 | Home / NTP | New tab (+) | Hero, recent, browse actions |
| 2 | Files + formats | Open a folder tab | Breadcrumbs, file-type badges, one PDF or image thumb |
| 3 | Command palette | Ctrl+Shift+P | Fuzzy list with a chord visible |
| 4 | Settings | `unibrowser://settings` | Theme + one distinctive pref (accessibility or checklist) |
| 5 | Project | New project → Notes template | Board cards + timeline strip |
| 6 | Viewer | Open EPUB or DOCX | In-document search or chapter nav |

Name files `unibrowser-01-ntp.png`, `unibrowser-02-files.png`, and so on.

## Capture script (Linux)

Optional helper when a graphical session is available (`$DISPLAY` set):

```bash
./scripts/capture-report-screenshots.sh --guide-only
```

Prints the shot list and paths without starting the app or requiring a display.

Interactive capture (graphical session, `$DISPLAY` set):

```bash
./scripts/capture-report-screenshots.sh
```

The script does **not** drive the UI automatically. It prints the shot list, waits for you to arrange each scene, then calls `scrot` (or `import` from ImageMagick) into `./report-screenshots/` (gitignored).

Install tools if needed:

```bash
sudo apt-get install -y scrot   # or imagemagick
```

## Headless / CI

Do not rely on screenshots from CI. For agents or VMs without a display, record steps in [qa-desktop.md](qa-desktop.md) and attach captures from a local `tauri dev` session.

## Video (optional)

A 30–60s screen recording covering: open app → folder tab → open file → palette → settings is enough for most reports. Keep the cursor steady and avoid unrelated windows.
