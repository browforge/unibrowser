import type { DocumentEditorMode } from "./documentEditorModes";

export function isMarkdownFilePath(path: string | null | undefined): boolean {
  if (!path) {
    return false;
  }
  const lower = path.toLowerCase();
  return lower.endsWith(".md") || lower.endsWith(".markdown");
}

/** Markdown Visual uses Monaco live-block editing on the same buffer as Source/Split. */
export function markdownUsesMonacoLiveVisual(
  mode: DocumentEditorMode,
  path: string | null | undefined,
): boolean {
  if (!isMarkdownFilePath(path)) {
    return false;
  }
  return mode === "wysiwyg" || mode === "reading";
}

/** Modes that keep the Monaco model mounted (shared undo stack for markdown). */
export function markdownKeepsMonacoEditor(
  mode: DocumentEditorMode,
  path: string | null | undefined,
): boolean {
  if (!isMarkdownFilePath(path)) {
    return mode === "source" || mode === "split";
  }
  return mode === "source" || mode === "split" || mode === "wysiwyg" || mode === "reading";
}

export function modeSwitchPreservesMonacoUndoStack(
  from: DocumentEditorMode,
  to: DocumentEditorMode,
  path: string | null | undefined,
): boolean {
  if (!isMarkdownFilePath(path)) {
    return from === to;
  }
  return markdownKeepsMonacoEditor(from, path) && markdownKeepsMonacoEditor(to, path);
}
