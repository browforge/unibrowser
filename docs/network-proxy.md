# Network proxy (web tabs)

## Settings

**Settings → Advanced → Web tab proxy** stores how https tabs should reach the network:

| Mode | Meaning |
| --- | --- |
| **System** | Use the OS proxy (default). |
| **Direct** | No proxy. |
| **Manual** | Host + port fields (persisted when mode is Manual). |

Values live in `preferences.json` under `web_proxy`.

## Applied in the shell?

**Not in the blitz build.** Tauri / WRY content webviews are created without a per-tab proxy override in `webview.rs`. Until a supported API exists on each platform (WebView2, WebKitGTK, WKWebView), changing this setting only updates prefs and diagnostics expectations.

## Workarounds

- Set the system proxy before launching Unibrowser (mode **System**).
- Use **Direct** when you need to confirm traffic is not using a stale manual config (prefs only today).
- Corporate proxies that require auth or PAC files are out of scope for this stub.
