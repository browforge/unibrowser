function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function hasVisibleText(selector: string, minLen = 1): boolean {
  const el = document.querySelector(selector);
  if (!el || el instanceof HTMLElement && el.classList.contains("hidden")) {
    return false;
  }
  const text = (el.textContent ?? "").trim();
  return text.length >= minLen;
}

export function editorReadyForCapture(minChars = 80): boolean {
  const host = document.getElementById("text-editor-host");
  if (!host || host.classList.contains("hidden")) {
    return false;
  }
  const monacoRoot = host.querySelector(".monaco-editor") as HTMLElement | null;
  if (!monacoRoot) {
    return false;
  }
  const hostH = host.clientHeight;
  const editorH = monacoRoot.clientHeight;
  if (hostH > 80 && editorH < hostH * 0.35) {
    return false;
  }
  const lines = document.querySelector(".monaco-editor .view-lines");
  if (!lines) {
    return false;
  }
  const text = (lines.textContent ?? "").trim();
  return text.length >= minChars;
}

function monacoReady(): boolean {
  if (document.body.classList.contains("doc-capture-mode")) {
    return editorReadyForCapture(80);
  }
  const host = document.getElementById("text-editor-host");
  if (!host || host.classList.contains("hidden")) {
    return false;
  }
  const lines = document.querySelector(".monaco-editor .view-lines");
  return (lines?.textContent ?? "").trim().length >= 8;
}

function unibrowBoardReady(): boolean {
  const host = document.querySelector("#document-viewer-host");
  if (!host || host.classList.contains("hidden")) {
    return false;
  }
  const board = host.querySelector(".unibrow-board-canvas, .unibrow-board-hint, .unibrow-live-panel");
  return Boolean(board);
}

function liveBadgeReady(): boolean {
  const badge = document.querySelector(".unibrow-live-indicator");
  return Boolean(badge && !badge.classList.contains("hidden"));
}

function aiPanelReady(): boolean {
  const panel = document.getElementById("ai-assistant-panel");
  if (!panel || panel.classList.contains("hidden")) {
    return false;
  }
  return panel.querySelector(".ai-assistant-composer-input") !== null;
}

function ntpReady(): boolean {
  const host = document.getElementById("new-tab-page-host");
  if (!host || host.classList.contains("hidden")) {
    return false;
  }
  return hasVisibleText(".ntp-hero h1", 2);
}

function browseFolderReady(): boolean {
  const panel = document.getElementById("files-panel");
  if (!panel || panel.classList.contains("hidden")) {
    return false;
  }
  return panel.querySelector(".files-entry, .files-row, [data-path], tbody tr") !== null;
}

function docViewerReady(): boolean {
  const host = document.querySelector("#document-viewer-host");
  if (!host || host.classList.contains("hidden")) {
    return false;
  }
  return host.querySelector(".docx-reader, .pdf-page, .viewer-note, section") !== null;
}

function problemsReady(): boolean {
  const panel = document.getElementById("script-output-panel");
  if (!panel || panel.classList.contains("hidden")) {
    return false;
  }
  const pane = document.getElementById("bottom-panel-problems-pane");
  if (!pane || pane.classList.contains("hidden")) {
    return false;
  }
  const rows = panel.querySelector(".problems-table tbody tr");
  const countEl = document.getElementById("status-problems-count");
  if (rows) {
    return true;
  }
  return Boolean(countEl && !countEl.classList.contains("hidden"));
}

function paletteReady(): boolean {
  const palette = document.getElementById("palette");
  return Boolean(palette && !palette.classList.contains("hidden"));
}

function settingsReady(): boolean {
  const host = document.querySelector("#document-viewer-host .settings-page, [data-settings-section]");
  return Boolean(host);
}

