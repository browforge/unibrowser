import type { editor as MonacoEditorNS } from "monaco-editor";
import { t } from "./i18n";

type Monaco = typeof import("monaco-editor");
type DiffEditor = MonacoEditorNS.IStandaloneDiffEditor;

let monacoApi: Monaco | null = null;
let diffEditor: DiffEditor | null = null;
let hostEl: HTMLElement | null = null;
let sideBySide = true;

function isRecoveryPreviewPath(path: string): boolean {
  const lower = path.toLowerCase();
  return lower.includes("edit-recovery") || lower.includes("preview");
}

function diffSideLabel(path: string, side: "left" | "right"): string {
  if (side === "left" && !isRecoveryPreviewPath(path)) {
    return t("diff.sideSavedFile");
  }
  if (side === "right" && isRecoveryPreviewPath(path)) {
    return t("diff.sideRecoveredChanges");
  }
  const base = path.split(/[/\\]/).pop();
  return base && base.length > 0 ? base : path;
}

export function diffInsertLineCaptureRegion(
  lineNumber: number,
): { x: number; y: number; width: number; height: number } | null {
  if (!diffEditor || !hostEl) {
    return null;
  }
  const modified = diffEditor.getModifiedEditor();
  const domNode = modified.getDomNode();
  if (!domNode) {
    return null;
  }
  const model = modified.getModel();
  if (!model || lineNumber < 1 || lineNumber > model.getLineCount()) {
    return null;
  }
  const pos = modified.getScrolledVisiblePosition({
    lineNumber,
    column: 2,
  });
  if (!pos) {
    return null;
  }
  const editorRect = domNode.getBoundingClientRect();
  const x = editorRect.left + 48;
  const y = editorRect.top + pos.top;
  const width = Math.max(40, editorRect.width - 64);
  const height = Math.max(14, pos.height || 18);
  return {
    x: Math.round(x),
    y: Math.round(y),
    width: Math.round(width),
    height: Math.round(height),
  };
}

export async function waitForDiffHighlightPainted(timeoutMs = 8_000): Promise<boolean> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const host = hostEl ?? document.getElementById("diff-editor-host");
    const painted = Boolean(
      host?.querySelector(".monaco-diff-editor .line-insert, .monaco-diff-editor .char-insert"),
    );
    const changes = diffEditor?.getLineChanges()?.length ?? 0;
    if (painted && changes > 0) {
      return true;
    }
    diffEditor?.layout();
    await new Promise<void>((resolve) => {
      window.requestAnimationFrame(() => {
        window.requestAnimationFrame(() => resolve());
      });
    });
  }
  return Boolean(
    hostEl?.querySelector(".monaco-diff-editor .line-insert, .monaco-diff-editor .char-insert"),
  );
}

async function getMonaco(): Promise<Monaco> {
  if (!monacoApi) {
    const loader = (await import("@monaco-editor/loader")).default;
    const monaco = await import("monaco-editor");
    loader.config({ monaco });
    monacoApi = await loader.init();
  }
  return monacoApi as Monaco;
}

export interface DiffViewDto {
  left_path: string;
  right_path: string;
  left_content: string;
  right_content: string;
  language: string;
}

