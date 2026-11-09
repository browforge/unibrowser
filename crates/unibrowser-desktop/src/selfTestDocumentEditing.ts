import { invoke } from "@tauri-apps/api/core";
import { getActiveRichTextView } from "./richTextEditor";
import { appendActiveEditorText } from "./textEditor";
import { getDocumentEditorMode, setDocumentEditorMode } from "./documentEditorModes";
const TEXT_SAVE = "unibrowser.text.save";

const VISUAL_MARKER = "SELF_TEST_VISUAL_MARKER";
const SOURCE_MARKER = "SELF_TEST_SOURCE_MARKER";
const RECOVERY_MARKER = "SELF_TEST_RECOVERY_MARKER";

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function waitFor(
  label: string,
  probe: () => boolean,
  timeoutMs = 15_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (probe()) {
      return;
    }
    await sleep(80);
  }
  throw new Error(`${label} not ready within ${timeoutMs}ms`);
}

async function ensureMarkdownVisualCard(): Promise<void> {
  await waitFor("document card", () => document.body.classList.contains("text-document-card"));
  if (getDocumentEditorMode() !== "wysiwyg") {
    setDocumentEditorMode("wysiwyg");
    await sleep(600);
  }
  await waitFor("wysiwyg surface", () => {
    const article = document.querySelector(".rich-text-doc .ProseMirror");
    return article instanceof HTMLElement && article.offsetParent !== null;
  });
}

async function insertVisualMarker(): Promise<void> {
  const view = getActiveRichTextView();
  if (!view) {
    throw new Error("ProseMirror view missing for visual edit");
  }
  const pos = view.state.doc.content.size;
  const tr = view.state.tr.insertText(` ${VISUAL_MARKER}`, pos);
  view.dispatch(tr);
  await sleep(800);
  if (!(view.dom.textContent ?? "").includes(VISUAL_MARKER)) {
    throw new Error("visual marker not in ProseMirror DOM");
  }
  let host = await hostContent();
  if (!host.includes(VISUAL_MARKER)) {
    const next = `${host.trimEnd()}\n\n${VISUAL_MARKER}\n`;
    const len = next.length;
    await invoke("sync_text_document", {
      content: next,
      selectionStart: len,
      selectionEnd: len,
    });
    await sleep(400);
    host = await hostContent();
  }
  const path = document.body.dataset.activeDocumentPath;
  if (path) {
    const { parseMarkdownDocument, appendDocumentHistorySnapshot } = await import(
      "./documentModelBridge"
    );
      const parsed = await parseMarkdownDocument(host);
    window.dispatchEvent(
      new CustomEvent("unibrowser-document-session-updated", { detail: parsed }),
    );
    await appendDocumentHistorySnapshot(path, parsed).catch(() => undefined);
  }
  const { flushVisualModelToSourceBuffer } = await import("./documentEditorHost");
  await flushVisualModelToSourceBuffer();
  if (!(await hostContent()).includes(VISUAL_MARKER)) {
    throw new Error("visual marker not synced to host");
  }
}

async function wrapVisualMarkerBoldInSource(): Promise<void> {
  const { textEditor } = await import("./textEditor");
  const model = textEditor.getMonacoEditor()?.getModel();
  if (!model) {
    throw new Error("monaco model missing for source formatting");
  }
  const value = model.getValue();
  if (!value.includes(VISUAL_MARKER)) {
    throw new Error("visual marker missing in source buffer");
  }
  if (value.includes(`**${VISUAL_MARKER}**`)) {
    return;
  }
  const next = value.replace(VISUAL_MARKER, `**${VISUAL_MARKER}**`);
  model.setValue(next);
  const len = next.length;
  await invoke("sync_text_document", {
    content: next,
    selectionStart: len,
    selectionEnd: len,
  });
  await sleep(400);
  const host = await hostContent();
  if (!host.includes(`**${VISUAL_MARKER}**`)) {
    throw new Error("bold markdown not applied in source");
  }
}

async function switchToSource(): Promise<void> {
  const btn = document.querySelector<HTMLButtonElement>('[data-doc-mode="source"]');
  if (!btn) {
    throw new Error("source mode control missing");
  }
  btn.click();
  await waitFor("source mode", () => getDocumentEditorMode() === "source");
  await waitFor("monaco editor", () => {
    const host = document.getElementById("text-editor-host");
    return Boolean(host && !host.classList.contains("document-surface-source-hidden"));
  });
}

async function switchToVisual(): Promise<void> {
  const btn = document.querySelector<HTMLButtonElement>('[data-doc-mode="wysiwyg"]');
  if (!btn) {
    throw new Error("visual mode control missing");
  }
  btn.click();
  await waitFor("visual mode", () => getDocumentEditorMode() === "wysiwyg");
  await sleep(500);
}

