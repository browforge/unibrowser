import { hideBottomPanel, selectBottomPanelTab, showTerminalPanel } from "./terminalPanel";

let host: HTMLElement | null = null;
let logEl: HTMLElement | null = null;

export function syncBottomPanelEmptyState() {
  const panel = document.getElementById("script-output-panel");
  const log = document.getElementById("script-output-log");
  if (!panel || !log) {
    return;
  }
  const hasOutput = Boolean(log.textContent?.trim());
  const problemRows = document.querySelectorAll(".problems-table tbody tr").length;
  const terminalPane = document.getElementById("bottom-panel-terminal-pane");
  const terminalOpen =
    terminalPane &&
    !terminalPane.classList.contains("hidden") &&
    !panel.classList.contains("hidden");
  if (!hasOutput && problemRows === 0 && !terminalOpen) {
    panel.classList.add("hidden");
  }
}

export function initScriptOutputPanel() {
  host = document.getElementById("script-output-panel");
  logEl = document.getElementById("script-output-log");
  syncBottomPanelEmptyState();
  const close = document.getElementById("script-output-close");
  if (close) {
    close.onclick = () => hideScriptOutputPanel();
  }
  const outputTab = document.getElementById("bottom-panel-tab-output");
  const terminalTab = document.getElementById("bottom-panel-tab-terminal");
  outputTab?.addEventListener("click", () => selectBottomPanelTab("output"));
  terminalTab?.addEventListener("click", () => {
    host?.classList.remove("hidden");
    selectBottomPanelTab("terminal");
    void showTerminalPanel();
  });
}

export function appendScriptOutput(line: string) {
  if (!host || !logEl) {
    return;
  }
  selectBottomPanelTab("output");
  host.classList.remove("hidden");
  const row = document.createElement("div");
  row.className = "script-output-line";
  row.textContent = line;
  logEl.appendChild(row);
  logEl.scrollTop = logEl.scrollHeight;
}

export function hideScriptOutputPanel() {
  hideBottomPanel();
}
