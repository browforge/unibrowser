#!/usr/bin/env python3
"""Write built-in template packs under crates/unibrowser-desktop/builtin-templates/."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "crates/unibrowser-desktop/builtin-templates"

PACKS = [
    (
        "unibrowser.template.blank",
        "Blank",
        "Empty project graph (same as New).",
        ["notes"],
        ["blank"],
        {"format_version": 2, "title": "Untitled project", "nodes": [], "timeline": {"clips": [], "transitions": [], "markers": []}},
        "browse",
    ),
    (
        "unibrowser.template.document.office",
        "Office document",
        "Single rich-text frame with print-oriented page setup.",
        ["document"],
        ["office", "document"],
        {
            "format_version": 2,
            "title": "Office document",
            "nodes": [
                {"id": "frame-1", "type": "document", "label": "Document", "x": 48, "y": 56}
            ],
            "timeline": {"clips": [], "transitions": [], "markers": []},
        },
        "write",
    ),
    (
        "unibrowser.template.presentation",
        "Presentation",
        "Slide board with speaker notes region.",
        ["presentation"],
        ["slides", "presentation"],
        {
            "format_version": 2,
            "title": "Presentation",
            "nodes": [
                {"id": "board-1", "type": "presentation", "label": "Slides", "x": 40, "y": 48},
                {"id": "notes-1", "type": "notes", "label": "Speaker notes", "x": 40, "y": 280},
            ],
            "timeline": {"clips": [], "transitions": [], "markers": []},
        },
        "create",
    ),
    (
        "unibrowser.template.video.simple",
        "Simple video",
        "Timeline with title card placeholder.",
        ["video"],
        ["video", "timeline"],
        {
            "format_version": 2,
            "title": "Simple video",
            "nodes": [
                {"id": "node-1", "type": "video", "label": "Title card", "x": 40, "y": 48, "color": "blue"}
            ],
            "timeline": {
                "clips": [
                    {
                        "id": "clip-1",
                        "node_id": "node-1",
                        "start_sec": 0,
                        "duration_sec": 5,
                        "label": "Title",
                        "title_text": "Your title",
                    }
                ],
                "transitions": [],
                "markers": [],
            },
        },
        "create",
    ),
    (
        "unibrowser.template.design.space",
        "Design space",
        "Infinite canvas with layout grids.",
        ["design"],
        ["canvas", "design"],
        {
            "format_version": 2,
            "title": "Design space",
            "nodes": [
                {"id": "canvas-1", "type": "canvas", "label": "Canvas", "x": 0, "y": 0}
            ],
            "timeline": {"clips": [], "transitions": [], "markers": []},
        },
        "create",
    ),
    (
        "unibrowser.template.notes",
        "Notes board",
        "Lightweight board with markdown note nodes.",
        ["notes"],
        ["notes", "board"],
        {
            "format_version": 2,
            "title": "Notes board",
            "nodes": [
                {"id": "note-1", "type": "note", "label": "Note", "x": 48, "y": 56},
                {"id": "note-2", "type": "note", "label": "Another note", "x": 280, "y": 160},
            ],
            "timeline": {"clips": [], "transitions": [], "markers": []},
        },
        "browse",
    ),
]


def main() -> None:
    for tid, title, desc, categories, tags, project, workspace in PACKS:
        pack_dir = ROOT / tid
        pack_dir.mkdir(parents=True, exist_ok=True)
        manifest = {
            "template_schema_version": 1,
            "id": tid,
            "version": "1.0.0",
            "title": title,
            "description": desc,
            "categories": categories,
            "tags": tags,
            "relationships": {
                "requires": {},
                "recommends": {},
                "optional": {},
                "incompatible": {},
            },
            "default_workspace": workspace,
        }
        (pack_dir / "template.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        (pack_dir / "project.json").write_text(json.dumps(project, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
