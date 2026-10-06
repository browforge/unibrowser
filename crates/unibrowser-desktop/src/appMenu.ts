import { t } from "./i18n";

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

function buildMenus(): MenuDef[] {
  return [
    {
      id: "file",
      label: t("menu.file"),
      items: [
        { label: t("menu.newWindow"), commandId: "unibrowser.window.new" },
        { label: t("menu.newProject"), path: "unibrowser://action/new-project" },
        { divider: true, label: "" },
        { label: t("settings.open"), path: "unibrowser://settings" },
      ],
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

export function initAppMenu(run: MenuRun, openPath: MenuOpenPath) {
  const bar = document.getElementById("app-menu-bar");
  if (!bar) {
    return;
  }
  bar.innerHTML = "";
  let openMenu: HTMLElement | null = null;

  const closeOpen = () => {
    openMenu?.remove();
    openMenu = null;
  };

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
      const rect = trigger.getBoundingClientRect();
      panel.style.left = `${rect.left}px`;
      panel.style.top = `${rect.bottom}px`;
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
}
