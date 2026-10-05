import type { editor as MonacoEditorNS } from "monaco-editor";

type Monaco = typeof import("monaco-editor");
type DiffEditor = MonacoEditorNS.IStandaloneDiffEditor;

let monacoApi: Monaco | null = null;
let diffEditor: DiffEditor | null = null;
let hostEl: HTMLElement | null = null;

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
      const close = document.createElement("button");
      close.type = "button";
      close.className = "diff-close";
      close.dataset.action = "close-diff";
      toolbar.appendChild(close);
      const body = document.createElement("div");
      body.className = "diff-monaco-host";
      host.appendChild(toolbar);
      host.appendChild(body);
      const theme =
        document.documentElement.dataset.theme === "light" ? "vs" : "vs-dark";
      diffEditor = api.editor.createDiffEditor(body, {
        theme,
        automaticLayout: true,
        readOnly: true,
        renderSideBySide: true,
      });
    }
    const titleEl = host.querySelector(".diff-title");
    const closeBtn = host.querySelector(".diff-close") as HTMLButtonElement | null;
    if (titleEl) {
      titleEl.textContent = `${view.left_path} ↔ ${view.right_path}`;
    }
    if (closeBtn && !closeBtn.textContent) {
      const { t } = await import("./i18n");
      closeBtn.textContent = t("diff.close");
    }
    const left = api.editor.createModel(view.left_content, view.language);
    const right = api.editor.createModel(view.right_content, view.language);
    diffEditor!.setModel({ original: left, modified: right });
  },
};
