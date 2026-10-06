# Network (local-first)

Unibrowser does not phone home. Outbound traffic happens only when **you** do something that needs the network.

## Offline mode (Settings → Network)

When **Offline mode** is on:

| Action | Allowed? |
| --- | --- |
| Open VSX search / VSIX download | No |
| Bundled mod catalog filter (offline list) | Yes (local) |
| Omnibox `https://` / `http://` navigation | Yes (you navigated) |
| In-app download fetch / WebView downloads | Yes (you started the download) |
| Settings **Check for updates** (opens releases URL) | Yes (you clicked) |
| Report issue / external links from About | Yes (you clicked) |

Turn offline mode off to use Open VSX; you still must tick **Allow network** in the mod portal before each catalog search.

## Open VSX and extensions

- **Search** calls `https://open-vsx.org/api/-/search` only with mod-portal opt-in and offline mode off.
- **Install from search** downloads the `.vsix` over `https` with the same gates.
- **Sideload** from disk does not use Open VSX.

## Web and proxy

- Remote web tabs behave like a normal browser.
- Manual proxy prefs apply when new content webviews are created ([network-proxy.md](network-proxy.md)).

## Not used

- No analytics, crash upload, or background update checker.
- No cloud autosave or account sync.

See also [privacy.md](privacy.md).
