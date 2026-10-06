import { invoke } from "@tauri-apps/api/core";
import { navigateToTarget, type NavigationTarget } from "./locatorNavigate";
import { selectBottomPanelTab } from "./terminalPanel";

interface ProblemDto {
  id: string;
  source: string;
  severity: string;
  message: string;
  locator_kind: string;
  navigate_hint: string | null;
  navigation: NavigationTarget;
}

let panelEl: HTMLElement | null = null;
let severityFilter = "";
let sourceFilter = "";

async function refresh() {
  if (!panelEl) {
    return;
  }
  const list = await invoke<ProblemDto[]>("list_problems_panel", {
    severity: severityFilter || null,
    source: sourceFilter || null,
  });
  const ul = panelEl.querySelector(".problems-list");
  if (!ul) {
    return;
  }
  ul.innerHTML = "";
  for (const p of list) {
    const li = document.createElement("li");
    li.className = `problems-item problems-${p.severity}`;
    li.textContent = `[${p.source}] ${p.message}`;
    if (p.navigate_hint) {
      li.title = `${p.locator_kind}: ${p.navigate_hint}`;
    }
    li.addEventListener("click", () => {
      void navigateToTarget(p.navigation);
    });
    ul.appendChild(li);
  }
}

export function showProblemsBottomPanel() {
  document.getElementById("script-output-panel")?.classList.remove("hidden");
  selectBottomPanelTab("problems");
  void refresh();
}

export function initProblemsPanel() {
  panelEl = document.getElementById("problems-panel");
  if (!panelEl) {
    return;
  }
  document.getElementById("bottom-panel-tab-problems")?.addEventListener("click", () => {
    showProblemsBottomPanel();
  });
  panelEl.querySelector(".problems-close")?.addEventListener("click", () => {
    document.getElementById("script-output-panel")?.classList.add("hidden");
  });
  const sev = panelEl.querySelector<HTMLSelectElement>(".problems-filter-severity");
  sev?.addEventListener("change", () => {
    severityFilter = sev.value;
    void refresh();
  });
  const src = panelEl.querySelector<HTMLInputElement>(".problems-filter-source");
  src?.addEventListener("input", () => {
    sourceFilter = src.value.trim();
    void refresh();
  });
}

export function toggleProblemsPanel() {
  if (!panelEl) {
    return;
  }
  const open = panelEl.classList.toggle("hidden");
  if (!open) {
    void refresh();
  }
}

export function refreshProblemsPanel() {
  if (panelEl && !panelEl.classList.contains("hidden")) {
    void refresh();
  }
}
