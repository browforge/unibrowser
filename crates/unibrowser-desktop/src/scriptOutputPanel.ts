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
  terminalTab?.addEventListener("click", (ev) => {
    ev.preventDefault();
  });
}

function selectBottomPanelTab(which: "output" | "terminal") {
  if (which === "terminal") {
    return;
  }
  const outputPane = document.getElementById("bottom-panel-output-pane");
  const terminalPane = document.getElementById("bottom-panel-terminal-pane");
  const outputTab = document.getElementById("bottom-panel-tab-output");
  const terminalTab = document.getElementById("bottom-panel-tab-terminal");
  outputPane?.classList.remove("hidden");
  terminalPane?.classList.add("hidden");
  terminalPane?.setAttribute("aria-hidden", "true");
  outputPane?.setAttribute("aria-hidden", "false");
  outputTab?.classList.add("is-active");
  outputTab?.setAttribute("aria-selected", "true");
  terminalTab?.classList.remove("is-active");
  terminalTab?.setAttribute("aria-selected", "false");
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
  host?.classList.add("hidden");
}
