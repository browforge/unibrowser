import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { DocumentModelDto } from "./documentModelBridge";
import { isWebPlatform } from "./platform/isWeb";
import { mountProseMirrorEditor, type RichTextEditorHandle } from "./richTextProseMirror";
import { markdownLiveBlockPlugins } from "./markdownLiveBlock";

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
  markdownLiveBlock?: boolean;
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
    lower.startsWith("file://") ||
    lower.startsWith("asset://") ||
    lower.startsWith("blob:")
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
    const base = import.meta.env.BASE_URL ?? "/";
    const normalized = joined.replace(/^\.\//, "");
    if (normalized.startsWith("/") || normalized.startsWith("http")) {
      return normalized;
    }
    return `${base}${normalized}`;
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

function richEditorImageBasePath(basePath: string): string {
  const override = document.body.dataset.documentImageBasePath?.trim();
  return override || basePath;
}

async function resolveDesktopImageHref(
  resolvedBase: string,
  raw: string,
): Promise<string | null> {
  const repoRelative = joinDocRelative(resolvedBase, raw);
  const fileUrl = await invoke<string>("resolve_preview_link", {
    basePath: resolvedBase,
    href: raw,
  });
  const filePath = fileUrl.replace(/^file:\/\//i, "");
  await invoke("allow_asset_path", { path: repoRelative }).catch(() => undefined);
  await invoke("allow_asset_path", { path: filePath }).catch(() => undefined);
  for (const readPath of [repoRelative, filePath]) {
    try {
      const data = await invoke<number[]>("read_document_bytes", { path: readPath });
      const bytes = new Uint8Array(data);
      const mime = guessImageMime(readPath);
      return `data:${mime};base64,${bytesToBase64(bytes)}`;
    } catch {
      /* try next path */
    }
  }
  return convertFileSrc(filePath);
}

/** Inline relative image hrefs in the model so ProseMirror keeps resolved src (DOM-only patches get overwritten). */
export async function resolveDocumentModelImageAssets(
  model: DocumentModelDto,
  basePath: string,
): Promise<DocumentModelDto> {
  const resolvedBase = richEditorImageBasePath(basePath);
  const blocks = model.blocks ?? [];
  let changed = false;
  const nextBlocks: unknown[] = [];
  for (const block of blocks) {
    const b = block as BlockDto;
    if (b.type !== "image" || !b.src || isAbsoluteUrl(b.src)) {
      nextBlocks.push(block);
      continue;
    }
    if (isWebPlatform()) {
      const url = joinDocRelative(resolvedBase, b.src);
      const dataUrl = await loadImageDataUrl(url, guessImageMime(b.src));
      nextBlocks.push({ ...b, src: dataUrl ?? url });
      changed = true;
      continue;
    }
    try {
      const resolved = await resolveDesktopImageHref(resolvedBase, b.src);
      nextBlocks.push({ ...b, src: resolved ?? b.src });
      changed = true;
    } catch {
      nextBlocks.push(block);
    }
  }
  if (!changed) {
    return model;
  }
  return { ...model, blocks: nextBlocks };
}

async function loadImageDataUrl(url: string, mimeHint?: string): Promise<string | null> {
  try {
    const res = await fetch(url);
    if (!res.ok) {
      return null;
    }
    const bytes = new Uint8Array(await res.arrayBuffer());
    const mime = mimeHint ?? res.headers.get("content-type") ?? guessImageMime(url);
    return `data:${mime};base64,${bytesToBase64(bytes)}`;
  } catch {
    return null;
  }
}

export async function resolveRichImages(host: HTMLElement, basePath: string): Promise<void> {
  const resolvedBase = richEditorImageBasePath(basePath);
  const imgs = host.querySelectorAll<HTMLImageElement>("img");
  for (const img of imgs) {
    img.classList.remove("rich-text-img--broken");
    const raw = img.dataset.docSrc ?? img.getAttribute("src") ?? "";
    if (!raw || isAbsoluteUrl(raw)) {
      continue;
    }
    if (isWebPlatform()) {
      const url = joinDocRelative(resolvedBase, raw);
      const dataUrl = await loadImageDataUrl(url, guessImageMime(raw));
      img.src = dataUrl ?? url;
      continue;
    }
    try {
      const resolved = await resolveDesktopImageHref(resolvedBase, raw);
      if (resolved) {
        img.src = resolved;
        await img.decode().catch(() => undefined);
      }
    } catch {
      img.classList.add("rich-text-img--broken");
    }
  }
}

async function mountReadonlyHtml(
  host: HTMLElement,
  model: DocumentModelDto,
  readonly: boolean,
  basePath?: string | null,
): Promise<HTMLElement> {
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
    await resolveRichImages(host, basePath);
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
  let markdownLiveBlock = false;
  if (typeof options === "string" || options === null) {
    basePath = options;
  } else if (options) {
    basePath = options.basePath ?? basePath;
    onModelChange = options.onModelChange;
    markdownLiveBlock = Boolean(options.markdownLiveBlock);
  }

  if (readonly || !onModelChange) {
    activeHandle?.destroy();
    activeHandle = null;
    return await mountReadonlyHtml(host, model, readonly, basePath);
  }

  activeHandle?.destroy();
  const liveEnabled = () => markdownLiveBlock && !readonly;
  activeHandle = mountProseMirrorEditor(
    host,
    model,
    readonly,
    (next, touched) => {
      onModelChange?.(next, touched);
    },
    { extraPlugins: markdownLiveBlock ? markdownLiveBlockPlugins(liveEnabled) : [] },
  );
  if (basePath) {
    await resolveRichImages(host, basePath);
    await new Promise((r) => requestAnimationFrame(() => r(undefined)));
    await resolveRichImages(host, basePath);
  }
  return host.querySelector("article") ?? host;
}

export function destroyRichTextEditor() {
  activeHandle?.destroy();
  activeHandle = null;
}

export function getActiveRichTextView() {
  return activeHandle?.view ?? null;
}

/** Wait until every image in the document card rich surface has decoded (naturalWidth > 0). */
export async function waitForRichCardImagesLoaded(timeoutMs = 12_000): Promise<void> {
  const shell = document.getElementById("document-card-shell");
  const rich =
    document.querySelector<HTMLElement>("#rich-text-editor-host:not(.hidden) .rich-text-doc") ??
    document.querySelector<HTMLElement>(
      "#document-split-preview-host:not(.hidden) .rich-text-doc",
    );
  if (!shell || shell.classList.contains("hidden") || !rich) {
    return;
  }
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const imgs = Array.from(rich.querySelectorAll<HTMLImageElement>("img"));
    if (imgs.length === 0) {
      return;
    }
    const pending = imgs.filter((img) => !img.naturalWidth);
    if (pending.length === 0) {
      return;
    }
    await Promise.all(
      pending.map(
        (img) =>
          new Promise<void>((resolve) => {
            if (img.complete && img.naturalWidth > 0) {
              resolve();
              return;
            }
            const perImgMs = Math.min(4_000, Math.max(500, deadline - Date.now()));
            const timer = window.setTimeout(() => {
              console.error(
                `[doc-capture] rich image wait timeout src=${img.currentSrc || img.src}`,
              );
              resolve();
            }, perImgMs);
            const done = () => {
              window.clearTimeout(timer);
              resolve();
            };
            img.addEventListener("load", done, { once: true });
            img.addEventListener("error", done, { once: true });
          }),
      ),
    );
    await new Promise((r) => requestAnimationFrame(() => r(undefined)));
  }
}
