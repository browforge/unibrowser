import { listDocumentHistory } from "./documentModelBridge";
import {
  formatHistoryTimestamp,
  groupHistoryRows,
  type HistoryRow,
} from "./historyPanelEntries";
import {
  jumpHistoryToSeq,
  setHistoryPanelSelection,
} from "./historyCommands";

let mounted = false;
let selectedSeq: number | null = null;

function renderConfirmStrip(
  parent: HTMLElement,
  message: string,
  confirmLabel: string,
  onConfirm: () => void,
) {
  parent.querySelector(".history-panel-confirm")?.remove();
  const strip = document.createElement("div");
  strip.className = "history-panel-confirm";
  strip.innerHTML = `<p></p><div class="history-panel-confirm-actions"></div>`;
  strip.querySelector("p")!.textContent = message;
  const actions = strip.querySelector(".history-panel-confirm-actions")!;
  const cancel = document.createElement("button");
  cancel.type = "button";
  cancel.textContent = "Cancel";
  cancel.onclick = () => strip.remove();
  const ok = document.createElement("button");
  ok.type = "button";
  ok.textContent = confirmLabel;
  ok.onclick = () => {
    strip.remove();
    onConfirm();
  };
  actions.appendChild(cancel);
  actions.appendChild(ok);
  parent.prepend(strip);
}

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
  let rows: HistoryRow[] = [];
  try {
    rows = (await listDocumentHistory(path)) as HistoryRow[];
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
  const grouped = groupHistoryRows(rows);
  const table = document.createElement("table");
  table.className = "history-panel-table";
  table.innerHTML =
    "<thead><tr><th>Seq</th><th>Time</th><th>Source</th><th>Summary</th></tr></thead>";
  const body = document.createElement("tbody");
  for (const entry of grouped) {
    const tr = document.createElement("tr");
    tr.dataset.seq = String(entry.seq);
    tr.dataset.endSeq = String(entry.endSeq);
    tr.tabIndex = 0;
    if (selectedSeq === entry.seq) {
      tr.classList.add("is-selected");
    }
    const seqLabel = entry.opCount > 1 ? `${entry.seq}–${entry.endSeq}` : String(entry.seq);
    tr.innerHTML = `<td>${seqLabel}</td><td>${formatHistoryTimestamp(entry.timestamp)}</td><td>${entry.plugin_id}</td><td>${entry.label}</td>`;
    tr.addEventListener("click", () => {
      selectedSeq = entry.seq;
      setHistoryPanelSelection(entry.seq, entry.endSeq);
      body.querySelectorAll("tr.is-selected").forEach((r) => r.classList.remove("is-selected"));
      tr.classList.add("is-selected");
    });
    tr.addEventListener("dblclick", () => {
      void jumpHistoryToSeq(path, entry.endSeq);
    });
    tr.addEventListener("keydown", (ev) => {
      if (ev.key === "Enter") {
        ev.preventDefault();
        renderConfirmStrip(
          list,
          `Restore document to seq ${entry.endSeq}? A new snapshot is appended; later log entries stay on disk.`,
          "Jump to here",
          () => {
            void jumpHistoryToSeq(path, entry.endSeq).then(() => refreshHistoryPanel(path));
          },
        );
      }
    });
    body.appendChild(tr);
  }
  table.appendChild(body);
  list.appendChild(table);
  if (selectedSeq == null && grouped[0]) {
    setHistoryPanelSelection(grouped[0].seq, grouped[0].endSeq);
    selectedSeq = grouped[0].seq;
    body.querySelector("tr")?.classList.add("is-selected");
  }
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
