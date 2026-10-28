import { invoke } from "@tauri-apps/api/core";

export type ProofingDiagnostic = {
  start: number;
  end: number;
  kind: string;
  message: string;
  suggestions: string[];
};

const MARKER_OWNER = "unibrowser-proofing";
const SPELL_INLINE_CLASS = "ub-proofing-spell-squiggle";
const GRAMMAR_INLINE_CLASS = "ub-proofing-grammar-squiggle";

let squigglesEnabled = true;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let lastDiagnostics: ProofingDiagnostic[] = [];
let problemIndex = 0;
let inlineDecorationIds: string[] = [];
let decorationsPainted = false;
let activeProofingEditor: import("monaco-editor").editor.IStandaloneCodeEditor | null =
  null;

export async function loadProofingPrefs(): Promise<void> {
  try {
    const prefs = await invoke<{
      proofing_squiggles_enabled: boolean;
    }>("get_preferences");
    squigglesEnabled = prefs.proofing_squiggles_enabled;
  } catch {
    squigglesEnabled = true;
  }
}

export function isProofingEnabled(): boolean {
  return squigglesEnabled;
}

export function proofingHasSpellingDiagnostic(): boolean {
  return lastDiagnostics.some((d) => d.kind === "spelling");
}

export function proofingDecorationsArePainted(): boolean {
  if (!decorationsPainted) {
    return false;
  }
  const host = document.getElementById("text-editor-host");
  if (!host) {
    return false;
  }
  return Boolean(
    host.querySelector(
      `.${SPELL_INLINE_CLASS}, .${GRAMMAR_INLINE_CLASS}, .squiggly-error, .squiggly-warning`,
    ),
  );
}

export async function waitForProofingDecorationsPainted(
  editor: import("monaco-editor").editor.IStandaloneCodeEditor,
  surface: "plaintext" | "markdown" | "code",
  codeLanguage?: string,
  timeoutMs = 8_000,
): Promise<boolean> {
  const monaco = (await import("monaco-editor")) as typeof import("monaco-editor");
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (proofingHasSpellingDiagnostic() && proofingDecorationsArePainted()) {
      return true;
    }
    await runEditorProofing(monaco, editor, surface, codeLanguage);
    await new Promise<void>((resolve) => {
      window.requestAnimationFrame(() => {
        window.requestAnimationFrame(() => resolve());
      });
    });
  }
  return proofingHasSpellingDiagnostic() && proofingDecorationsArePainted();
}

export function getProofingWordCaptureRegion(
  editor: import("monaco-editor").editor.IStandaloneCodeEditor,
  needle: string,
): { x: number; y: number; width: number; height: number } | null {
  const model = editor.getModel();
  if (!model) {
    return null;
  }
  const hit = lastDiagnostics.find((d) => {
    const slice = model.getValue().slice(d.start, d.end);
    return slice.toLowerCase() === needle.toLowerCase();
  });
  if (!hit) {
    return null;
  }
  const start = model.getPositionAt(hit.start);
  const end = model.getPositionAt(hit.end);
  const domNode = editor.getDomNode();
  if (!domNode) {
    return null;
  }
  const startPx = editor.getScrolledVisiblePosition(start);
  const endPx = editor.getScrolledVisiblePosition(end);
  if (!startPx || !endPx) {
    return null;
  }
  const editorRect = domNode.getBoundingClientRect();
  const x = editorRect.left + Math.min(startPx.left, endPx.left);
  const y = editorRect.top + startPx.top;
  const width = Math.max(8, Math.abs(endPx.left - startPx.left) + 10);
  const height = Math.max(14, startPx.height || 18);
  return {
    x: Math.round(x),
    y: Math.round(y),
    width: Math.round(width),
    height: Math.round(height),
  };
}

export function getProofingSpellingCount(): number {
  return lastDiagnostics.filter((d) => d.kind === "spelling").length;
}

export async function setProofingSquigglesEnabled(enabled: boolean): Promise<void> {
  await invoke("set_proofing_squiggles_enabled", { enabled });
  squigglesEnabled = enabled;
  window.dispatchEvent(new CustomEvent("unibrowser-proofing-pref"));
}

export async function toggleProofingSquiggles(): Promise<void> {
  await setProofingSquigglesEnabled(!squigglesEnabled);
}

export function scheduleEditorProofing(
  monaco: typeof import("monaco-editor"),
  editor: import("monaco-editor").editor.IStandaloneCodeEditor,
  surface: "plaintext" | "markdown" | "code",
  codeLanguage?: string,
): void {
  if (!squigglesEnabled) {
    clearEditorMarkers(monaco, editor);
    return;
  }
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  debounceTimer = setTimeout(() => {
    void runEditorProofing(monaco, editor, surface, codeLanguage);
  }, 400);
  const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
  if (demo.includes("rec-capture")) {
    void runEditorProofing(monaco, editor, surface, codeLanguage);
  }
}

