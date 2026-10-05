import { marked } from "marked";
import { invoke } from "@tauri-apps/api/core";

export async function renderDocs(root: HTMLElement, slug: string) {
  root.innerHTML = "<p class='viewer-loading'>Loading help…</p>";
  const doc = await invoke<{ title: string; markdown: string }>("get_help_doc_markdown", {
    slug,
  });
  root.innerHTML = "";
  const title = document.createElement("h2");
  title.className = "viewer-title";
  title.textContent = doc.title;
  root.appendChild(title);
  const article = document.createElement("article");
  article.className = "docs-help-body";
  article.innerHTML = marked.parse(doc.markdown, { async: false }) as string;
  root.appendChild(article);
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = `Source: docs/${slug}.md in the repository (dev checkout).`;
  root.appendChild(note);
}
