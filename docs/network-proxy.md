# Network proxy (web tabs)

## Settings

**Settings → Advanced → Web tab proxy** (`preferences.json` → `web_proxy`):

| Mode | Applied in shell |
| --- | --- |
| **System** | No override (OS / WebView default). |
| **Direct** | No `proxy_url` on the content webview (not a forced bypass on all platforms). |
| **Manual** | `http://host:port` via Tauri `WebviewBuilder::proxy_url` when the content webview is **created**. |

## Platform notes

| Platform | Mechanism | Limits |
| --- | --- | --- |
| Windows (WebView2) | WRY maps manual proxy to `--proxy-server=…` | SOCKS/auth/PAC not implemented; existing https tabs keep old proxy until recreated |
| Linux (WebKitGTK) | Tauri `proxy_url` | Same recreation caveat; system mode still uses OS env |
| macOS | `proxy_url` needs macOS 14+ / feature flags in upstream Tauri | Not verified in blitz |

Changing proxy prefs does **not** hot-reload open https tabs; open a new tab or restart the window to pick up manual proxy on new webviews.

## Workarounds

- Set OS proxy before launch when using **System**.
- Corporate PAC / NTLM proxies remain out of scope.