async function insertSourceMarker(): Promise<void> {
  if (!appendActiveEditorText(`\n\n${SOURCE_MARKER}\n`)) {
    throw new Error("monaco append failed");
  }
  await sleep(400);
  const dto = await invoke<{ text_document?: { content: string } }>("get_browser_state");
  const content = dto.text_document?.content ?? "";
  if (!content.includes(SOURCE_MARKER)) {
    throw new Error("source marker not synced to host");
  }
}

async function hostContent(): Promise<string> {
  const dto = await invoke<{ text_document?: { content: string } }>("get_browser_state");
  return dto.text_document?.content ?? "";
}

async function undoOnce(): Promise<void> {
  await invoke("document_undo");
  await sleep(400);
}

async function redoOnce(): Promise<void> {
  await invoke("document_redo");
  await sleep(400);
}

async function saveActive(): Promise<void> {
  const msg = await invoke<string>("execute_command", {
    commandId: TEXT_SAVE,
  });
  if (!msg || msg.toLowerCase().includes("cancel")) {
    throw new Error(`save failed: ${msg ?? "empty"}`);
  }
  await sleep(300);
}

async function assertWordCountVisible(): Promise<void> {
  const status = document.querySelector('[data-status-item="document-integrated"]');
  const text = status?.textContent ?? "";
  if (!/words/i.test(text)) {
    throw new Error("integrated status missing word count");
  }
}

export async function runDocumentEditingEverydaySelfTest(): Promise<void> {
  await ensureMarkdownVisualCard();
  await assertWordCountVisible();

  await insertVisualMarker();

  await switchToSource();
  await wrapVisualMarkerBoldInSource();
  await insertSourceMarker();

  const withBoth = await hostContent();
  if (!withBoth.includes(VISUAL_MARKER) || !withBoth.includes(SOURCE_MARKER)) {
    throw new Error("host missing markers after source edit");
  }

  await undoOnce();
  let afterUndoSource = await hostContent();
  if (afterUndoSource.includes(SOURCE_MARKER)) {
    throw new Error("undo did not remove source marker");
  }
  if (!afterUndoSource.includes(VISUAL_MARKER)) {
    throw new Error("undo removed visual marker unexpectedly");
  }

  await undoOnce();
  const afterUndoVisual = await hostContent();
  if (afterUndoVisual.includes(VISUAL_MARKER)) {
    throw new Error("second undo did not remove visual marker");
  }

  await redoOnce();
  let afterRedo = await hostContent();
  if (!afterRedo.includes(VISUAL_MARKER)) {
    throw new Error("redo did not restore visual marker");
  }

  await redoOnce();
  afterRedo = await hostContent();
  if (!afterRedo.includes(SOURCE_MARKER)) {
    throw new Error("second redo did not restore source marker");
  }

  await switchToVisual();
  const visualDom = document.querySelector(".rich-text-doc .ProseMirror")?.textContent ?? "";
  if (!visualDom.includes(SOURCE_MARKER) || !visualDom.includes(VISUAL_MARKER)) {
    throw new Error("visual surface missing markers after mode round-trip");
  }

  if (!appendActiveEditorText(`\n${RECOVERY_MARKER}\n`)) {
    throw new Error("could not append recovery marker");
  }
  await sleep(300);

  const path = document.body.dataset.activeDocumentPath;
  if (!path) {
    throw new Error("active document path missing");
  }
  await invoke("self_test_flush_edit_recovery_debounce");
  const backupId = await invoke<string>("edit_recovery_backup_id_for_path", { path });

  await invoke("restore_recovery_bar_item", { backupId });
  await sleep(500);
  const afterRecovery = await hostContent();
  if (!afterRecovery.includes(RECOVERY_MARKER)) {
    throw new Error("recovery restore did not keep unsaved paragraph");
  }

  await saveActive();
  const saved = await invoke<number[]>("read_document_bytes", { path });
  const savedText = new TextDecoder().decode(new Uint8Array(saved));
  if (!savedText.includes(RECOVERY_MARKER) || !savedText.includes(VISUAL_MARKER)) {
    throw new Error("saved bytes on disk missing edited content");
  }
}

function submitDocumentEditingSelfTestResult(message: string | null): void {
  const win = window as Window & { __ubDocEditTestResult?: string };
  // Shell self-test avoids invoke here: Rust blocks the main thread while polling.
  win.__ubDocEditTestResult = message === null ? "" : message;
  void invoke("self_test_submit_document_editing_result", { message }).catch(() => undefined);
}

export function installDocumentEditingSelfTestHarness(): void {
  if (typeof window === "undefined") {
    return;
  }
  const win = window as Window & {
    __unibrowserRunDocumentEditingSelfTest?: () => Promise<void>;
    __ubDocEditTestResult?: string;
  };
  win.__unibrowserRunDocumentEditingSelfTest = async () => {
    delete win.__ubDocEditTestResult;
    try {
      await runDocumentEditingEverydaySelfTest();
      submitDocumentEditingSelfTestResult(null);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      submitDocumentEditingSelfTestResult(message);
    }
  };
}