export const diffEditorView = {
  isActive(): boolean {
    return diffEditor !== null;
  },

  hide(host: HTMLElement) {
    host.classList.add("hidden");
    host.innerHTML = "";
    diffEditor?.dispose();
    diffEditor = null;
    hostEl = null;
  },

  async apply(view: DiffViewDto | null, host: HTMLElement) {
    if (!view) {
      this.hide(host);
      return;
    }
    const api = await getMonaco();
    host.classList.remove("hidden");
    if (hostEl !== host) {
      diffEditor?.dispose();
      host.innerHTML = "";
      hostEl = host;
      const toolbar = document.createElement("div");
      toolbar.className = "diff-toolbar";
      const title = document.createElement("span");
      title.className = "diff-title";
      toolbar.appendChild(title);
      const layoutToggle = document.createElement("button");
      layoutToggle.type = "button";
      layoutToggle.className = "diff-layout-toggle";
      layoutToggle.dataset.action = "toggle-diff-layout";
      toolbar.appendChild(layoutToggle);
      const close = document.createElement("button");
      close.type = "button";
      close.className = "diff-close";
      close.dataset.action = "close-diff";
      toolbar.appendChild(close);
      const body = document.createElement("div");
      body.className = "diff-monaco-host";
      host.appendChild(toolbar);
      host.appendChild(body);
      if (!document.getElementById("rec-diff-insert-style")) {
        const style = document.createElement("style");
        style.id = "rec-diff-insert-style";
        style.textContent = `
.monaco-diff-editor .line-insert,
.monaco-diff-editor .char-insert,
.monaco-diff-editor .insert-sign,
.monaco-diff-editor .view-line.line-insert {
  background-color: rgba(88, 124, 14, 0.38) !important;
}
.monaco-diff-editor .line-delete,
.monaco-diff-editor .char-delete,
.monaco-diff-editor .delete-sign,
.monaco-diff-editor .view-line.line-delete {
  background-color: rgba(161, 38, 13, 0.28) !important;
}
body[data-ub-doc-capture="1"] .monaco-diff-editor .line-insert,
body[data-ub-doc-capture="1"] .monaco-diff-editor .char-insert,
body[data-ub-doc-capture="1"] .monaco-diff-editor .view-line.line-insert {
  background-color: #d9e8b8 !important;
}
body[data-ub-doc-capture="1"] .monaco-diff-editor .line-delete,
body[data-ub-doc-capture="1"] .monaco-diff-editor .char-delete,
body[data-ub-doc-capture="1"] .monaco-diff-editor .view-line.line-delete {
  background-color: #f0d2cc !important;
}
`;
        document.head.appendChild(style);
      }
      const theme =
        document.documentElement.dataset.theme === "light" ? "vs" : "vs-dark";
      const diffThemeId = "unibrowser-rec-diff";
      api.editor.defineTheme(diffThemeId, {
        base: theme,
        inherit: true,
        rules: [],
        colors: {
          "diffEditor.insertedTextBackground": "#587c0e33",
          "diffEditor.insertedLineBackground": "#587c0e40",
          "diffEditor.removedTextBackground": "#a1260d33",
          "diffEditor.removedLineBackground": "#a1260d40",
          "editor.background": theme === "vs" ? "#faf9f6" : "#1e1e24",
        },
      });
      diffEditor = api.editor.createDiffEditor(body, {
        theme: diffThemeId,
        automaticLayout: true,
        readOnly: true,
        renderSideBySide: true,
        renderOverviewRuler: false,
        renderIndicators: true,
        renderMarginRevertIcon: false,
        ignoreTrimWhitespace: false,
        minimap: { enabled: false },
        scrollBeyondLastLine: false,
        overviewRulerLanes: 0,
        hideUnchangedRegions: { enabled: false },
        diffAlgorithm: "advanced",
        useInlineViewWhenSpaceIsLimited: false,
        experimentalGpuAcceleration: "off",
      });
    }
    const titleEl = host.querySelector(".diff-title");
    const closeBtn = host.querySelector(".diff-close") as HTMLButtonElement | null;
    const layoutBtn = host.querySelector(
      ".diff-layout-toggle",
    ) as HTMLButtonElement | null;
    if (titleEl) {
      titleEl.textContent = `${diffSideLabel(view.left_path, "left")} ↔ ${diffSideLabel(view.right_path, "right")}`;
    }
    if (closeBtn) {
      closeBtn.textContent = t("diff.close");
    }
    if (layoutBtn) {
      layoutBtn.textContent = sideBySide ? t("diff.unified") : t("diff.sideBySide");
    }
    diffEditor!.updateOptions({ renderSideBySide: sideBySide });
    const left = api.editor.createModel(view.left_content, view.language);
    const right = api.editor.createModel(view.right_content, view.language);
    const leftLines = view.left_content.replace(/\r\n/g, "\n").split("\n");
    const rightLines = view.right_content.replace(/\r\n/g, "\n").split("\n");
    let changedRows = 0;
    for (let i = 0; i < Math.max(leftLines.length, rightLines.length); i += 1) {
      if ((leftLines[i] ?? "") !== (rightLines[i] ?? "")) {
        changedRows += 1;
      }
    }
    diffEditor!.setModel({ original: left, modified: right });
    diffEditor!.layout();
    await waitForDiffHighlightPainted();
    const monacoChanges = diffEditor!.getLineChanges()?.length ?? 0;
    host.dataset.diffChangeCount = String(Math.max(changedRows, monacoChanges));
    host.dataset.diffHighlightPainted = host.querySelector(
      ".monaco-diff-editor .line-insert, .monaco-diff-editor .char-insert",
    )
      ? "1"
      : "0";
    window.requestAnimationFrame(() => {
      diffEditor?.layout();
      window.requestAnimationFrame(() => diffEditor?.layout());
    });
  },

  toggleLayout(host: HTMLElement) {
    sideBySide = !sideBySide;
    diffEditor?.updateOptions({ renderSideBySide: sideBySide });
    const layoutBtn = host.querySelector(
      ".diff-layout-toggle",
    ) as HTMLButtonElement | null;
    if (layoutBtn) {
      layoutBtn.textContent = sideBySide ? t("diff.unified") : t("diff.sideBySide");
    }
  },
};
