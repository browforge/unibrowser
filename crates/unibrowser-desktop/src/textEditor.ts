import { marked } from "marked";
import { invoke } from "@tauri-apps/api/core";
import { applyMarkdownPreview } from "./markdownPreview";
import { exportMarkdownToHtmlFile } from "./markdownExport";
import { textReadingStats } from "./textStats";
import { t } from "./i18n";
import { openPrintPreviewDocument, plainTextPrintBody } from "./printExport";
import {
  attachSpellcheckPane,
  detachSpellcheckPane,
  isExperimentalSpellcheckEnabled,
  syncSpellcheckPane,
} from "./experimentalSpellcheck";
import { applyMonacoEditorTheme, resolveEditorThemeId } from "./monacoEditorTheme";

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
let previewShellEl: HTMLElement | null = null;
let previewTocEl: HTMLElement | null = null;
let previewEl: HTMLElement | null = null;
let toolbarEl: HTMLElement | null = null;
let plainToolbarEl: HTMLDivElement | null = null;
let plainToolbarPath: string | null = null;
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

let editorThemePref = "auto";

export function setEditorThemePreference(themeId: string) {
  editorThemePref = themeId;
  void applyMonacoEditorTheme(themeId);
}

window.addEventListener("unibrowser-theme", () => {
  void applyMonacoEditorTheme(editorThemePref);
});

window.addEventListener("unibrowser-editor-theme", () => {
  // other surfaces (e.g. markdown code blocks) may listen
});

export interface TextDocumentDto {
  path: string;
  content: string;
  font_size_pt: number;
  language: string;
  dirty: boolean;
  encoding: string;
  reveal_line?: number | null;
}

export interface CursorStatus {
  line: number;
  column: number;
}

let cursorListener: ((status: CursorStatus) => void) | null = null;
let readingStatsListener: ((label: string) => void) | null = null;

export function setCursorStatusListener(fn: ((status: CursorStatus) => void) | null) {
  cursorListener = fn;
}

export function setReadingStatsListener(fn: ((label: string) => void) | null) {
  readingStatsListener = fn;
}

function formatReadingLabel(text: string): string {
  const { words, minutes } = textReadingStats(text);
  if (words === 0) {
    return "";
  }
  return t("status.reading", { words: String(words), minutes: String(minutes) });
}

function emitReadingStats() {
  if (!readingStatsListener || !editor) {
    return;
  }
  readingStatsListener(formatReadingLabel(editor.getModel()?.getValue() ?? ""));
}

export function getActiveTextContent(): string | null {
  if (!editor) {
    return null;
  }
  return editor.getModel()?.getValue() ?? null;
}

