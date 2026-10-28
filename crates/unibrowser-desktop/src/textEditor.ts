import { marked } from "marked";
import { invoke } from "@tauri-apps/api/core";
import { applyMarkdownPreview } from "./markdownPreview";
import { textReadingStats } from "./textStats";
import { inferSemanticBlockStyleAtCursor as inferFromLine } from "./semanticBlockStyle";
import { t } from "./i18n";
import {
  attachSpellcheckPane,
  detachSpellcheckPane,
  isExperimentalSpellcheckEnabled,
  syncSpellcheckPane,
} from "./experimentalSpellcheck";
import {
  clearEditorMarkers,
  focusNextProofingProblem,
  focusPreviousProofingProblem,
  isProofingEnabled,
  runEditorProofing,
  scheduleEditorProofing,
} from "./proofing";
import { applyMonacoEditorTheme, resolveEditorThemeId } from "./monacoEditorTheme";
import { refreshVsixTextmateGrammars } from "./monacoTextmate";

type Monaco = typeof import("monaco-editor");
type MonacoEditor = import("monaco-editor").editor.IStandaloneCodeEditor;
type MonacoModel = import("monaco-editor").editor.ITextModel;

let monacoApi: Monaco | null = null;
let editor: MonacoEditor | null = null;
let hostEl: HTMLElement | null = null;
let hostResizeObserver: ResizeObserver | null = null;
let currentFontPt = 14;

export function getEditorFontSizePt(): number {
  if (editor && monacoApi) {
    return editor.getOption(monacoApi.editor.EditorOption.fontSize);
  }
  return currentFontPt;
}
let loadedPath: string | null = null;
let syncTimer: ReturnType<typeof setTimeout> | null = null;
let suppressSync = false;
let markdownPreview = false;
let previewShellEl: HTMLElement | null = null;
let previewTocEl: HTMLElement | null = null;
let previewEl: HTMLElement | null = null;
let toolbarEl: HTMLElement | null = null;
let plainToolbarEl: HTMLDivElement | null = null;
let findBound = false;

async function getMonaco(): Promise<Monaco> {
  if (!monacoApi) {
    const loader = (await import("@monaco-editor/loader")).default;
    const monaco = await import("monaco-editor");
    loader.config({ monaco });
    monacoApi = await loader.init();
    await refreshVsixTextmateGrammars(monacoApi as Monaco).catch(() => {});
    if (import.meta.env.VITE_UB_PLATFORM === "web") {
      (window as unknown as { monaco?: Monaco }).monaco = monacoApi as Monaco;
    }
  }
  return monacoApi as Monaco;
}

export async function reloadVsixEditorGrammars(): Promise<void> {
  const api = await getMonaco();
  await refreshVsixTextmateGrammars(api);
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

/** Block-style picker trigger label from the cursor line (markdown/plain); docx uses demo dataset when set. */
export function inferSemanticBlockStyleAtCursor(): { commandId: string; label: string } {
  if (!editor || !loadedPath) {
    return inferFromLine(() => null, 1, false);
  }
  const model = editor.getModel();
  const line = editor.getPosition()?.lineNumber ?? 1;
  return inferFromLine(
    (n) => model?.getLineContent(n) ?? null,
    line,
    isMarkdownPath(loadedPath),
  );
}

/** Self-test / automation: append text at end of active Monaco document. */
export function appendActiveEditorText(chunk: string): boolean {
  if (!editor || !monacoApi) {
    return false;
  }
  const model = editor.getModel();
  if (!model) {
    return false;
  }
  const end = model.getFullModelRange().getEndPosition();
  const range = new monacoApi.Range(end.lineNumber, end.column, end.lineNumber, end.column);
  editor.executeEdits("self-test", [{ range, text: chunk, forceMoveMarkers: true }]);
  void pushToHost();
  return true;
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
  if (document.body.classList.contains("text-compact-visible")) {
    void import("./textCompactToolbar").then(({ refreshCompactBlockPickerLabel }) => {
      refreshCompactBlockPickerLabel();
    });
  }
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
  window.dispatchEvent(new CustomEvent("unibrowser-text-synced"));
  if (markdownPreview && previewEl && loadedPath) {
    await applyMarkdownPreview(previewEl, model.getValue(), loadedPath, previewTocEl);
  }
  if (isExperimentalSpellcheckEnabled()) {
    syncSpellcheckPane(model.getValue());
  }
  if (isProofingEnabled() && editor && monacoApi) {
    const surface = proofingSurfaceForPath(loadedPath, model.getLanguageId());
    scheduleEditorProofing(monacoApi, editor, surface, model.getLanguageId());
  }
  emitReadingStats();
}

function proofingSurfaceForPath(
  path: string | null,
  languageId: string,
): "plaintext" | "markdown" | "code" {
  if (path && isMarkdownPath(path)) {
    return "markdown";
  }
  if (languageId !== "plaintext" && languageId !== "markdown") {
    return "code";
  }
  return "plaintext";
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
    if (loadedPath && isMarkdownPath(loadedPath) && editor) {
      void import("./documentEditorHost").then((m) =>
        m.onMonacoMarkdownContentChanged(editor!.getValue()),
      );
    }
  }, 250);
}

