import { getCurrentWindow } from "@tauri-apps/api/window";
import { createLucideIcon } from "./lucideIcon";

export interface TitleBarTabHint {
  is_http: boolean;
  is_directory: boolean;
}

export interface CompactTitleBarDeps {
  runCommand: (commandId: string) => void | Promise<void>;
  openTitleOverflow: (anchor: HTMLElement) => void;
  onBack: () => void;
  onForward: () => void;
  onReload: () => void;
  navigateOmnibox: () => void;
}

function fillIconButton(id: string, icon: Parameters<typeof createLucideIcon>[0], title: string) {
  const el = document.getElementById(id);
  if (!el) {
    return;
  }
  el.innerHTML = "";
  el.title = title;
  el.appendChild(createLucideIcon(icon, 16, "ub-icon svg-icon"));
}

export function initCompactTitleBar(deps: CompactTitleBarDeps) {
  fillIconButton("title-menu-btn", "menu", "Main menu (Alt)");
  fillIconButton("btn-back", "arrow-left", "Back");
  fillIconButton("btn-forward", "arrow-right", "Forward");
  fillIconButton("btn-reload", "rotate-cw", "Reload");
  const saveBtn = document.getElementById("btn-save");
  if (saveBtn) {
    saveBtn.title = "Save";
    if (!saveBtn.textContent?.trim()) {
      saveBtn.textContent = "Save";
    }
  }
  fillIconButton("title-overflow", "ellipsis", "Layout and more options");

  fillIconButton("win-minimize", "minus", "Minimize");
  fillIconButton("win-maximize", "square", "Maximize");
  fillIconButton("win-close", "x", "Close");
  const winMin = document.getElementById("win-minimize");
  const winMax = document.getElementById("win-maximize");
  const winClose = document.getElementById("win-close");

  const appWin = getCurrentWindow();
  document.getElementById("title-drag-region")?.addEventListener("mousedown", (e) => {
    if (e.button === 0) {
      void appWin.startDragging();
    }
  });
  document.getElementById("title-bar")?.addEventListener("mousedown", (e) => {
    const t = e.target as HTMLElement;
    if (t.closest("button, input, [role='tab'], .tab-wrap")) {
      return;
    }
    if (e.button === 0) {
      void appWin.startDragging();
    }
  });

  const syncMaximizeIcon = async () => {
    const maximized = await appWin.isMaximized();
    if (winMax) {
      winMax.innerHTML = "";
      winMax.appendChild(
        createLucideIcon(maximized ? "copy" : "square", 14, "ub-icon svg-icon"),
      );
      winMax.setAttribute("aria-label", maximized ? "Restore" : "Maximize");
      winMax.title = maximized ? "Restore" : "Maximize";
    }
  };
  void syncMaximizeIcon();
  void appWin.onResized(() => {
    void syncMaximizeIcon();
  });

  winMin?.addEventListener("click", () => void appWin.minimize());
  winMax?.addEventListener("click", () => void appWin.toggleMaximize());
  winClose?.addEventListener("click", () => void appWin.close());

  document.getElementById("btn-back")!.onclick = () => deps.onBack();
  document.getElementById("btn-forward")!.onclick = () => deps.onForward();
  document.getElementById("btn-reload")!.onclick = () => deps.onReload();

  document.getElementById("btn-save")!.onclick = () => {
    void deps.runCommand("unibrowser.text.save");
  };

  document.getElementById("title-overflow")!.onclick = (e) => {
    e.stopPropagation();
    const anchor = document.getElementById("title-overflow");
    if (anchor) {
      deps.openTitleOverflow(anchor);
    }
  };
}

export function updateTitleBarNavVisibility(tab: TitleBarTabHint | null | undefined) {
  const nav = document.getElementById("title-nav");
  if (!nav) {
    return;
  }
  const show = Boolean(tab && (tab.is_http || tab.is_directory));
  nav.classList.toggle("hidden", !show);
}

export function syncTitleBar(_data?: unknown) {
  /* Reserved for omnibox crumbs / security chips in later UI batches. */
}
