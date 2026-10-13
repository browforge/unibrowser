/** Bear concept Render 1: light paper document card with in-card formatting toolbar. */

import { refreshDocumentCardStatusFooter } from "./documentCardStatus";

const CARD_SHELL_ID = "document-card-shell";
const CARD_TOOLBAR_MOUNT_ID = "document-card-toolbar-mount";
const CARD_BODY_MOUNT_ID = "document-card-body-mount";
const CARD_TAB_ID = "document-card-tab";
const COMPACT_TOOLBAR_ID = "text-compact-toolbar-host";

function activeDocumentPane(): HTMLElement | null {
  const editor = document.getElementById("text-editor-host");
  const richVisible = Boolean(
    document.querySelector<HTMLElement>(
      "#rich-text-editor-host:not(.hidden) .rich-text-doc, #document-split-preview-host:not(.hidden) .rich-text-doc",
    ),
  );
  if (editor && (!editor.classList.contains("hidden") || richVisible)) {
    if (richVisible && editor.classList.contains("hidden")) {
      editor.classList.remove("hidden");
    }
    return editor;
  }
  const viewer = document.getElementById("document-viewer-host");
  if (viewer && !viewer.classList.contains("hidden")) {
    return viewer;
  }
  return null;
}

function restorePaneToSplit(pane: HTMLElement) {
  const split = document.getElementById("split-primary");
  if (split && pane.parentElement?.id === CARD_BODY_MOUNT_ID) {
    split.appendChild(pane);
  }
}

function restoreToolbarToChrome(toolbar: HTMLElement) {
  const chrome = document.getElementById("chrome");
  const modHost = document.getElementById("text-mod-toolbar-host");
  if (chrome && toolbar.parentElement?.id === CARD_TOOLBAR_MOUNT_ID) {
    if (modHost?.nextSibling) {
      chrome.insertBefore(toolbar, modHost);
    } else {
      chrome.appendChild(toolbar);
    }
  }
}

export function syncTextDocumentCard(active: boolean, fileLabel?: string | null): void {
  const shell = document.getElementById(CARD_SHELL_ID);
  const toolbarMount = document.getElementById(CARD_TOOLBAR_MOUNT_ID);
  const bodyMount = document.getElementById(CARD_BODY_MOUNT_ID);
  const toolbar = document.getElementById(COMPACT_TOOLBAR_ID);
  const tab = document.getElementById(CARD_TAB_ID);
  const pane = activeDocumentPane();

  const useCard =
    active &&
    !document.body.classList.contains("text-document-card-disabled") &&
    Boolean(pane && toolbar && shell && toolbarMount && bodyMount);

  document.body.classList.toggle("text-document-card", useCard);
  shell?.classList.toggle("hidden", !useCard);

  if (!useCard) {
    if (pane) {
      restorePaneToSplit(pane);
    }
    if (toolbar) {
      restoreToolbarToChrome(toolbar);
    }
    return;
  }

  if (pane && bodyMount && pane.parentElement !== bodyMount) {
    bodyMount.appendChild(pane);
    void import("../documentEditorHost").then(({ relocateDocumentEditorSurfaces }) => {
      relocateDocumentEditorSurfaces(pane);
    });
  }
  if (toolbar && toolbarMount && toolbar.parentElement !== toolbarMount) {
    toolbarMount.appendChild(toolbar);
    window.requestAnimationFrame(() => {
      window.dispatchEvent(new Event("resize"));
      window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
    });
  }

  const name = fileLabel?.trim() || "Untitled";
  if (tab) {
    tab.innerHTML = "";
    const label = document.createElement("span");
    label.className = "document-card-tab-label";
    label.textContent = name;
    const close = document.createElement("button");
    close.type = "button";
    close.className = "document-card-tab-close";
    close.setAttribute("aria-label", `Close ${name}`);
    close.textContent = "×";
    tab.append(label, close);
  }

  refreshDocumentCardStatusFooter(Boolean(document.body.dataset.documentCardDirty === "1"));
}

