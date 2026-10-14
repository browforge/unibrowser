import { documentViewer } from "../documentViewer";
import { getActiveTextContent } from "../textEditor";
import { textReadingStats } from "../textStats";

const CARD_STATUS_ID = "document-card-status";

let lastAutosaveAtMs = 0;

export function setDocumentCardAutosaveAtMs(ms: number): void {
  lastAutosaveAtMs = ms > 0 ? ms : 0;
}

export function documentTextForWordCount(): string {
  const editorText = getActiveTextContent();
  if (editorText !== null) {
    return editorText;
  }
  const previewShell = document.querySelector<HTMLElement>(".md-preview-shell:not(.hidden)");
  const preview = previewShell?.querySelector<HTMLElement>(".md-preview-body");
  if (preview) {
    return preview.innerText || preview.textContent || "";
  }
  const viewer = document.getElementById("document-viewer-host");
  if (viewer && !viewer.classList.contains("hidden")) {
    return documentViewer.getReadableText(viewer) ?? "";
  }
  return "";
}

export function documentWordCount(): number {
  return textReadingStats(documentTextForWordCount()).words;
}

function formatSaveClock(ms: number): string {
  return new Date(ms).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatDocumentCardSaveStatus(dirty: boolean): string {
  if (dirty) {
    return "Not saved";
  }
  if (lastAutosaveAtMs > 0) {
    return `Saved ${formatSaveClock(lastAutosaveAtMs)}`;
  }
  return "Saved";
}

export function formatDocumentCardStatusLeft(kindLabel: string, dirty: boolean): string {
  const words = documentWordCount();
  const wordLabel = words === 1 ? "1 word" : `${words} words`;
  const save = formatDocumentCardSaveStatus(dirty);
  return `${kindLabel} · ${wordLabel} · ${save}`;
}

export function refreshDocumentCardStatusFooter(dirty: boolean): void {
  if (!document.body.classList.contains("text-document-card")) {
    return;
  }
  const status = document.getElementById(CARD_STATUS_ID);
  if (!status) {
    return;
  }
  const ctx = document.body.dataset.cmAutoContext ?? "text";
  const activePath = (document.body.dataset.activeDocumentPath ?? "").toLowerCase();
  const tabLabel =
    document
      .querySelector<HTMLElement>("#document-card-tab .document-card-tab-label")
      ?.textContent?.trim()
      .toLowerCase() ?? "";
  const officeDoc =
    activePath.endsWith(".docx") ||
    activePath.endsWith(".doc") ||
    activePath.endsWith(".odt") ||
    activePath.includes("/assets/document.md") ||
    tabLabel.endsWith(".docx") ||
    tabLabel.endsWith(".doc") ||
    tabLabel.endsWith(".odt");
  const kind = officeDoc || ctx === "office"
    ? "Word"
    : ctx === "text"
      ? "Markdown"
      : "Plain text";
  const zoom =
    document.querySelector<HTMLElement>(
      '[data-control-id="unibrowser.control.editor.size"] .cm-cmd-label, [data-control-id="unibrowser.control.text.size"] .cm-cmd-label',
    )?.textContent?.trim() ?? "100%";
  const modeHost = document.getElementById("document-mode-inline-host");
  status.replaceChildren();
  const left = document.createElement("span");
  left.className = "document-card-status-main";
  left.textContent = formatDocumentCardStatusLeft(kind, dirty);
  const zoomSpan = document.createElement("span");
  zoomSpan.className = "document-card-status-zoom";
  zoomSpan.textContent = zoom;
  status.append(left, zoomSpan);
  if (modeHost) {
    status.append(modeHost);
  }
}
