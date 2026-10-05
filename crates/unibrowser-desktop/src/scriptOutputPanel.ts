import { hideBottomPanel, selectBottomPanelTab, showTerminalPanel } from "./terminalPanel";

let host: HTMLElement | null = null;
let logEl: HTMLElement | null = null;

export function initScriptOutputPanel() {
  host = document.getElementById("script-output-panel");
  logEl = document.getElementById("script-output-log");
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