export function syncDocumentCardDirtyFlag(dirty: boolean): void {
  document.body.dataset.documentCardDirty = dirty ? "1" : "0";
  refreshDocumentCardStatusFooter(dirty);
}

export function exposeTextDocumentCard() {
  (window as unknown as { __unibrowserSyncTextDocumentCard?: typeof syncTextDocumentCard }).__unibrowserSyncTextDocumentCard =
    syncTextDocumentCard;
}

function compactToolbarReady(): boolean {
  const toolbar = document.getElementById(COMPACT_TOOLBAR_ID);
  if (!toolbar || toolbar.classList.contains("hidden")) {
    return false;
  }
  const ctx = document.body.dataset.cmAutoContext ?? "";
  return (
    (ctx === "text" || ctx === "office") &&
    toolbar.querySelector(".text-compact-tools") !== null
  );
}

function docFeatureCardLabel(demoRaw: string): string | null {
  const demo = demoRaw.toLowerCase();
  if (demo.includes("doc-opaque-card")) {
    return "sample-opaque.md";
  }
  if (
    demo.includes("doc-history-panel") ||
    demo.includes("doc-visual-md-docx") ||
    demo.includes("doc-mode-") ||
    demo.includes("doc-merge-banner")
  ) {
    return "sample.md";
  }
  return null;
}

/** Mount paper card + compact toolbar after CM context is Text (doc-feature captures). */
export async function ensureDocFeatureCaptureLayout(
  demoRaw: string,
  deps: {
    refreshState: () => Promise<void>;
    refreshCommandManager: () => Promise<void>;
    applyDemoToken: (token: string) => Promise<void>;
    skipDemoApply?: boolean;
  },
): Promise<void> {
  const demo = demoRaw.toLowerCase();
  const { dismissDocCaptureChrome } = await import("../docCaptureDismiss");
  dismissDocCaptureChrome();
  const { setRailMode } = await import("./activityRail");
  setRailMode("collapsed");

  const docxShot = demo.includes("doc-visual-md-docx");
  if (docxShot) {
    const cardReady = () => {
      const tabLabel =
        document.querySelector<HTMLElement>(".document-card-tab-label")?.textContent?.trim() ?? "";
      const footer =
        document.querySelector<HTMLElement>(".document-card-status-main")?.textContent ?? "";
      return (
        tabLabel === "sample.docx" &&
        footer.includes("Word") &&
        document.body.classList.contains("text-document-card") &&
        !document.getElementById(CARD_SHELL_ID)?.classList.contains("hidden") &&
        Boolean(
          document.querySelector(
            "#rich-text-editor-host:not(.hidden) .rich-text-doc, #document-split-preview-host:not(.hidden) .rich-text-doc",
          ),
        ) &&
        !document.querySelector<HTMLElement>("#document-viewer-host:not(.hidden)")
      );
    };
    if (!cardReady() && !deps.skipDemoApply) {
      await deps.applyDemoToken("doc-visual-md-docx");
      await deps.refreshState();
    }
    for (let attempt = 0; attempt < 30; attempt += 1) {
      if (!deps.skipDemoApply && attempt % 4 === 0) {
        await deps.refreshCommandManager();
      }
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("select_command_manager_tab", { tabId: "office" }).catch(() => undefined);
      await invoke("select_command_manager_tab", { tabId: "text" }).catch(() => undefined);
      window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
      const toolbarOk = deps.skipDemoApply || compactToolbarReady();
      if (cardReady() && toolbarOk) {
        forceDocFeatureCaptureLayout("sample.docx");
        dismissDocCaptureChrome();
        return;
      }
      await new Promise((resolve) => window.setTimeout(resolve, 100));
    }
    dismissDocCaptureChrome();
    return;
  }

  const { invoke } = await import("@tauri-apps/api/core");
  for (let attempt = 0; attempt < 60; attempt += 1) {
    await deps.refreshState();
    await deps.refreshCommandManager();
    if (compactToolbarReady()) {
      break;
    }
    await invoke("select_command_manager_tab", { tabId: "text" }).catch(() => undefined);
    await new Promise((resolve) => window.setTimeout(resolve, 70));
  }

  const cardLabel = docFeatureCardLabel(demoRaw);
  const activePath = document.body.dataset.activeDocumentPath ?? "";
  const label = cardLabel ?? activePath.split(/[/\\]/).pop() ?? null;
  forceDocFeatureCaptureLayout(label);

  if (demo.includes("doc-history-panel")) {
    const { setDocumentEditorMode } = await import("../documentEditorModes");
    setDocumentEditorMode("wysiwyg", false);
    const { syncDocumentEditorSurfaceVisibility } = await import("../documentEditorHost");
    syncDocumentEditorSurfaceVisibility();
    document.getElementById("text-editor-host")?.classList.remove("hidden");
    const { showHistoryPanelTab, refreshHistoryPanel } = await import("../historyPanel");
    showHistoryPanelTab();
    const historyPath =
      document.body.dataset.activeDocumentPath ||
      activePath ||
      `examples/demo-pack/${cardLabel ?? "sample.md"}`;
    await refreshHistoryPanel(historyPath);
    forceDocFeatureCaptureLayout(label ?? "sample.md");
    syncDocumentEditorSurfaceVisibility();
  }

  await new Promise((resolve) => window.setTimeout(resolve, 180));
}

