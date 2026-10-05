# Web developer tools (FOSS debugging)

Embedded **https** tabs use a per-window content webview (WebKit on Linux, WebView2 on Windows). For FOSS contributors debugging layout, network, and console output:

## Toggle inspector

| Action | How |
| --- | --- |
| **Keyboard** | **F12** (default chord for `unibrowser.web.devtools.toggle`) |
| **Palette** | `Ctrl+Shift+P` → **Toggle developer tools** |
| **Stay open** | Settings → Advanced → **Embedded web inspector** keeps tools open when you navigate |

The command only runs when the active tab is an **https** page (greyed in the palette on file or internal pages).

## What it inspects

- The **content** webview (`https://…`), not the Unibrowser chrome (tabs, Command Manager, Monaco).
- Chrome UI debugging still uses the desktop toolkit / Tauri dev build when you run `npm run tauri dev`.

## Privacy

Developer tools can reveal cookies, storage, and page scripts. They do not send telemetry; everything stays on your machine.
