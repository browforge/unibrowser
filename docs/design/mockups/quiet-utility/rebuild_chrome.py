#!/usr/bin/env python3
"""Apply the accepted compact chrome and activity rail to Quiet Utility studies."""

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent


def icon(path: str, label: str) -> str:
    accessibility = (
        f'role="img" aria-label="{label}"'
        if label
        else 'aria-hidden="true"'
    )
    return (
        f'<svg class="svg-icon" viewBox="0 0 24 24" {accessibility}>'
        f'{path}</svg>'
    )


ICONS = {
    "menu": '<path d="M4 6h16M4 12h16M4 18h16"/>',
    "file": '<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/>',
    "pdf": '<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/><path d="M8 13h2a2 2 0 0 1 0 4H8v-6"/><path d="M14 17v-6h2"/>',
    "project": '<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18M3 9h6M3 15h6"/>',
    "folder": '<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>',
    "search": '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>',
    "assets": '<path d="m12 2 7 4v8l-7 4-7-4V6z"/><path d="m5 6 7 4 7-4M12 10v8"/>',
    "settings": '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.38a2 2 0 0 0-.73-2.73l-.15-.09a2 2 0 0 1-1-1.74v-.51a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2Z"/><circle cx="12" cy="12" r="3"/>',
    "collapse": '<path d="m11 17-5-5 5-5M18 17l-5-5 5-5"/>',
    "undo": '<path d="M9 14 4 9l5-5"/><path d="M4 9h10a6 6 0 0 1 6 6v1"/>',
    "redo": '<path d="m15 14 5-5-5-5"/><path d="M20 9H10a6 6 0 0 0-6 6v1"/>',
    "back": '<path d="m15 18-6-6 6-6"/><path d="M9 12h10"/>',
    "forward": '<path d="m9 18 6-6-6-6"/><path d="M5 12h10"/>',
    "reload": '<path d="M20 6v5h-5"/><path d="M4 18v-5h5"/><path d="M18.5 9A7 7 0 0 0 6.2 6.2L4 8"/><path d="M5.5 15A7 7 0 0 0 17.8 17.8L20 16"/>',
}


def tab(kind: str, label: str, active: bool = False) -> str:
    cls = "title-tab active" if active else "title-tab"
    return (
        f'<div class="{cls}">{icon(ICONS[kind], "")}'
        f'<span>{label}</span><span class="close">x</span></div>'
    )


def chrome(context: str, location: str, tabs: list[tuple[str, str]], commands: list[str],
           browser_navigation: bool = False, save: bool = True) -> str:
    tab_html = "".join(tab(kind, label, i == 0) for i, (kind, label) in enumerate(tabs))
    navigation = ""
    if browser_navigation:
        navigation = "".join(
            f'<button class="title-icon" aria-label="{label}">{icon(ICONS[name], "")}</button>'
            for name, label in (("back", "Back"), ("forward", "Forward"), ("reload", "Reload"))
        )
    tools = "".join(f'<button class="tool">{command}</button>' for command in commands)
    save_line = (
        '      <button class="button primary title-save">Save</button>\n'
        if save
        else ""
    )
    navigation_line = f"      {navigation}\n" if navigation else ""
    return f"""<header class="titlebar" aria-label="Window and document navigation">
      <button class="title-icon app-menu" aria-label="Main menu" title="Main menu (Alt)">{icon(ICONS["menu"], "")}</button>
      <div class="title-tabs" role="tablist" aria-label="Open tabs">{tab_html}<button class="title-icon" aria-label="New tab">+</button></div>
{navigation_line}\
      <div class="command-centre" aria-label="Location and commands">{location}</div>
      <button class="title-icon" aria-label="Undo">{icon(ICONS["undo"], "")}</button>
      <button class="title-icon" aria-label="Redo">{icon(ICONS["redo"], "")}</button>
{save_line}\
      <button class="title-icon" aria-label="Layout and more options">...</button>
      <span class="drag-region" aria-hidden="true"></span>
      <div class="window-controls" aria-label="Window controls"><button aria-label="Minimise">-</button><button aria-label="Maximise">[]</button><button aria-label="Close">x</button></div>
    </header>
    <div class="command-manager" aria-label="{context} commands">
      <button class="context-label">{context}<span aria-hidden="true">v</span></button>
      <span class="separator"></span>{tools}
      <span class="spacer"></span>
      <span class="follow">Following: <strong>{context}</strong></span>
      <button class="pin-button">Pin</button>
      <button class="title-icon collapse-manager" aria-label="Collapse Command Manager">^</button>
    </div>"""


