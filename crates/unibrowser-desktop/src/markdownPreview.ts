import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { marked } from "marked";
import { bindPreviewLinks } from "./previewLinks";
import { t } from "./i18n";

export interface MarkdownRenderOptions {
  embedImages?: boolean;
  includeMermaid?: boolean;
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

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
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

async function resolvePreviewImages(
  container: HTMLElement,
  basePath: string,
  embedImages: boolean,
) {
  const imgs = container.querySelectorAll<HTMLImageElement>("img[src]");
  for (const img of imgs) {
    const src = img.getAttribute("src");
    if (!src) {
      continue;
    }
    if (isAbsoluteUrl(src) && !src.toLowerCase().startsWith("file://")) {
      continue;
    }
    let filePath = "";
    if (src.toLowerCase().startsWith("file://")) {
      filePath = src.replace(/^file:\/\//i, "");
    } else {
      try {
        const fileUrl = await invoke<string>("resolve_preview_link", {
          basePath,
          href: src,
        });
        filePath = fileUrl.replace(/^file:\/\//i, "");
      } catch {
        img.classList.add("md-preview-img--broken");
        if (!img.alt) {
          img.alt = "Image not found";
        }
        continue;
      }
    }
    if (embedImages) {
      try {
        const data = await invoke<number[]>("read_document_bytes", { path: filePath });
        const bytes = new Uint8Array(data);
        const mime = guessImageMime(filePath);
        img.src = `data:${mime};base64,${bytesToBase64(bytes)}`;
      } catch {
        img.classList.add("md-preview-img--broken");
      }
    } else {
      img.src = convertFileSrc(filePath);
    }
  }
}

function slugifyHeading(text: string, used: Set<string>): string {
  let base =
    text
      .trim()
      .toLowerCase()
      .replace(/[^\w\s-]/g, "")
      .replace(/\s+/g, "-")
      .replace(/-+/g, "-")
      .replace(/^-|-$/g, "") || "section";
  let id = base;
  let n = 2;
  while (used.has(id)) {
    id = `${base}-${n}`;
    n += 1;
  }
  used.add(id);
  return id;
}

export interface MarkdownHeading {
  level: number;
  id: string;
  text: string;
}

export function collectHeadingAnchors(container: HTMLElement): MarkdownHeading[] {
  const used = new Set<string>();
  const headings: MarkdownHeading[] = [];
  container.querySelectorAll("h1, h2, h3, h4, h5, h6").forEach((node) => {
    const el = node as HTMLElement;
    const level = Number.parseInt(el.tagName[1], 10);
    const text = el.textContent?.trim() ?? "";
    if (!el.id) {
      el.id = slugifyHeading(text, used);
    } else {
      used.add(el.id);
    }
    headings.push({ level, id: el.id, text });
  });
  return headings;
}

export function buildTocHtml(headings: MarkdownHeading[]): string {
  if (!headings.length) {
    return "";
  }
  const items = headings
    .map(
      (h) =>
        `<li class="md-toc-item md-toc-h${h.level}"><a href="#${escapeAttr(h.id)}">${escapeHtml(h.text)}</a></li>`,
    )
    .join("");
  return `<p class="md-toc-title">${escapeHtml(t("markdown.tocTitle"))}</p><ul class="md-toc-list">${items}</ul>`;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function escapeAttr(text: string): string {
  return escapeHtml(text).replace(/"/g, "&quot;");
}

export function mountPreviewToc(tocEl: HTMLElement, headings: MarkdownHeading[], scrollRoot: HTMLElement) {
  if (!headings.length) {
    tocEl.innerHTML = "";
    tocEl.classList.add("hidden");
    return;
  }
  tocEl.classList.remove("hidden");
  tocEl.innerHTML = "";
  const title = document.createElement("p");
  title.className = "md-toc-title";
  title.textContent = t("markdown.tocTitle");
  tocEl.appendChild(title);
  const ul = document.createElement("ul");
  ul.className = "md-toc-list";
  headings.forEach((h) => {
    const li = document.createElement("li");
    li.className = `md-toc-item md-toc-h${h.level}`;
    const a = document.createElement("a");
    a.href = `#${h.id}`;
    a.textContent = h.text;
    a.onclick = (ev) => {
      ev.preventDefault();
      const target = scrollRoot.querySelector<HTMLElement>(`#${CSS.escape(h.id)}`);
      target?.scrollIntoView({ behavior: "smooth", block: "start" });
    };
    li.appendChild(a);
    ul.appendChild(li);
  });
  tocEl.appendChild(ul);
}

function monacoLanguageId(className: string): string {
  const match = className.match(/language-([\w#+-]+)/i);
  const raw = match?.[1]?.toLowerCase() ?? "plaintext";
  const aliases: Record<string, string> = {
    js: "javascript",
    jsx: "javascript",
    ts: "typescript",
    tsx: "typescript",
    py: "python",
    rs: "rust",
    yml: "yaml",
    sh: "shell",
    bash: "shell",
    zsh: "shell",
    md: "markdown",
    "c++": "cpp",
    h: "c",
  };
  return aliases[raw] ?? raw;
}

async function highlightPreviewCodeBlocks(container: HTMLElement) {
  const blocks = container.querySelectorAll<HTMLElement>("pre > code[class*='language-']");
  if (!blocks.length) {
    return;
  }
  const monaco = await import("monaco-editor");
  const { resolveEditorThemeId } = await import("./monacoEditorTheme");
  const pref =
    (document.documentElement.dataset.editorTheme as string | undefined) ?? "auto";
  monaco.editor.setTheme(resolveEditorThemeId(pref));
  for (const code of blocks) {
    if (code.classList.contains("language-mermaid")) {
      continue;
    }
    const lang = monacoLanguageId(code.className);
    const text = code.textContent ?? "";
    const pre = code.parentElement;
    if (!pre) {
      continue;
    }
    try {
      const html = await monaco.editor.colorize(text, lang, {});
      pre.classList.add("md-code-highlighted");
      pre.innerHTML = html;
    } catch {
      // keep plain code block
    }
  }
}

let mermaidModule: typeof import("mermaid") | null = null;

async function loadMermaid() {
  if (!mermaidModule) {
    mermaidModule = await import("mermaid");
  }
  return mermaidModule.default;
}

async function renderMermaidInPreview(container: HTMLElement) {
  const codes = container.querySelectorAll<HTMLElement>("pre > code.language-mermaid");
  if (!codes.length) {
    return;
  }
  const mounts: HTMLElement[] = [];
  codes.forEach((code) => {
    const pre = code.parentElement;
    if (!pre || !pre.parentElement) {
      return;
    }
    const source = code.textContent ?? "";
    const wrap = document.createElement("div");
    wrap.className = "mermaid-block";
    const diagram = document.createElement("div");
    diagram.className = "mermaid";
    diagram.textContent = source;
    const fallback = pre.cloneNode(true) as HTMLElement;
    fallback.classList.add("mermaid-fallback", "hidden");
    wrap.appendChild(diagram);
    wrap.appendChild(fallback);
    pre.replaceWith(wrap);
    mounts.push(diagram);
  });
  if (!mounts.length) {
    return;
  }
  try {
    const mermaid = await loadMermaid();
    const theme =
      document.documentElement.dataset.theme === "light" ? "default" : "dark";
    mermaid.initialize({
      startOnLoad: false,
      securityLevel: "strict",
      theme,
    });
    await mermaid.run({ nodes: mounts });
  } catch {
    mounts.forEach((diagram) => {
      const wrap = diagram.parentElement;
      const fallback = wrap?.querySelector<HTMLElement>(".mermaid-fallback");
      diagram.remove();
      fallback?.classList.remove("hidden");
    });
  }
}

async function hydrateMarkdownContainer(
  container: HTMLElement,
  markdown: string,
  basePath: string,
  options: MarkdownRenderOptions,
) {
  container.innerHTML = marked.parse(markdown, { async: false }) as string;
  await highlightPreviewCodeBlocks(container);
  if (options.includeMermaid !== false) {
    await renderMermaidInPreview(container);
  }
  await resolvePreviewImages(container, basePath, options.embedImages === true);
  bindPreviewLinks(container, basePath);
}

/** Full render for export (no live preview chrome). */
export async function renderMarkdownBodyHtml(
  markdown: string,
  basePath: string,
  options: MarkdownRenderOptions = {},
): Promise<{ bodyHtml: string; tocHtml: string }> {
  const host = document.createElement("div");
  await hydrateMarkdownContainer(host, markdown, basePath, options);
  const headings = collectHeadingAnchors(host);
  return { bodyHtml: host.innerHTML, tocHtml: buildTocHtml(headings) };
}

/** Parse Markdown, resolve local images, bind links, lazy-render Mermaid. */
export async function applyMarkdownPreview(
  container: HTMLElement,
  markdown: string,
  basePath: string,
  tocEl?: HTMLElement | null,
  options?: MarkdownRenderOptions,
) {
  await hydrateMarkdownContainer(container, markdown, basePath, {
    embedImages: options?.embedImages ?? false,
    includeMermaid: options?.includeMermaid ?? true,
  });
  const headings = collectHeadingAnchors(container);
  if (tocEl) {
    mountPreviewToc(tocEl, headings, container);
  }
}
