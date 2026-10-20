import { invoke } from "@tauri-apps/api/core";
import { showInlineConfirm } from "./inlineUi";

export const HISTORY_TRIM_BEFORE_ID = "unibrowser.history.trimBefore";
export const HISTORY_SQUASH_SELECTION_ID = "unibrowser.history.squashSelection";

let selectedSeq: number | null = null;
let selectedEndSeq: number | null = null;

export function setHistoryPanelSelection(seq: number, endSeq: number) {
  selectedSeq = seq;
  selectedEndSeq = endSeq;
}

export function getHistoryPanelSelection(): { from: number; to: number } | null {
  if (selectedSeq == null || selectedEndSeq == null) {
    return null;
  }
  return { from: selectedSeq, to: selectedEndSeq };
}

function activeDocumentPath(): string | null {
  return document.body.dataset.activeDocumentPath ?? null;
}

async function applyJumpContent(content: string) {
  const { textEditor } = await import("./textEditor");
  await textEditor.setContentFromHost(content);
  const { syncDocumentModelFromMarkdown } = await import("./documentEditorHost");
  await syncDocumentModelFromMarkdown(content);
}

export async function jumpHistoryToSeq(path: string, seq: number): Promise<void> {
  const content = await invoke<string>("document_history_jump_to_seq", { path, seq });
  await applyJumpContent(content);
}

export async function runHistoryCommand(commandId: string): Promise<boolean> {
  const path = activeDocumentPath();
  if (!path || (!path.endsWith(".md") && !path.endsWith(".markdown"))) {
    return false;
  }
  const selection = getHistoryPanelSelection();
  if (!selection) {
    return false;
  }
  if (commandId === HISTORY_TRIM_BEFORE_ID) {
    const ok = await showInlineConfirm(
      `Remove all history entries before seq ${selection.from}? Older edits cannot be restored from the log.`,
      "Trim history",
      "Cancel",
    );
    if (!ok) {
      return true;
    }
    await invoke("document_history_trim_before", {
      path,
      keepFromSeq: selection.from,
    });
    const { refreshHistoryPanel } = await import("./historyPanel");
    await refreshHistoryPanel(path);
    return true;
  }
  if (commandId === HISTORY_SQUASH_SELECTION_ID) {
    const ok = await showInlineConfirm(
      `Squash seq ${selection.from}–${selection.to} into one snapshot? Undo depth for that range is lost.`,
      "Squash",
      "Cancel",
    );
    if (!ok) {
      return true;
    }
    await invoke("document_history_squash_range", {
      path,
      fromSeq: selection.from,
      toSeq: selection.to,
    });
    const { refreshHistoryPanel } = await import("./historyPanel");
    await refreshHistoryPanel(path);
    return true;
  }
  return false;
}
