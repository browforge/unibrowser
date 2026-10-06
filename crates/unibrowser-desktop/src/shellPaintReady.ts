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

function monacoReady(): boolean {
  const lines = document.querySelector(".monaco-editor .view-lines");
  if (!lines) {
    return false;
  }
  return (lines.textContent ?? "").trim().length > 0;
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
  const grants = panel.querySelector(".ai-assistant-grants .ai-grants-row");
  if (grants) {
    return true;
  }
  return panel.querySelector(".ai-assistant-body") !== null;
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
  const panel = document.getElementById("problems-bottom-panel");
  return Boolean(panel && !panel.classList.contains("hidden"));
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
  if (joined.includes(".txt") || joined.includes(".js") || joined.includes("spell-test")) {
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

/** Wait for shell/webview paint before doc-capture screenshots. */
export async function waitForShellPaintReady(
  demoRaw: string | null,
  launchPaths: string[],
  timeoutMs = 18_000,
): Promise<void> {
  const checks = checksForDemo(demoRaw, launchPaths);
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (checks.every((fn) => fn())) {
      await sleep(120);
      return;
    }
    await sleep(80);
  }
}
