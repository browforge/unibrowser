import type { editor as MonacoEditorNS } from "monaco-editor";
import { t } from "./i18n";

type Monaco = typeof import("monaco-editor");
type DiffEditor = MonacoEditorNS.IStandaloneDiffEditor;

let monacoApi: Monaco | null = null;
let diffEditor: DiffEditor | null = null;
let hostEl: HTMLElement | null = null;
let sideBySide = true;

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
.monaco-diff-editor .insert-sign {
  background-color: rgba(88, 124, 14, 0.28) !important;
}
.monaco-diff-editor .line-delete,
.monaco-diff-editor .char-delete,
.monaco-diff-editor .delete-sign {
  background-color: rgba(161, 38, 13, 0.22) !important;
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
      });
    }
    const titleEl = host.querySelector(".diff-title");
    const closeBtn = host.querySelector(".diff-close") as HTMLButtonElement | null;
    const layoutBtn = host.querySelector(
      ".diff-layout-toggle",
    ) as HTMLButtonElement | null;
    if (titleEl) {
      titleEl.textContent = `${view.left_path} ↔ ${view.right_path}`;
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
    await new Promise((resolve) => window.setTimeout(resolve, 250));
    const monacoChanges = diffEditor!.getLineChanges()?.length ?? 0;
    host.dataset.diffChangeCount = String(Math.max(changedRows, monacoChanges));
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
