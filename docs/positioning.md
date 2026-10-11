# How Unibrowser is different

Unibrowser is a **local-first browser for files and the web**: one tab strip for folders, `https://` pages, and the documents you open from them. It is **not** trying to be a single install that clones every screen of Microsoft Office or Adobe Creative Suite the way LibreOffice or an all-in-one Craft-style suite does.

Instead, Unibrowser is a **stable platform** with a small core and **plugins** that add what you need: file formats, panels, commands, toolbars, scripts, and services. The community ships features; the core keeps them **in the same places** and behaving **the same way** wherever the underlying content allows.

## Platform, not a frozen suite

| Monolithic suite clone | Unibrowser |
| --- | --- |
| One vendor ships every feature | Core ships shell, commands, trust, and Save |
| Features tied to app boundaries | Plugins register capabilities into fixed slots |
| Same tool reimplemented per app | Tools target **content kinds**, not just file extensions |
| Upgrade means waiting for the suite release | Install or remove a plugin from the Mod Portal |

You can still use many formats out of the box via bundled `base-*` mods. Anything beyond that is optional, inspectable, and uninstallable.

## Content kinds, not just extensions

File extensions tell you how a file is stored on disk. They do not tell you what you are editing **right now**.

Unibrowser uses **content kinds**: image, plain text run, rich text, audio clip, video frame, vector shape, table, 3D mesh, and so on. Format plugins expose their documents as trees of these nodes, including **embedded** content (a picture inside a word processor file, a frame in a video project, an image in Markdown).

**Example — colour profile on an image**

A plugin provides “Assign colour profile” for the **image** content kind. That control appears in the same family and tab position whether you selected:

- a standalone PNG on disk,
- a photo embedded in a DOCX,
- a still frame from a video timeline, or
- an inline image in a Markdown note.

You edit in place. You do not export to a separate image editor and re-import unless you choose to.

When the **host format** cannot store a profile in the file, the control stays visible: **supported** if the backend can persist it, **emulated** if the effect is baked into pixels or markup, or **unsupported** with a short reason — never silently removed.

## Local-first and Save

- No account and no cloud are required to edit.
- Network use is opt-in (opening URLs, browsing extension catalogues when you allow it).
- **Save, Save As, and autosave always work** for the current editor state. Spellcheck, validation, and capability gaps may warn or grey out tools; they do not refuse save.

## Consistent chrome, modular guts

The Command Manager, docks, palette, and status bar use **fixed slots**. Plugins contribute families, groups, toolbars, panels, and status items into those slots — they do not redraw the shell or park controls over your document.

Grey-out-not-hide applies everywhere: if a tool cannot run on the current selection, it stays where you expect it and explains why.

## Who it is for

- **Everyday use:** open common files from the same browser-like shell; install a mod when the portal says a format or tool is missing.
- **Power users and FOSS contributors:** build or sideload plugins, compose APIs ([ADR 0025](adr/0025-mod-apis-and-dependencies.md)), and keep the core small.

For philosophy and decision rules, see [principles.md](principles.md). For technical contracts, see [ADR 0028](adr/0028-platform-content-kinds-and-plugin-scope.md) and [command-system.md](design/command-system.md).
