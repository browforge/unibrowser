import { invoke } from "@tauri-apps/api/core";
import { setAiPanelVisible } from "./aiAssistantPanel";
import { showProblemsBottomPanel } from "./problemsPanel";
import { setRailMode } from "./ui/activityRail";

let active = false;
let overlay: HTMLElement | null = null;

function target(
  label: string,
  detail: string,
  className: string,
  run: () => void | Promise<void>,
): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = `layout-dock-target ${className}`;
  const title = document.createElement("strong");
  title.textContent = label;
  const copy = document.createElement("span");
  copy.textContent = detail;
  button.append(title, copy);
  button.onclick = () => void run();
  return button;
}

function close() {
  active = false;
  document.body.classList.remove("layout-edit-mode");
  overlay?.remove();
  overlay = null;
  window.dispatchEvent(new Event("unibrowser-layout-edit-changed"));
}

function open() {
  const workArea = document.getElementById("work-area");
  if (!workArea || active) {
    return;
  }
  active = true;
  document.body.classList.add("layout-edit-mode");

  overlay = document.createElement("section");
  overlay.className = "layout-edit-overlay";
  overlay.setAttribute("aria-label", "Workspace layout edit mode");

  const toolbar = document.createElement("header");
  toolbar.className = "layout-edit-toolbar";
  const heading = document.createElement("div");
  const title = document.createElement("strong");
  title.textContent = "Layout edit";
  const lead = document.createElement("span");
  lead.textContent = "Choose dock regions, move floating toolbars by their handles, then save.";
  heading.append(title, lead);

  const name = document.createElement("input");
  name.type = "text";
  name.placeholder = "Layout name";
  name.setAttribute("aria-label", "Layout name");
  const status = document.createElement("span");
  status.className = "layout-edit-status";
  status.setAttribute("role", "status");
  const save = document.createElement("button");
  save.type = "button";
  save.textContent = "Save layout";
  save.onclick = async () => {
    const value = name.value.trim();
    if (!value) {
      status.textContent = "Enter a layout name.";
      name.focus();
      return;
    }
    save.disabled = true;
    try {
      await invoke("save_workspace_layout_preset", { name: value });
      status.textContent = `Saved ${value}.`;
      name.value = "";
    } catch (err) {
      status.textContent = err instanceof Error ? err.message : String(err);
    } finally {
      save.disabled = false;
    }
  };
  const done = document.createElement("button");
  done.type = "button";
  done.textContent = "Done";
  done.onclick = close;
  const actions = document.createElement("div");
  actions.className = "layout-edit-actions";
  actions.append(name, save, done);
  toolbar.append(heading, status, actions);

  const targets = document.createElement("div");
  targets.className = "layout-dock-targets";
  targets.append(
    target("Left dock", "Expand Files and supporting panels", "layout-dock-target-left", () =>
      setRailMode("expanded"),
    ),
    target("Right dock", "Open AI Assistant beside the document", "layout-dock-target-right", () =>
      setAiPanelVisible(true),
    ),
    target("Bottom dock", "Open Problems below the document", "layout-dock-target-bottom", () =>
      showProblemsBottomPanel(),
    ),
  );

  overlay.append(toolbar, targets);
  workArea.appendChild(overlay);
  window.dispatchEvent(new Event("unibrowser-layout-edit-changed"));
}

export function setLayoutEditMode(enabled: boolean) {
  if (enabled) {
    open();
    return;
  }
  close();
}

export function toggleLayoutEditMode() {
  setLayoutEditMode(!active);
}

export function isLayoutEditMode() {
  return active;
}
