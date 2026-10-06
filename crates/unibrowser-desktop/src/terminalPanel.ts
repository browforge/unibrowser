import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "@xterm/xterm/css/xterm.css";

let sessionId: number | null = null;
let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let hostEl: HTMLElement | null = null;
let panelEl: HTMLElement | null = null;
let outputListenerReady = false;

export function initTerminalPanel() {
  panelEl = document.getElementById("script-output-panel");
  hostEl = document.getElementById("terminal-host");
  window.addEventListener("unibrowser-bottom-panel-resize", () => fitTerminal());
  void ensureOutputListener();
}

async function ensureOutputListener() {
  if (outputListenerReady) {
    return;
  }
  outputListenerReady = true;
  await listen<{ sessionId: number; data: string }>("terminal-output", (event) => {
    if (sessionId !== event.payload.sessionId || !term) {
      return;
    }
    term.write(event.payload.data);
  });
}

async function ensureTerminal(): Promise<void> {
  if (!hostEl) {
    return;
  }
  if (term) {
    return;
  }
  term = new Terminal({
    cursorBlink: true,
    fontSize: 13,
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
    theme: { background: "#1e1e24" },
  });
  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  term.open(hostEl);
  term.onData((data) => {
    if (sessionId === null) {
      return;
    }
    void invoke("terminal_write", { sessionId, data });
  });
  if (sessionId === null) {
    sessionId = await invoke<number>("terminal_spawn", { cwd: null });
  }
  fitTerminal();
}

function fitTerminal() {
  if (!term || !fitAddon || !hostEl) {
    return;
  }
  fitAddon.fit();
  if (sessionId !== null) {
    void invoke("terminal_resize", {
      sessionId,
      cols: term.cols,
      rows: term.rows,
    });
  }
}

export async function showTerminalPanel() {
  if (!panelEl) {
    panelEl = document.getElementById("script-output-panel");
  }
  panelEl?.classList.remove("hidden");
  selectBottomPanelTab("terminal");
  await ensureTerminal();
  fitTerminal();
  term?.focus();
}

export function hideBottomPanel() {
  panelEl?.classList.add("hidden");
}

export function toggleTerminalPanel() {
  if (!panelEl) {
    return;
  }
  const terminalVisible =
    !panelEl.classList.contains("hidden") &&
    document.getElementById("bottom-panel-terminal-pane")?.classList.contains("hidden") === false;
  if (terminalVisible) {
    panelEl.classList.add("hidden");
    return;
  }
  void showTerminalPanel();
}

export function selectBottomPanelTab(which: "output" | "terminal" | "problems") {
  const outputPane = document.getElementById("bottom-panel-output-pane");
  const terminalPane = document.getElementById("bottom-panel-terminal-pane");
  const problemsPane = document.getElementById("bottom-panel-problems-pane");
  const outputTab = document.getElementById("bottom-panel-tab-output");
  const terminalTab = document.getElementById("bottom-panel-tab-terminal");
  const problemsTab = document.getElementById("bottom-panel-tab-problems");
  const deactivateAll = () => {
    outputPane?.classList.add("hidden");
    terminalPane?.classList.add("hidden");
    problemsPane?.classList.add("hidden");
    outputPane?.setAttribute("aria-hidden", "true");
    terminalPane?.setAttribute("aria-hidden", "true");
    problemsPane?.setAttribute("aria-hidden", "true");
    outputTab?.classList.remove("is-active");
    terminalTab?.classList.remove("is-active");
    problemsTab?.classList.remove("is-active");
    outputTab?.setAttribute("aria-selected", "false");
    terminalTab?.setAttribute("aria-selected", "false");
    problemsTab?.setAttribute("aria-selected", "false");
  };
  if (which === "problems") {
    deactivateAll();
    problemsPane?.classList.remove("hidden");
    problemsPane?.setAttribute("aria-hidden", "false");
    problemsTab?.classList.add("is-active");
    problemsTab?.setAttribute("aria-selected", "true");
    return;
  }
  if (which === "terminal") {
    deactivateAll();
    terminalPane?.classList.remove("hidden");
    terminalPane?.setAttribute("aria-hidden", "false");
    terminalTab?.classList.add("is-active");
    terminalTab?.setAttribute("aria-selected", "true");
    return;
  }
  deactivateAll();
  outputPane?.classList.remove("hidden");
  outputPane?.setAttribute("aria-hidden", "false");
  outputTab?.classList.add("is-active");
  outputTab?.setAttribute("aria-selected", "true");
}
