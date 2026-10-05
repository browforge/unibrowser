import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { marked } from "marked";
import { bindPreviewLinks } from "./previewLinks";

function isAbsoluteUrl(src: string): boolean {
  const lower = src.trim().toLowerCase();
  return (
    lower.startsWith("https://") ||
    lower.startsWith("http://") ||
    lower.startsWith("data:") ||
    lower.startsWith("file://")
  );
}

async function resolvePreviewImages(container: HTMLElement, basePath: string) {
  const imgs = container.querySelectorAll<HTMLImageElement>("img[src]");
  for (const img of imgs) {
    const src = img.getAttribute("src");
    if (!src || isAbsoluteUrl(src)) {
      if (src?.toLowerCase().startsWith("file://")) {
        const path = src.replace(/^file:\/\//i, "");
        img.src = convertFileSrc(path);
      }
      continue;
    }
    try {
      const fileUrl = await invoke<string>("resolve_preview_link", {
        basePath,
        href: src,
      });
      const path = fileUrl.replace(/^file:\/\//i, "");
      img.src = convertFileSrc(path);
    } catch {
      img.classList.add("md-preview-img--broken");
      if (!img.alt) {
        img.alt = "Image not found";
      }
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

/** Parse Markdown, resolve local images, bind links, lazy-render Mermaid. */
export async function applyMarkdownPreview(
  container: HTMLElement,
  markdown: string,
  basePath: string,
) {
  container.innerHTML = marked.parse(markdown, { async: false }) as string;
  await resolvePreviewImages(container, basePath);
  bindPreviewLinks(container, basePath);
  await renderMermaidInPreview(container);
}
