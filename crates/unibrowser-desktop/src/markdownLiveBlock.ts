import { type Node as PmNode, type Schema, type MarkType } from "prosemirror-model";
import { Plugin, PluginKey, TextSelection, type EditorState, type Transaction } from "prosemirror-state";
import { Decoration, DecorationSet, type EditorView } from "prosemirror-view";
import { richTextSchema } from "./richTextProseMirror.ts";

export const markdownLiveBlockKey = new PluginKey("markdownLiveBlock");

export type LiveBlockTarget =
  | { kind: "heading"; level: number; pos: number; node: PmNode }
  | { kind: "paragraph"; pos: number; node: PmNode }
  | { kind: "blockquote"; pos: number; node: PmNode }
  | { kind: "list_item"; pos: number; node: PmNode; ordered: boolean; index: number }
  | { kind: "code_block"; pos: number; node: PmNode; language: string | null };

export function liveBlockTargetAtSelection(doc: PmNode, anchor: number): LiveBlockTarget | null {
  const $pos = doc.resolve(anchor);
  for (let depth = $pos.depth; depth > 0; depth -= 1) {
    const node = $pos.node(depth);
    const name = node.type.name;
    if (name === "list_item") {
      const parent = $pos.node(depth - 1);
      const ordered = parent.type.name === "ordered_list";
      return {
        kind: "list_item",
        pos: $pos.before(depth),
        node,
        ordered,
        index: $pos.index(depth - 1),
      };
    }
    if (name === "blockquote") {
      const inner = node.firstChild;
      if (!inner || inner.textContent.length === 0) {
        return null;
      }
      return { kind: "blockquote", pos: $pos.before(depth), node };
    }
    if (name === "heading") {
      if (node.textContent.length === 0) {
        return null;
      }
      return { kind: "heading", level: Number(node.attrs.level ?? 1), pos: $pos.before(depth), node };
    }
    if (name === "code_block") {
      return {
        kind: "code_block",
        pos: $pos.before(depth),
        node,
        language: (node.attrs.language as string | null) ?? null,
      };
    }
    if (name === "paragraph") {
      const parentName = $pos.node(depth - 1).type.name;
      if (parentName === "blockquote" || parentName === "list_item") {
        continue;
      }
      if (node.textContent.length === 0) {
        return null;
      }
      return { kind: "paragraph", pos: $pos.before(depth), node };
    }
  }
  return null;
}

function markerWidget(label: string): HTMLElement {
  const span = document.createElement("span");
  span.className = "md-live-marker";
  span.textContent = label;
  span.setAttribute("aria-hidden", "true");
  span.setAttribute("contenteditable", "false");
  return span;
}

function inlineMarkDecorations(
  block: PmNode,
  blockPos: number,
  markFilter: (name: string) => boolean,
): Decoration[] {
  const decos: Decoration[] = [];
  block.forEach((child, offset) => {
    if (!child.isText) {
      return;
    }
    for (const mark of child.marks) {
      if (!markFilter(mark.type.name)) {
        continue;
      }
      const open = markOpen(mark.type.name);
      const close = markClose(mark.type.name);
      const from = blockPos + 1 + offset;
      const to = from + (child.text?.length ?? 0);
      decos.push(
        Decoration.widget(from, () => markerWidget(open), { side: -1, key: `open-${from}-${mark.type.name}` }),
      );
      decos.push(
        Decoration.widget(to, () => markerWidget(close), { side: 1, key: `close-${to}-${mark.type.name}` }),
      );
    }
  });
  return decos;
}

function markOpen(name: string): string {
  if (name === "bold") return "**";
  if (name === "italic") return "*";
  if (name === "code") return "`";
  return "";
}

function markClose(name: string): string {
  return markOpen(name);
}