function ensurePlainTextPrintToolbar(_host: HTMLElement, path: string) {
  if (plainToolbarEl) {
    plainToolbarEl.remove();
    plainToolbarEl = null;
  }
  if (isMarkdownPath(path)) {
    return;
  }
  if (toolbarEl) {
    toolbarEl.remove();
    toolbarEl = null;
  }
  markdownPreview = false;
  tearDownPreviewShell();
}

function ensureMarkdownPreviewShell(host: HTMLElement) {
  if (previewShellEl && previewEl) {
    return;
  }
  const parent = host.parentElement;
  if (!parent) {
    return;
  }
  previewShellEl = document.createElement("div");
  previewShellEl.className = "md-preview-shell hidden";
  previewTocEl = document.createElement("nav");
  previewTocEl.className = "md-preview-toc";
  previewTocEl.setAttribute("aria-label", "Table of contents");
  previewEl = document.createElement("div");
  previewEl.className = "md-preview md-preview-body";
  previewShellEl.append(previewTocEl, previewEl);
  parent.appendChild(previewShellEl);
}

export async function setMarkdownPreviewMode(
  enabled: boolean,
  host?: HTMLElement,
): Promise<void> {
  const editorHost = host ?? hostEl;
  if (!editorHost || !loadedPath || !isMarkdownPath(loadedPath) || !editor) {
    return;
  }
  ensureMarkdownPreviewShell(editorHost);
  if (!previewShellEl || !previewEl) {
    return;
  }
  markdownPreview = enabled;
  if (enabled) {
    previewShellEl.classList.remove("hidden");
    editorHost.classList.add("md-split");
    const model = editor.getModel();
    if (model) {
      await applyMarkdownPreview(
        previewEl,
        model.getValue(),
        loadedPath,
        previewTocEl,
        {
          embedImages: document.documentElement.classList.contains(
            "ub-capture-md-preview",
          ),
        },
      );
    }
  } else {
    previewShellEl.classList.add("hidden");
    editorHost.classList.remove("md-split");
  }
}

function tearDownPreviewShell() {
  if (previewShellEl) {
    previewShellEl.remove();
    previewShellEl = null;
    previewTocEl = null;
    previewEl = null;
  }
}

