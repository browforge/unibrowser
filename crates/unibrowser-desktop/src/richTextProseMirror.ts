import { Schema, type Node as PmNode, type Mark, type NodeSpec, type MarkSpec } from "prosemirror-model";
import { EditorState, type Transaction } from "prosemirror-state";
import { EditorView } from "prosemirror-view";
import { keymap } from "prosemirror-keymap";
import { baseKeymap } from "prosemirror-commands";
import { history, undo, redo } from "prosemirror-history";
import type { DocumentModelDto } from "./documentModelBridge";

type BlockDto = {
  type: string;
  id?: string;
  level?: number;
  inlines?: { text: string; marks?: { kind: string; href?: string }[] }[];
  text?: string;
  language?: string | null;
  label?: string | null;
  raw?: string;
  items?: { inlines: { text: string; marks?: { kind: string }[] }[] }[];
  src?: string;
  alt?: string;
};

const nodes: Record<string, NodeSpec> = {
  doc: { content: "block+" },
  paragraph: {
    group: "block",
    content: "inline*",
    attrs: { blockId: { default: "" } },
    parseDOM: [{ tag: "p" }],
    toDOM() {
      return ["p", 0];
    },
  },
  heading: {
    group: "block",
    content: "inline*",
    attrs: { level: { default: 1 }, blockId: { default: "" } },
    parseDOM: [
      { tag: "h1", attrs: { level: 1 } },
      { tag: "h2", attrs: { level: 2 } },
      { tag: "h3", attrs: { level: 3 } },
      { tag: "h4", attrs: { level: 4 } },
      { tag: "h5", attrs: { level: 5 } },
      { tag: "h6", attrs: { level: 6 } },
    ],
    toDOM(node: PmNode): [string, number] {
      return [`h${node.attrs.level}`, 0] as [string, number];
    },
  },
  blockquote: {
    group: "block",
    content: "paragraph",
    attrs: { blockId: { default: "" } },
    parseDOM: [{ tag: "blockquote" }],
    toDOM() {
      return ["blockquote", 0];
    },
  },
  code_block: {
    group: "block",
    content: "text*",
    marks: "",
    code: true,
    attrs: { blockId: { default: "" }, language: { default: null } },
    parseDOM: [{ tag: "pre", preserveWhitespace: "full" }],
    toDOM() {
      return ["pre", ["code", 0]];
    },
  },
  bullet_list: {
    group: "block",
    content: "list_item+",
    attrs: { blockId: { default: "" } },
    parseDOM: [{ tag: "ul" }],
    toDOM() {
      return ["ul", 0];
    },
  },
  ordered_list: {
    group: "block",
    content: "list_item+",
    attrs: { blockId: { default: "" } },
    parseDOM: [{ tag: "ol" }],
    toDOM() {
      return ["ol", 0];
    },
  },
  list_item: {
    content: "paragraph",
    parseDOM: [{ tag: "li" }],
    toDOM() {
      return ["li", 0];
    },
  },
  horizontal_rule: {
    group: "block",
    attrs: { blockId: { default: "" } },
    parseDOM: [{ tag: "hr" }],
    toDOM() {
      return ["hr"];
    },
  },
  image: {
    group: "block",
    inline: false,
    attrs: { blockId: { default: "" }, src: { default: "" }, alt: { default: "" } },
    parseDOM: [{ tag: "img[src]", getAttrs(dom: HTMLElement) {
      return { src: dom.getAttribute("src"), alt: dom.getAttribute("alt") ?? "" };
    } }],
    toDOM(node: PmNode): [string, Record<string, string>] {
      return ["img", { src: node.attrs.src, alt: node.attrs.alt }] as [string, Record<string, string>];
    },
  },
  opaque: {
    group: "block",
    attrs: { blockId: { default: "" }, raw: { default: "" }, label: { default: "Raw Markdown (preserved)" } },
    parseDOM: [{ tag: "div.rich-text-opaque" }],
    toDOM(node: PmNode) {
      return [
        "div",
        { class: "rich-text-opaque", "data-block-id": node.attrs.blockId },
        ["span", { class: "rich-text-opaque-label" }, node.attrs.label],
        ["pre", node.attrs.raw],
        [
          "button",
          {
            type: "button",
            class: "rich-text-opaque-parse",
            "data-command": "parse-opaque",
          },
          "Parse as Markdown",
        ],
      ] as const;
    },
  },
  text: { group: "inline" },
};

