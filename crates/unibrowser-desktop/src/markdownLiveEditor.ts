type MonacoApi = typeof import("monaco-editor");
import {
  applySmartEnter,
  activeLiveBlockIndex,
  buildBlockRenderMap,
  splitLiveBlocks,
  sourceOffsetFromRenderClick,
  type LiveBlock,
} from "./markdownLiveMapping";
import { renderLiveBlockHtml } from "./markdownLiveRender";

type Editor = import("monaco-editor").editor.IStandaloneCodeEditor;
type Model = import("monaco-editor").editor.ITextModel;

let attached = false;
let composing = false;
let decorationIds: string[] = [];
let zoneIds: string[] = [];
let refreshTimer: ReturnType<typeof setTimeout> | null = null;
let monacoRef: MonacoApi | null = null;
let editorRef: Editor | null = null;
let disposables: import("monaco-editor").IDisposable[] = [];

function offsetToPosition(model: Model, offset: number): import("monaco-editor").Position {
  return model.getPositionAt(offset);
}

function positionToOffset(model: Model, pos: import("monaco-editor").Position): number {
  return model.getOffsetAt(pos);
}

function scheduleRefresh() {
  if (composing) {
    return;
  }
  if (refreshTimer) {
    clearTimeout(refreshTimer);
  }
  refreshTimer = setTimeout(() => {
    refreshTimer = null;
    refreshLiveSurface();
  }, 16);
}

function clearZones(editor: Editor) {
  editor.changeViewZones((accessor) => {
    for (const id of zoneIds) {
      accessor.removeZone(id);
    }
  });
  zoneIds = [];
}

function refreshLiveSurface() {
  const editor = editorRef;
  const monaco = monacoRef;
  if (!editor || !monaco || !attached) {
    return;
  }
  const model = editor.getModel();
  if (!model) {
    return;
  }
  const source = model.getValue();
  const blocks = splitLiveBlocks(source);
  const cursor = editor.getPosition();
  const cursorOffset = cursor ? positionToOffset(model, cursor) : 0;
  const activeIndex = activeLiveBlockIndex(blocks, cursorOffset);

  const decorations: import("monaco-editor").editor.IModelDeltaDecoration[] = [];
  for (let i = 0; i < blocks.length; i += 1) {
    const block = blocks[i]!;
    const startPos = offsetToPosition(model, block.start);
    const endPos = offsetToPosition(model, Math.max(block.start, block.end - 1));
    const isActive = activeIndex === i;
    const allRendered = activeIndex < 0;
    if ((allRendered || !isActive) && block.kind !== "unsupported") {
      decorations.push({
        range: new monaco.Range(
          startPos.lineNumber,
          1,
          endPos.lineNumber,
          model.getLineMaxColumn(endPos.lineNumber),
        ),
        options: {
          isWholeLine: true,
          className: "md-live-line-hidden",
          stickiness: monaco.editor.TrackedRangeStickiness.AlwaysGrowsWhenTypingAtEdges,
        },
      });
    } else if (isActive) {
      const map = buildBlockRenderMap(block);
      if (map) {
        for (const marker of map.markers) {
          const mStart = offsetToPosition(model, marker.start);
          const mEnd = offsetToPosition(model, marker.end);
          decorations.push({
            range: new monaco.Range(
              mStart.lineNumber,
              mStart.column,
              mEnd.lineNumber,
              mEnd.column,
            ),
            options: {
              inlineClassName: "md-live-marker",
              stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
            },
          });
        }
      }
    }
  }
  decorationIds = editor.deltaDecorations(decorationIds, decorations);

  clearZones(editor);
  editor.changeViewZones((accessor) => {
    for (let i = 0; i < blocks.length; i += 1) {
      if (i === activeIndex) {
        continue;
      }
      const block = blocks[i]!;
      if (block.kind === "unsupported") {
        continue;
      }
      const html = renderLiveBlockHtml(block);
      if (!html) {
        continue;
      }
      const startPos = offsetToPosition(model, block.start);
      const endPos = offsetToPosition(model, Math.max(block.start, block.end - 1));
      const dom = document.createElement("div");
      dom.className = "md-live-render-zone";
      dom.innerHTML = html;
      dom.dataset.liveBlockIndex = String(i);
      dom.addEventListener("mousedown", (ev) => {
        ev.preventDefault();
        const textNode = findTextNodeAtPoint(dom, ev.clientX, ev.clientY);
        const renderOffset = textOffsetInElement(dom, textNode, ev.clientX);
        const sourceOffset = sourceOffsetFromRenderClick(block, renderOffset);
        const pos = offsetToPosition(model, sourceOffset);
        editor.focus();
        editor.setPosition(pos);
        scheduleRefresh();
      });
      const lineSpan = endPos.lineNumber - startPos.lineNumber + 1;
      const zoneId = accessor.addZone({
        afterLineNumber: Math.max(0, startPos.lineNumber - 1),
        heightInLines: Math.max(lineSpan, estimateLines(html)),
        domNode: dom,
        suppressMouseDown: false,
      });
      zoneIds.push(zoneId);
    }
  });
}

