import {
  appendDocumentHistoryOp,
  parseMarkdownDocument,
  serializeMarkdownDocument,
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
import { destroyRichTextEditor, mountRichTextEditor } from "./richTextEditor";
import { hideSourceMergeBanner, seedDemoMergeBanner } from "./documentSourceMerge";

let session: DocumentParseResponse | null = null;
let richHost: HTMLElement | null = null;
let monacoHost: HTMLElement | null = null;
let splitPreviewHost: HTMLElement | null = null;
let documentPath: string | null = null;

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

async function commitModelToMarkdown(touchedBlockIds: string[]) {
  if (!session || !documentPath) {
    return;
  }
  const dirty = Array.from(new Set([...touchedBlockIds]));
  const content = await serializeMarkdownDocument({
    model: session.model,
    source_map: session.source_map,
    original_base64: session.original_base64,
    dirty_block_ids: dirty,
    touched_block_ids: dirty,
  });
  const { textEditor } = await import("./textEditor");
  await textEditor.setContentFromHost(content);
  for (const blockId of dirty) {
    await appendDocumentHistoryOp(documentPath, {
      op_type: "base-text.text.replace",
      block_id: blockId,
      from: 0,
      to: 0,
      text: "",
      marks: [],
    }).catch(() => undefined);
  }
}

function renderRich() {
  if (!session || !richHost) {
    return;
  }
  const readonly = getDocumentEditorMode() === "reading";
  mountRichTextEditor(richHost, session.model, readonly, (nextModel, touched) => {
    if (!session) {
      return;
    }
    session = { ...session, model: nextModel };
    void commitModelToMarkdown(touched);
    if (splitPreviewHost && getDocumentEditorMode() === "split") {
      mountRichTextEditor(splitPreviewHost, session.model, true);
    }
  });
  if (splitPreviewHost && getDocumentEditorMode() === "split") {
    mountRichTextEditor(splitPreviewHost, session.model, true);
  }
}

function installSessionListeners() {
  if (document.body.dataset.documentEditorListeners) {
    return;
  }
  document.body.dataset.documentEditorListeners = "1";
  window.addEventListener("unibrowser-document-session-updated", (ev) => {
    const detail = (ev as CustomEvent).detail as DocumentParseResponse & {
      model: DocumentModelDto;
      source_map: SourceMapDto;
      original_base64: string;
      dirty_block_ids?: string[];
      touched_block_ids?: string[];
    };
    session = {
      model: detail.model,
      source_map: detail.source_map,
      original_base64: detail.original_base64,
    };
    renderRich();
  });
  window.addEventListener("unibrowser-document-parse-opaque", async (ev) => {
    const blockId = (ev as CustomEvent).detail?.blockId as string | undefined;
    if (!blockId || !session || !documentPath) {
      return;
    }
    await appendDocumentHistoryOp(documentPath, {
      op_type: "base-text.opaque.replace",
      block_id: blockId,
      raw: "",
    }).catch(() => undefined);
    const { invoke } = await import("@tauri-apps/api/core");
    const promoted = await invoke<DocumentParseResponse>("document_promote_opaque", {
      model: session.model,
      source_map: session.source_map,
      original_base64: session.original_base64,
      dirty_block_ids: [],
      touched_block_ids: [],
      block_id: blockId,
    }).catch(() => null);
    if (promoted) {
      session = promoted;
      renderRich();
      await commitModelToMarkdown([blockId]);
    }
  });
}

export async function syncDocumentEditorForTextTab(
  monacoEditorHost: HTMLElement,
  path: string,
  content: string,
): Promise<void> {
  monacoHost = monacoEditorHost;
  documentPath = path;
  document.body.dataset.activeDocumentPath = path;
  installSessionListeners();
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
  if (demo.includes("doc-mode-split") && demo.includes("doc-merge-banner")) {
    const { setDocumentEditorMode } = await import("./documentEditorModes");
    setDocumentEditorMode("split", false);
    seedDemoMergeBanner(monacoEditorHost.parentElement ?? monacoEditorHost);
  }
  const parent = monacoEditorHost.parentElement ?? monacoEditorHost;
  ensureRichHost(parent);
  ensureSplitPreview(parent);
  applySurfaceVisibility(getDocumentEditorMode());
  renderRich();
}

export async function onMonacoMarkdownContentChanged(content: string) {
  if (!monacoHost?.parentElement) {
    return;
  }
  const { checkSourceBuffer } = await import("./documentSourceMerge");
  await checkSourceBuffer(monacoHost.parentElement, content);
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

export function teardownDocumentEditor() {
  destroyRichTextEditor();
  hideSourceMergeBanner();
  session = null;
}
