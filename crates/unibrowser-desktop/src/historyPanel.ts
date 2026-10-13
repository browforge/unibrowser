import { listDocumentHistory } from "./documentModelBridge";

let mounted = false;

export async function refreshHistoryPanel(path: string | null) {
  const list = document.getElementById("history-panel-list");
  if (!list) {
    return;
  }
  list.innerHTML = "";
  if (!path) {
    const empty = document.createElement("p");
    empty.className = "history-panel-empty";
    empty.textContent = "No document open.";
    list.appendChild(empty);
    return;
  }
  let rows: Awaited<ReturnType<typeof listDocumentHistory>> = [];
  try {
    rows = await listDocumentHistory(path);
  } catch {
    const empty = document.createElement("p");
    empty.className = "history-panel-empty";
    empty.textContent =
      "Edits in this session are not recorded to disk yet. Save to enable .unibrow-history.";
    list.appendChild(empty);
    return;
  }
  if (!rows.length) {
    const empty = document.createElement("p");
    empty.className = "history-panel-empty";
    empty.textContent = "No committed document ops yet.";
    list.appendChild(empty);
    return;
  }
  const table = document.createElement("table");
  table.className = "history-panel-table";
  table.innerHTML =
    "<thead><tr><th>Seq</th><th>Time</th><th>Source</th><th>Summary</th></tr></thead>";
  const body = document.createElement("tbody");
  for (const row of [...rows].reverse()) {
    const tr = document.createElement("tr");
    tr.innerHTML = `<td>${row.seq}</td><td>${row.timestamp}</td><td>${row.plugin_id}</td><td>${row.op_type}</td>`;
    body.appendChild(tr);
  }
  table.appendChild(body);
  list.appendChild(table);
}

export function mountHistoryPanel() {
  if (mounted) {
    return;
  }
  mounted = true;
  const tab = document.getElementById("bottom-panel-tab-history");
  const pane = document.getElementById("bottom-panel-history-pane");
  if (!tab || !pane) {
    return;
  }
  pane.innerHTML = `<div id="history-panel-list" class="history-panel-list" role="log" aria-live="polite"></div>`;
  tab.addEventListener("click", () => {
    document.querySelectorAll(".bottom-panel-tab").forEach((el) => {
      el.classList.remove("is-active");
      el.setAttribute("aria-selected", "false");
    });
    tab.classList.add("is-active");
    tab.setAttribute("aria-selected", "true");
    document.querySelectorAll(".bottom-panel-pane").forEach((el) => el.classList.add("hidden"));
    pane.classList.remove("hidden");
    const path = document.body.dataset.activeDocumentPath ?? null;
    void refreshHistoryPanel(path);
  });
}

export function showHistoryPanelTab() {
  document.getElementById("script-output-panel")?.classList.remove("hidden");
  document.getElementById("bottom-panel-tab-history")?.click();
}
