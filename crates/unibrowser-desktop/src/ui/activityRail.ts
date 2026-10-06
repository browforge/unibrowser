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

function setActiveButton(rail: HTMLElement, id: RailDestination | null) {
  activeDestination = id;
  rail.querySelectorAll(".rail-destination").forEach((el) => {
    el.classList.toggle("active", el.getAttribute("data-rail-id") === id);
  });
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
  rail.appendChild(settingsBtn);

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
}
