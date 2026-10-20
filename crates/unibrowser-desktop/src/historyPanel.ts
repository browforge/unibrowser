import { listDocumentHistory } from "./documentModelBridge";
import {
  formatHistoryTimestamp,
  groupHistoryRows,
  type HistoryRow,
} from "./historyPanelEntries";
import {
  jumpHistoryToSeq,
  runHistoryCommand,
  setHistoryPanelSelection,
  HISTORY_SQUASH_SELECTION_ID,
  HISTORY_TRIM_BEFORE_ID,
} from "./historyCommands";

let mounted = false;
let selectedSeq: number | null = null;
let refreshInFlight: Promise<void> | null = null;
let refreshEpoch = 0;

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

function selectRow(body: HTMLElement, tr: HTMLTableRowElement, entry: { seq: number; endSeq: number }) {
  selectedSeq = entry.seq;
  setHistoryPanelSelection(entry.seq, entry.endSeq);
  body.querySelectorAll("tr.is-selected").forEach((r) => r.classList.remove("is-selected"));
  tr.classList.add("is-selected");
  document.body.dataset.ubHistorySelectedSeq = String(entry.seq);
}

export function selectHistoryPanelFirstRow() {
  selectedSeq = null;
  const body = document.querySelector("#history-panel-list tbody");
  const tr = body?.querySelector<HTMLTableRowElement>("tr");
  if (!body || !tr) {
    return;
  }
  const seq = Number.parseInt(tr.dataset.seq ?? "0", 10);
  const endSeq = Number.parseInt(tr.dataset.endSeq ?? String(seq), 10);
  selectRow(body as HTMLElement, tr, { seq, endSeq });
}

export function selectHistoryPanelRow(seq: number) {
  selectedSeq = seq;
  const body = document.querySelector("#history-panel-list tbody");
  if (!body) {
    return;
  }
  const tr = body.querySelector<HTMLTableRowElement>(`tr[data-seq="${seq}"]`);
  if (!tr) {
    return;
  }
  const endSeq = Number.parseInt(tr.dataset.endSeq ?? String(seq), 10);
  selectRow(body as HTMLElement, tr, { seq, endSeq });
}

export function selectHistoryPanelRowByEndSeq(endSeq: number) {
  const body = document.querySelector("#history-panel-list tbody");
  if (!body) {
    return;
  }
  const tr = body.querySelector<HTMLTableRowElement>(`tr[data-end-seq="${endSeq}"]`);
  if (!tr) {
    return;
  }
  const seq = Number.parseInt(tr.dataset.seq ?? "0", 10);
  selectRow(body as HTMLElement, tr, { seq, endSeq });
}

