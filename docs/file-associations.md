# System file associations (future packaging)

Unibrowser is built to **open** local paths and `file://` URLs you give it (omnibox, drag-drop, session restore, Places). It does **not** register itself as the OS default application for file types or protocols in this blitz.

## Today

| Mechanism | Behaviour |
| --- | --- |
| User action | You open files inside Unibrowser; the shell picks a bundled `base-*` mod or shows the mod portal for unknown extensions. |
| OS “Open with” | Works only if the user or distro explicitly chooses Unibrowser; we do not ship `.desktop` `MimeType=` defaults or Windows `ProgId` handlers yet. |
| `https://` | Embedded webview tabs; not a default browser claim. |

Bundled format handlers stay in the app. Disabling a **user** native mod is per-mod in the mod portal (see [safe-mode.md](safe-mode.md)); that is separate from claiming system defaults.

## FOSS packaging later

When installers land ([packaging.md](packaging.md)), association work should be **opt-in** and documented:

- **Linux:** optional `.desktop` file with `MimeType=` and `x-scheme-handler/file` only if maintainers want it; respect XDG defaults and document how to unset.
- **Windows:** optional installer checkbox for “Register Unibrowser for supported file types”; use Known Folder and registry patterns without hijacking unrelated ProgIds.
- **macOS:** `CFBundleDocumentTypes` / UTI entries when signing exists.

Until then, release notes should state clearly: **Unibrowser can open many formats but does not replace your system default handlers.**

## Related

- [packaging.md](packaging.md)
- [mod-system.md](mod-system.md)
- [safe-mode.md](safe-mode.md)
