import { invoke } from "@tauri-apps/api/core";
import { TextSelection } from "prosemirror-state";
import { getActiveRichTextView } from "./richTextEditor";
import { appendActiveEditorText, getActiveTextContent } from "./textEditor";
import { getDocumentEditorMode, setDocumentEditorMode } from "./documentEditorModes";
import {
  appendDocumentHistorySnapshot,
  parseMarkdownDocument,
  sourceProjectionDocument,
} from "./documentModelBridge";
import { richTextSchema } from "./richTextProseMirror";

const TEXT_SAVE = "unibrowser.text.save";

const VISUAL_MARKER = "SELF_TEST_VISUAL_MARKER";
const SOURCE_MARKER = "SELF_TEST_SOURCE_MARKER";
const KILL_RECOVERY_MARKER = "SELF_TEST_KILL_RECOVERY_MARKER";

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function waitForHostIncludes(
  fragment: string,
  timeoutMs = 30_000,
  readContent: () => Promise<string> = hostContent,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if ((await readContent()).includes(fragment)) {
      return;
    }
    await sleep(100);
  }
  throw new Error(`host missing ${fragment}`);
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

async function syncMonacoSourceBuffer(): Promise<void> {
  const sync = (
    window as unknown as { __unibrowserSyncMonacoSourceBuffer?: () => Promise<void> }
  ).__unibrowserSyncMonacoSourceBuffer;
  if (!sync) {
    throw new Error("monaco source sync hook missing");
  }
  await sync();
}

async function applyShellState(dto: Record<string, unknown>): Promise<void> {
  const apply = (
    window as unknown as { __selfTestApplyState?: (d: unknown) => void | Promise<void> }
  ).__selfTestApplyState;
  if (apply) {
    await Promise.resolve(apply(dto));
  }
  await sleep(600);
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
  const { getDocumentModelSession } = await import("./documentEditorHost");
  await waitFor("document session", () => Boolean(getDocumentModelSession()), 30_000);
}

async function editorMarkdown(): Promise<string> {
  const { getActiveTextContent } = await import("./textEditor");
  const monaco = getActiveTextContent();
  if (monaco) {
    return monaco;
  }
  const dto = await invoke<{ text_document?: { content: string } }>("get_browser_state");
  return dto.text_document?.content ?? "";
}

async function hostContent(): Promise<string> {
  const dto = await invoke<{ text_document?: { content: string } }>("get_browser_state");
  return dto.text_document?.content ?? "";
}

async function insertVisualMarker(): Promise<void> {
  const view = getActiveRichTextView();
  if (!view) {
    throw new Error("ProseMirror view missing for visual edit");
  }
  const pos = view.state.doc.content.size;
  const tr = view.state.tr.insertText(` ${VISUAL_MARKER}`, pos);
  view.dispatch(tr);
  await sleep(300);
  await waitFor("visual marker in editor", () => {
    const surface = getActiveRichTextView();
    return (surface?.state.doc.textContent ?? "").includes(VISUAL_MARKER);
  });
}

async function boldVisualMarkerInProseMirror(): Promise<void> {
  const view = getActiveRichTextView();
  if (!view) {
    throw new Error("ProseMirror view missing for visual formatting");
  }
  let from = -1;
  let to = -1;
  view.state.doc.descendants((node, pos) => {
    if (from >= 0 || !node.isText) {
      return;
    }
    const text = node.text ?? "";
    const idx = text.indexOf(VISUAL_MARKER);
    if (idx >= 0) {
      from = pos + idx;
      to = from + VISUAL_MARKER.length;
    }
  });
  if (from < 0 || to < 0) {
    throw new Error("visual marker position not found");
  }
  const bold = richTextSchema.marks.bold;
  if (!bold) {
    throw new Error("ProseMirror bold mark missing");
  }
  const markTr = view.state.tr
    .setSelection(TextSelection.create(view.state.doc, from, to))
    .addMark(from, to, bold.create());
  view.dispatch(markTr);
  const { flushVisualEditsToHostDocument } = await import("./documentEditorHost");
  await flushVisualEditsToHostDocument();
  await waitForHostIncludes(`**${VISUAL_MARKER}**`, 15_000).catch(async () => {
    const host = await hostContent();
    if (!host.includes("**") || !host.includes(VISUAL_MARKER)) {
      throw new Error("visual bold not reflected in host markdown");
    }
  });
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
  await waitFor("prosemirror surface", () => Boolean(getActiveRichTextView()));
}