export function buildLiveBlockDecorations(
  doc: PmNode,
  selection: { from: number; to: number },
  enabled: boolean,
): DecorationSet {
  if (!enabled) {
    return DecorationSet.empty;
  }
  const target = liveBlockTargetAtSelection(doc, selection.from);
  if (!target) {
    return DecorationSet.empty;
  }
  const decos: Decoration[] = [];
  const blockPos = target.pos;
  switch (target.kind) {
    case "heading": {
      const prefix = `${"#".repeat(Math.min(6, Math.max(1, target.level)))} `;
      decos.push(
        Decoration.widget(
          blockPos + 1,
          () => markerWidget(prefix),
          { side: -1, key: `h-${blockPos}` },
        ),
      );
      decos.push(...inlineMarkDecorations(target.node, blockPos, () => true));
      break;
    }
    case "paragraph":
      decos.push(...inlineMarkDecorations(target.node, blockPos, () => true));
      break;
    case "blockquote": {
      const inner = target.node.firstChild;
      if (inner) {
        decos.push(
          Decoration.widget(
            blockPos + 1,
            () => markerWidget("> "),
            { side: -1, key: `bq-${blockPos}` },
          ),
        );
        decos.push(...inlineMarkDecorations(inner, blockPos + 1, () => true));
      }
      break;
    }
    case "list_item": {
      const para = target.node.firstChild;
      const marker = target.ordered ? `${target.index + 1}. ` : "- ";
      if (para) {
        decos.push(
          Decoration.widget(
            blockPos + 1,
            () => markerWidget(marker),
            { side: -1, key: `li-${blockPos}` },
          ),
        );
        decos.push(...inlineMarkDecorations(para, blockPos + 1, () => true));
      }
      break;
    }
    case "code_block": {
      const lang = target.language?.trim() ?? "";
      const open = lang ? `\`\`\`${lang}\n` : "```\n";
      decos.push(
        Decoration.widget(
          blockPos + 1,
          () => markerWidget(open),
          { side: -1, key: `fence-open-${blockPos}` },
        ),
      );
      const end = blockPos + target.node.nodeSize - 1;
      decos.push(
        Decoration.widget(end, () => markerWidget("```"), { side: 1, key: `fence-close-${blockPos}` }),
      );
      break;
    }
    default:
      break;
  }
  return decos.length ? DecorationSet.create(doc, decos) : DecorationSet.empty;
}

