import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { DocumentModelDto } from "./documentModelBridge";
import { isWebPlatform } from "./platform/isWeb";
import { mountProseMirrorEditor, type RichTextEditorHandle } from "./richTextProseMirror";

type BlockDto = {
  type: string;
  id?: string;
  level?: number;
  inlines?: { text: string; marks?: InlineMarkDto[] }[];
  text?: string;
  language?: string | null;
  label?: string | null;
  raw?: string;
  items?: { inlines: { text: string; marks?: InlineMarkDto[] }[] }[];
  src?: string;
  alt?: string;
};

type InlineMarkDto = {
  kind: string;
  href?: string | null;
};

export type MountRichTextOptions = {
  basePath?: string | null;
  onModelChange?: (model: DocumentModelDto, touchedBlockIds: string[]) => void;
};

let activeHandle: RichTextEditorHandle | null = null;

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function escapeAttr(text: string): string {
  return escapeHtml(text).replace(/"/g, "&quot;");
}

function isAbsoluteUrl(src: string): boolean {
  const lower = src.trim().toLowerCase();
  return (
    lower.startsWith("https://") ||
    lower.startsWith("http://") ||
    lower.startsWith("data:") ||
    lower.startsWith("file://")
  );
}

function guessImageMime(path: string): string {
  const lower = path.toLowerCase();
  if (lower.endsWith(".png")) {
    return "image/png";
  }
  if (lower.endsWith(".gif")) {
    return "image/gif";
  }
  if (lower.endsWith(".webp")) {
    return "image/webp";
  }
  if (lower.endsWith(".svg")) {
    return "image/svg+xml";
  }
  return "image/jpeg";
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

function joinDocRelative(basePath: string, href: string): string {
  const trimmed = href.trim();
  if (isAbsoluteUrl(trimmed)) {
    return trimmed;
  }
  const normalizedBase = basePath.replace(/\\/g, "/");
  const slash = normalizedBase.lastIndexOf("/");
  const dir = slash >= 0 ? normalizedBase.slice(0, slash + 1) : "";
  const joined = `${dir}${trimmed}`.replace(/\/+/g, "/");
  if (isWebPlatform()) {
    return joined.startsWith("./") || joined.startsWith("/") ? joined : `./${joined}`;
  }
  return joined;
}

function renderMarkedText(text: string, marks: InlineMarkDto[] | undefined): string {
  let html = escapeHtml(text);
  for (const mark of marks ?? []) {
    switch (mark.kind) {
      case "bold":
        html = `<strong>${html}</strong>`;
        break;
      case "italic":
        html = `<em>${html}</em>`;
        break;
      case "strikethrough":
        html = `<s>${html}</s>`;
        break;
      case "code":
        html = `<code>${html}</code>`;
        break;
      case "link": {
        const href = escapeAttr(mark.href ?? "#");
        html = `<a href="${href}" class="rich-text-link">${html}</a>`;
        break;
      }
      case "underline":
        html = `<u>${html}</u>`;
        break;
      default:
        break;
    }
  }
  return html;
}

function renderInlines(inlines: { text: string; marks?: InlineMarkDto[] }[] | undefined): string {
  if (!inlines?.length) {
    return "";
  }
  return inlines.map((span) => renderMarkedText(span.text, span.marks)).join("");
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
          `<figure data-block-id="${block.id ?? ""}"><img alt="${escapeHtml(block.alt ?? "")}" src="${escapeAttr(block.src ?? "")}" data-doc-src="${escapeAttr(block.src ?? "")}"/></figure>`,
        );
        break;
      case "horizontal_rule":
        parts.push(`<hr data-block-id="${block.id ?? ""}"/>`);
        break;
      case "opaque":
        parts.push(
          `<div class="rich-text-opaque" data-block-id="${block.id ?? ""}"><span class="rich-text-opaque-label">${escapeHtml(block.label ?? "Raw Markdown (preserved)")}</span><pre>${escapeHtml(block.raw ?? "")}</pre><button type="button" class="rich-text-opaque-parse" data-command="parse-opaque">Parse as Markdown</button></div>`,
        );
        break;
      default:
        parts.push(`<p data-block-id="${block.id ?? ""}">${renderInlines(block.inlines)}</p>`);
    }
  }
  parts.push("</article>");
  return parts.join("");
}

async function resolveRichImages(host: HTMLElement, basePath: string): Promise<void> {
  const imgs = host.querySelectorAll<HTMLImageElement>("img");
  for (const img of imgs) {
    const raw = img.dataset.docSrc ?? img.getAttribute("src") ?? "";
    if (!raw || isAbsoluteUrl(raw)) {
      continue;
    }
    if (isWebPlatform()) {
      img.src = joinDocRelative(basePath, raw);
      continue;
    }
    try {
      const fileUrl = await invoke<string>("resolve_preview_link", {
        basePath,
        href: raw,
      });
      const filePath = fileUrl.replace(/^file:\/\//i, "");
      try {
        const data = await invoke<number[]>("read_document_bytes", { path: filePath });
        const bytes = new Uint8Array(data);
        const mime = guessImageMime(filePath);
        img.src = `data:${mime};base64,${bytesToBase64(bytes)}`;
      } catch {
        img.src = convertFileSrc(filePath);
      }
    } catch {
      img.classList.add("rich-text-img--broken");
    }
  }
}

function mountReadonlyHtml(
  host: HTMLElement,
  model: DocumentModelDto,
  readonly: boolean,
  basePath?: string | null,
): HTMLElement {
  host.innerHTML = renderDocumentModelToHtml(model, readonly);
  host.classList.add("rich-text-editor-host");
  host.querySelector("article")?.addEventListener("click", (ev) => {
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
  if (basePath) {
    void resolveRichImages(host, basePath);
  }
  return host.querySelector("article") ?? host;
}

export async function mountRichTextEditor(
  host: HTMLElement,
  model: DocumentModelDto,
  readonly: boolean,
  options?: MountRichTextOptions | string | null,
  legacyBasePath?: string | null,
): Promise<HTMLElement> {
  let basePath: string | null | undefined = legacyBasePath;
  let onModelChange: MountRichTextOptions["onModelChange"];
  if (typeof options === "string" || options === null) {
    basePath = options;
  } else if (options) {
    basePath = options.basePath ?? basePath;
    onModelChange = options.onModelChange;
  }

  if (readonly || !onModelChange) {
    activeHandle?.destroy();
    activeHandle = null;
    return mountReadonlyHtml(host, model, readonly, basePath);
  }

  activeHandle?.destroy();
  activeHandle = mountProseMirrorEditor(host, model, readonly, (next, touched) => {
    onModelChange?.(next, touched);
  });
  if (basePath) {
    await resolveRichImages(host, basePath);
  }
  return host.querySelector("article") ?? host;
}

export function destroyRichTextEditor() {
  activeHandle?.destroy();
  activeHandle = null;
}
