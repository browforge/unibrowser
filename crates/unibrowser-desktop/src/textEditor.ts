import { marked } from "marked";
import { invoke } from "@tauri-apps/api/core";

type Monaco = typeof import("monaco-editor");
type MonacoEditor = import("monaco-editor").editor.IStandaloneCodeEditor;
type MonacoModel = import("monaco-editor").editor.ITextModel;

let monacoApi: Monaco | null = null;
let editor: MonacoEditor | null = null;
let hostEl: HTMLElement | null = null;
let loadedPath: string | null = null;
let syncTimer: ReturnType<typeof setTimeout> | null = null;
let suppressSync = false;
let markdownPreview = false;
let previewEl: HTMLElement | null = null;
let toolbarEl: HTMLElement | null = null;
let findBound = false;

async function getMonaco(): Promise<Monaco> {
  if (!monacoApi) {
    const loader = (await import("@monaco-editor/loader")).default;
    const monaco = await import("monaco-editor");
    loader.config({ monaco });
    monacoApi = await loader.init();
  }
  return monacoApi as Monaco;
}

window.addEventListener("unibrowser-theme", () => {
  if (!editor || !monacoApi) {
    return;
  }
  const theme =
    document.documentElement.dataset.theme === "light" ? "vs" : "vs-dark";
  monacoApi.editor.setTheme(theme);
});

export interface TextDocumentDto {
  path: string;
  content: string;
  font_size_pt: number;
  language: string;
  dirty: boolean;
}

function isMarkdownPath(path: string): boolean {
  const lower = path.toLowerCase();
  return lower.endsWith(".md") || lower.endsWith(".markdown");
}

function selectionOffsets(model: MonacoModel) {
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

function bindFindShortcut(api: Monaco) {
  if (!editor || findBound) {
    return;
  }
  editor.addCommand(api.KeyMod.CtrlCmd | api.KeyCode.KeyF, () => {
    editor?.trigger("keyboard", "actions.find", null);
  });
  findBound = true;
}

async function ensureEditor(host: HTMLElement) {
  if (editor && hostEl === host) {
    return;
  }
  findBound = false;
  const api = await getMonaco();
  hostEl = host;
  host.innerHTML = "";
  const monacoTheme =
    document.documentElement.dataset.theme === "light" ? "vs" : "vs-dark";
  editor = api.editor.create(host, {
    theme: monacoTheme,
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 14,
    wordWrap: "on",
  });
  bindFindShortcut(api);
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
    bindFindShortcut(api);
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
