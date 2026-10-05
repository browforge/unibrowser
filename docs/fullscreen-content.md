# Fullscreen content (chrome hidden)

**F11** toggles a **content-focused** layout: the tab strip, omnibox, Command Manager, status bar, and other chrome are hidden so the active document or webview uses the window. This is not the browser Fullscreen API (no OS-level fullscreen).

## Exit

- **F11** again toggles chrome back.
- **Escape** exits content fullscreen (palette, tab search, and find bar close first if open).

## Webview layout

After entering or leaving content fullscreen, the shell recalculates embedded https webview bounds so the page still fills the visible region.

## Scope

- Applies to the whole window (all panes in split view share the same chrome).
- Floating toolbars and notification drawer are hidden while active.
- Internal pages (`unibrowser://settings`, Files browse, Monaco) benefit the same way as PDF/video viewers.
