import { t } from "./i18n";
import { isWebPlatform } from "./platform/isWeb";

export type MenuRun = (commandId: string) => void | Promise<void>;
export type MenuOpenPath = (path: string) => void | Promise<void>;

interface MenuAction {
  label: string;
  commandId?: string;
  path?: string;
  divider?: boolean;
}

interface MenuDef {
  id: string;
  label: string;
  items: MenuAction[];
}

function fileMenuItems(): MenuAction[] {
  const items: MenuAction[] = [
    { label: t("menu.new"), path: "unibrowser://action/new-project" },
    { label: t("menu.newFromTemplate"), path: "unibrowser://action/new-from-template" },
    { label: t("menu.openFile"), commandId: "unibrowser.control.start.openFile" },
    { label: t("menu.openFolder"), commandId: "unibrowser.control.start.openFolder" },
    { label: t("menu.saveAsTemplate"), path: "unibrowser://action/save-as-template" },
  ];
  if (!isWebPlatform()) {
    items.push(
      { divider: true, label: "" },
      { label: t("pkg.menu.unpack"), path: "unibrowser://action/pkg-unpack" },
      { label: t("pkg.menu.pack"), path: "unibrowser://action/pkg-pack" },
    );
  }
  items.push(
    { divider: true, label: "" },
    { label: t("menu.newWindow"), commandId: "unibrowser.window.new" },
    { divider: true, label: "" },
    { label: t("settings.open"), path: "unibrowser://settings" },
  );
  return items;
}

function buildMenus(): MenuDef[] {
  return [
    {
      id: "file",
      label: t("menu.file"),
      items: fileMenuItems(),
    },
    {
      id: "edit",
      label: t("menu.edit"),
      items: [
        { label: t("menu.findInDocument"), commandId: "unibrowser.find.inDocument" },
        { label: t("menu.findFiles"), commandId: "unibrowser.find.filesDeep" },
      ],
    },
    {
      id: "view",
      label: t("menu.view"),
      items: [
        { label: t("menu.commandPalette"), commandId: "unibrowser.palette.toggle" },
        { label: t("menu.problems"), commandId: "unibrowser.problems.toggle" },
        { label: t("menu.terminal"), commandId: "unibrowser.terminal.toggle" },
        { divider: true, label: "" },
        { label: "Workspaces: Edit layout", commandId: "unibrowser.workspace.editLayout" },
        { label: "Workspaces: Manage saved layouts", path: "unibrowser://settings" },
        { divider: true, label: "" },
        { label: t("menu.aiAssistant"), commandId: "ai-assistant.toggle" },
        { label: t("menu.mods"), path: "unibrowser://mods" },
        { label: t("bookmark.downloadManager"), path: "unibrowser://downloads" },
      ],
    },
    {
      id: "go",
      label: t("menu.go"),
      items: [
        { label: t("chrome.tabs"), commandId: "unibrowser.tabs.goTo" },
        { label: t("menu.workspaceQuickOpen"), commandId: "unibrowser.workspace.quickOpen" },
      ],
    },
    {
      id: "tools",
      label: t("menu.tools"),
      items: [
        { label: t("chrome.compare"), commandId: "unibrowser.image.compare" },
        { label: t("menu.keyboardShortcuts"), commandId: "unibrowser.keys.showOverlay" },
      ],
    },
    {
      id: "help",
      label: t("menu.help"),
      items: [
        { label: t("menu.contextHelp"), commandId: "unibrowser.help.context" },
      ],
    },
  ];
}

let closeOpenMenu: (() => void) | null = null;

function compactMenuAnchorPosition(anchor: HTMLElement): { left: number; top: number } {
  const anchorRect = anchor.getBoundingClientRect();
  const titleBar = document.getElementById("title-bar");
  const barRect = titleBar?.getBoundingClientRect();
  const top =
    barRect && barRect.height > 0 ? barRect.bottom : Math.max(anchorRect.bottom, 32);
  const left = anchorRect.width > 0 && anchorRect.left >= 0 ? anchorRect.left : 4;
  return { left, top };
}

function positionDropdownPanel(anchor: HTMLElement, panel: HTMLElement) {
  const { left, top } = compactMenuAnchorPosition(anchor);
  panel.style.position = "fixed";
  panel.style.left = `${left}px`;
  panel.style.top = `${top}px`;
}

function openMenuPanelAt(anchor: HTMLElement, panel: HTMLElement) {
  closeOpenMenu?.();
  positionDropdownPanel(anchor, panel);
  document.body.appendChild(panel);
  closeOpenMenu = () => {
    panel.remove();
    closeOpenMenu = null;
  };
  const first = panel.querySelector<HTMLButtonElement>(".app-menu-item, .app-menu-root-item");
  first?.focus();
}

