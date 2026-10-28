import { invoke } from "@tauri-apps/api/core";

export type ProofingDiagnostic = {
  start: number;
  end: number;
  kind: string;
  message: string;
  suggestions: string[];
};

const MARKER_OWNER = "unibrowser-proofing";

let squigglesEnabled = true;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let lastDiagnostics: ProofingDiagnostic[] = [];
let problemIndex = 0;

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
  lastDiagnostics = [];
  problemIndex = 0;
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
        code_language: codeLanguage ?? model.getLanguageId(),
        grammar_enabled: null,
      },
    });
    lastDiagnostics = diags;
    void import("./problemsPanel").then((m) => m.refreshProblemsPanel());
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
