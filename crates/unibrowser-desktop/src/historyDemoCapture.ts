import { invoke } from "@tauri-apps/api/core";
import { applyDocumentHistoryHead } from "./historyCommands";

export const HISTORY_CAPTURE_BASELINE_MD = `# Demo markdown

Unibrowser renders Markdown in the document viewer.

> Render 1 block quotes use a pale blue panel and a left accent bar on the paper card.

![Demo pack image](sample.png)

- **Bold** and _italic_
- [Architecture](../../docs/architecture.md) (link may not resolve in preview; open from repo)

## Next steps

Open \`sample.unibrow\` for the native project format.
`;

export const HISTORY_CAPTURE_EDITED_MD = `# Edited heading

Unibrowser renders Markdown in the document viewer.

Edit three

> Render 1 block quotes use a pale blue panel and a left accent bar on the paper card.

![Demo pack image](sample.png)

- **Bold** and _italic_
- [Architecture](../../docs/architecture.md) (link may not resolve in preview; open from repo)

## Next steps

Open \`sample.unibrow\` for the native project format.
`;

export function expandBottomPanelForHistoryCapture() {
  const panel = document.getElementById("script-output-panel");
  if (!panel) {
    return;
  }
  const height = `${Math.round(window.innerHeight * 0.35)}px`;
  panel.style.height = height;
  panel.style.setProperty("--ub-bottom-panel-height", height);
  panel.classList.remove("hidden");
}

export async function waitForRichDocText(match: (text: string) => boolean): Promise<void> {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    const text = document.querySelector(".rich-text-doc")?.textContent ?? "";
    if (match(text)) {
      return;
    }
    await new Promise((resolve) => window.setTimeout(resolve, 50));
  }
  throw new Error("rich document text did not match capture predicate");
}

export function resolveHistoryDocumentPath(fallback?: string): string {
  return (
    document.body.dataset.activeDocumentPath?.trim() ||
    fallback?.trim() ||
    "examples/demo-pack/sample.md"
  );
}

/** Baseline checkpoint seq (always 1 after rust seed). */
export async function seedHistoryCaptureDemo(docPath?: string): Promise<number> {
  const path = resolveHistoryDocumentPath(docPath);
  document.body.dataset.activeDocumentPath = path;
  const { bindDocumentPath } = await import("./documentEditorModes");
  await bindDocumentPath(path);
  const baselineSeq = await invoke<number>("document_history_seed_capture_demo", { path });
  await applyDocumentHistoryHead(path);
  document.body.dataset.documentCardDirty = "0";
  return baselineSeq;
}
