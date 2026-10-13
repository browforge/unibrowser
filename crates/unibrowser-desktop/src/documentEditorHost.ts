import {
  parseMarkdownDocument,
  type DocumentParseResponse,
  type DocumentModelDto,
  type SourceMapDto,
} from "./documentModelBridge";
import {
  bindDocumentPath,
  getDocumentEditorMode,
  mountDocumentModeStrip,
  onDocumentEditorModeChange,
} from "./documentEditorModes";
import { mountRichTextEditor } from "./richTextEditor";

let session: DocumentParseResponse | null = null;
let richHost: HTMLElement | null = null;
let monacoHost: HTMLElement | null = null;
let splitPreviewHost: HTMLElement | null = null;

function isMarkdownPath(path: string): boolean {
  const lower = path.toLowerCase();
  return lower.endsWith(".md") || lower.endsWith(".markdown");
}

function ensureRichHost(parent: HTMLElement): HTMLElement {
  if (!richHost) {
    richHost = document.createElement("div");
    richHost.id = "rich-text-editor-host";
    richHost.className = "rich-text-editor-shell hidden";
    parent.insertBefore(richHost, parent.firstChild);
  }
  return richHost;
}

function ensureSplitPreview(parent: HTMLElement): HTMLElement {
  if (!splitPreviewHost) {
    splitPreviewHost = document.createElement("div");
    splitPreviewHost.id = "document-split-preview-host";
    splitPreviewHost.className = "document-split-preview hidden";
    parent.appendChild(splitPreviewHost);
  }
  return splitPreviewHost;
}

function applySurfaceVisibility(mode: ReturnType<typeof getDocumentEditorMode>) {
  const reading = mode === "reading";
  const wysiwyg = mode === "wysiwyg" || reading;
  const source = mode === "source" || mode === "split";
  const split = mode === "split";
  monacoHost?.classList.remove("hidden");
  monacoHost?.classList.toggle("document-surface-source-hidden", !source);
  monacoHost?.classList.toggle("document-split-source", split);
  richHost?.classList.toggle("hidden", !wysiwyg);
  splitPreviewHost?.classList.toggle("hidden", !split);
  monacoHost?.parentElement?.classList.toggle("document-editor-split-layout", split);
  document.body.classList.toggle("document-editor-reading", reading);
}

function renderRich() {
  if (!session || !richHost) {
    return;
  }
  const readonly = getDocumentEditorMode() === "reading";
  mountRichTextEditor(richHost, session.model, readonly);
  if (splitPreviewHost && getDocumentEditorMode() === "split") {
    mountRichTextEditor(splitPreviewHost, session.model, true);
  }
}

export async function syncDocumentEditorForTextTab(
  monacoEditorHost: HTMLElement,
  path: string,
  content: string,
): Promise<void> {
  monacoHost = monacoEditorHost;
  const strip = document.getElementById("document-mode-strip-host");
  if (strip && !strip.dataset.mounted) {
    mountDocumentModeStrip(strip);
    strip.dataset.mounted = "1";
    onDocumentEditorModeChange(() => {
      applySurfaceVisibility(getDocumentEditorMode());
      renderRich();
    });
  }
  if (!isMarkdownPath(path)) {
    strip?.classList.add("hidden");
    richHost?.classList.add("hidden");
    splitPreviewHost?.classList.add("hidden");
    monacoHost.classList.remove("hidden");
    session = null;
    return;
  }
  strip?.classList.remove("hidden");
  try {
    session = await parseMarkdownDocument(content);
  } catch {
    session = null;
    strip?.classList.add("hidden");
    monacoHost.classList.remove("hidden");
    return;
  }
  await bindDocumentPath(path);
  const demo = new URLSearchParams(window.location.search).get("demo")?.toLowerCase() ?? "";
  const wantsVisual =
    demo.includes("doc-mode-wysiwyg") ||
    demo.includes("doc-mode-reading") ||
    demo.includes("doc-mode-split");
  const toolbarCapture =
    demo.includes("txt-apply-h2") || demo.includes("txt-block-picker-open");
  if (toolbarCapture && !wantsVisual) {
    const { setDocumentEditorMode } = await import("./documentEditorModes");
    setDocumentEditorMode("source", false);
  }
  const parent = monacoEditorHost.parentElement ?? monacoEditorHost;
  ensureRichHost(parent);
  ensureSplitPreview(parent);
  applySurfaceVisibility(getDocumentEditorMode());
  renderRich();
}

export function getDocumentModelSession(): {
  model: DocumentModelDto;
  source_map: SourceMapDto;
  original_base64: string;
} | null {
  if (!session) {
    return null;
  }
  return session;
}
