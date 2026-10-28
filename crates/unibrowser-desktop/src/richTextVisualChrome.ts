import type { Node as PmNode } from "prosemirror-model";
import type { EditorView } from "prosemirror-view";
import type { SourceMapDto } from "./documentModelBridge";
import { getDocumentModelSession } from "./documentEditorHost";
import { getDocumentEditorMode } from "./documentEditorModes";
import { getActiveTextContent } from "./textEditor";
import { refreshCompactBlockPickerLabel } from "./textCompactToolbar";
import { refreshDocumentCardStatusFooter } from "./ui/documentCardStatus";
import { blockStyleAtProseMirrorPos } from "./markdownLiveBlock";

function blockIdForTextblock(doc: PmNode, textblockPos: number): string {
  const $pos = doc.resolve(textblockPos + 1);
  let blockId = "";
  for (let depth = $pos.depth; depth > 0; depth -= 1) {
    const id = String($pos.node(depth).attrs.blockId ?? "").trim();
    if (id) {
      blockId = id;
    }
  }
  return blockId;
}

function blockIdAndTextOffset(doc: PmNode, pos: number): { blockId: string; textOffset: number } | null {
  const $pos = doc.resolve(Math.max(1, Math.min(pos, doc.content.size - 1)));
  let blockId = "";
  let textOffset = 0;
  for (let depth = $pos.depth; depth > 0; depth -= 1) {
    const node = $pos.node(depth);
    const id = String(node.attrs.blockId ?? "").trim();
    if (id) {
      blockId = id;
    }
    if (node.isTextblock) {
      textOffset = Math.max(0, pos - $pos.start(depth));
    }
  }
  if (!blockId) {
    for (let depth = $pos.depth; depth > 0; depth -= 1) {
      const node = $pos.node(depth);
      if (node.type.name === "paragraph" && node.textContent.length === 0) {
        return { blockId: "", textOffset: 0 };
      }
    }
    return null;
  }
  return { blockId, textOffset };
}

function mapPlainOffsetIntoBlockMarkdown(blockMarkdown: string, pmText: string, pmOffset: number): number {
  let pm = 0;
  let src = 0;
  const target = Math.min(pmOffset, pmText.length);
  while (pm < target && src < blockMarkdown.length) {
    if (blockMarkdown[src] === pmText[pm]) {
      pm += 1;
      src += 1;
      continue;
    }
    src += 1;
  }
  return src;
}

export function sourceLineColFromProseMirror(
  doc: PmNode,
  pos: number,
  markdown: string,
  sourceMap: SourceMapDto,
): { line: number; column: number } | null {
  const ctx = blockIdAndTextOffset(doc, pos);
  if (!ctx) {
    return null;
  }
  if (!ctx.blockId) {
    const lines = markdown.split("\n");
    const line = lines.length;
    const column = (lines[lines.length - 1]?.length ?? 0) + 1;
    return { line, column };
  }
  const entry = sourceMap.entries.find((e) => e.block_id === ctx.blockId);
  if (!entry) {
    return null;
  }
  const slice = markdown.slice(entry.byte_start, entry.byte_end);
  let pmText = "";
  doc.descendants((node, nodePos) => {
    if (pmText || !node.isTextblock) {
      return pmText === "";
    }
    if (blockIdForTextblock(doc, nodePos) !== ctx.blockId) {
      return true;
    }
    pmText = node.textContent;
    return false;
  });
  const inBlockSrc = mapPlainOffsetIntoBlockMarkdown(slice, pmText, ctx.textOffset);
  const absolute = entry.byte_start + inBlockSrc;
  const clamped = Math.max(0, Math.min(absolute, markdown.length));
  const before = markdown.slice(0, clamped);
  const lines = before.split("\n");
  const line = lines.length;
  const column = (lines[lines.length - 1]?.length ?? 0) + 1;
  return { line, column };
}

export function syncVisualChromeFromProseMirror(view: EditorView): void {
  const mode = getDocumentEditorMode();
  if (mode !== "wysiwyg" && mode !== "reading") {
    return;
  }
  const session = getDocumentModelSession();
  const markdown = getActiveTextContent();
  const pos = view.state.selection.from;
  if (session && markdown) {
    const mapped = sourceLineColFromProseMirror(
      view.state.doc,
      pos,
      markdown,
      session.source_map,
    );
    if (mapped) {
      const win = window as unknown as {
        __unibrowserPatchStatusCursor?: (label: string) => void;
      };
      win.__unibrowserPatchStatusCursor?.(`Ln ${mapped.line}, Col ${mapped.column}`);
      const setListener = (
        window as unknown as {
          __unibrowserSetEditorCursor?: (line: number, column: number) => void;
        }
      ).__unibrowserSetEditorCursor;
      setListener?.(mapped.line, mapped.column);
    }
  }
  const blockStyle = blockStyleAtProseMirrorPos(view.state.doc, pos);
  const blockLabel = document.getElementById("text-compact-toolbar-host")?.querySelector<HTMLElement>(
    ".text-compact-block-label",
  );
  if (blockLabel) {
    blockLabel.textContent = blockStyle.label;
  }
  refreshCompactBlockPickerLabel();
  refreshDocumentCardStatusFooter(Boolean(document.body.dataset.documentCardDirty === "1"));
}
