import { mergeSourceDocument, sourceProjectionDocument } from "./documentModelBridge";
import { getDocumentModelSession } from "./documentEditorHost";
import { getDocumentEditorMode } from "./documentEditorModes";

let bannerEl: HTMLElement | null = null;
let pendingSource: string | null = null;

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
  const merged = await mergeSourceDocument(session, pendingSource);
  if (!merged.clean) {
    return;
  }
  window.dispatchEvent(
    new CustomEvent("unibrowser-document-session-updated", {
      detail: merged,
    }),
  );
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
  const projection = await sourceProjectionDocument(session);
  if (monacoContent === projection) {
    hideSourceMergeBanner();
    return;
  }
  const merged = await mergeSourceDocument(session, monacoContent);
  if (merged.clean) {
    window.dispatchEvent(
      new CustomEvent("unibrowser-document-session-updated", { detail: merged }),
    );
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