function headingInputMatch(text: string): { level: number; body: string } | null {
  const m = /^(#{1,6}) (.*)$/.exec(text);
  if (!m) {
    return null;
  }
  return { level: m[1]!.length, body: m[2] ?? "" };
}

function applyBlockInputRules(tr: Transaction): Transaction | null {
  let changed = false;
  tr.doc.descendants((node, pos) => {
    if (changed) {
      return false;
    }
    if (node.type.name !== "paragraph") {
      return true;
    }
    const parsed = headingInputMatch(node.textContent);
    if (parsed !== null) {
      tr.setBlockType(pos, pos + node.nodeSize, richTextSchema.nodes.heading!, {
        level: parsed.level,
        blockId: node.attrs.blockId ?? "",
      });
      const from = pos + 1;
      const to = from + node.textContent.length;
      if (parsed.body) {
        tr.replaceWith(from, to, richTextSchema.text(parsed.body));
      } else {
        tr.delete(from, to);
      }
      changed = true;
      return false;
    }
    return true;
  });
  return changed ? tr : null;
}

function applyBoldInputRule(tr: Transaction): Transaction | null {
  const bold: MarkType = richTextSchema.marks.bold!;
  let changed = false;
  tr.doc.descendants((node, pos) => {
    if (changed || !node.isTextblock) {
      return !changed;
    }
    const text = node.textContent;
    const re = /\*\*([^*]+)\*\*$/;
    const m = re.exec(text);
    if (!m) {
      return true;
    }
    const inner = m[1]!;
    const start = pos + 1 + text.length - m[0]!.length;
    const end = pos + 1 + text.length;
    tr.replaceWith(start, end, richTextSchema.text(inner, [bold.create()]));
    tr.setSelection(TextSelection.create(tr.doc, start + inner.length));
    changed = true;
    return false;
  });
  return changed ? tr : null;
}

export function markdownLiveBlockPlugins(enabled: () => boolean): Plugin[] {
  const decoPlugin = new Plugin({
    key: markdownLiveBlockKey,
    state: {
      init(_, state) {
        return buildLiveBlockDecorations(state.doc, state.selection, enabled());
      },
      apply(tr, old, _oldState, newState) {
        if (tr.docChanged || tr.selectionSet) {
          return buildLiveBlockDecorations(newState.doc, newState.selection, enabled());
        }
        return old.map(tr.mapping, tr.doc);
      },
    },
    props: {
      decorations(state) {
        return markdownLiveBlockKey.getState(state);
      },
    },
  });

  const inputPlugin = new Plugin({
    appendTransaction(transactions, _oldState, newState) {
      if (!enabled() || !transactions.some((tr) => tr.docChanged)) {
        return null;
      }
      if (transactions.some((tr) => tr.getMeta("history$"))) {
        return null;
      }
      let tr = newState.tr;
      let changed = false;
      const headingTr = applyBlockInputRules(tr);
      if (headingTr) {
        tr = headingTr;
        changed = true;
      }
      const boldTr = applyBoldInputRule(tr);
      if (boldTr) {
        return boldTr;
      }
      return changed ? tr : null;
    },
  });

  return [decoPlugin, inputPlugin];
}

/** Apply block/input rules outside appendTransaction (tests and capture helpers). */
export function applyLiveInputRules(state: EditorState): EditorState {
  let tr = state.tr;
  const headingTr = applyBlockInputRules(tr);
  if (headingTr) {
    tr = headingTr;
  }
  const boldTr = applyBoldInputRule(tr);
  if (boldTr) {
    return state.apply(boldTr);
  }
  if (headingTr) {
    return state.apply(headingTr);
  }
  return state;
}

export function findDocTextPos(doc: PmNode, needle: string): number | null {
  let found: number | null = null;
  doc.descendants((node, pos) => {
    if (found !== null || !node.isTextblock) {
      return found === null;
    }
    const idx = node.textContent.indexOf(needle);
    if (idx >= 0) {
      found = pos + 1 + idx;
      return false;
    }
    return true;
  });
  return found;
}

export function findEmptyTrailingParagraphPos(doc: PmNode): number | null {
  let lastEmpty: number | null = null;
  doc.forEach((node, offset, index) => {
    if (index === doc.childCount - 1 && node.type.name === "paragraph" && node.textContent.length === 0) {
      lastEmpty = offset + 1;
    }
  });
  return lastEmpty;
}

export type LiveCaptureCursor = "outside" | "heading" | "bold";

export function selectionForLiveCapture(doc: PmNode, kind: LiveCaptureCursor): number | null {
  if (kind === "outside") {
    const trailing = findEmptyTrailingParagraphPos(doc);
    if (trailing !== null) {
      return trailing;
    }
    return doc.content.size - 1;
  }
  if (kind === "heading") {
    const pos = findDocTextPos(doc, "Active heading sample");
    return pos !== null ? pos + 3 : null;
  }
  const bold = findDocTextPos(doc, "bold");
  return bold !== null ? bold + 2 : null;
}

export function applyLiveCaptureSelection(view: EditorView, kind: LiveCaptureCursor): boolean {
  if (kind === "outside") {
    const doc = view.state.doc;
    const last = doc.lastChild;
    if (!last || last.type.name !== "paragraph" || last.textContent.length > 0) {
      const para = richTextSchema.nodes.paragraph!.create({ blockId: "" });
      view.dispatch(view.state.tr.insert(doc.content.size, para));
    }
  }
  const pos = selectionForLiveCapture(view.state.doc, kind);
  if (pos === null || pos < 0 || pos > view.state.doc.content.size) {
    return false;
  }
  const tr = view.state.tr.setSelection(TextSelection.create(view.state.doc, pos));
  view.dispatch(tr);
  view.focus();
  document.body.dataset.liveCaptureReady = "1";
  return true;
}

export function createMinimalLiveDoc(schema: Schema = richTextSchema): PmNode {
  const bold = schema.marks.bold!.create();
  return schema.node("doc", null, [
    schema.node("heading", { level: 2, blockId: "h" }, [schema.text("Title")]),
    schema.node("paragraph", { blockId: "p" }, [
      schema.text("Say "),
      schema.text("bold", [bold]),
      schema.text(" here"),
    ]),
    schema.node("paragraph", { blockId: "empty" }),
  ]);
}
