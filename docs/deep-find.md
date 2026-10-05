# Deep find (Ctrl+Shift+F)

**Deep find** searches under a folder you choose (defaults to the last Files browse path). It is separate from **Ctrl+P** workspace quick open (pinned workspace root, filename fuzzy match only).

## Behaviour

| Match | Result |
| --- | --- |
| Filename contains query (case-insensitive) | Listed; if the file is searchable text, the first matching **line** is attached when found |
| Text file content contains query | Listed with **line number**; opening jumps the editor to that line |
| Binary / huge files | Skipped for content scan |

Opening a hit calls `open_path` with an optional line. Monaco reveals that line when the tab uses the text editor.

## Limits (defaults)

| Limit | Value |
| --- | --- |
| Max depth | 8 (shell), hard cap 16 |
| Max results | 200 (shell), hard cap 500 |
| Content scan max file size | 512 KiB |
| Binary detection | Null byte in first 4 KiB, or over size cap |

Content scan applies to common source and text extensions (`.rs`, `.ts`, `.js`, `.md`, `.json`, etc.). Other extensions are filename-only.

## Cancel

**Cancel search** sets the cancel flag; in-flight walks stop with "search cancelled".

## Related

- Workspace quick open: [workspace-folders.md](workspace-folders.md)
- Plain-text editor: bundled `base-text` (Monaco)