/** Desktop headless doc-capture: markdown → visual DOCX card without Save dialog or full refreshState. */
export async function prepareDocVisualMdDocxDesktopCapture(): Promise<void> {
  if (document.body.dataset.ubDocVisualMdDocxApplied === "1") {
    return;
  }
  const savedPath = "examples/demo-pack/sample.docx";
  const editorHost = document.getElementById("text-editor-host");
  if (!editorHost) {
    throw new Error("doc-visual-md-docx capture requires text-editor-host");
  }
  const { setDocumentEditorMode } = await import("../documentEditorModes");
  setDocumentEditorMode("wysiwyg", false);
  const {
    refreshVisualDocxRichSurface,
    syncDocumentEditorSurfaceVisibility,
  } = await import("../documentEditorHost");
  const { documentViewer } = await import("../documentViewer");
  const markdown = await (
    await import("../demoPackSample")
  ).readDemoPackSampleMarkdownDesktopCapture();
  const { parseMarkdownDocument } = await import("../documentModelBridge");
  const { seedDocumentModelSessionForDemo } = await import("../documentEditorHost");
  seedDocumentModelSessionForDemo(await parseMarkdownDocument(markdown));
  document.body.dataset.documentImageBasePath = "examples/demo-pack/sample.md";
  document.body.dataset.activeDocumentPath = savedPath;
  document.body.dataset.cmAutoContext = "office";
  const viewerHost = document.getElementById("document-viewer-host");
  if (viewerHost) {
    documentViewer.hide(viewerHost);
  }
  editorHost.classList.remove("hidden");
  syncDocumentEditorSurfaceVisibility();
  await refreshVisualDocxRichSurface(savedPath, "examples/demo-pack/sample.md");
  const { dismissDocCaptureChrome } = await import("../docCaptureDismiss");
  dismissDocCaptureChrome();
  syncTextDocumentCard(true, "sample.docx");
  forceDocFeatureCaptureLayout("sample.docx");
  refreshDocumentCardStatusFooter(false);
  const compactHost = document.getElementById(COMPACT_TOOLBAR_ID);
  if (compactHost) {
    const { seedDocVisualCompactToolbarFallback } = await import("../textCompactToolbar");
    seedDocVisualCompactToolbarFallback(compactHost, true);
  }
  await (await import("../richTextEditor")).waitForRichCardImagesLoaded(4_000);
  const refreshStatusBarOnly = (
    window as unknown as { __unibrowserRefreshStatusBarOnly?: () => Promise<void> }
  ).__unibrowserRefreshStatusBarOnly;
  if (refreshStatusBarOnly) {
    await refreshStatusBarOnly();
  }
  document.body.dataset.ubDocVisualMdDocxApplied = "1";
}