function ensureMarkdownToolbar(host: HTMLElement, path: string) {
  if (toolbarEl) {
    toolbarEl.remove();
    toolbarEl = null;
  }
  if (!isMarkdownPath(path)) {
    markdownPreview = false;
    tearDownPreviewShell();
    host.classList.remove("md-split");
    ensurePlainTextPrintToolbar(host, path);
    return;
  }
  if (plainToolbarEl) {
    plainToolbarEl.remove();
    plainToolbarEl = null;
  }
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

async function syncMonacoTheme(): Promise<void> {
  const api = await getMonaco();
  await applyMonacoEditorTheme(editorThemePref);
  api.editor.setTheme(resolveEditorThemeId(editorThemePref));
  editor?.layout();
}

async function ensureEditor(host: HTMLElement) {
  if (editor && hostEl === host) {
    await syncMonacoTheme();
    return;
  }
  findBound = false;
  const api = await getMonaco();
  if (editor) {
    editor.dispose();
    editor = null;
  }
  for (const orphan of api.editor.getEditors()) {
    const node = orphan.getDomNode();
    if (node && host.contains(node)) {
      orphan.dispose();
    }
  }
  hostEl = host;
  const preserved: HTMLElement[] = [];
  for (const id of ["rich-text-editor-host", "document-split-preview-host"]) {
    const el = document.getElementById(id);
    if (el && el.parentElement === host) {
      preserved.push(el);
    }
  }
  const existingSlot = host.querySelector<HTMLElement>(".document-monaco-slot");
  if (existingSlot) {
    preserved.push(existingSlot);
  }
  host.innerHTML = "";
  for (const el of preserved) {
    host.appendChild(el);
  }
  let monacoSlot = host.querySelector<HTMLElement>(".document-monaco-slot");
  if (!monacoSlot) {
    monacoSlot = document.createElement("div");
    monacoSlot.className = "document-monaco-slot";
    host.appendChild(monacoSlot);
  }
  await applyMonacoEditorTheme(editorThemePref);
  const monacoTheme = resolveEditorThemeId(editorThemePref);
  // Spellcheck: Monaco has no built-in dictionary; see docs/qa-desktop.md.
  editor = api.editor.create(monacoSlot, {
    theme: monacoTheme,
    automaticLayout: true,
    minimap: { enabled: false },
    fontSize: currentFontPt,
    fontFamily: '"Source Code Pro", "DejaVu Sans Mono", monospace',
    wordWrap: "on",
    renderValidationDecorations: "on",
    experimentalGpuAcceleration: "off",
  });
  hostResizeObserver?.disconnect();
  hostResizeObserver = new ResizeObserver(() => {
    editor?.layout();
  });
  hostResizeObserver.observe(host);
  hostResizeObserver.observe(monacoSlot);
  host.classList.add("monaco-host-ready");
  bindFindShortcut(api);
  editor.onDidChangeModelContent(() => scheduleSync());
  editor.onDidChangeCursorSelection(() => {
    scheduleSync();
    emitCursorStatus();
  });
  editor.onDidChangeCursorPosition(() => emitCursorStatus());
  for (const orphan of api.editor.getEditors()) {
    if (orphan === editor) {
      continue;
    }
    const node = orphan.getDomNode();
    if (node && host.contains(node)) {
      orphan.dispose();
    }
  }
}

export function openTextFind() {
  editor?.getAction("actions.find")?.run();
}

/** Recompute Monaco after shell regions or sidebars change size. */
export function relayoutTextEditor(): void {
  editor?.layout();
  window.dispatchEvent(new Event("resize"));
}

window.addEventListener("unibrowser-shell-layout-changed", () => {
  relayoutTextEditor();
});

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
    toolbarEl?.classList.remove("hidden");
    plainToolbarEl?.classList.remove("hidden");
  },

  hide(host: HTMLElement) {
    host.classList.add("hidden");
    toolbarEl?.classList.add("hidden");
    plainToolbarEl?.classList.add("hidden");
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
    await syncMonacoTheme();
    const api = await getMonaco();
    bindFindShortcut(api);
    const model = editor!.getModel();
    const uri = api.Uri.parse(`file://${doc.path}`);
    const language = doc.language || "plaintext";
    const pathChanged =
      loadedPath !== doc.path || model?.uri.toString() !== uri.toString();
    suppressSync = true;
    try {
      if (pathChanged) {
        const next =
          api.editor.getModel(uri) ??
          api.editor.createModel(doc.content, language, uri);
        if (next.getValue() !== doc.content) {
          next.setValue(doc.content);
        }
        api.editor.setModelLanguage(next, language);
        editor!.setModel(next);
      }
      const activeModel = editor!.getModel();
      if (activeModel && activeModel.getValue() !== doc.content) {
        activeModel.setValue(doc.content);
        api.editor.setModelLanguage(activeModel, language);
      }
      loadedPath = doc.path;
    } finally {
      suppressSync = false;
    }
    host.dataset.documentContentLength = String(doc.content.length);
    host.dataset.modelContentLength = String(editor!.getModel()?.getValueLength() ?? 0);
    currentFontPt = doc.font_size_pt;
    editor!.updateOptions({ fontSize: doc.font_size_pt });
    editor!.layout();
    window.requestAnimationFrame(() => {
      editor?.layout();
      window.requestAnimationFrame(() => editor?.layout());
    });
    const { syncDocumentEditorForTextTab } = await import("./documentEditorHost");
    await syncDocumentEditorForTextTab(host, doc.path, doc.content);
    if (doc.reveal_line && doc.reveal_line > 0) {
      const line = doc.reveal_line;
      editor!.revealLineInCenter(line);
      editor!.setPosition({ lineNumber: line, column: 1 });
    }
    if (markdownPreview && previewEl) {
      await applyMarkdownPreview(previewEl, doc.content, doc.path, previewTocEl, {
        embedImages: document.documentElement.classList.contains(
          "ub-capture-md-preview",
        ),
      });
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
    if (isProofingEnabled() && editor && monacoApi) {
      const model = editor.getModel();
      if (model) {
        const surface = proofingSurfaceForPath(doc.path, model.getLanguageId());
        scheduleEditorProofing(monacoApi, editor, surface, model.getLanguageId());
      }
    }
  },

  async ensureProofingApplied(): Promise<void> {
    if (!isProofingEnabled() || !editor || !monacoApi) {
      return;
    }
    const model = editor.getModel();
    if (!model) {
      return;
    }
    const surface = proofingSurfaceForPath(loadedPath, model.getLanguageId());
    await runEditorProofing(monacoApi, editor, surface, model.getLanguageId());
    const { refreshProblemsPanel } = await import("./problemsPanel");
    await refreshProblemsPanel();
    await new Promise<void>((resolve) => {
      window.requestAnimationFrame(() => {
        window.requestAnimationFrame(() => resolve());
      });
    });
  },

  focusNextProofingInEditor() {
    if (editor) {
      focusNextProofingProblem(editor);
    }
  },

  focusPreviousProofingInEditor() {
    if (editor) {
      focusPreviousProofingProblem(editor);
    }
  },

  getLineText(lineNumber: number): string | null {
    const model = editor?.getModel();
    if (!model || lineNumber < 1 || lineNumber > model.getLineCount()) {
      return null;
    }
    return model.getLineContent(lineNumber);
  },

  setCursorLine(lineNumber: number): boolean {
    if (!editor || !monacoApi) {
      return false;
    }
    const model = editor.getModel();
    if (!model || lineNumber < 1 || lineNumber > model.getLineCount()) {
      return false;
    }
    editor.setPosition({ lineNumber, column: 1 });
    editor.revealLineInCenter(lineNumber);
    editor.focus();
    return true;
  },

  async applySemanticBlockStyle(commandId: string): Promise<boolean> {
    if (!editor || !monacoApi || !loadedPath) {
      return false;
    }
    const model = editor.getModel();
    if (!model) {
      return false;
    }
    const line = editor.getPosition()?.lineNumber ?? 1;
    const text = model.getLineContent(line);
    const stripped = text.replace(/^#{1,6}\s+/, "").replace(/^>\s+/, "");
    let nextLine = stripped;
    if (commandId.endsWith("paragraph")) {
      nextLine = stripped;
    } else if (commandId.endsWith("heading_1")) {
      nextLine = `# ${stripped}`;
    } else if (commandId.endsWith("heading_2")) {
      nextLine = `## ${stripped}`;
    } else if (commandId.endsWith("heading_3")) {
      nextLine = `### ${stripped}`;
    } else if (commandId.endsWith("heading_4")) {
      nextLine = `#### ${stripped}`;
    } else if (commandId.endsWith("heading_5")) {
      nextLine = `##### ${stripped}`;
    } else if (commandId.endsWith("heading_6")) {
      nextLine = `###### ${stripped}`;
    } else if (commandId.endsWith("blockquote")) {
      nextLine = `> ${stripped}`;
    } else if (commandId.endsWith("code_block")) {
      nextLine = `\`\`\`\n${stripped}\n\`\`\``;
    } else {
      return false;
    }
    const range = new monacoApi.Range(line, 1, line, text.length + 1);
    editor.executeEdits("block-style", [{ range, text: nextLine, forceMoveMarkers: true }]);
    editor.focus();
    await pushToHost();
    return true;
  },

  async applyMarkdownInlineWrap(before: string, after: string): Promise<boolean> {
    if (!editor || !monacoApi || !loadedPath || !isMarkdownPath(loadedPath)) {
      return false;
    }
    const model = editor.getModel();
    const selection = editor.getSelection();
    if (!model || !selection) {
      return false;
    }
    const selected = model.getValueInRange(selection);
    const replacement = selected ? `${before}${selected}${after}` : `${before}${after}`;
    editor.executeEdits("markdown-inline", [
      {
        range: selection,
        text: replacement,
        forceMoveMarkers: true,
      },
    ]);
    if (!selected) {
      editor.setPosition({
        lineNumber: selection.startLineNumber,
        column: selection.startColumn + before.length,
      });
    }
    editor.focus();
    await pushToHost();
    return true;
  },

  async setContentFromHost(content: string) {
    if (!editor) {
      return;
    }
    suppressSync = true;
    try {
      const model = editor.getModel();
      if (model && model.getValue() !== content) {
        model.setValue(content);
      }
    } finally {
      suppressSync = false;
    }
  },

  async refreshProofingMarkers() {
    if (!editor || !monacoApi) {
      return;
    }
    if (!isProofingEnabled()) {
      clearEditorMarkers(monacoApi, editor);
      return;
    }
    const model = editor.getModel();
    if (!model) {
      return;
    }
    const surface = proofingSurfaceForPath(loadedPath, model.getLanguageId());
    scheduleEditorProofing(monacoApi, editor, surface, model.getLanguageId());
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

window.addEventListener("unibrowser-proofing-pref", () => {
  void textEditor.refreshProofingMarkers();
});

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

export function navigateToTextLineCol(
  line: number,
  col: number,
  endLine?: number,
  endCol?: number,
) {
  if (!editor) {
    return;
  }
  const startLine = Math.max(1, line);
  const startCol = Math.max(1, col);
  const endL = Math.max(1, endLine ?? line);
  const endC = Math.max(1, endCol ?? col);
  editor.setSelection({
    startLineNumber: startLine,
    startColumn: startCol,
    endLineNumber: endL,
    endColumn: endC,
  });
  editor.revealLineInCenter(startLine);
}

export function navigateToTextByteRange(startByte: number, endByte: number) {
  if (!editor) {
    return;
  }
  const model = editor.getModel();
  if (!model) {
    return;
  }
  const max = model.getValueLength();
  const start = model.getPositionAt(Math.min(Math.max(0, startByte), max));
  const end = model.getPositionAt(Math.min(Math.max(0, endByte), max));
  editor.setSelection({
    startLineNumber: start.lineNumber,
    startColumn: start.column,
    endLineNumber: end.lineNumber,
    endColumn: end.column,
  });
  editor.revealRangeInCenter({
    startLineNumber: start.lineNumber,
    startColumn: start.column,
    endLineNumber: end.lineNumber,
    endColumn: end.column,
  });
}

if (typeof window !== "undefined") {
  (window as unknown as { __ubTextEditor?: typeof textEditor }).__ubTextEditor = textEditor;
}