async function insertSourceMarker(): Promise<void> {
  if (!(await appendActiveEditorText(`\n\n${SOURCE_MARKER}\n`))) {
    throw new Error("monaco append failed");
  }
  await sleep(500);
  await syncMonacoSourceBuffer();
  await sleep(400);
  if (!(await editorMarkdown()).includes(SOURCE_MARKER)) {
    throw new Error("source marker not synced to editor");
  }
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

async function closeActiveTabAndReopen(path: string): Promise<void> {
  const saved = await invoke<number[]>("read_document_bytes", { path });
  const savedText = new TextDecoder().decode(new Uint8Array(saved));
  const reopened = await invoke<Record<string, unknown>>("open_path", { path, line: null });
  await applyShellState(reopened);
  await ensureMarkdownVisualCard();
  const content = await hostContent();
  if (!content.includes(SOURCE_MARKER) || !content.includes(VISUAL_MARKER)) {
    throw new Error("reopened document missing edited markers");
  }
  if (savedText !== content) {
    throw new Error("reopened host content differs from on-disk bytes");
  }
}

function submitDocumentEditingSelfTestResult(message: string | null): void {
  void invoke("self_test_submit_document_editing_result", { message }).catch(() => undefined);
}

export async function runDocumentEditingEverydayPhase1(): Promise<void> {
  await invoke("self_test_ping", { phase: "doc-edit-begin" }).catch(() => undefined);
  await ensureMarkdownVisualCard();
  await assertWordCountVisible();

  await insertVisualMarker();
  await invoke("self_test_ping", { phase: "doc-edit-visual-done" }).catch(() => undefined);
  await boldVisualMarkerInProseMirror();
  await invoke("self_test_ping", { phase: "doc-edit-bold-done" }).catch(() => undefined);

  await switchToSource();
  await sleep(1200);
  await invoke("self_test_ping", { phase: "doc-edit-source-mode" }).catch(() => undefined);
  await insertSourceMarker();
  await invoke("self_test_ping", { phase: "doc-edit-source-marker" }).catch(() => undefined);

  const monacoText = getActiveTextContent() ?? "";
  if (!monacoText.includes(VISUAL_MARKER) || !monacoText.includes(SOURCE_MARKER)) {
    throw new Error("source buffer missing markers after source edit");
  }
  const path = document.body.dataset.activeDocumentPath;
  if (!path) {
    throw new Error("active document path missing after phase1");
  }
  const monacoSnapshot = getActiveTextContent() ?? "";
  const parsed = await parseMarkdownDocument(monacoSnapshot);
  const parsedProjection = await sourceProjectionDocument(parsed);
  if (!parsedProjection.includes(SOURCE_MARKER) || !parsedProjection.includes(VISUAL_MARKER)) {
    throw new Error("parsed monaco projection missing markers after phase1");
  }
  await invoke("document_history_reset", { path });
  await appendDocumentHistorySnapshot(path, {
    model: parsed.model,
    source_map: parsed.source_map,
    original_base64: parsed.original_base64,
  });
  await invoke("self_test_ping", { phase: "doc-edit-phase1-done" }).catch(() => undefined);
}

export async function runDocumentEditingEverydayPhase2(): Promise<void> {
  const content = await hostContent();
  if (!content.includes(SOURCE_MARKER) || !content.includes(VISUAL_MARKER)) {
    throw new Error("host missing markers before undo/redo");
  }

  const undoDto = await invoke<Record<string, unknown>>("document_undo");
  await applyShellState(undoDto);
  const afterUndo = await hostContent();
  if (afterUndo.includes(SOURCE_MARKER) || afterUndo.includes(VISUAL_MARKER)) {
    throw new Error("undo did not revert squashed document edit");
  }

  const redoDto = await invoke<{ text_document?: { content: string } }>("document_redo");
  const redoneInDto = redoDto.text_document?.content ?? "";
  if (!redoneInDto.includes(SOURCE_MARKER) || !redoneInDto.includes(VISUAL_MARKER)) {
    throw new Error(
      `document_redo missing markers (src=${redoneInDto.includes(SOURCE_MARKER)} vis=${redoneInDto.includes(VISUAL_MARKER)} tail=${JSON.stringify(redoneInDto.slice(-120))})`,
    );
  }
  await applyShellState(redoDto);
  const afterRedo = await hostContent();
  if (!afterRedo.includes(SOURCE_MARKER) || !afterRedo.includes(VISUAL_MARKER)) {
    throw new Error("redo did not restore source and visual markers in host");
  }

  await switchToVisual();
  const docPath = document.body.dataset.activeDocumentPath;
  if (docPath) {
    const { syncDocumentModelFromMarkdown } = await import("./documentEditorHost");
    await syncDocumentModelFromMarkdown(afterRedo, docPath);
  }
  await invoke("self_test_ping", { phase: "doc-edit-undo-redo-done" }).catch(() => undefined);
  const visualHost = await hostContent();
  if (!visualHost.includes(SOURCE_MARKER) || !visualHost.includes(VISUAL_MARKER)) {
    throw new Error("visual mode host missing markers after round-trip");
  }

  await saveActive();
  const path = document.body.dataset.activeDocumentPath;
  if (!path) {
    throw new Error("active document path missing");
  }
  const saved = await invoke<number[]>("read_document_bytes", { path });
  const savedText = new TextDecoder().decode(new Uint8Array(saved));
  if (!savedText.includes(SOURCE_MARKER) || !savedText.includes(VISUAL_MARKER)) {
    throw new Error("saved bytes on disk missing edited content");
  }

  await closeActiveTabAndReopen(path);
  await invoke("self_test_ping", { phase: "doc-edit-reopen-done" }).catch(() => undefined);
}

export async function runDocumentEditingEverydaySelfTest(): Promise<void> {
  await runDocumentEditingEverydayPhase1();
  await runDocumentEditingEverydayPhase2();
}

export async function runDocumentEditingKillPrepSelfTest(): Promise<void> {
  await ensureMarkdownVisualCard();
  if (!(await appendActiveEditorText(`\n${KILL_RECOVERY_MARKER}\n`))) {
    throw new Error("could not append kill recovery marker");
  }
  await syncMonacoSourceBuffer();
  await sleep(200);
  await invoke("self_test_flush_edit_recovery_debounce");
  const path = document.body.dataset.activeDocumentPath;
  if (!path) {
    throw new Error("active document path missing for kill prep");
  }
  const backupId = await invoke<string>("edit_recovery_backup_id_for_path", { path });
  if (!backupId || backupId.length < 8) {
    throw new Error("edit recovery journal missing before abort");
  }
  await invoke("self_test_abort_for_recovery");
}

export async function runDocumentEditingRelaunchSelfTest(): Promise<void> {
  await ensureMarkdownVisualCard();
  const path = document.body.dataset.activeDocumentPath;
  if (!path) {
    throw new Error("active document path missing for relaunch");
  }
  const dto = await invoke<{
    recovery_bar?: { backup_id: string; path: string }[];
  }>("get_browser_state");
  const normalized = path.replace(/\\/g, "/");
  const row = (dto.recovery_bar ?? []).find(
    (item) => item.path.replace(/\\/g, "/") === normalized,
  );
  if (!row) {
    throw new Error("recovery bar missing pending edit for document");
  }
  const host = document.getElementById("recovery-bar-host");
  if (!host || host.classList.contains("hidden")) {
    throw new Error("recovery bar host not visible after relaunch");
  }
  await invoke("restore_recovery_bar_item", { backupId: row.backup_id });
  await sleep(500);
  const afterRecovery = await hostContent();
  if (!afterRecovery.includes(KILL_RECOVERY_MARKER)) {
    throw new Error("recovery restore did not keep unsaved paragraph");
  }
}

export function installDocumentEditingSelfTestHarness(): void {
  if (typeof window === "undefined") {
    return;
  }
  const win = window as unknown as {
    __unibrowserRunDocumentEditingSelfTest?: () => Promise<void>;
    __unibrowserRunDocumentEditingEverydayPhase1?: () => Promise<void>;
    __unibrowserRunDocumentEditingEverydayPhase2?: () => Promise<void>;
    __unibrowserRunDocumentEditingKillPrepSelfTest?: () => Promise<void>;
    __unibrowserRunDocumentEditingRelaunchSelfTest?: () => Promise<void>;
  };
  const runner = async (fn: () => Promise<void>) => {
    try {
      await fn();
      submitDocumentEditingSelfTestResult(null);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      submitDocumentEditingSelfTestResult(message);
    }
  };
  win.__unibrowserRunDocumentEditingEverydayPhase1 = () => runner(runDocumentEditingEverydayPhase1);
  win.__unibrowserRunDocumentEditingEverydayPhase2 = () => runner(runDocumentEditingEverydayPhase2);
  win.__unibrowserRunDocumentEditingSelfTest = () => runner(runDocumentEditingEverydaySelfTest);
  win.__unibrowserRunDocumentEditingKillPrepSelfTest = () =>
    runner(runDocumentEditingKillPrepSelfTest);
  win.__unibrowserRunDocumentEditingRelaunchSelfTest = () =>
    runner(runDocumentEditingRelaunchSelfTest);
}
