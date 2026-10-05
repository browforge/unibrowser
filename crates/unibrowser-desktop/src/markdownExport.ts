import { invoke } from "@tauri-apps/api/core";
import { renderMarkdownBodyHtml } from "./markdownPreview";
import { showErrorToast, showToast } from "./toast";
import { t } from "./i18n";

/** Inline CSS for exported Markdown HTML (no app bundle). */
export const MARKDOWN_STANDALONE_CSS = `
  :root { color-scheme: light dark; }
  body {
    margin: 0;
    padding: 1.5rem 2rem;
    font-family: system-ui, sans-serif;
    font-size: 11pt;
    line-height: 1.55;
    color: #1a1a1a;
    background: #fafafa;
  }
  main { max-width: 48rem; margin: 0 auto; }
  h1, h2, h3, h4 { line-height: 1.25; margin-top: 1.5em; }
  h1 { font-size: 1.75rem; }
  h2 { font-size: 1.35rem; }
  h3 { font-size: 1.15rem; }
  a { color: #0b57d0; }
  img { max-width: 100%; height: auto; }
  table { border-collapse: collapse; width: 100%; margin: 1em 0; }
  th, td { border: 1px solid #ccc; padding: 0.35rem 0.5rem; text-align: left; }
  pre {
    overflow-x: auto;
    padding: 0.75rem 1rem;
    border-radius: 4px;
    background: #1e1e28;
    color: #d4d4d4;
    font-size: 0.85rem;
  }
  code { font-family: ui-monospace, monospace; }
  .md-export-toc {
    margin-bottom: 1.5rem;
    padding: 0.75rem 1rem;
    background: #eee;
    border-radius: 4px;
  }
  .md-export-toc ul { margin: 0.35rem 0 0; padding-left: 1.25rem; }
  .md-export-toc .md-toc-h3 { margin-left: 0.75rem; }
  .md-export-toc .md-toc-h4 { margin-left: 1.5rem; }
  @media (prefers-color-scheme: dark) {
    body { color: #e8e8e8; background: #121218; }
    th, td { border-color: #444; }
    .md-export-toc { background: #1e1e28; }
  }
`;

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function standaloneMarkdownHtmlDocument(
  title: string,
  bodyInnerHtml: string,
  tocHtml?: string,
): string {
  const safeTitle = escapeHtml(title);
  const tocBlock = tocHtml ? `<nav class="md-export-toc" aria-label="Table of contents">${tocHtml}</nav>` : "";
  return `<!DOCTYPE html>
<html lang="en-NZ">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${safeTitle}</title>
  <style>${MARKDOWN_STANDALONE_CSS}</style>
</head>
<body>
  <main>
    ${tocBlock}
    <div class="md-export-body">${bodyInnerHtml}</div>
  </main>
</body>
</html>`;
}

function suggestedHtmlName(mdPath: string): string {
  const base = mdPath.split(/[/\\]/).pop() ?? "document.md";
  return base.replace(/\.(md|markdown)$/i, "") + ".html";
}

export async function exportMarkdownToHtmlFile(mdPath: string, markdown: string): Promise<void> {
  const { bodyHtml, tocHtml } = await renderMarkdownBodyHtml(markdown, mdPath, {
    embedImages: true,
    includeMermaid: false,
  });
  const doc = standaloneMarkdownHtmlDocument(mdPath, bodyHtml, tocHtml);
  const suggested = suggestedHtmlName(mdPath);
  try {
    const dest = await invoke<string | null>("pick_save_html_path", { suggestedName: suggested });
    if (!dest) {
      return;
    }
    const bytes = Array.from(new TextEncoder().encode(doc));
    await invoke("write_document_bytes", { path: dest, bytes });
    showToast(t("markdown.exportedHtml", { path: dest }), "info");
  } catch (err) {
    showErrorToast(err);
  }
}
