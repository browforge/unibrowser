import { invoke } from "@tauri-apps/api/core";
import { navigateToTarget, type NavigationTarget } from "./locatorNavigate";
import { selectBottomPanelTab } from "./terminalPanel";
import { createLucideIcon, type LucideName } from "./ui/lucideIcon";

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
let messageFilter = "";
let selectedId: string | null = null;

function severityIcon(sev: string): LucideName {
  if (sev === "error") {
    return "circle-x";
  }
  if (sev === "warning") {
    return "triangle-alert";
  }
  return "info";
}

function severityClass(sev: string): string {
  if (sev === "error") {
    return "problems-sev-error";
  }
  if (sev === "warning") {
    return "problems-sev-warning";
  }
  return "problems-sev-info";
}

async function refresh() {
  if (!panelEl) {
    return;
  }
  const list = await invoke<ProblemDto[]>("list_problems_panel", {
    severity: severityFilter || null,
    source: sourceFilter || null,
  });
  const problemsTab = document.getElementById("bottom-panel-tab-problems");
  if (problemsTab) {
    problemsTab.textContent = list.length ? `Problems ${list.length}` : "Problems";
  }
  const countEl = document.getElementById("status-problems-count");
  if (countEl) {
    if (list.length) {
      countEl.textContent = `${list.length} Problems`;
      countEl.classList.remove("hidden");
    } else {
      countEl.classList.add("hidden");
    }
  }
  const wrap = panelEl.querySelector(".problems-table-wrap");
  const detail = panelEl.querySelector(".problems-detail");
  const empty = panelEl.querySelector(".problems-empty");
  const table = panelEl.querySelector<HTMLTableElement>(".problems-table");
  if (!wrap || !table || !detail || !empty) {
    return;
  }

  const q = messageFilter.trim().toLowerCase();
  const filtered = q
    ? list.filter((p) => p.message.toLowerCase().includes(q) || p.source.toLowerCase().includes(q))
    : list;

  table.querySelector("tbody")?.remove();
  const tbody = document.createElement("tbody");

  if (!list.length) {
    empty.textContent = "No problems in this workspace.";
    empty.classList.remove("hidden");
    wrap.classList.add("hidden");
    detail.classList.add("hidden");
    table.classList.add("hidden");
    if (!document.body.dataset.demoProblems && !document.body.dataset.demoMockupProblems) {
      document.getElementById("script-output-panel")?.classList.add("hidden");
    }
    return;
  }
  document.getElementById("script-output-panel")?.classList.remove("hidden");
  table.classList.remove("hidden");
  empty.classList.add("hidden");

  if (!filtered.length) {
    empty.textContent = "No problems match the current filters.";
    empty.classList.remove("hidden");
    wrap.classList.add("hidden");
    detail.classList.add("hidden");
    return;
  }

  empty.classList.add("hidden");
  wrap.classList.remove("hidden");

  for (const p of filtered) {
    const tr = document.createElement("tr");
    tr.dataset.problemId = p.id;
    if (p.id === selectedId) {
      tr.classList.add("is-selected");
    }
    const sevTd = document.createElement("td");
    sevTd.className = `problems-sev ${severityClass(p.severity)}`;
    const sevWrap = document.createElement("span");
    sevWrap.appendChild(createLucideIcon(severityIcon(p.severity), 14));
    sevWrap.appendChild(document.createTextNode(` ${p.severity}`));
    sevTd.appendChild(sevWrap);

    const msgTd = document.createElement("td");
    msgTd.textContent = p.message;

    const srcTd = document.createElement("td");
    srcTd.textContent = p.source;

    const locTd = document.createElement("td");
    locTd.className = "problems-loc";
    locTd.textContent = p.navigate_hint ?? p.locator_kind;

    tr.append(sevTd, msgTd, srcTd, locTd);
    tr.addEventListener("click", () => {
      selectedId = p.id;
      void refresh();
      void navigateToTarget(p.navigation);
    });
    tbody.appendChild(tr);
  }
  table.appendChild(tbody);

  const selected = filtered.find((p) => p.id === selectedId) ?? filtered[0];
  if (selected) {
    selectedId = selected.id;
    detail.textContent = `${selected.message} (${selected.source}${selected.navigate_hint ? ` · ${selected.navigate_hint}` : ""})`;
    detail.classList.remove("hidden");
  } else {
    detail.classList.add("hidden");
  }
}

export async function showProblemsBottomPanel() {
  document.getElementById("script-output-panel")?.classList.remove("hidden");
  selectBottomPanelTab("problems");
  await refresh();
}

export function initProblemsPanel() {
  panelEl = document.getElementById("problems-panel");
  if (!panelEl) {
    return;
  }
  document.getElementById("bottom-panel-tab-problems")?.addEventListener("click", () => {
    showProblemsBottomPanel();
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
  const msg = panelEl.querySelector<HTMLInputElement>(".problems-filter-message");
  msg?.addEventListener("input", () => {
    messageFilter = msg.value;
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
  const pane = document.getElementById("bottom-panel-problems-pane");
  if (pane && !pane.classList.contains("hidden")) {
    void refresh();
  }
}
