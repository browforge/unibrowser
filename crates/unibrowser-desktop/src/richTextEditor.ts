import type { DocumentModelDto } from "./documentModelBridge";
import { mountProseMirrorEditor, type RichTextEditorHandle } from "./richTextProseMirror";

let activeHandle: RichTextEditorHandle | null = null;

export function renderDocumentModelToHtml(model: DocumentModelDto, readonly = false): string {
  const host = document.createElement("div");
  mountRichTextEditor(host, model, readonly);
  return host.innerHTML;
}

export function mountRichTextEditor(
  host: HTMLElement,
  model: DocumentModelDto,
  readonly: boolean,
  onModelChange?: (model: DocumentModelDto, touchedBlockIds: string[]) => void,
): HTMLElement {
  activeHandle?.destroy();
  activeHandle = mountProseMirrorEditor(host, model, readonly, (next, touched) => {
    onModelChange?.(next, touched);
  });
  return host.querySelector("article") ?? host;
}

export function destroyRichTextEditor() {
  activeHandle?.destroy();
  activeHandle = null;
}