export function clearEditorMarkers(
  monaco: typeof import("monaco-editor"),
  editor: import("monaco-editor").editor.IStandaloneCodeEditor,
): void {
  const model = editor.getModel();
  if (!model) {
    return;
  }
  monaco.editor.setModelMarkers(model, MARKER_OWNER, []);
  inlineDecorationIds = editor.deltaDecorations(inlineDecorationIds, []);
  lastDiagnostics = [];
  problemIndex = 0;
  decorationsPainted = false;
  activeProofingEditor = null;
}

function applyInlineProofingDecorations(
  monaco: typeof import("monaco-editor"),
  editor: import("monaco-editor").editor.IStandaloneCodeEditor,
  diags: ProofingDiagnostic[],
): void {
  const model = editor.getModel();
  if (!model) {
    inlineDecorationIds = editor.deltaDecorations(inlineDecorationIds, []);
    decorationsPainted = false;
    return;
  }
  const decorations = diags.map((d) => {
    const start = model.getPositionAt(d.start);
    const end = model.getPositionAt(d.end);
    return {
      range: new monaco.Range(
        start.lineNumber,
        start.column,
        end.lineNumber,
        end.column,
      ),
      options: {
        inlineClassName:
          d.kind === "grammar" ? GRAMMAR_INLINE_CLASS : SPELL_INLINE_CLASS,
        stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
      },
    };
  });
  inlineDecorationIds = editor.deltaDecorations(inlineDecorationIds, decorations);
  activeProofingEditor = editor;
  decorationsPainted = decorations.length > 0;
  window.dispatchEvent(new CustomEvent("unibrowser-proofing-decorations-applied"));
}

export async function runEditorProofing(
  monaco: typeof import("monaco-editor"),
  editor: import("monaco-editor").editor.IStandaloneCodeEditor,
  surface: "plaintext" | "markdown" | "code",
  codeLanguage?: string,
): Promise<void> {
  const model = editor.getModel();
  if (!model) {
    return;
  }
  const text = model.getValue();
  try {
    const diags = await invoke<ProofingDiagnostic[]>("proofing_check", {
      req: {
        text,
        surface,
        language: "en-NZ",
        ...(surface === "code"
          ? { code_language: codeLanguage ?? model.getLanguageId() }
          : {}),
        grammar_enabled: null,
      },
    });
    lastDiagnostics = diags;
    void import("./problemsPanel").then((m) => m.syncProblemsStatusBar());
    const markers = diags.map((d) => {
      const start = model.getPositionAt(d.start);
      const end = model.getPositionAt(d.end);
      return {
        severity:
          d.kind === "grammar"
            ? monaco.MarkerSeverity.Warning
            : monaco.MarkerSeverity.Error,
        message: d.message,
        startLineNumber: start.lineNumber,
        startColumn: start.column,
        endLineNumber: end.lineNumber,
        endColumn: end.column,
      };
    });
    monaco.editor.setModelMarkers(model, MARKER_OWNER, markers);
    applyInlineProofingDecorations(monaco, editor, diags);
  } catch {
    clearEditorMarkers(monaco, editor);
  }
}

export function focusNextProofingProblem(
  editor: import("monaco-editor").editor.IStandaloneCodeEditor,
): void {
  if (!lastDiagnostics.length) {
    return;
  }
  problemIndex = (problemIndex + 1) % lastDiagnostics.length;
  focusProblem(editor, problemIndex);
}

export function focusPreviousProofingProblem(
  editor: import("monaco-editor").editor.IStandaloneCodeEditor,
): void {
  if (!lastDiagnostics.length) {
    return;
  }
  problemIndex = (problemIndex + lastDiagnostics.length - 1) % lastDiagnostics.length;
  focusProblem(editor, problemIndex);
}

function focusProblem(
  editor: import("monaco-editor").editor.IStandaloneCodeEditor,
  index: number,
): void {
  const d = lastDiagnostics[index];
  const model = editor.getModel();
  if (!model) {
    return;
  }
  const start = model.getPositionAt(d.start);
  const end = model.getPositionAt(d.end);
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

export const PROOFING_TOGGLE = "unibrowser.proofing.toggle";
export const PROOFING_NEXT = "unibrowser.proofing.nextProblem";
export const PROOFING_PREVIOUS = "unibrowser.proofing.previousProblem";
