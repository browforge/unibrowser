# Workspace folders (Files manager)

Pin a **workspace root** so the Files manager shows a capped recursive tree alongside the normal folder listing. This is for day-to-day project navigation, not a full IDE project system.

## Set or clear workspace

1. Open a folder tab (home or any directory).
2. In the Files browser nav row, choose **Set workspace** to store the current browse path as the workspace root (saved in preferences).
3. **Clear workspace** removes the root and hides the tree.

The workspace root is global (shared across windows), like other preferences.

## Workspace tree

When a root is set, a **Workspace** section appears above the file table:

- Breadth-first walk from the root, **max 1200 entries** and **depth 10** (`workspace_tree.rs`).
- Folders open in the same Files tab; files open in a **new tab**.
- If the walk hits the cap, the note under the heading still shows the root path; some branches may be omitted.

## Git status lite (optional)

If the browse folder lies inside a git repository and `git` is on `PATH`:

- Changed paths from `git status --porcelain` show a **●** badge in the file list and workspace tree.
- No commit UI: **Commit**, **Push**, and **Sync** appear in the command registry but stay **disabled** with grey reasons (`base-files` mod).

Refresh happens on each shell snapshot when you browse folders; there is no background watcher.

## Related code

| Piece | Location |
| --- | --- |
| Tree walk | `crates/unibrowser-desktop/src-tauri/src/workspace_tree.rs` |
| Git hints | `crates/unibrowser-desktop/src-tauri/src/git_status.rs` |
| Commands | `set_workspace_root`, `clear_workspace_root`, `list_workspace_tree` |
| UI | `crates/unibrowser-desktop/src/filesPanel.ts` |
