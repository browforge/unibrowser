import * as monaco from "monaco-editor";
import loader from "@monaco-editor/loader";
import { marked } from "marked";
import { invoke } from "@tauri-apps/api/core";

loader.config({ monaco });

export interface TextDocumentDto {
  path: string;
  content: string;
  font_size_pt: number;
  language: string;
  dirty: boolean;
}

type Monaco = typeof monaco;
let monacoApi: Monaco | null = null;
let editor: monaco.editor.IStandaloneCodeEditor | null = null;
let hostEl: HTMLElement | null = null;
let loadedPath: string | null = null;
let syncTimer: ReturnType<typeof setTimeout> | null = null;
let suppressSync = false;
let markdownPreview = false;
let previewEl: HTMLElement | null = null;
let toolbarEl: HTMLElement | null = null;

async function getMonaco(): Promise<Monaco> {
  if (!monacoApi) {
    monacoApi = await loader.init();
  }
  return monacoApi as Monaco;
}

function isMarkdownPath(path: string): boolean {
  const lower = path.toLowerCase();
  return lower.endsWith(".md") || lower.endsWith(".markdown");
}

function selectionOffsets(model: monaco.editor.ITextModel) {
  const sel = editor?.getSelection();
  if (!sel) {
    return { start: 0, end: 0 };
  }
  const start = model.getOffsetAt(sel.getStartPosition());
  const end = model.getOffsetAt(sel.getEndPosition());
  return { start, end };
}

async function pushToHost() {
  if (!editor) {
    return;
  }
  const model = editor.getModel();
  if (!model) {
    return;
  }
  const { start, end } = selectionOffsets(model);
  await invoke("sync_text_document", {
    content: model.getValue(),
    selectionStart: start,
    selectionEnd: end,
  });
  if (markdownPreview && previewEl) {
    previewEl.innerHTML = marked.parse(model.getValue(), { async: false }) as string;
  }
}

function scheduleSync() {
  if (suppressSync) {
    return;
  }
  if (syncTimer) {
    clearTimeout(syncTimer);
  }
  syncTimer = setTimeout(() => {
    pushToHost().catch(() => {});
  }, 250);
}

function ensureMarkdownToolbar(host: HTMLElement, path: string) {
  if (!isMarkdownPath(path)) {
    if (toolbarEl) {
      toolbarEl.remove();
      toolbarEl = null;
    }
    markdownPreview = false;
    if (previewEl) {
      previewEl.remove();
      previewEl = null;
    }
    return;
  }
  if (toolbarEl) {
    return;
  }
  toolbarEl = document.createElement("div");
  toolbarEl.className = "md-editor-toolbar";
  const toggle = document.createElement("button");
  toggle.type = "button";
  toggle.textContent = "Preview";
  toggle.onclick = () => {
    markdownPreview = !markdownPreview;
    toggle.textContent = markdownPreview ? "Edit only" : "Preview";
    if (!previewEl && host.parentElement) {
      previewEl = document.createElement("article");
      previewEl.className = "md-preview";
      host.parentElement.insertBefore(previewEl, host.nextSibling);
    }
    if (previewEl) {
      previewEl.classList.toggle("hidden", !markdownPreview);
    }
    if (markdownPreview && editor && previewEl) {
      previewEl.innerHTML = marked.parse(editor.getValue(), { async: false }) as string;
    }
    host.classList.toggle("md-split", markdownPreview);
  };
  toolbarEl.appendChild(toggle);
  host.parentElement?.insertBefore(toolbarEl, host);
}

async function ensureEditor(host: HTMLElement) {
  if (editor && hostEl === host) {
    return;
  }
  const api = await getMonaco();
  hostEl = host;
  host.innerHTML = "";
  editor = api.editor.create(host, {
    theme: "vs-dark",
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 14,
    wordWrap: "on",
  });
  editor.onDidChangeModelContent(() => scheduleSync());
  editor.onDidChangeCursorSelection(() => scheduleSync());
}

export const textEditor = {
  isActive(): boolean {
    return editor !== null && loadedPath !== null;
  },

  async flush(): Promise<void> {
    if (syncTimer) {
      clearTimeout(syncTimer);
      syncTimer = null;
    }
    await pushToHost();
  },

  show(host: HTMLElement) {
    host.classList.remove("hidden");
  },

  hide(host: HTMLElement) {
    host.classList.add("hidden");
    loadedPath = null;
    markdownPreview = false;
    if (previewEl) {
      previewEl.classList.add("hidden");
    }
    host.classList.remove("md-split");
  },

  async apply(doc: TextDocumentDto | null, host: HTMLElement) {
    if (!doc) {
      this.hide(host);
      return;
    }
    this.show(host);
    ensureMarkdownToolbar(host, doc.path);
    await ensureEditor(host);
    const api = await getMonaco();
    const model = editor!.getModel();
    const pathChanged = loadedPath !== doc.path;
    if (pathChanged) {
      suppressSync = true;
      loadedPath = doc.path;
      const uri = api.Uri.parse(`file://${doc.path}`);
      const language = doc.language || "plaintext";
      if (model && model.uri.toString() === uri.toString()) {
        model.setValue(doc.content);
        api.editor.setModelLanguage(model, language);
      } else {
        const next =
          api.editor.getModel(uri) ??
          api.editor.createModel(doc.content, language, uri);
        if (next.getValue() !== doc.content) {
          next.setValue(doc.content);
        }
        api.editor.setModelLanguage(next, language);
        editor!.setModel(next);
      }
      suppressSync = false;
    }
    editor!.updateOptions({ fontSize: doc.font_size_pt });
    if (markdownPreview && previewEl) {
      previewEl.innerHTML = marked.parse(doc.content, { async: false }) as string;
    }
  },
};