/** After Save As DOCX, keep the Render 1 paper card in Visual mode (not the browse DOCX viewer). */
export async function finalizeVisualDocxAfterSave(savedPath: string): Promise<void> {
  const { dismissDocCaptureChrome } = await import("../docCaptureDismiss");
  const { setDocumentEditorMode, bindDocumentPath } = await import("../documentEditorModes");
  const { syncDocumentEditorSurfaceVisibility } = await import("../documentEditorHost");
  const { documentViewer } = await import("../documentViewer");
  const label = savedPath.split(/[/\\]/).pop() ?? "sample.docx";
  document.body.dataset.activeDocumentPath = savedPath;
  document.body.dataset.cmAutoContext = "office";
  const viewerHost = document.getElementById("document-viewer-host");
  if (viewerHost) {
    documentViewer.hide(viewerHost);
  }
  const editorHost = document.getElementById("text-editor-host");
  editorHost?.classList.remove("hidden");
  await bindDocumentPath(savedPath);
  setDocumentEditorMode("wysiwyg", false);
  syncDocumentEditorSurfaceVisibility();
  const imageBase =
    document.body.dataset.documentImageBasePath?.trim() ||
    savedPath.replace(/[/\\][^/\\]+$/u, "/sample.md");
  const { refreshVisualDocxRichSurface } = await import("../documentEditorHost");
  await refreshVisualDocxRichSurface(savedPath, imageBase);
  forceDocFeatureCaptureLayout(label);
  dismissDocCaptureChrome();
  const { invoke } = await import("@tauri-apps/api/core");
  for (let attempt = 0; attempt < 35; attempt += 1) {
    await invoke("select_command_manager_tab", { tabId: "office" }).catch(() => undefined);
    window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
    if (compactToolbarReady()) {
      break;
    }
    await invoke("select_command_manager_tab", { tabId: "text" }).catch(() => undefined);
    window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
    if (compactToolbarReady()) {
      break;
    }
    await new Promise((resolve) => window.setTimeout(resolve, 80));
  }
  await new Promise((resolve) => window.setTimeout(resolve, 120));
  const compactHost = document.getElementById(COMPACT_TOOLBAR_ID);
  if (compactHost) {
    const { seedDocVisualCompactToolbarFallback } = await import("../textCompactToolbar");
    seedDocVisualCompactToolbarFallback(compactHost, true);
  }
  forceDocFeatureCaptureLayout(label);
  const { refreshDocumentCardStatusFooter } = await import("./documentCardStatus");
  refreshDocumentCardStatusFooter(false);
  const { waitForRichCardImagesLoaded } = await import("../richTextEditor");
  await waitForRichCardImagesLoaded();
  const refreshStatusBarOnly = (
    window as unknown as { __unibrowserRefreshStatusBarOnly?: () => Promise<void> }
  ).__unibrowserRefreshStatusBarOnly;
  if (refreshStatusBarOnly) {
    await refreshStatusBarOnly();
  }
}

export function forceDocFeatureCaptureLayout(fileLabel: string | null): void {
  if (!fileLabel) {
    return;
  }
  document.body.classList.add("cm-collapsed");
  if (compactToolbarReady()) {
    document.body.classList.add("text-compact-visible");
  }
  const shell = document.getElementById(CARD_SHELL_ID);
  const bodyMount = document.getElementById(CARD_BODY_MOUNT_ID);
  shell?.classList.remove("hidden");
  const pane = activeDocumentPane();
  if (pane && bodyMount && pane.parentElement !== bodyMount) {
    bodyMount.appendChild(pane);
  }
  syncTextDocumentCard(true, fileLabel);
  shell?.classList.remove("hidden");
  if (document.body.classList.contains("text-document-card")) {
    document.body.classList.add("text-compact-visible");
  }
}
