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
    "type": '<path d="M4 7V4h16v3"/><path d="M9 20h6"/><path d="M12 4v16"/>',
    "copy": '<rect width="14" height="14" x="8" y="8" rx="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>',
    "find": '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>',
    "spell": '<path d="m4 16 6-12 6 12"/><path d="M6 12h8"/><path d="m16 19 2 2 4-4"/>',
    "export": '<path d="M15 3h6v6"/><path d="m10 14 11-11"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
    "pointer": '<path d="M22 14a8 8 0 0 1-8 8"/><path d="M18 11v-1a2 2 0 0 0-4 0V9a2 2 0 0 0-4 0V8a2 2 0 0 0-4 0v5l-1.7-1.7a2 2 0 0 0-2.8 2.8l5.4 5.4A8 8 0 0 0 12.5 22H14"/><path d="M18 11V9a2 2 0 1 1 4 0v5"/>',
    "orbit": '<circle cx="12" cy="12" r="3"/><path d="M3 12a9 3 0 0 0 18 0M12 3a3 9 0 0 0 0 18"/>',
    "ruler": '<path d="M21.3 8.7 8.7 21.3a2.4 2.4 0 0 1-3.4 0l-2.6-2.6a2.4 2.4 0 0 1 0-3.4L15.3 2.7a2.4 2.4 0 0 1 3.4 0l2.6 2.6a2.4 2.4 0 0 1 0 3.4Z"/><path d="m7.5 10.5 2 2M10.5 7.5l2 2M13.5 4.5l2 2M4.5 13.5l2 2"/>',
    "section": '<path d="m12 2 9 4.9v10.2L12 22l-9-4.9V6.9z"/><path d="m3.6 7.2 8.4 4.6 8.4-4.6M12 22V11.8"/><path d="M8 4.2 16.5 9"/>',
    "highlight": '<path d="m9 11-6 6v3h9l3-3"/><path d="m22 12-4.6 4.6a2.1 2.1 0 0 1-3 0l-7-7a2.1 2.1 0 0 1 0-3L12 2z"/>',
    "note": '<path d="M21 15a4 4 0 0 1-4 4H8l-5 3V7a4 4 0 0 1 4-4h10a4 4 0 0 1 4 4z"/>',
    "area": '<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M3 9h18M9 21V9"/>',
    "page": '<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/>',
    "zoom": '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3M8 11h6"/>',
    "format": '<path d="m3 3 3 3M19 3l2 2M11 7l6 6M9 21l12-12-6-6L3 15v6z"/>',
    "import": '<path d="M12 3v12M8 11l4 4 4-4"/><path d="M5 21h14"/>',
    "split": '<path d="M12 3v18M3 12h18"/><rect width="18" height="18" x="3" y="3" rx="2"/>',
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


def command_button(command: dict[str, object]) -> str:
    label = str(command["label"])
    icon_name = str(command["icon"])
    disabled = bool(command.get("disabled", False))
    split = bool(command.get("split", False))
    reason = str(command.get("reason", label))
    classes = ["cm-command"]
    if disabled:
        classes.append("disabled")
    if split:
        classes.append("split-button")
    disabled_attr = ' disabled aria-disabled="true"' if disabled else ""
    tooltip = reason if disabled else label
    caret = '<span class="split-caret" aria-hidden="true">v</span>' if split else ""
    return (
        f'<button class="{" ".join(classes)}" title="{tooltip}"'
        f'{disabled_attr}>{icon(ICONS[icon_name], "")}'
        f'<span class="cm-command-label">{label}</span>{caret}</button>'
    )


def command_group(group: dict[str, object]) -> str:
    name = str(group["name"])
    priority = str(group.get("priority", "normal"))
    commands = "".join(
        command_button(command)
        for command in group["commands"]  # type: ignore[union-attr]
    )
    return (
        f'<div class="cm-group priority-{priority}" role="group" '
        f'aria-label="{name}">'
        f'<div class="cm-group-tools">{commands}</div>'
        f'<span class="cm-group-name">{name}</span></div>'
    )


def chrome(context: str, location: str, tabs: list[tuple[str, str]], groups: list[dict[str, object]],
           browser_navigation: bool = False, save: bool = True) -> str:
    tab_html = "".join(tab(kind, label, i == 0) for i, (kind, label) in enumerate(tabs))
    navigation = ""
    if browser_navigation:
        navigation = "".join(
            f'<button class="title-icon" aria-label="{label}">{icon(ICONS[name], "")}</button>'
            for name, label in (("back", "Back"), ("forward", "Forward"), ("reload", "Reload"))
        )
    tools = '<span class="cm-divider"></span>'.join(command_group(group) for group in groups)
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
      <span class="cm-divider context-divider"></span>{tools}
      <button class="cm-overflow" aria-label="More commands" title="More commands; least-used groups move here first">v</button>
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


