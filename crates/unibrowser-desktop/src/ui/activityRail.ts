import { createLucideIcon, type LucideName } from "./lucideIcon";

export type RailMode = "collapsed" | "expanded" | "auto-hide" | "hidden";

const STORAGE_KEY = "ub-rail-mode";

type RailDestination = "files" | "search" | "project" | "assets" | "settings";

interface RailItem {
  id: RailDestination;
  label: string;
  icon: LucideName;
  settings?: boolean;
}

const ITEMS: RailItem[] = [
  { id: "files", label: "Files", icon: "folder" },
  { id: "search", label: "Search", icon: "search" },
  { id: "project", label: "Project", icon: "panels-top-left" },
  { id: "assets", label: "Assets", icon: "package-open" },
  { id: "settings", label: "Settings", icon: "settings", settings: true },
];

const PANEL_COPY: Record<RailDestination, { title: string; lines: string[] }> = {
  files: {
    title: "Files",
    lines: ["Browse the workspace tree.", "Open folders from pinned places."],
  },
  search: {
    title: "Search",
    lines: ["Search across open files.", "Filter by name or symbol."],
  },
  project: {
    title: "Project",
    lines: ["Workspace roots and tasks.", "Switch between project folders."],
  },
  assets: {
    title: "Assets",
    lines: ["Images, fonts, and bundled resources.", "Drag assets into documents."],
  },
  settings: {
    title: "Settings",
    lines: ["Preferences and appearance.", "Keyboard shortcuts and proofing."],
  },
};

let activeDestination: RailDestination | null = null;
let mode: RailMode = "collapsed";
let edgeHitEl: HTMLElement | null = null;

function readMode(): RailMode {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (
      raw === "collapsed" ||
      raw === "expanded" ||
      raw === "auto-hide" ||
      raw === "hidden"
    ) {
      return raw;
    }
  } catch {
    /* ignore */
  }
  return "collapsed";
}

function writeMode(next: RailMode) {
  mode = next;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    /* ignore */
  }
}

function applyModeClasses(rail: HTMLElement) {
  rail.classList.remove("collapsed", "expanded", "auto-hide", "hidden", "rail-revealed");
  if (mode === "hidden") {
    rail.classList.add("hidden");
    return;
  }
  if (mode === "expanded") {
    rail.classList.add("expanded");
  } else {
    rail.classList.add("collapsed");
  }
  if (mode === "auto-hide") {
    rail.classList.add("auto-hide");
  }
}

function renderRailPanel(rail: HTMLElement, id: RailDestination | null) {
  const panel = rail.querySelector<HTMLElement>(".rail-side-panel");
  if (!panel) {
    return;
  }
  if (!id || mode === "hidden") {
    panel.classList.add("hidden");
    panel.innerHTML = "";
    return;
  }
  const copy = PANEL_COPY[id];
  panel.classList.remove("hidden");
  panel.innerHTML = "";
  const title = document.createElement("h2");
  title.className = "rail-panel-title";
  title.textContent = copy.title;
  panel.appendChild(title);
  const list = document.createElement("ul");
  list.className = "rail-panel-list";
  copy.lines.forEach((line) => {
    const li = document.createElement("li");
    li.textContent = line;
    list.appendChild(li);
  });
  panel.appendChild(list);
}

function hidePanelIfIdle(rail: HTMLElement) {
  const focused = rail.querySelector(".rail-destination:focus");
  if (focused instanceof HTMLElement) {
    const focusedId = focused.dataset.railId as RailDestination | undefined;
    if (focusedId && focusedId in PANEL_COPY) {
      renderRailPanel(rail, focusedId);
      return;
    }
  }
  renderRailPanel(rail, activeDestination);
}

function setActiveButton(rail: HTMLElement, id: RailDestination | null) {
  activeDestination = id;
  rail.querySelectorAll(".rail-destination").forEach((el) => {
    el.classList.toggle("active", el.getAttribute("data-rail-id") === id);
  });
  renderRailPanel(rail, id);
}

