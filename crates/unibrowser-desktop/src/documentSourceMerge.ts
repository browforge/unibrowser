import { invoke } from "@tauri-apps/api/core";
import {
  appendDocumentHistorySnapshot,
  mergeSourceDocument,
  sourceProjectionDocument,
  type DocumentParseResponse,
} from "./documentModelBridge";
import { getDocumentModelSession } from "./documentEditorHost";
import { getDocumentEditorMode } from "./documentEditorModes";

let bannerEl: HTMLElement | null = null;
let pendingSource: string | null = null;

type SessionSnapshot = Pick<DocumentParseResponse, "model" | "source_map" | "original_base64">;

async function appendVisualCheckpointBeforeSourceMerge(
  path: string,
  session: SessionSnapshot,
): Promise<void> {
  try {
    const head = await invoke<DocumentParseResponse>("document_history_head_session", { path });
    const headProj = await sourceProjectionDocument(head);
    const sessProj = await sourceProjectionDocument(session);
    if (headProj.trim() === sessProj.trim()) {
      return;
    }
  } catch {
    // No history sidecar yet — record the visual checkpoint below.
  }
  await appendDocumentHistorySnapshot(path, {
    model: session.model,
    source_map: session.source_map,
    original_base64: session.original_base64,
  });
}

function ensureBanner(parent: HTMLElement): HTMLElement {
  if (!bannerEl) {
    bannerEl = document.createElement("div");
    bannerEl.className = "document-source-merge-banner hidden";
    bannerEl.setAttribute("role", "status");
    bannerEl.innerHTML = `
      <span class="document-source-merge-banner-text"></span>
      <div class="document-source-merge-banner-actions">
        <button type="button" class="document-source-merge-apply">Apply</button>
        <button type="button" class="document-source-merge-discard">Discard</button>
      </div>`;
    parent.insertBefore(bannerEl, parent.firstChild);
    bannerEl.querySelector(".document-source-merge-apply")?.addEventListener("click", () => {
      void applyPendingMerge();
    });
    bannerEl.querySelector(".document-source-merge-discard")?.addEventListener("click", () => {
      void discardPendingMerge();
    });
  }
  return bannerEl;
}

export function hideSourceMergeBanner() {
  pendingSource = null;
  bannerEl?.classList.add("hidden");
}

async function applyPendingMerge() {
  const session = getDocumentModelSession();
  if (!session || !pendingSource) {
    hideSourceMergeBanner();
    return;
  }
  const path = document.body.dataset.activeDocumentPath;
  if (path) {
    await appendVisualCheckpointBeforeSourceMerge(path, session);
  }
  const merged = await mergeSourceDocument(session, pendingSource);
  if (!merged.clean) {
    return;
  }
  window.dispatchEvent(
    new CustomEvent("unibrowser-document-session-updated", {
      detail: merged,
    }),
  );
  if (path) {
    await appendDocumentHistorySnapshot(path, {
      model: merged.model,
      source_map: merged.source_map,
      original_base64: merged.original_base64,
    }).catch(() => undefined);
  }
  hideSourceMergeBanner();
  const { textEditor } = await import("./textEditor");
  await textEditor.setContentFromHost(merged.projectionContent ?? pendingSource);
}

async function discardPendingMerge() {
  const session = getDocumentModelSession();
  if (!session) {
    hideSourceMergeBanner();
    return;
  }
  const projection = await sourceProjectionDocument(session);
  const { textEditor } = await import("./textEditor");
  await textEditor.setContentFromHost(projection);
  hideSourceMergeBanner();
}

export async function checkSourceBuffer(
  hostParent: HTMLElement,
  monacoContent: string,
): Promise<void> {
  const mode = getDocumentEditorMode();
  if (mode !== "source" && mode !== "split") {
    hideSourceMergeBanner();
    return;
  }
  const session = getDocumentModelSession();
  if (!session) {
    return;
  }
  const { getActiveTextContent } = await import("./textEditor");
  let projection = getActiveTextContent() ?? "";
  try {
    projection = await sourceProjectionDocument(session);
  } catch {
    projection = getActiveTextContent() ?? "";
  }
  if (monacoContent === projection) {
    hideSourceMergeBanner();
    return;
  }
  const path = document.body.dataset.activeDocumentPath;
  if (path) {
    await appendVisualCheckpointBeforeSourceMerge(path, session);
  }
  const merged = await mergeSourceDocument(session, monacoContent);
  if (merged.clean) {
    window.dispatchEvent(
      new CustomEvent("unibrowser-document-session-updated", { detail: merged }),
    );
    if (path) {
      await appendDocumentHistorySnapshot(path, {
        model: merged.model,
        source_map: merged.source_map,
        original_base64: merged.original_base64,
      });
    }
    const { textEditor } = await import("./textEditor");
    await textEditor.setContentFromHost(merged.projectionContent ?? monacoContent);
    hideSourceMergeBanner();
    return;
  }
  pendingSource = monacoContent;
  const banner = ensureBanner(hostParent);
  const text = banner.querySelector(".document-source-merge-banner-text");
  if (text) {
    text.textContent =
      merged.conflict_reason ??
      "Source edit could not be mapped cleanly to the document model.";
  }
  banner.classList.remove("hidden");
}

export function seedDemoMergeBanner(hostParent: HTMLElement) {
  const banner = ensureBanner(hostParent);
  const text = banner.querySelector(".document-source-merge-banner-text");
  if (text) {
    text.textContent =
      "Source edit could not be mapped cleanly to the document model.";
  }
  banner.classList.remove("hidden");
  pendingSource = "demo";
}
