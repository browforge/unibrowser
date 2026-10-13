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

export function syncDocumentEditorSurfaceVisibility(): void {
  applySurfaceVisibility(getDocumentEditorMode());
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
  const { appendDocumentHistorySnapshot } = await import("./documentModelBridge");
  await appendDocumentHistorySnapshot(documentPath, {
    model: session.model,
    source_map: session.source_map,
    original_base64: session.original_base64,
  }).catch(() => undefined);
}

async function renderRich() {
  if (!session || !richHost || !documentPath) {
    return;
  }
  const mode = getDocumentEditorMode();
  const readonly = mode === "reading";
  const split = mode === "split";

  if (!split) {
    if (readonly) {
      await mountRichTextEditor(richHost, session.model, true, { basePath: documentPath });
    } else if (mode === "wysiwyg") {
      await mountRichTextEditor(richHost, session.model, false, {
        basePath: documentPath,
        onModelChange: (nextModel, touched) => {
          if (!session) {
            return;
          }
          session = { ...session, model: nextModel };
          void commitModelToMarkdown(touched);
        },
      });
    }
  }

  if (splitPreviewHost && split) {
    await mountRichTextEditor(splitPreviewHost, session.model, true, { basePath: documentPath });
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
    void renderRich();
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
      void renderRich();
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
  relocateDocumentEditorSurfaces(monacoEditorHost);
  document.body.dataset.activeDocumentPath = path;
  installSessionListeners();
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
  if (demo.includes("doc-mode-split") && demo.includes("doc-merge-banner")) {
    const { setDocumentEditorMode } = await import("./documentEditorModes");
    setDocumentEditorMode("split", false);
    seedDemoMergeBanner(monacoEditorHost.parentElement ?? monacoEditorHost);
  }
  ensureRichHost(monacoEditorHost);
  ensureSplitPreview(monacoEditorHost);
  hideLegacyMarkdownPreview();
  applySurfaceVisibility(getDocumentEditorMode());
  await renderRich();
  const { syncTextDocumentCard } = await import("./ui/textDocumentCard");
  syncTextDocumentCard(true, path.split(/[/\\]/).pop() ?? null);
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

/** Demo/capture fallback when the editor surface has not finished mounting yet. */
export function seedDocumentModelSessionForDemo(response: DocumentParseResponse): void {
  session = {
    model: response.model,
    source_map: response.source_map,
    original_base64: response.original_base64,
  };
}

export function teardownDocumentEditor() {
  destroyRichTextEditor();
  hideSourceMergeBanner();
  session = null;
}