const marks: Record<string, MarkSpec> = {
  bold: {
    parseDOM: [{ tag: "strong" }, { tag: "b" }],
    toDOM() {
      return ["strong", 0];
    },
  },
  italic: {
    parseDOM: [{ tag: "em" }, { tag: "i" }],
    toDOM() {
      return ["em", 0];
    },
  },
  code: {
    parseDOM: [{ tag: "code" }],
    toDOM() {
      return ["code", 0];
    },
  },
  link: {
    attrs: { href: { default: "" } },
    inclusive: false,
    parseDOM: [{ tag: "a[href]", getAttrs(dom: HTMLElement) {
      return { href: dom.getAttribute("href") };
    } }],
    toDOM(mark: Mark): [string, Record<string, string>, number] {
      return ["a", { href: mark.attrs.href }, 0] as [string, Record<string, string>, number];
    },
  },
};

export const richTextSchema = new Schema({ nodes, marks });

function inlinesFromPm(node: PmNode): { text: string; marks: { kind: string; href?: string }[] }[] {
  const spans: { text: string; marks: { kind: string; href?: string }[] }[] = [];
  node.forEach((child) => {
    if (!child.isText) {
      return;
    }
    const markKinds = child.marks.map((m) => {
      if (m.type.name === "link") {
        return { kind: "link", href: String(m.attrs.href ?? "") };
      }
      return { kind: m.type.name };
    });
    spans.push({ text: child.text ?? "", marks: markKinds });
  });
  if (!spans.length) {
    spans.push({ text: "", marks: [] });
  }
  return spans;
}

function blockFromPm(node: PmNode): BlockDto {
  const blockId = String(node.attrs.blockId ?? "");
  switch (node.type.name) {
    case "paragraph":
      return { type: "paragraph", id: blockId, inlines: inlinesFromPm(node) };
    case "heading":
      return { type: "heading", id: blockId, level: node.attrs.level, inlines: inlinesFromPm(node) };
    case "blockquote": {
      const inner = node.firstChild;
      return {
        type: "blockquote",
        id: blockId,
        inlines: inner ? inlinesFromPm(inner) : [{ text: "", marks: [] }],
      };
    }
    case "code_block":
      return { type: "code_block", id: blockId, text: node.textContent, language: node.attrs.language };
    case "bullet_list":
      return {
        type: "bullet_list",
        id: blockId,
        items: listItemsFromPm(node),
      };
    case "ordered_list":
      return {
        type: "ordered_list",
        id: blockId,
        items: listItemsFromPm(node),
      };
    case "horizontal_rule":
      return { type: "horizontal_rule", id: blockId };
    case "image":
      return { type: "image", id: blockId, src: node.attrs.src, alt: node.attrs.alt };
    case "opaque":
      return {
        type: "opaque",
        id: blockId,
        raw: node.attrs.raw,
        label: node.attrs.label,
      };
    default:
      return { type: "paragraph", id: blockId, inlines: inlinesFromPm(node) };
  }
}

function listItemsFromPm(node: PmNode) {
  const items: { inlines: { text: string; marks?: { kind: string }[] }[] }[] = [];
  node.forEach((child) => {
    const p = child.firstChild;
    items.push({ inlines: p ? inlinesFromPm(p) : [{ text: "", marks: [] }] });
  });
  return items;
}

export function modelToDoc(model: DocumentModelDto): PmNode {
  const blocks = (model.blocks as BlockDto[]).map((block) => {
    switch (block.type) {
      case "paragraph":
        return richTextSchema.node("paragraph", { blockId: block.id ?? "" }, textNodes(block.inlines));
      case "heading":
        return richTextSchema.node(
          "heading",
          { level: block.level ?? 1, blockId: block.id ?? "" },
          textNodes(block.inlines),
        );
      case "blockquote":
        return richTextSchema.node("blockquote", { blockId: block.id ?? "" }, [
          richTextSchema.node("paragraph", { blockId: block.id ?? "" }, textNodes(block.inlines)),
        ]);
      case "code_block":
        return richTextSchema.node("code_block", {
          blockId: block.id ?? "",
          language: block.language ?? null,
        }, block.text ? [richTextSchema.text(block.text)] : []);
      case "bullet_list":
        return richTextSchema.node(
          "bullet_list",
          { blockId: block.id ?? "" },
          (block.items ?? []).map((item) =>
            richTextSchema.node("list_item", null, [
              richTextSchema.node("paragraph", null, textNodes(item.inlines)),
            ]),
          ),
        );
      case "ordered_list":
        return richTextSchema.node(
          "ordered_list",
          { blockId: block.id ?? "" },
          (block.items ?? []).map((item) =>
            richTextSchema.node("list_item", null, [
              richTextSchema.node("paragraph", null, textNodes(item.inlines)),
            ]),
          ),
        );
      case "horizontal_rule":
        return richTextSchema.node("horizontal_rule", { blockId: block.id ?? "" });
      case "image":
        return richTextSchema.node("image", {
          blockId: block.id ?? "",
          src: block.src ?? "",
          alt: block.alt ?? "",
        });
      case "opaque":
        return richTextSchema.node("opaque", {
          blockId: block.id ?? "",
          raw: block.raw ?? "",
          label: block.label ?? "Raw Markdown (preserved)",
        });
      default:
        return richTextSchema.node("paragraph", { blockId: block.id ?? "" }, textNodes(block.inlines));
    }
  });
  return richTextSchema.node("doc", null, blocks);
}

