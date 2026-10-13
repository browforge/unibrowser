import { invoke } from "@tauri-apps/api/core";

export type DocumentEditorMode = "wysiwyg" | "source" | "split" | "reading";

const MODE_STORAGE_KEY = "unibrowser.document.mode.byPath";
const LAST_EDIT_MODE_KEY = "unibrowser.document.lastEditMode";

let stripEl: HTMLElement | null = null;
let currentPath: string | null = null;
let currentMode: DocumentEditorMode = "wysiwyg";
let lastEditMode: DocumentEditorMode = "wysiwyg";
let splitRatio = 0.5;
let listeners: ((mode: DocumentEditorMode) => void)[] = [];

function readModeMap(): Record<string, DocumentEditorMode> {
  try {
    return JSON.parse(localStorage.getItem(MODE_STORAGE_KEY) ?? "{}") as Record<
      string,
      DocumentEditorMode
    >;
  } catch {
    return {};
  }
}

function writeModeForPath(path: string, mode: DocumentEditorMode) {
  const map = readModeMap();
  map[path] = mode;
  localStorage.setItem(MODE_STORAGE_KEY, JSON.stringify(map));
}

export async function defaultModeForNewProfiles(): Promise<DocumentEditorMode> {
  try {
    const prefs = await invoke<{ document_default_mode?: string }>("get_preferences");
    const raw = prefs.document_default_mode?.toLowerCase() ?? "wysiwyg";
    if (raw === "source" || raw === "split" || raw === "reading") {
      return raw;
    }
    return "wysiwyg";
  } catch {
    return "wysiwyg";
  }
}

export function getDocumentEditorMode(): DocumentEditorMode {
  return currentMode;
}

export function getSplitRatio(): number {
  return splitRatio;
}

export function onDocumentEditorModeChange(listener: (mode: DocumentEditorMode) => void): () => void {
  listeners.push(listener);
  return () => {
    listeners = listeners.filter((l) => l !== listener);
  };
}

function notifyMode() {
  for (const listener of listeners) {
    listener(currentMode);
  }
  document.body.dataset.documentEditorMode = currentMode;
  stripEl?.querySelectorAll<HTMLButtonElement>("[data-doc-mode]").forEach((btn) => {
    const active = btn.dataset.docMode === currentMode;
    btn.setAttribute("aria-selected", active ? "true" : "false");
    btn.classList.toggle("is-active", active);
  });
  const chip = document.getElementById("document-mode-status-chip");
  if (chip) {
    chip.textContent = modeLabel(currentMode);
  }
}

function modeLabel(mode: DocumentEditorMode): string {
  switch (mode) {
    case "wysiwyg":
      return "Visual";
    case "source":
      return "Source";
    case "split":
      return "Split";
    case "reading":
      return "Read";
    default:
      return mode;
  }
}

export function setDocumentEditorMode(mode: DocumentEditorMode, persist = true) {
  if (mode !== "reading") {
    lastEditMode = mode;
    localStorage.setItem(LAST_EDIT_MODE_KEY, mode);
  }
  currentMode = mode;
  if (persist && currentPath) {
    writeModeForPath(currentPath, mode);
  }
  document.body.classList.toggle("document-mode-reading", mode === "reading");
  document.body.classList.toggle("document-mode-split", mode === "split");
  document.body.classList.toggle("document-mode-wysiwyg", mode === "wysiwyg");
  document.body.classList.toggle("document-mode-source", mode === "source");
  notifyMode();
}

export async function bindDocumentPath(path: string | null) {
  currentPath = path;
  if (!path) {
    return;
  }
  const stored = readModeMap()[path];
  const mode = stored ?? (await defaultModeForNewProfiles());
  setDocumentEditorMode(mode, false);
}

export function toggleReadingMode() {
  if (currentMode === "reading") {
    const restore =
      (localStorage.getItem(LAST_EDIT_MODE_KEY) as DocumentEditorMode | null) ?? lastEditMode;
    setDocumentEditorMode(restore);
    return;
  }
  setDocumentEditorMode("reading");
}

export function mountDocumentModeStrip(host: HTMLElement) {
  stripEl = host;
  host.innerHTML = "";
  host.className = "document-mode-strip";
  host.setAttribute("role", "tablist");
  host.setAttribute("aria-label", "Document editing mode");
  const modes: { id: DocumentEditorMode; label: string }[] = [
    { id: "wysiwyg", label: "Visual" },
    { id: "source", label: "Source" },
    { id: "split", label: "Split" },
    { id: "reading", label: "Read" },
  ];
  for (const mode of modes) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "document-mode-strip-btn";
    btn.dataset.docMode = mode.id;
    btn.setAttribute("role", "tab");
    btn.textContent = mode.label;
    btn.onclick = () => setDocumentEditorMode(mode.id);
    host.appendChild(btn);
  }
  notifyMode();
}

export function setSplitRatio(ratio: number) {
  splitRatio = Math.min(0.85, Math.max(0.15, ratio));
  document.documentElement.style.setProperty(
    "--document-split-ratio",
    String(splitRatio),
  );
}