def cmd(label: str, icon_name: str, *, split: bool = False,
        disabled: bool = False, reason: str = "") -> dict[str, object]:
    return {
        "label": label,
        "icon": icon_name,
        "split": split,
        "disabled": disabled,
        "reason": reason or label,
    }


def group(name: str, commands: list[dict[str, object]],
          priority: str = "normal") -> dict[str, object]:
    return {"name": name, "commands": commands, "priority": priority}


CONFIG = {
    "document-problems-ai.html": dict(
        context="Text",
        location='<span class="crumb-muted">Projects / Field kit /</span><strong>sync-notes.ts</strong>',
        tabs=[("file", "sync-notes.ts"), ("pdf", "field-report.pdf")],
        groups=[
            group("History", [cmd("Undo", "undo"), cmd("Redo", "redo")]),
            group("Type", [
                cmd("14 pt", "type", split=True),
                cmd("Format", "format", disabled=True,
                    reason="Format document is unavailable for plain text."),
            ]),
            group("Find", [cmd("Find", "find")]),
            group("Proofing", [cmd("Check", "spell")], "low"),
            group("Export", [cmd("Export", "export", split=True)], "low"),
        ],
        active_rail="Files",
    ),
    "pdf-annotation.html": dict(
        context="PDF",
        location='<span class="crumb-muted">Documents / Site visit /</span><strong>field-report.pdf</strong>',
        tabs=[("pdf", "field-report.pdf"), ("file", "sync-notes.ts")],
        groups=[
            group("Tools", [cmd("Select", "pointer", split=True)]),
            group("Annotate", [
                cmd("Highlight", "highlight", split=True),
                cmd("Note", "note"),
                cmd("Area", "area"),
            ]),
            group("Page", [cmd("3 / 12", "page", split=True)]),
            group("View", [cmd("90%", "zoom", split=True)], "low"),
            group("Export", [cmd("Export PDF", "export")], "low"),
        ],
        active_rail="Files",
    ),
    "unibrow-create.html": dict(
        context="Create",
        location='<span class="crumb-muted">Projects / Workshop tour /</span><strong>workshop-tour.unibrow</strong>',
        tabs=[("project", "workshop-tour.unibrow"), ("folder", "Workshop media")],
        groups=[
            group("History", [cmd("Undo", "undo"), cmd("Redo", "redo")]),
            group("Add", [cmd("Import", "import"), cmd("Title", "type", split=True)]),
            group("Edit", [cmd("Split", "split")]),
            group("Timeline", [cmd("80%", "zoom", split=True)], "low"),
            group("Export", [cmd("Export", "export", split=True)], "low"),
        ],
        active_rail="Project",
    ),
    "new-tab.html": dict(
        context="Start",
        location='<span class="crumb-muted">Search, open a path, or enter a web address</span>',
        tabs=[("file", "New tab"), ("file", "sync-notes.ts")],
        groups=[
            group("Open", [cmd("File", "file"), cmd("Folder", "folder")]),
            group("Create", [cmd("Project", "project")]),
            group("Recent", [cmd("Recent", "page")], "low"),
        ],
        active_rail="Files",
        save=False,
    ),
    "multimodal-workshop.html": dict(
        context="CAD / 3D",
        location='<span class="crumb-muted">Projects / Ventilation retrofit /</span><strong>north-bay-bracket.dxf</strong>',
        tabs=[("project", "north-bay-bracket.dxf"), ("project", "workshop-tour.unibrow"), ("file", "bracket-check.ts")],
        groups=[
            group("Select", [cmd("Select", "pointer", split=True)]),
            group("View", [cmd("Orbit", "orbit", split=True), cmd("Front", "area")]),
            group("Measure", [cmd("Measure", "ruler")]),
            group("Section", [cmd("Section", "section", split=True)]),
            group("Export", [cmd("Export DXF", "export", split=True)], "low"),
        ],
        active_rail="Project",
    ),
}


for filename, config in CONFIG.items():
    path = ROOT / filename
    source = path.read_text()
    if "data-command-density" not in source:
        source = re.sub(
            r"(<body[^>]*>)",
            r"""\1
  <script data-command-density>
    if (new URLSearchParams(location.search).get("density") === "large") {
      document.body.classList.add("command-large");
    }
  </script>""",
            source,
            count=1,
        )
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