function estimateLines(html: string): number {
  const plain = html.replace(/<[^>]+>/g, " ");
  return Math.max(1, Math.ceil(plain.length / 72));
}

function findTextNodeAtPoint(root: HTMLElement, x: number, y: number): Text | null {
  const doc = root.ownerDocument;
  if (!doc) {
    return null;
  }
  const range = doc.caretRangeFromPoint?.(x, y);
  if (range?.startContainer.nodeType === Node.TEXT_NODE) {
    return range.startContainer as Text;
  }
  const walker = doc.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let last: Text | null = null;
  while (walker.nextNode()) {
    last = walker.currentNode as Text;
  }
  return last;
}

function textOffsetInElement(root: HTMLElement, prefer: Text | null, x: number): number {
  let offset = 0;
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  while (walker.nextNode()) {
    const text = walker.currentNode as Text;
    if (text === prefer) {
      for (let i = 0; i < text.length; i += 1) {
        const range = document.createRange();
        range.setStart(text, i);
        range.setEnd(text, i + 1);
        const rect = range.getBoundingClientRect();
        if (rect.left <= x && x <= rect.right) {
          return offset + i;
        }
      }
      return offset + text.length;
    }
    offset += text.length;
  }
  return offset;
}

function handleEnter(editor: Editor, shiftKey: boolean): boolean {
  const model = editor.getModel();
  if (!model) {
    return false;
  }
  const pos = editor.getPosition();
  if (!pos) {
    return false;
  }
  const offset = positionToOffset(model, pos);
  const result = applySmartEnter(model.getValue(), offset, shiftKey);
  if (result.text === model.getValue()) {
    return false;
  }
  editor.executeEdits("md-live-enter", [
    {
      range: model.getFullModelRange(),
      text: result.text,
      forceMoveMarkers: true,
    },
  ]);
  editor.setPosition(offsetToPosition(model, result.cursor));
  scheduleRefresh();
  return true;
}

export function attachMarkdownLiveEditor(monaco: MonacoApi, editor: Editor): void {
  detachMarkdownLiveEditor();
  attached = true;
  monacoRef = monaco;
  editorRef = editor;
  editor.getDomNode()?.closest("#text-editor-host")?.classList.add("document-mode-live-active");

  disposables.push(
    editor.onDidChangeModelContent(() => scheduleRefresh()),
    editor.onDidChangeCursorPosition(() => scheduleRefresh()),
    editor.onDidCompositionStart(() => {
      composing = true;
    }),
    editor.onDidCompositionEnd(() => {
      composing = false;
      scheduleRefresh();
    }),
    editor.onKeyDown((ev) => {
      if (ev.keyCode === monaco.KeyCode.Enter) {
        if (handleEnter(editor, ev.shiftKey)) {
          ev.preventDefault();
          ev.stopPropagation();
        }
      }
    }),
  );
  scheduleRefresh();
}

export function detachMarkdownLiveEditor(): void {
  attached = false;
  composing = false;
  if (refreshTimer) {
    clearTimeout(refreshTimer);
    refreshTimer = null;
  }
  const editor = editorRef;
  if (editor) {
    clearZones(editor);
    decorationIds = editor.deltaDecorations(decorationIds, []);
  }
  for (const d of disposables) {
    d.dispose();
  }
  disposables = [];
  document.querySelector("#text-editor-host")?.classList.remove("document-mode-live-active");
  editorRef = null;
  monacoRef = null;
}

export function isMarkdownLiveEditorAttached(): boolean {
  return attached;
}

export function liveCaptureProbe(
  blocks: LiveBlock[],
  activeIndex: number,
): { markerCount: number; hiddenLineBlocks: number } {
  let markerCount = 0;
  let hiddenLineBlocks = 0;
  for (let i = 0; i < blocks.length; i += 1) {
    if (i !== activeIndex && blocks[i]!.kind !== "unsupported") {
      hiddenLineBlocks += 1;
    }
    if (i === activeIndex) {
      markerCount = buildBlockRenderMap(blocks[i]!)?.markers.length ?? 0;
    }
  }
  return { markerCount, hiddenLineBlocks };
}
