import type { DocumentModelDto } from "./documentModelBridge";

type BlockDto = {
  type: string;
  id?: string;
  level?: number;
  inlines?: { text: string; marks?: unknown[] }[];
  text?: string;
  language?: string | null;
  label?: string | null;
  raw?: string;
  items?: { inlines: { text: string }[] }[];
  src?: string;
  alt?: string;
};

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function renderInlines(inlines: { text: string }[] | undefined): string {
  if (!inlines?.length) {
    return "";
  }
  return inlines.map((span) => escapeHtml(span.text)).join("");
}

export function renderDocumentModelToHtml(model: DocumentModelDto, readonly = false): string {
  const parts: string[] = [];
  parts.push(`<article class="rich-text-doc${readonly ? " rich-text-doc--readonly" : ""}"${
    readonly ? ' aria-readonly="true"' : ""
  } contenteditable="${readonly ? "false" : "true"}">`);
  for (const block of model.blocks as BlockDto[]) {
    switch (block.type) {
      case "paragraph":
        parts.push(
          `<p data-block-id="${block.id ?? ""}">${renderInlines(block.inlines)}</p>`,
        );
        break;
      case "heading":
        parts.push(
          `<h${block.level ?? 1} data-block-id="${block.id ?? ""}">${renderInlines(block.inlines)}</h${block.level ?? 1}>`,
        );
        break;
      case "blockquote":
        parts.push(
          `<blockquote data-block-id="${block.id ?? ""}"><p>${renderInlines(block.inlines)}</p></blockquote>`,
        );
        break;
      case "code_block":
        parts.push(
          `<pre data-block-id="${block.id ?? ""}"><code>${escapeHtml(block.text ?? "")}</code></pre>`,
        );
        break;
      case "bullet_list":
        parts.push(`<ul data-block-id="${block.id ?? ""}">`);
        for (const item of block.items ?? []) {
          parts.push(`<li>${renderInlines(item.inlines)}</li>`);
        }
        parts.push("</ul>");
        break;
      case "ordered_list":
        parts.push(`<ol data-block-id="${block.id ?? ""}">`);
        for (const item of block.items ?? []) {
          parts.push(`<li>${renderInlines(item.inlines)}</li>`);
        }
        parts.push("</ol>");
        break;
      case "image":
        parts.push(
          `<figure data-block-id="${block.id ?? ""}"><img alt="${escapeHtml(block.alt ?? "")}" src="${escapeHtml(block.src ?? "")}"/></figure>`,
        );
        break;
      case "horizontal_rule":
        parts.push(`<hr data-block-id="${block.id ?? ""}"/>`);
        break;
      case "opaque":
        parts.push(
          `<div class="rich-text-opaque" data-block-id="${block.id ?? ""}"><span class="rich-text-opaque-label">${escapeHtml(block.label ?? "Raw Markdown (preserved)")}</span><pre>${escapeHtml(block.raw ?? "")}</pre></div>`,
        );
        break;
      default:
        parts.push(`<p data-block-id="${block.id ?? ""}">${renderInlines(block.inlines)}</p>`);
    }
  }
  parts.push("</article>");
  return parts.join("");
}

export function mountRichTextEditor(
  host: HTMLElement,
  model: DocumentModelDto,
  readonly: boolean,
): HTMLElement {
  host.innerHTML = renderDocumentModelToHtml(model, readonly);
  host.classList.add("rich-text-editor-host");
  return host.querySelector("article") ?? host;
}
