import * as monaco from "monaco-editor";
import loader from "@monaco-editor/loader";
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

async function getMonaco(): Promise<Monaco> {
  if (!monacoApi) {
    monacoApi = await loader.init();
  }
  return monacoApi as Monaco;
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
  },

  async apply(doc: TextDocumentDto | null, host: HTMLElement) {
    if (!doc) {
      this.hide(host);
      return;
    }
    this.show(host);
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
  },
};
