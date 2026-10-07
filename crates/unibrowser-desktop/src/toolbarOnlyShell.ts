export function isToolbarWindowLabel(label: string): boolean {
  return label.startsWith("toolbar-");
}

/** Remove full-app regions so detached toolbar windows never mount the workbench shell. */
export function mountToolbarOnlyDocument(): void {
  document.body.classList.add("toolbar-window-mode");
  const shell = document.getElementById("app-shell");
  if (shell) {
    shell.dataset.ubToolbarShell = "1";
  }
  for (const id of [
    "work-area",
    "command-manager",
    "app-menu-bar",
    "title-bar",
    "bookmark-bar",
    "recent-panel",
    "shell-fallback",
    "shell-subsystem-notice",
  ]) {
    document.getElementById(id)?.remove();
  }
  const chrome = document.getElementById("chrome");
  if (chrome) {
    chrome.classList.add("toolbar-window-chrome");
  }
}

/** True when the main workbench shell should not be present (for tests). */
export function mainShellShouldBeAbsent(doc: Pick<Document, "getElementById" | "body">): boolean {
  return (
    !doc.getElementById("work-area") &&
    !doc.getElementById("content-host") &&
    !doc.getElementById("command-manager") &&
    !doc.getElementById("omnibox")
  );
}