function textNodes(inlines?: { text: string; marks?: { kind: string; href?: string }[] }[]): PmNode[] {
  if (!inlines?.length) {
    return [];
  }
  return inlines.map((span) => {
    const pmMarks = (span.marks ?? []).flatMap((m) => {
      if (m.kind === "bold") return richTextSchema.mark("bold");
      if (m.kind === "italic") return richTextSchema.mark("italic");
      if (m.kind === "code") return richTextSchema.mark("code");
      if (m.kind === "link") return richTextSchema.mark("link", { href: m.href ?? "" });
      return null;
    }).filter(Boolean) as ReturnType<typeof richTextSchema.mark>[];
    return richTextSchema.text(span.text, pmMarks);
  });
}

export function docToModel(doc: PmNode, base: DocumentModelDto): DocumentModelDto {
  const blocks: BlockDto[] = [];
  doc.forEach((child) => {
    blocks.push(blockFromPm(child));
  });
  return {
    ...base,
    blocks,
  };
}

export type RichTextEditorHandle = {
  view: EditorView;
  destroy: () => void;
};

export function mountProseMirrorEditor(
  host: HTMLElement,
  model: DocumentModelDto,
  readonly: boolean,
  onModelChange: (next: DocumentModelDto, touchedBlockIds: string[]) => void,
): RichTextEditorHandle {
  host.innerHTML = "";
  host.classList.add("rich-text-editor-host");
  const article = document.createElement("article");
  article.className = `rich-text-doc${readonly ? " rich-text-doc--readonly" : ""}`;
  if (readonly) {
    article.setAttribute("aria-readonly", "true");
  }
  host.appendChild(article);

  const doc = modelToDoc(model);
  let baseModel = model;
  const state = EditorState.create({
    doc,
    plugins: [
      history(),
      keymap({ "Mod-z": undo, "Mod-y": redo, "Mod-Shift-z": redo }),
      keymap(baseKeymap),
    ],
  });

  const view = new EditorView(article, {
    state,
    editable: () => !readonly,
    dispatchTransaction(tr: Transaction) {
      const next = view.state.apply(tr);
      view.updateState(next);
      if (!tr.docChanged) {
        return;
      }
      const nextModel = docToModel(next.doc, baseModel);
      const touched = diffTouchedBlocks(baseModel.blocks as BlockDto[], nextModel.blocks as BlockDto[]);
      baseModel = nextModel;
      onModelChange(nextModel, touched);
    },
  });

  article.addEventListener("click", (ev) => {
    const btn = (ev.target as HTMLElement).closest<HTMLButtonElement>("[data-command='parse-opaque']");
    if (!btn) {
      return;
    }
    ev.preventDefault();
    window.dispatchEvent(
      new CustomEvent("unibrowser-document-parse-opaque", {
        detail: { blockId: btn.closest("[data-block-id]")?.getAttribute("data-block-id") },
      }),
    );
  });

  return {
    view,
    destroy: () => view.destroy(),
  };
}

function diffTouchedBlocks(before: BlockDto[], after: BlockDto[]): string[] {
  const touched: string[] = [];
  const afterById = new Map(after.map((b) => [b.id ?? "", b]));
  for (const block of before) {
    const id = block.id ?? "";
    const next = afterById.get(id);
    if (!next || JSON.stringify(next) !== JSON.stringify(block)) {
      touched.push(id);
    }
  }
  for (const block of after) {
    const id = block.id ?? "";
    if (!before.some((b) => b.id === id)) {
      touched.push(id);
    }
  }
  return touched;
}
