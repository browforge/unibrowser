import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { t } from "./i18n";

export function isToolbarOnlyWindow(label?: string): boolean {
  const l = label ?? getCurrentWebviewWindow().label;
  return l.startsWith("toolbar-") || new URLSearchParams(window.location.search).has("toolbar");
}

export function toolbarBarIdFromWindow(label?: string): string | null {
  const fromQuery = new URLSearchParams(window.location.search).get("toolbar");
  const l = label ?? fromQuery ?? getCurrentWebviewWindow().label;
  if (!l.startsWith("toolbar-")) {
    return null;
  }
  return l.slice("toolbar-".length).replace(/__dot__/g, ".");
}

/** Minimal shell for detached toolbar windows. Returns true when this window is toolbar-only. */
export async function initToolbarOnlyShell(
  refreshCommandManager: () => Promise<void>,
): Promise<boolean> {
  if (!isToolbarOnlyWindow()) {
    return false;
  }
  document.body.classList.add("toolbar-window-mode");
  const barId = toolbarBarIdFromWindow();
  const header = document.getElementById("app-header");
  if (header && barId) {
    const dock = document.createElement("button");
    dock.type = "button";
    dock.className = "toolbar-window-dock";
    dock.textContent = t("commandManager.dockToolbar");
    dock.onclick = async () => {
      await invoke("dock_floating_toolbar_window", { barId });
      await getCurrentWebviewWindow().close();
    };
    header.prepend(dock);
  }
  await refreshCommandManager();
  return true;
}