function emitCursorStatus() {
  if (!editor || !cursorListener) {
    return;
  }
  const pos = editor.getPosition();
  if (!pos) {
    return;
  }
  cursorListener({ line: pos.lineNumber, column: pos.column });
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
  if (markdownPreview && previewEl && loadedPath) {
    await applyMarkdownPreview(previewEl, model.getValue(), loadedPath, previewTocEl);
  }
  if (isExperimentalSpellcheckEnabled()) {
    syncSpellcheckPane(model.getValue());
  }
  emitReadingStats();
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

function ensurePlainTextPrintToolbar(host: HTMLElement, path: string) {
  if (isMarkdownPath(path)) {
    return;
  }
  if (toolbarEl) {
    toolbarEl.remove();
    toolbarEl = null;
  }
  markdownPreview = false;
  tearDownPreviewShell();
  if (plainToolbarEl && plainToolbarPath === path) {
    return;
  }
  if (plainToolbarEl) {
    plainToolbarEl.remove();
    plainToolbarEl = null;
  }
  plainToolbarPath = path;
  plainToolbarEl = document.createElement("div");
  plainToolbarEl.className = "md-editor-toolbar text-print-toolbar";
  const printBtn = document.createElement("button");
  printBtn.type = "button";
  printBtn.textContent = t("print.preview");
  printBtn.onclick = () => {
    if (!editor) {
      return;
    }
    openPrintPreviewDocument(path, plainTextPrintBody(editor.getValue()));
  };
  plainToolbarEl.appendChild(printBtn);
  host.parentElement?.insertBefore(plainToolbarEl, host);
}

function tearDownPreviewShell() {
  if (previewShellEl) {
    previewShellEl.remove();
    previewShellEl = null;
    previewTocEl = null;
    previewEl = null;
  }
}

function ensurePreviewShell(host: HTMLElement) {
  if (previewShellEl) {
    return;
  }
  const parent = host.parentElement;
  if (!parent) {
    return;
  }
  previewShellEl = document.createElement("div");
  previewShellEl.className = "md-preview-shell hidden";
  previewTocEl = document.createElement("nav");
  previewTocEl.className = "md-toc hidden";
  previewTocEl.setAttribute("aria-label", t("markdown.tocAria"));
  previewEl = document.createElement("article");
  previewEl.className = "md-preview-body";
  previewShellEl.appendChild(previewTocEl);
  previewShellEl.appendChild(previewEl);
  parent.insertBefore(previewShellEl, host.nextSibling);
}

function ensureMarkdownToolbar(host: HTMLElement, path: string) {
  if (!isMarkdownPath(path)) {
    if (plainToolbarEl) {
      plainToolbarEl.remove();
      plainToolbarEl = null;
      plainToolbarPath = null;
    }
    ensurePlainTextPrintToolbar(host, path);
    return;
  }
  if (plainToolbarEl) {
    plainToolbarEl.remove();
    plainToolbarEl = null;
    plainToolbarPath = null;
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
    ensurePreviewShell(host);
    if (previewShellEl) {
      previewShellEl.classList.toggle("hidden", !markdownPreview);
    }
    if (markdownPreview && editor && previewEl) {
      void applyMarkdownPreview(previewEl, editor.getValue(), path, previewTocEl);
    }
    host.classList.toggle("md-split", markdownPreview);
  };
  toolbarEl.appendChild(toggle);
  const exportHtmlBtn = document.createElement("button");
  exportHtmlBtn.type = "button";
  exportHtmlBtn.textContent = t("markdown.exportHtml");
  exportHtmlBtn.onclick = () => {
    if (!editor) {
      return;
    }
    void exportMarkdownToHtmlFile(path, editor.getValue());
  };
  toolbarEl.appendChild(exportHtmlBtn);
  const printBtn = document.createElement("button");
  printBtn.type = "button";
  printBtn.textContent = t("print.preview");
  printBtn.onclick = () => {
    if (!editor) {
      return;
    }
    const html = markdownPreview && previewEl
      ? previewEl.innerHTML
      : `<pre>${editor.getValue()}</pre>`;
    openPrintPreviewDocument(path, html);
  };
  toolbarEl.appendChild(printBtn);
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
  const monacoTheme = resolveEditorThemeId(editorThemePref);
  // Spellcheck: Monaco has no built-in dictionary; see docs/qa-desktop.md.
  editor = api.editor.create(host, {
    theme: monacoTheme,
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: 14,
    wordWrap: "on",
  });
  bindFindShortcut(api);
  editor.onDidChangeModelContent(() => scheduleSync());
  editor.onDidChangeCursorSelection(() => {
    scheduleSync();
    emitCursorStatus();
  });
  editor.onDidChangeCursorPosition(() => emitCursorStatus());
}

export function openTextFind() {
  editor?.getAction("actions.find")?.run();
}

export function runTextFind(query: string, forward: boolean) {
  if (!editor) {
    return;
  }
  if (markdownPreview && previewEl && query.trim()) {
    const finder = window as unknown as Window & {
      find: (
        query: string,
        caseSensitive: boolean,
        backwards: boolean,
        wrapAround: boolean,
        wholeWord: boolean,
        searchInFrames: boolean,
        showDialog: boolean,
      ) => boolean;
    };
    finder.find(query, false, !forward, false, false, false, false);
    return;
  }
  const model = editor.getModel();
  if (!model || !query.trim()) {
    openTextFind();
    return;
  }
  const api = monacoApi;
  if (!api) {
    openTextFind();
    return;
  }
  const matches = model.findMatches(query, false, false, false, null, false);
  if (!matches.length) {
    return;
  }
  const sel = editor.getSelection();
  const pos = sel ? model.getOffsetAt(sel.getStartPosition()) : 0;
  let idx = matches.findIndex((m) => model.getOffsetAt(m.range.getStartPosition()) > pos);
  if (idx < 0) {
    idx = forward ? 0 : matches.length - 1;
  } else if (!forward) {
    idx = Math.max(0, idx - 1);
  }
  const match = matches[idx];
  editor.setSelection(match.range);
  editor.revealRangeInCenter(match.range);
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
    detachSpellcheckPane();
    loadedPath = null;
    markdownPreview = false;
    if (previewShellEl) {
      previewShellEl.classList.add("hidden");
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
    if (doc.reveal_line && doc.reveal_line > 0) {
      const line = doc.reveal_line;
      editor!.revealLineInCenter(line);
      editor!.setPosition({ lineNumber: line, column: 1 });
    }
    if (markdownPreview && previewEl) {
      await applyMarkdownPreview(previewEl, doc.content, doc.path, previewTocEl);
    }
    const editorWrap = host.parentElement;
    if (isExperimentalSpellcheckEnabled() && editorWrap) {
      attachSpellcheckPane(editorWrap);
      syncSpellcheckPane(doc.content);
    } else {
      detachSpellcheckPane();
    }
    emitCursorStatus();
    emitReadingStats();
  },
};

function escapeForPre(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

export function getPrintSelectionHtml(): { title: string; html: string } | null {
  if (!loadedPath || !editor) {
    return null;
  }
  const model = editor.getModel();
  const sel = editor.getSelection();
  if (!model || !sel || sel.isEmpty()) {
    return null;
  }
  const text = model.getValueInRange(sel);
  if (!text.trim()) {
    return null;
  }
  const html = isMarkdownPath(loadedPath)
    ? (marked.parse(text, { async: false }) as string)
    : `<pre>${escapeForPre(text)}</pre>`;
  return { title: `${loadedPath} (selection)`, html };
}

export function getPrintHtmlForPdf(): { title: string; html: string } | null {
  if (!loadedPath || !editor || !isMarkdownPath(loadedPath)) {
    return null;
  }
  const html =
    markdownPreview && previewEl
      ? previewEl.innerHTML
      : `<pre>${escapeForPre(editor.getValue())}</pre>`;
  return { title: loadedPath, html };
}

window.addEventListener("unibrowser-spellcheck-pref", () => {
  if (!hostEl || !editor || !loadedPath) {
    return;
  }
  const wrap = hostEl.parentElement;
  if (isExperimentalSpellcheckEnabled() && wrap) {
    attachSpellcheckPane(wrap);
    syncSpellcheckPane(editor.getValue());
  } else {
    detachSpellcheckPane();
  }
});
