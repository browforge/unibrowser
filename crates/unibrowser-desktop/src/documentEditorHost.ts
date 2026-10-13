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

const RICH_HOST_ID = "rich-text-editor-host";
const SPLIT_PREVIEW_ID = "document-split-preview-host";

let session: DocumentParseResponse | null = null;
let monacoHost: HTMLElement | null = null;
let richHost: HTMLElement | null = null;
let splitPreviewHost: HTMLElement | null = null;
let documentPath: string | null = null;

async function resolveDemoTokens(): Promise<string> {
  const fromUrl = new URLSearchParams(window.location.search).get("demo");
  if (fromUrl?.trim()) {
    return fromUrl.toLowerCase();
  }
  const fromBody = document.body.dataset.ubDemoTokens;
  if (fromBody?.trim()) {
    return fromBody.toLowerCase();
  }
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const state = await invoke<string | null>("get_demo_state");
    return state?.toLowerCase() ?? "";
  } catch {
    return "";
  }
}

function isMarkdownPath(path: string): boolean {
  const lower = path.toLowerCase();
  return lower.endsWith(".md") || lower.endsWith(".markdown");
}

function hideLegacyMarkdownPreview(): void {
  document.querySelectorAll<HTMLElement>(".md-preview-shell").forEach((shell) => {
    shell.classList.add("hidden");
  });
  monacoHost?.classList.remove("md-split");
}

function ensureRichHost(editorHost: HTMLElement): HTMLElement {
  let el = document.getElementById(RICH_HOST_ID) as HTMLElement | null;
  if (!el) {
    el = document.createElement("div");
    el.id = RICH_HOST_ID;
    el.className = "rich-text-editor-shell hidden";
    editorHost.appendChild(el);
  } else if (el.parentElement !== editorHost) {
    editorHost.appendChild(el);
  }
  richHost = el;
  return el;
}

function ensureSplitPreview(editorHost: HTMLElement): HTMLElement {
  let el = document.getElementById(SPLIT_PREVIEW_ID) as HTMLElement | null;
  if (!el) {
    el = document.createElement("div");
    el.id = SPLIT_PREVIEW_ID;
    el.className = "document-split-preview hidden";
    editorHost.appendChild(el);
  } else if (el.parentElement !== editorHost) {
    editorHost.appendChild(el);
  }
  splitPreviewHost = el;
  return el;
}

/** Keep WYSIWYG surfaces inside the editor host when the document card moves the host. */
export function relocateDocumentEditorSurfaces(editorHost: HTMLElement): void {
  if (richHost && richHost.parentElement !== editorHost) {
    editorHost.appendChild(richHost);
  }
  if (splitPreviewHost && splitPreviewHost.parentElement !== editorHost) {
    editorHost.appendChild(splitPreviewHost);
  }
}

export function ensureDocumentModeFooterMounted(): void {
  mountModeControls();
}

function mountModeControls(): void {
  const chromeStrip = document.getElementById("document-mode-strip-host");
  chromeStrip?.classList.add("hidden");
  const cardFooter = document.getElementById("document-card-status");
  let inlineHost = document.getElementById("document-mode-inline-host");
  if (cardFooter && !inlineHost) {
    inlineHost = document.createElement("div");
    inlineHost.id = "document-mode-inline-host";
    cardFooter.appendChild(inlineHost);
  }
  const mountTarget = inlineHost ?? cardFooter ?? chromeStrip;
  if (!mountTarget) {
    return;
  }
  if (!mountTarget.dataset.docModeMounted) {
    mountDocumentModeStrip(mountTarget, inlineHost ? "inline" : "chrome");
    mountTarget.dataset.docModeMounted = "1";
    onDocumentEditorModeChange(() => {
      applySurfaceVisibility(getDocumentEditorMode());
      void renderRich();
    });
  }
}

function applySurfaceVisibility(mode: ReturnType<typeof getDocumentEditorMode>) {
  const reading = mode === "reading";
  const wysiwyg = mode === "wysiwyg" || reading;
  const source = mode === "source" || mode === "split";
  const split = mode === "split";
  monacoHost?.classList.toggle("document-surface-source-hidden", !source);
  monacoHost?.classList.toggle("document-editor-split-layout", split);
  richHost?.classList.toggle("hidden", !wysiwyg || split);
  splitPreviewHost?.classList.toggle("hidden", !split);
  document.body.classList.toggle("document-editor-reading", reading);
  hideLegacyMarkdownPreview();
  void import("./textEditor").then(({ relayoutTextEditor }) => {
    window.requestAnimationFrame(() => relayoutTextEditor());
  });
}

async function renderRich() {
  if (!session || !richHost || !documentPath) {
    return;
  }
  const readonly = getDocumentEditorMode() === "reading";
  await mountRichTextEditor(richHost, session.model, readonly, documentPath);
  if (splitPreviewHost && getDocumentEditorMode() === "split") {
    await mountRichTextEditor(splitPreviewHost, session.model, true, documentPath);
  }
}

export async function syncDocumentEditorForTextTab(
  monacoEditorHost: HTMLElement,
  path: string,
  content: string,
): Promise<void> {
  monacoHost = monacoEditorHost;
  documentPath = path;
  relocateDocumentEditorSurfaces(monacoEditorHost);
  if (!isMarkdownPath(path)) {
    document.getElementById("document-mode-inline-host")?.remove();
    document.getElementById("document-mode-strip-host")?.classList.add("hidden");
    richHost?.classList.add("hidden");
    splitPreviewHost?.classList.add("hidden");
    monacoHost.classList.remove("document-surface-source-hidden", "document-editor-split-layout");
    session = null;
    documentPath = null;
    return;
  }
  try {
    session = await parseMarkdownDocument(content);
  } catch {
    session = null;
    document.getElementById("document-mode-strip-host")?.classList.add("hidden");
    monacoHost.classList.remove("document-surface-source-hidden");
    return;
  }
  mountModeControls();
  const { refreshDocumentCardStatusFooter } = await import("./ui/documentCardStatus");
  refreshDocumentCardStatusFooter(Boolean(document.body.dataset.documentCardDirty === "1"));
  const demo = await resolveDemoTokens();
  await bindDocumentPath(path);
  if (demo.includes("doc-mode-wysiwyg")) {
    const { setDocumentEditorMode } = await import("./documentEditorModes");
    setDocumentEditorMode("wysiwyg", false);
  } else if (demo.includes("doc-mode-source")) {
    const { setDocumentEditorMode } = await import("./documentEditorModes");
    setDocumentEditorMode("source", false);
  } else if (demo.includes("doc-mode-split")) {
    const { setDocumentEditorMode } = await import("./documentEditorModes");
    setDocumentEditorMode("split", false);
  } else if (demo.includes("doc-mode-reading")) {
    const { setDocumentEditorMode } = await import("./documentEditorModes");
    setDocumentEditorMode("reading", false);
  }
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
  ensureRichHost(monacoEditorHost);
  ensureSplitPreview(monacoEditorHost);
  hideLegacyMarkdownPreview();
  applySurfaceVisibility(getDocumentEditorMode());
  await renderRich();
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