def rail(active: str) -> str:
    items = [
        ("Files", "folder"),
        ("Search", "search"),
        ("Project", "project"),
        ("Assets", "assets"),
        ("Settings", "settings"),
    ]
    rows = []
    for label, name in items:
        classes = ["rail-destination"]
        if label == active:
            classes.append("active")
        if label == "Settings":
            classes.append("settings")
        rows.append(
            f'<button class="{" ".join(classes)}" aria-label="{label}" title="{label}">'
            f'{icon(ICONS[name], "")}<span class="rail-label">{label}</span></button>'
        )
    rows.insert(
        -1,
        f'<button class="rail-destination rail-collapse" aria-label="Collapse activity rail" '
        f'title="Collapse activity rail">{icon(ICONS["collapse"], "")}</button>',
    )
    return '<nav class="workspace-rail collapsed" aria-label="Side panels">' + "".join(rows) + "</nav>"


CONFIG = {
    "document-problems-ai.html": dict(
        context="Text",
        location='<span class="crumb-muted">Projects / Field kit /</span><strong>sync-notes.ts</strong>',
        tabs=[("file", "sync-notes.ts"), ("pdf", "field-report.pdf")],
        commands=["Undo", "Redo", "A-", "14 pt", "A+", "Find", "Proofing", "Export"],
        active_rail="Files",
    ),
    "pdf-annotation.html": dict(
        context="PDF",
        location='<span class="crumb-muted">Documents / Site visit /</span><strong>field-report.pdf</strong>',
        tabs=[("pdf", "field-report.pdf"), ("file", "sync-notes.ts")],
        commands=["Select", "Highlight", "Note", "Area", "Page 3 of 12", "90%", "Export PDF"],
        active_rail="Files",
    ),
    "unibrow-create.html": dict(
        context="Create",
        location='<span class="crumb-muted">Projects / Workshop tour /</span><strong>workshop-tour.unibrow</strong>',
        tabs=[("project", "workshop-tour.unibrow"), ("folder", "Workshop media")],
        commands=["Undo", "Redo", "Import media", "Add title", "Split clip", "Timeline 80%", "Export video"],
        active_rail="Project",
    ),
    "new-tab.html": dict(
        context="Start",
        location='<span class="crumb-muted">Search, open a path, or enter a web address</span>',
        tabs=[("file", "New tab"), ("file", "sync-notes.ts")],
        commands=["Open file", "Open folder", "New project", "Recent files"],
        active_rail="Files",
        save=False,
    ),
    "multimodal-workshop.html": dict(
        context="CAD / 3D",
        location='<span class="crumb-muted">Projects / Ventilation retrofit /</span><strong>north-bay-bracket.dxf</strong>',
        tabs=[("project", "north-bay-bracket.dxf"), ("project", "workshop-tour.unibrow"), ("file", "bracket-check.ts")],
        commands=["Select", "Orbit", "Measure", "Section", "Add to timeline", "Capture view", "Export DXF"],
        active_rail="Project",
    ),
}


for filename, config in CONFIG.items():
    path = ROOT / filename
    source = path.read_text()
    replacement = chrome(**{k: v for k, v in config.items() if k != "active_rail"})
    source, count = re.subn(
        r'(?:<header class="menu">|<header class="titlebar").*?<section class="workspace">',
        replacement + '\n    <section class="workspace">',
        source,
        count=1,
        flags=re.DOTALL,
    )
    if count != 1:
        raise RuntimeError(f"Could not replace chrome in {filename}")
    source, count = re.subn(
        r'<nav class="workspace-rail[^"]*".*?</nav>',
        rail(config["active_rail"]),
        source,
        count=1,
        flags=re.DOTALL,
    )
    if count != 1:
        raise RuntimeError(f"Could not replace rail in {filename}")
    path.write_text(source)
