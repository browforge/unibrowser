import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { t } from "./i18n";
import {
  isToolbarWindowLabel,
  mainShellShouldBeAbsent,
  mountToolbarOnlyDocument,
} from "./toolbarOnlyShell";

export { isToolbarWindowLabel, mainShellShouldBeAbsent, mountToolbarOnlyDocument };

export function isToolbarOnlyWindow(label?: string): boolean {
  if (label !== undefined) {
    return isToolbarWindowLabel(label);
  }
  const fromQuery = new URLSearchParams(window.location.search).has("toolbar");
  if (fromQuery) {
    return true;
  }
  return isToolbarWindowLabel(getCurrentWebviewWindow().label);
}

export function toolbarBarIdFromWindow(label?: string): string | null {
  const fromQuery = new URLSearchParams(window.location.search).get("toolbar");
  const l = label ?? fromQuery ?? getCurrentWebviewWindow().label;
  if (!isToolbarWindowLabel(l)) {
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
  mountToolbarOnlyDocument();
  const barId = toolbarBarIdFromWindow();
  const chrome = document.getElementById("chrome");
  if (chrome && barId) {
    const dock = document.createElement("button");
    dock.type = "button";
    dock.className = "toolbar-window-dock";
    dock.textContent = t("commandManager.dockToolbar");
    dock.onclick = async () => {
      await invoke("dock_floating_toolbar_window", { barId });
      await getCurrentWebviewWindow().close();
    };
    chrome.prepend(dock);
  }
  await refreshCommandManager();
  return true;
}
