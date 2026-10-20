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
  const historyPane = document.getElementById("bottom-panel-history-pane");
  if (historyPane && !historyPane.classList.contains("hidden")) {
    return true;
  }
  if (richTextDocumentReady()) {
    return true;
  }
  const host = document.getElementById("text-editor-host");
  if (!host || host.classList.contains("hidden")) {
    return false;
  }
  const mode = document.body.dataset.documentEditorMode;
  if (mode === "wysiwyg" || mode === "reading") {
    return richTextDocumentReady();
  }
  const monacoRoot = host.querySelector(".monaco-editor") as HTMLElement | null;
  if (!monacoRoot || monacoRoot.getBoundingClientRect().width <= 0) {
    return false;
  }
  const lines = host.querySelector(".view-lines");
  if (!lines) {
    return false;
  }
  const text = (lines.textContent ?? "").trim();
  return text.length >= minChars;
}

function monacoReady(): boolean {
  return editorReadyForCapture(80);
}

function richTextDocumentReady(): boolean {
  const article =
    document.querySelector<HTMLElement>(
      "#rich-text-editor-host:not(.hidden) .rich-text-doc",
    ) ??
    document.querySelector<HTMLElement>(
      "#rich-text-editor-host:not(.hidden) .ProseMirror",
    ) ??
    document.querySelector<HTMLElement>(
      "#document-split-preview-host:not(.hidden) .rich-text-doc",
    );
  if (!article || article.getBoundingClientRect().height < 48) {
    return false;
  }
  return (article.textContent ?? "").trim().length >= 20;
}

function richTextSplitPreviewReady(): boolean {
  const article = document.querySelector<HTMLElement>(
    "#document-split-preview-host:not(.hidden) .rich-text-doc",
  );
  if (!article || article.getBoundingClientRect().width < 80) {
    return false;
  }
  return (article.textContent ?? "").trim().length >= 20;
}

function markdownPreviewReady(): boolean {
  const shell = document.querySelector<HTMLElement>(".md-preview-shell");
  if (!shell || shell.classList.contains("hidden")) {
    return false;
  }
  const preview = shell.querySelector<HTMLElement>(".md-preview-body");
  if (!preview || preview.getBoundingClientRect().height <= 0) {
    return false;
  }
  const rect = preview.getBoundingClientRect();
  if (rect.height < 64 || rect.width < 160) {
    return false;
  }
  return (preview.textContent ?? "").trim().length >= 10;
}

function unibrowBoardReady(): boolean {
  const host = document.querySelector("#document-viewer-host");
  if (!host || host.classList.contains("hidden")) {
    return false;
  }
  const board = host.querySelector(
    ".workshop-stage-frame, .workshop-programme-card--live, .workshop-timeline-canvas",
  );
  return Boolean(board);
}

function liveBadgeReady(): boolean {
  const state = document.querySelector(".workshop-state");
  if (state) {
    return state.getAttribute("data-state") === "live" ||
      state.getAttribute("data-state") === "recording";
  }
  const badge = document.querySelector(".workshop-programme-card--live");
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
  if (!hasVisibleText(".ntp-hero h1, #new-tab-page-host h1", 2)) {
    return false;
  }
  return host.querySelector(".ntp-actions, .ntp-recent-list") !== null;
}

function browseFolderReady(): boolean {
  const panel = document.getElementById("files-panel");
  if (!panel || panel.classList.contains("hidden")) {
    return false;
  }
  return panel.querySelector(".files-entry, .files-row, [data-path], tbody tr") !== null;
}