export function openAppMenuFromButton(
  anchor: HTMLElement,
  run: MenuRun,
  openPath: MenuOpenPath,
) {
  const panel = document.createElement("div");
  panel.className = "app-menu-dropdown";
  panel.setAttribute("role", "menu");
  for (const menu of buildMenus()) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "app-menu-root-item";
    btn.textContent = menu.label;
    btn.setAttribute("role", "menuitem");
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      panel.remove();
      closeOpenMenu = null;
      const sub = document.createElement("div");
      sub.className = "app-menu-dropdown";
      sub.setAttribute("role", "menu");
      for (const item of menu.items) {
        if (item.divider) {
          const hr = document.createElement("hr");
          hr.className = "app-menu-divider";
          sub.appendChild(hr);
          continue;
        }
        const action = document.createElement("button");
        action.type = "button";
        action.className = "app-menu-item";
        action.textContent = item.label;
        action.setAttribute("role", "menuitem");
        action.addEventListener("click", () => {
          sub.remove();
          if (item.path) {
            void openPath(item.path);
          } else if (item.commandId) {
            void run(item.commandId);
          }
        });
        sub.appendChild(action);
      }
      positionDropdownPanel(anchor, sub);
      document.body.appendChild(sub);
    });
    panel.appendChild(btn);
  }
  openMenuPanelAt(anchor, panel);
}

/** Compact title bar: File menu only, anchored below the hamburger button. */
export function openCompactFileMenuDropdown(
  anchor: HTMLElement,
  run: MenuRun,
  openPath: MenuOpenPath,
) {
  closeOpenMenu?.();
  const panel = document.createElement("div");
  panel.className = "app-menu-dropdown";
  panel.setAttribute("role", "menu");
  for (const item of fileMenuItems()) {
    if (item.divider) {
      const hr = document.createElement("hr");
      hr.className = "app-menu-divider";
      panel.appendChild(hr);
      continue;
    }
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "app-menu-item";
    btn.textContent = item.label;
    btn.setAttribute("role", "menuitem");
    btn.addEventListener("click", () => {
      closeOpenMenu?.();
      if (item.path) {
        void openPath(item.path);
      } else if (item.commandId) {
        void run(item.commandId);
      }
    });
    panel.appendChild(btn);
  }
  positionDropdownPanel(anchor, panel);
  document.body.appendChild(panel);
  closeOpenMenu = () => {
    panel.remove();
    closeOpenMenu = null;
  };
  panel.querySelector<HTMLButtonElement>(".app-menu-item")?.focus();
}

/** @deprecated Desktop captures use the real File menu (#app-menu-bar). */
export function showFileMenuCapturePanel(): void {
  const fileTrigger = document.querySelector<HTMLButtonElement>(
    "#app-menu-bar .app-menu-trigger",
  );
  if (fileTrigger) {
    fileTrigger.click();
  }
}

export function initAppMenu(
  run: MenuRun,
  openPath: MenuOpenPath,
  options?: { menuButtonId?: string },
) {
  const bar = document.getElementById("app-menu-bar");
  if (!bar) {
    return;
  }
  bar.innerHTML = "";
  let openMenu: HTMLElement | null = null;

  const closeOpen = () => {
    openMenu?.remove();
    openMenu = null;
    closeOpenMenu = null;
  };
  closeOpenMenu = closeOpen;

  document.addEventListener("click", (e) => {
    if (openMenu && !openMenu.contains(e.target as Node)) {
      closeOpen();
    }
  });

  for (const menu of buildMenus()) {
    const trigger = document.createElement("button");
    trigger.type = "button";
    trigger.className = "app-menu-trigger";
    trigger.textContent = menu.label;
    trigger.setAttribute("aria-haspopup", "true");
    trigger.addEventListener("click", (e) => {
      e.stopPropagation();
      closeOpen();
      const panel = document.createElement("div");
      panel.className = "app-menu-dropdown";
      panel.setAttribute("role", "menu");
      for (const item of menu.items) {
        if (item.divider) {
          const hr = document.createElement("hr");
          hr.className = "app-menu-divider";
          panel.appendChild(hr);
          continue;
        }
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "app-menu-item";
        btn.textContent = item.label;
        btn.setAttribute("role", "menuitem");
        btn.addEventListener("click", () => {
          closeOpen();
          if (item.path) {
            void openPath(item.path);
          } else if (item.commandId) {
            void run(item.commandId);
          }
        });
        panel.appendChild(btn);
      }
      positionDropdownPanel(trigger, panel);
      document.body.appendChild(panel);
      openMenu = panel;
      const first = panel.querySelector<HTMLButtonElement>(".app-menu-item");
      first?.focus();
    });
    trigger.addEventListener("keydown", (e) => {
      if (e.key === "ArrowDown" || e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        trigger.click();
      }
    });
    bar.appendChild(trigger);
  }

  document.addEventListener("keydown", (e) => {
    if (e.altKey && !e.ctrlKey && !e.metaKey) {
      const key = e.key.toLowerCase();
      const map: Record<string, string> = {
        f: "file",
        e: "edit",
        v: "view",
        g: "go",
        t: "tools",
        h: "help",
      };
      const id = map[key];
      if (id) {
        e.preventDefault();
        const btn = bar.querySelector<HTMLButtonElement>(`[data-menu-id="${id}"]`);
        btn?.click();
      }
    }
    if (e.key === "Escape") {
      closeOpen();
    }
  });

  bar.querySelectorAll(".app-menu-trigger").forEach((el, i) => {
    const menus = buildMenus();
    if (menus[i]) {
      el.setAttribute("data-menu-id", menus[i].id);
    }
  });

  if (options?.menuButtonId) {
    const compactBtn = document.getElementById(options.menuButtonId);
    compactBtn?.addEventListener("click", (e) => {
      e.stopPropagation();
      openAppMenuFromButton(compactBtn, run, openPath);
    });
  }
}