export async function refreshHistoryPanel(path: string | null): Promise<void> {
  if (refreshInFlight) {
    await refreshInFlight;
  }
  const epoch = ++refreshEpoch;
  refreshInFlight = (async () => {
    const list = document.getElementById("history-panel-list");
    if (!list) {
      return;
    }
    list.replaceChildren();
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
    if (epoch !== refreshEpoch) {
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
    const headEndSeq = grouped[0]?.endSeq ?? 0;
    const table = document.createElement("table");
    table.className = "history-panel-table";
    table.innerHTML =
      "<thead><tr><th class=\"col-seq\">Seq</th><th class=\"col-time\">Time</th><th class=\"col-source\">Source</th><th class=\"col-summary\">Summary</th><th class=\"col-actions\"></th></tr></thead>";
    const body = document.createElement("tbody");
    for (const entry of grouped) {
      const tr = document.createElement("tr");
      tr.dataset.seq = String(entry.seq);
      tr.dataset.endSeq = String(entry.endSeq);
      tr.tabIndex = 0;
      const isHead = entry.endSeq === headEndSeq;
      tr.dataset.isHead = isHead ? "1" : "0";
      if (selectedSeq === entry.seq) {
        tr.classList.add("is-selected");
      }
      const seqLabel = entry.opCount > 1 ? `${entry.seq}–${entry.endSeq}` : String(entry.seq);
      const time = formatHistoryTimestamp(entry.timestamp);
      const source = document.createElement("span");
      source.className = "history-source-pill";
      source.textContent = entry.plugin_id;
      const summaryCell = document.createElement("td");
      summaryCell.className = "col-summary";
      summaryCell.textContent = entry.label;
      if (isHead) {
        const pill = document.createElement("span");
        pill.className = "history-current-pill";
        pill.textContent = "Current";
        summaryCell.appendChild(document.createTextNode(" "));
        summaryCell.appendChild(pill);
      }
      tr.innerHTML = `<td class="col-seq">${seqLabel}</td><td class="col-time" title=""></td><td class="col-source"></td><td class="col-actions"></td>`;
      tr.insertBefore(summaryCell, tr.querySelector(".col-actions"));
      const timeCell = tr.querySelector(".col-time") as HTMLElement;
      timeCell.textContent = time.display;
      timeCell.title = time.title;
      tr.querySelector(".col-source")?.appendChild(source);
      if (!isHead) {
        const restore = document.createElement("button");
        restore.type = "button";
        restore.className = "history-restore-btn";
        restore.textContent = "Restore";
        restore.onclick = (ev) => {
          ev.stopPropagation();
          renderConfirmStrip(
            list,
            `Restore document to seq ${entry.endSeq}? A new entry is appended; later log entries stay on disk.`,
            "Restore",
            () => {
              void jumpHistoryToSeq(path, entry.endSeq).then(() => refreshHistoryPanel(path));
            },
          );
        };
        tr.querySelector(".col-actions")?.appendChild(restore);
        tr.addEventListener("keydown", (ev) => {
          if (ev.key === "Enter") {
            ev.preventDefault();
            restore.click();
          }
        });
      }
      tr.addEventListener("click", () => selectRow(body, tr, entry));
      body.appendChild(tr);
    }
    if (epoch !== refreshEpoch) {
      return;
    }
    table.appendChild(body);
    list.replaceChildren(table);
    document.body.dataset.ubHistoryTableCount = String(
      list.querySelectorAll(".history-panel-table").length,
    );
    document.body.dataset.ubHistoryHeaderCount = String(
      list.querySelectorAll(".history-panel-table thead").length,
    );
    if (selectedSeq == null && grouped[0] && grouped[0].endSeq !== headEndSeq) {
      selectRow(body, body.querySelector("tr")!, grouped[0]);
    } else if (selectedSeq == null && grouped[1]) {
      selectRow(body, body.querySelectorAll("tr")[1] as HTMLTableRowElement, grouped[1]);
    } else if (selectedSeq != null) {
      selectHistoryPanelRow(selectedSeq);
    }
  })();
  try {
    await refreshInFlight;
  } finally {
    refreshInFlight = null;
  }
}

function wireToolbarActions() {
  document.getElementById("history-action-trim")?.addEventListener("click", () => {
    void runHistoryCommand(HISTORY_TRIM_BEFORE_ID);
  });
  document.getElementById("history-action-squash")?.addEventListener("click", () => {
    void runHistoryCommand(HISTORY_SQUASH_SELECTION_ID);
  });
}

export function mountHistoryPanel() {
  if (mounted) {
    return;
  }
  mounted = true;
  const tab = document.getElementById("bottom-panel-tab-history");
  const pane = document.getElementById("bottom-panel-history-pane");
  if (!tab || !pane) {
    mounted = false;
    return;
  }
  pane.replaceChildren();
  const shell = document.createElement("div");
  shell.className = "history-panel-shell";
  shell.innerHTML = `<div class="history-panel-toolbar" role="toolbar" aria-label="History actions">
      <button type="button" id="history-action-trim" class="history-panel-action">Trim before here</button>
      <button type="button" id="history-action-squash" class="history-panel-action">Squash selection</button>
    </div>
    <div id="history-panel-list" class="history-panel-list" role="log" aria-live="polite"></div>`;
  pane.appendChild(shell);
  wireToolbarActions();
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
  const tab = document.getElementById("bottom-panel-tab-history");
  const pane = document.getElementById("bottom-panel-history-pane");
  if (!tab || !pane) {
    return;
  }
  document.querySelectorAll(".bottom-panel-tab").forEach((el) => {
    el.classList.remove("is-active");
    el.setAttribute("aria-selected", "false");
  });
  tab.classList.add("is-active");
  tab.setAttribute("aria-selected", "true");
  document.querySelectorAll(".bottom-panel-pane").forEach((el) => el.classList.add("hidden"));
  pane.classList.remove("hidden");
}