function checksForDemo(demoRaw: string | null, launchPaths: string[]): Array<() => boolean> {
  const tokens = (demoRaw ?? "")
    .split(/[,;]+/)
    .map((t) => t.trim().toLowerCase())
    .filter(Boolean);
  const checks: Array<() => boolean> = [];

  for (const token of tokens) {
    switch (token) {
      case "ntp":
        checks.push(ntpReady);
        break;
      case "palette":
        checks.push(paletteReady);
        break;
      case "settings":
        checks.push(settingsReady);
        break;
      case "problems":
      case "problems-open":
      case "document-mockup":
      case "document-mockup-problems":
        checks.push(problemsReady);
        break;
      case "ai":
      case "ai-sidebar":
      case "ai-open":
        checks.push(aiPanelReady);
        break;
      case "live":
      case "live-preview":
        checks.push(unibrowBoardReady, liveBadgeReady);
        break;
      default:
        break;
    }
  }

  const joined = launchPaths.join("|").toLowerCase();
  if (joined.includes(".unibrow")) {
    checks.push(unibrowBoardReady);
    if (tokens.some((t) => t === "live" || t === "live-preview")) {
      checks.push(liveBadgeReady);
    }
  }
  if (
    joined.includes(".txt") ||
    joined.includes(".md") ||
    joined.includes(".markdown") ||
    joined.includes(".ts") ||
    joined.includes(".js") ||
    joined.includes("spell-test") ||
    joined.includes("sync-notes")
  ) {
    checks.push(monacoReady);
  }
  if (joined.includes(".docx") || joined.includes(".pdf") || joined.includes(".svg")) {
    checks.push(docViewerReady);
  }
  if (tokens.includes("files") || (joined.includes("demo-pack") && !joined.includes(".unibrow"))) {
    checks.push(browseFolderReady);
  }

  if (checks.length === 0) {
    checks.push(() => document.querySelector("#shell-root, #app-shell, body") !== null);
  }
  return checks;
}

export function launchPathsNeedEditor(launchPaths: string[]): boolean {
  const joined = launchPaths.join("|").toLowerCase();
  return (
    joined.includes(".txt") ||
    joined.includes(".md") ||
    joined.includes(".markdown") ||
    joined.includes(".ts") ||
    joined.includes(".js") ||
    joined.includes("spell-test") ||
    joined.includes("sync-notes")
  );
}

export async function ensureTextTabActiveForCapture(launchPaths: string[]): Promise<void> {
  if (!launchPathsNeedEditor(launchPaths)) {
    return;
  }
  const host = document.getElementById("text-editor-host");
  if (host && !host.classList.contains("hidden")) {
    return;
  }
  const textIndex = launchPaths.findIndex((p) =>
    /\.tsx?|sync-notes|spell-test/i.test(p),
  );
  const { invoke } = await import("@tauri-apps/api/core");
  const pick = textIndex >= 0 ? textIndex : 0;
  await invoke("select_tab", { index: pick }).catch(() => undefined);
  await sleep(300);
  const hostAfter = document.getElementById("text-editor-host");
  if (hostAfter && !hostAfter.classList.contains("hidden")) {
    return;
  }
  const tabs = document.querySelectorAll<HTMLElement>(".title-tabs .tab");
  for (const tab of tabs) {
    const label = tab.textContent ?? "";
    if (/\.tsx?|sync-notes|spell-test/i.test(label)) {
      tab.click();
      await sleep(200);
      return;
    }
  }
}

async function nudgeEditorLayout(): Promise<void> {
  try {
    const { relayoutTextEditor } = await import("./textEditor");
    relayoutTextEditor();
  } catch {
    /* editor module not ready */
  }
}

/** Wait for shell/webview paint before doc-capture screenshots. */
export async function waitForShellPaintReady(
  demoRaw: string | null,
  launchPaths: string[],
  timeoutMs = 18_000,
): Promise<boolean> {
  const checks = checksForDemo(demoRaw, launchPaths);
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    await ensureTextTabActiveForCapture(launchPaths);
    await nudgeEditorLayout();
    if (checks.every((fn) => fn())) {
      await nudgeEditorLayout();
      await sleep(200);
      if (checks.every((fn) => fn())) {
        return true;
      }
    }
    await sleep(80);
  }
  const failing = checks.filter((fn) => !fn()).length;
  console.error(`[paint-wait] timed out (${failing}/${checks.length} checks failing)`);
  return false;
}

const win = window as unknown as {
  __unibrowserEditorReadyForCapture?: () => boolean;
  __unibrowserWaitForShellPaintReady?: (
    demoRaw: string | null,
    launchPaths: string[],
    timeoutMs?: number,
  ) => Promise<boolean>;
};

win.__unibrowserEditorReadyForCapture = () => editorReadyForCapture();
win.__unibrowserWaitForShellPaintReady = waitForShellPaintReady;