export function initActivityRail(deps: {
  onDestination: (id: RailDestination) => void;
  workAreaRow: HTMLElement;
}) {
  const rail = document.getElementById("activity-rail");
  if (!rail) {
    return;
  }
  mode = readMode();
  applyModeClasses(rail);
  rail.innerHTML = "";

  const sidePanel = document.createElement("div");
  sidePanel.className = "rail-side-panel hidden";
  sidePanel.setAttribute("aria-label", "Navigation panel");

  const makeBtn = (item: { label: string; icon: LucideName; className?: string; onClick: () => void }) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "rail-destination" + (item.className ? ` ${item.className}` : "");
    btn.title = item.label;
    btn.setAttribute("aria-label", item.label);
    btn.appendChild(createLucideIcon(item.icon, 16, "ub-icon svg-icon"));
    const label = document.createElement("span");
    label.className = "rail-label";
    label.textContent = item.label;
    btn.appendChild(label);
    btn.onclick = item.onClick;
    return btn;
  };

  const bindPanelPreview = (btn: HTMLButtonElement, id: RailDestination) => {
    btn.addEventListener("focus", () => {
      renderRailPanel(rail, id);
    });
    btn.addEventListener("blur", () => {
      window.setTimeout(() => hidePanelIfIdle(rail), 0);
    });
  };

  for (const item of ITEMS.filter((i) => !i.settings)) {
    const btn = makeBtn({
      label: item.label,
      icon: item.icon,
      onClick: () => {
        if (activeDestination === item.id) {
          setActiveButton(rail, null);
        } else {
          setActiveButton(rail, item.id);
        }
        deps.onDestination(item.id);
      },
    });
    btn.dataset.railId = item.id;
    bindPanelPreview(btn, item.id);
    rail.appendChild(btn);
  }

  rail.appendChild(
    makeBtn({
      label: "Collapse",
      icon: "chevrons-left",
      className: "rail-collapse",
      onClick: () => {
        if (mode === "expanded") {
          writeMode("collapsed");
        } else if (mode === "collapsed") {
          writeMode("expanded");
        } else if (mode === "auto-hide") {
          writeMode("collapsed");
        }
        applyModeClasses(rail);
        if (mode === "expanded" && activeDestination === null) {
          setActiveButton(rail, "files");
        } else {
          renderRailPanel(rail, activeDestination);
        }
      },
    }),
  );

  const settingsItem = ITEMS.find((i) => i.settings)!;
  const settingsBtn = makeBtn({
    label: settingsItem.label,
    icon: settingsItem.icon,
    className: "settings",
    onClick: () => {
      setActiveButton(rail, settingsItem.id);
      deps.onDestination(settingsItem.id);
    },
  });
  settingsBtn.dataset.railId = settingsItem.id;
  bindPanelPreview(settingsBtn, settingsItem.id);
  const collapseButton = rail.querySelector(".rail-collapse");
  rail.insertBefore(sidePanel, collapseButton ?? settingsBtn);
  rail.appendChild(settingsBtn);
  if (mode === "expanded") {
    setActiveButton(rail, "files");
  }

  if (mode === "auto-hide") {
    edgeHitEl = document.createElement("div");
    edgeHitEl.className = "rail-edge-hit";
    edgeHitEl.title = "Show activity rail";
    deps.workAreaRow.appendChild(edgeHitEl);
    const reveal = () => rail.classList.add("rail-revealed");
    const hide = () => rail.classList.remove("rail-revealed");
    edgeHitEl.addEventListener("mouseenter", reveal);
    rail.addEventListener("mouseenter", reveal);
    rail.addEventListener("mouseleave", hide);
    edgeHitEl.addEventListener("mouseleave", (e) => {
      if (!rail.matches(":hover")) {
        hide();
      }
      e.stopPropagation();
    });
  }

  document.addEventListener("keydown", (e) => {
    if (e.altKey && e.shiftKey && e.key.toLowerCase() === "a") {
      if (mode === "auto-hide") {
        rail.classList.toggle("rail-revealed");
      }
    }
  });
}

export function getRailMode(): RailMode {
  return mode;
}

export function setRailMode(next: RailMode) {
  const rail = document.getElementById("activity-rail");
  if (!rail) {
    return;
  }
  writeMode(next);
  applyModeClasses(rail);
  if (next === "expanded" && activeDestination === null) {
    setActiveButton(rail, "files");
    return;
  }
  renderRailPanel(rail, activeDestination);
}
