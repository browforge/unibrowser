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
  },
): Promise<void> {
  const demo = demoRaw.toLowerCase();
  const { dismissDocCaptureChrome } = await import("../docCaptureDismiss");
  dismissDocCaptureChrome();
  const { setRailMode } = await import("./activityRail");
  setRailMode("collapsed");

  const docxShot = demo.includes("doc-visual-md-docx");
  if (docxShot) {
    const viewerReady = () => {
      const viewer = document.getElementById("document-viewer-host");
      return Boolean(
        viewer &&
          !viewer.classList.contains("hidden") &&
          viewer.querySelector(".docx-reader, .viewer-body .docx-view"),
      );
    };
    if (!viewerReady()) {
      await deps.applyDemoToken("doc-visual-md-docx");
    }
    for (let attempt = 0; attempt < 180; attempt += 1) {
      if (viewerReady()) {
        syncTextDocumentCard(false);
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
    setDocumentEditorMode("source", false);
    document.getElementById("text-editor-host")?.classList.remove("hidden");
    const { showHistoryPanelTab, refreshHistoryPanel } = await import("../historyPanel");
    showHistoryPanelTab();
    const historyPath =
      document.body.dataset.activeDocumentPath ||
      activePath ||
      `examples/demo-pack/${cardLabel ?? "sample.md"}`;
    await refreshHistoryPanel(historyPath);
    forceDocFeatureCaptureLayout(label ?? "sample.md");
  }

  await new Promise((resolve) => window.setTimeout(resolve, 180));
}

/** Last-resort layout for doc-feature screenshots when demo state did not mount the card. */
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
