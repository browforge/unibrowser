/** Bear concept Render 1: light paper document card with in-card formatting toolbar. */

import { refreshDocumentCardStatusFooter } from "./documentCardStatus";

const CARD_SHELL_ID = "document-card-shell";
const CARD_TOOLBAR_MOUNT_ID = "document-card-toolbar-mount";
const CARD_BODY_MOUNT_ID = "document-card-body-mount";
const CARD_TAB_ID = "document-card-tab";
const COMPACT_TOOLBAR_ID = "text-compact-toolbar-host";

function activeDocumentPane(): HTMLElement | null {
  const editor = document.getElementById("text-editor-host");
  if (editor && !editor.classList.contains("hidden")) {
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