function docViewerReady(): boolean {
  const host = document.querySelector<HTMLElement>("#document-viewer-host");
  if (!host || host.classList.contains("hidden")) {
    return false;
  }
  if (host.querySelector(".image-viewer-layout")) {
    const image = host.querySelector<HTMLImageElement>(".image-view");
    const stage = host.querySelector<HTMLElement>(".image-stage");
    return Boolean(
      image &&
      image.getBoundingClientRect().width > 0 &&
      stage?.style.transform.includes("scale("),
    );
  }
  const mediaViewer = host.querySelector<HTMLElement>(".media-viewer");
  if (mediaViewer) {
    return mediaViewer.dataset.mediaReady === "1";
  }
  if (
    host.querySelector(
      ".docx-reader, .pdf-page, .pdf-workspace, .pdf-page-wrap, .svg-workspace, .svg-preview--stage, .viewer-note, section",
    )
  ) {
    return true;
  }
  return host.childElementCount > 0 && host.textContent.trim().length > 0;
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
  if (tokens.includes("new-project-capture-empty")) {
    return [
      () =>
        Boolean(
          document.querySelector(".project-home-card, .project-surface--empty"),
        ),
    ];
  }
  if (tokens.includes("new-project-capture-presentation-project")) {
    return [
      () =>
        Boolean(
          document.querySelector(
            ".project-surface--presentation .presentation-slide-frame",
          ),
        ),
    ];
  }
  if (tokens.includes("new-project-capture-office-project")) {
    return [
      () => document.body.classList.contains("text-document-card"),
      richTextDocumentReady,
      () => {
        const host = document.getElementById("text-compact-toolbar-host");
        if (!host) {
          return false;
        }
        const count = host.querySelectorAll<HTMLElement>(
          ".text-compact-btn[data-control-id], .text-compact-block-trigger[data-control-id]",
        ).length;
        return count >= 6;
      },
    ];
  }
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
      case "ui17-unknown-support":
        checks.push(() => {
          const panel = document.getElementById("unknown-file-banner");
          return Boolean(
            panel &&
              !panel.classList.contains("hidden") &&
              panel.classList.contains("unknown-support-panel"),
          );
        });
        break;
      case "ui17-mod-portal":
        checks.push(() => {
          const portal = document.getElementById("mod-portal-panel");
          return Boolean(
            portal &&
              !portal.classList.contains("hidden") &&
              portal.querySelector(".mod-result-card"),
          );
        });
        break;
      case "ui18-layout-edit":
        checks.push(() => {
          const overlay = document.querySelector(".layout-edit-overlay");
          return Boolean(overlay && document.body.classList.contains("layout-edit-mode"));
        });
        break;
      case "ui19-focus-visible":
        checks.push(
          () => document.activeElement?.classList.contains("ui-command") ?? false,
        );
        break;
      case "doc-opaque-card":
        checks.push(richTextDocumentReady);
        checks.push(() => Boolean(document.querySelector(".rich-text-opaque-parse")));
        break;
      case "doc-history-panel":
      case "doc-history-jump":
        checks.push(() => {
          const pane = document.getElementById("bottom-panel-history-pane");
          if (!pane || pane.classList.contains("hidden")) {
            return false;
          }
          if (document.body.dataset.ubHistoryCaptureReady !== "1") {
            return false;
          }
          const headerCount = document.querySelectorAll(".history-panel-table thead").length;
          if (headerCount !== 1) {
            return false;
          }
          const selected = document.querySelector(
            ".history-panel-table tr.is-selected",
          ) as HTMLElement | null;
          const restoreVisible = Boolean(selected?.querySelector(".history-restore-btn"));
          const toolbar = document.getElementById("history-action-trim");
          if (!restoreVisible || !toolbar) {
            return false;
          }
          if (selected?.dataset.isHead === "1") {
            return false;
          }
          const topRow = document.querySelector(
            ".history-panel-table tbody tr:first-child",
          ) as HTMLElement | null;
          const topIsCurrent = Boolean(topRow?.querySelector(".history-current-pill"));
          if (token === "doc-history-jump") {
            const topSummary = document.querySelector(
              ".history-panel-table tbody tr:first-child .col-summary",
            );
            return (
              document.body.dataset.ubHistoryJumpCapture === "1" &&
              topIsCurrent &&
              (topSummary?.textContent?.includes("Restored to #") ?? false)
            );
          }
          if (!topIsCurrent) {
            return false;
          }
          const docText = document.querySelector(".rich-text-doc")?.textContent ?? "";
          return docText.includes("Edit three") && docText.includes("Edited heading");
        });
        break;
      case "new-project-capture-empty":
        checks.push(() => {
          const host = document.getElementById("document-viewer-host");
          if (!host || host.classList.contains("hidden")) {
            return false;
          }
          return Boolean(
            host.querySelector(".project-surface--empty, .project-home-card, .project-surface"),
          );
        });
        break;
      case "new-project-capture-presentation-project":
        checks.push(() =>
          Boolean(
            document.querySelector(
              ".project-surface--presentation .presentation-slide-frame",
            ),
          ),
        );
        break;
      case "new-project-capture-office-project":
        checks.push(richTextDocumentReady);
        break;
      case "new-project-capture-gallery":
      case "new-project-capture-gallery-office":
      case "new-project-capture-configure":
        checks.push(() =>
          Boolean(document.querySelector(".template-gallery-panel")),
        );
        break;
      case "new-project-capture-save-template":
        checks.push(() =>
          Boolean(
            document.querySelector(
              ".save-template-overlay .save-template-panel, .template-gallery-panel",
            ),
          ),
        );
        break;
      case "new-project-capture-relationship-consent":
        checks.push(() =>
          Boolean(
            document.querySelector(
              ".relationship-consent-panel .relationship-consent-footer .relationship-consent-actions",
            ),
          ),
        );
        break;
      case "new-project-capture-first-save":
        checks.push(() =>
          Boolean(
            document.querySelector(
              ".first-save-panel .first-save-path-preview, .first-save-panel .first-save-actions .quiet-button",
            ),
          ),
        );
        break;
      default:
        break;
    }
  }

  const joined = launchPaths.join("|").toLowerCase();
  if (tokens.includes("settings")) {
    return [settingsReady];
  }
  if (joined.includes(".unibrow")) {
    checks.push(unibrowBoardReady);
    if (tokens.some((t) => t === "live" || t === "live-preview")) {
      checks.push(liveBadgeReady);
    }
  }
  const wantsMarkdownPreview = tokens.includes("content-kind-md-preview");
  if (
    wantsMarkdownPreview &&
    (joined.includes(".md") || joined.includes(".markdown"))
  ) {
    checks.push(markdownPreviewReady);
  } else if (
    tokens.includes("doc-mode-wysiwyg") ||
    tokens.includes("doc-mode-reading") ||
    tokens.includes("doc-visual-md-docx")
  ) {
    checks.push(richTextDocumentReady);
  } else if (tokens.includes("doc-mode-split")) {
    checks.push(monacoReady);
    checks.push(richTextSplitPreviewReady);
  } else if (
    !tokens.includes("doc-opaque-card") &&
    !tokens.some((t) => t.startsWith("doc-history-")) &&
    (joined.includes(".txt") ||
      joined.includes(".md") ||
      joined.includes(".markdown") ||
      joined.includes(".ts") ||
      joined.includes(".js") ||
      joined.includes("spell-test") ||
      joined.includes("sync-notes"))
  ) {
    checks.push(monacoReady);
  }
  if (
    joined.includes(".docx") ||
    joined.includes(".pdf") ||
    joined.includes(".svg") ||
    joined.includes(".png") ||
    joined.includes(".dxf")
  ) {
    checks.push(docViewerReady);
  }
  if (joined.includes(".dxf")) {
    checks.push(() => {
      const host = document.querySelector("#document-viewer-host");
      if (!host || host.classList.contains("hidden")) {
        return false;
      }
      const canvas = host.querySelector("canvas");
      if (!canvas) {
        return false;
      }
      return document.documentElement.dataset.dxfFitZoomReady === "1";
    });
  }
  if (tokens.includes("files")) {
    checks.push(browseFolderReady);
  }

  if (
    tokens.includes("capture-ready") &&
    launchPaths.length === 0 &&
    !tokens.some((t) => t.startsWith("new-project-capture-"))
  ) {
    checks.push(ntpReady);
  }

  if (checks.length === 0) {
    checks.push(() => document.querySelector("#shell-root, #app-shell, body") !== null);
  }
  if (tokens.includes("cm-collapsed")) {
    checks.push(() => {
      const title = document.getElementById("title-bar");
      const chrome = document.getElementById("chrome");
      if (!title || !chrome) {
        return false;
      }
      const collapsed = document.body.classList.contains("cm-collapsed");
      const chromeH = chrome.getBoundingClientRect().height;
      if (collapsed) {
        return chromeH >= 38 && chromeH <= 96;
      }
      return title.getBoundingClientRect().height >= 38;
    });
  }
  if (joined.includes(".pdf")) {
    return [
      () => {
        const host = document.getElementById("document-viewer-host");
        return Boolean(host && !host.classList.contains("hidden"));
      },
      docViewerReady,
    ];
  }
  if (joined.includes(".docx")) {
    return [
      () => {
        const host = document.getElementById("document-viewer-host");
        return Boolean(host && !host.classList.contains("hidden"));
      },
      () => {
        const host = document.getElementById("document-viewer-host");
        return Boolean(host?.querySelector(".docx-reader, .docx-view"));
      },
    ];
  }
  if (
    joined.includes(".svg") ||
    joined.includes(".png") ||
    joined.includes(".jpg") ||
    joined.includes(".jpeg") ||
    joined.includes(".webp")
  ) {
    return [
      () => {
        const host = document.getElementById("document-viewer-host");
        return Boolean(host && !host.classList.contains("hidden"));
      },
      docViewerReady,
    ];
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

function launchPathBasenames(launchPaths: string[]): string[] {
  return launchPaths
    .map((p) => p.split(/[/\\]/).pop()?.toLowerCase() ?? "")
    .filter((name) => name.length > 0);
}

async function syncBrowserStateForCapture(): Promise<void> {
  const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
  if (
    document.body.dataset.ubDocCapture === "1" &&
    (demo.includes("doc-visual-md-docx") ||
      document.body.dataset.ubDocVisualMdDocxApplied === "1")
  ) {
    return;
  }
  const apply = (
    window as unknown as { __selfTestApplyState?: (data: unknown) => Promise<void> }
  ).__selfTestApplyState;
  if (!apply) {
    return;
  }
  const { invoke } = await import("@tauri-apps/api/core");
  const data = await invoke("get_browser_state");
  await apply(data);
}

export function launchPathsNeedDocumentViewer(launchPaths: string[]): boolean {
  const joined = launchPaths.join("|").toLowerCase();
  return (
    joined.includes(".pdf") ||
    joined.includes(".svg") ||
    joined.includes(".png") ||
    joined.includes(".jpg") ||
    joined.includes(".jpeg") ||
    joined.includes(".webp") ||
    joined.includes(".docx")
  );
}

/** Select the tab that matches argv `--open` paths so the viewer host is shown. */
export async function ensureDocumentTabActiveForCapture(launchPaths: string[]): Promise<void> {
  if (!launchPathsNeedDocumentViewer(launchPaths)) {
    return;
  }
  const host = document.getElementById("document-viewer-host");
  if (host && !host.classList.contains("hidden")) {
    return;
  }
  const joined = launchPaths.join("|").toLowerCase();
  const ext =
    [".pdf", ".svg", ".png", ".jpg", ".jpeg", ".webp", ".docx"].find((e) => joined.includes(e)) ??
    "";
  const basenames = launchPathBasenames(launchPaths);
  const tabs = document.querySelectorAll<HTMLElement>(".title-tabs .tab");
  for (const tab of tabs) {
    const label = (tab.textContent ?? "").toLowerCase();
    const matches =
      basenames.some((name) => label === name || label.endsWith(name)) ||
      (ext ? label.includes(ext.replace(".", "")) : false);
    if (matches) {
      tab.click();
      await sleep(300);
      await syncBrowserStateForCapture();
      return;
    }
  }
  const { invoke } = await import("@tauri-apps/api/core");
  const state = await invoke<{ active_index: number; tabs: { label: string }[] }>("get_browser_state");
  const idx = state.tabs.findIndex((t) => {
    const label = t.label.toLowerCase();
    return (
      basenames.some((name) => label === name || label.endsWith(name)) ||
      (ext ? label.includes(ext.replace(".", "")) : false)
    );
  });
  const pick = idx >= 0 ? idx : state.tabs.length > 0 ? state.tabs.length - 1 : -1;
  if (pick >= 0 && pick !== state.active_index) {
    await invoke("select_tab", { index: pick }).catch(() => undefined);
    await sleep(300);
    await syncBrowserStateForCapture();
  } else if (pick >= 0) {
    await syncBrowserStateForCapture();
  }
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
  await syncBrowserStateForCapture();
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
  const preferDocumentViewer = launchPathsNeedDocumentViewer(launchPaths);
  while (Date.now() < deadline) {
    await ensureDocumentTabActiveForCapture(launchPaths);
    if (!preferDocumentViewer) {
      await ensureTextTabActiveForCapture(launchPaths);
    }
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
