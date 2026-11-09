import { documentViewer } from "../documentViewer";
import { getActiveTextContent } from "../textEditor";
import { textReadingStats } from "../textStats";
import { getDocumentModelSession } from "../documentEditorHost";
import { documentModelWordCount } from "../documentModelWordCount";

const INTEGRATED_STATUS_ID = "integrated-document-status";

let lastAutosaveAtMs = 0;

export function setDocumentCardAutosaveAtMs(ms: number): void {
  lastAutosaveAtMs = ms > 0 ? ms : 0;
}

function visibleRichTextPlainText(): string {
  const prose =
    document.querySelector<HTMLElement>(
      "#rich-text-editor-host:not(.hidden) .rich-text-doc .ProseMirror",
    ) ??
    document.querySelector<HTMLElement>(
      "#document-split-preview-host:not(.hidden) .rich-text-doc .ProseMirror",
    );
  if (prose) {
    return prose.innerText || prose.textContent || "";
  }
  const rich =
    document.querySelector<HTMLElement>("#rich-text-editor-host:not(.hidden) .rich-text-doc") ??
    document.querySelector<HTMLElement>(
      "#document-split-preview-host:not(.hidden) .rich-text-doc",
    );
  if (rich) {
    return rich.innerText || rich.textContent || "";
  }
  return "";
}

export function documentTextForWordCount(): string {
  const richText = visibleRichTextPlainText();
  if (richText.trim().length > 0) {
    return richText;
  }
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
  const session = getDocumentModelSession();
  if (session?.model) {
    const fromModel = documentModelWordCount(session.model);
    if (fromModel !== null) {
      return fromModel;
    }
  }
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

function activeTabFileName(): string {
  return (
    document.querySelector<HTMLElement>(".title-tabs .tab.active .tab-label")?.textContent?.trim() ??
    ""
  );
}

export function documentKindLabel(): string {
  const ctx = document.body.dataset.cmAutoContext ?? "text";
  const activePath = (document.body.dataset.activeDocumentPath ?? "").toLowerCase();
  const tabLabel = activeTabFileName().toLowerCase();
  const officeDoc =
    activePath.endsWith(".docx") ||
    activePath.endsWith(".doc") ||
    activePath.endsWith(".odt") ||
    activePath.includes("/assets/document.md") ||
    tabLabel.endsWith(".docx") ||
    tabLabel.endsWith(".doc") ||
    tabLabel.endsWith(".odt");
  if (officeDoc || ctx === "office") {
    return "Word";
  }
  if (
    activePath.endsWith(".md") ||
    activePath.endsWith(".markdown") ||
    tabLabel.endsWith(".md") ||
    tabLabel.endsWith(".markdown")
  ) {
    return "Markdown";
  }
  if (activePath.endsWith(".txt") || tabLabel.endsWith(".txt")) {
    return "Plain text";
  }
  return ctx === "text" ? "Plain text" : "Plain text";
}

export function formatDocumentCardStatusLeft(kindLabel: string, dirty: boolean): string {
  const words = documentWordCount();
  const wordLabel = words === 1 ? "1 word" : `${words} words`;
  const save = formatDocumentCardSaveStatus(dirty);
  return `${kindLabel} · ${wordLabel} · ${save}`;
}

function ensureIntegratedStatusProbe(): HTMLElement {
  let probe = document.getElementById(INTEGRATED_STATUS_ID);
  if (!probe) {
    probe = document.createElement("span");
    probe.id = INTEGRATED_STATUS_ID;
    probe.className = "document-card-status-main integrated-document-status";
    probe.setAttribute("data-integrated-doc-status", "1");
    probe.setAttribute("aria-hidden", "true");
    document.getElementById("status-bar")?.appendChild(probe);
  }
  return probe;
}

export function refreshDocumentCardStatusFooter(dirty: boolean): void {
  if (!document.body.classList.contains("text-document-card")) {
    document.getElementById(INTEGRATED_STATUS_ID)?.remove();
    return;
  }
  const kind = documentKindLabel();
  const leftText = formatDocumentCardStatusLeft(kind, dirty);
  const probe = ensureIntegratedStatusProbe();
  probe.textContent = leftText;
}
