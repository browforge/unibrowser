import { marked } from "marked";
import { invoke } from "@tauri-apps/api/core";

type DocTocEntry = { section: string; slug: string; label: string };

function navigateDocs(slug: string) {
  window.dispatchEvent(
    new CustomEvent("unibrowser-navigate", { detail: `unibrowser://docs/${slug}` }),
  );
}

function highlightSearch(article: HTMLElement, query: string) {
  const html = article.dataset.sourceHtml ?? article.innerHTML;
  if (!query.trim()) {
    article.innerHTML = html;
    return;
  }
  article.innerHTML = html;
  const q = query.trim();
  const walker = document.createTreeWalker(article, NodeFilter.SHOW_TEXT);
  const hits: Text[] = [];
  while (walker.nextNode()) {
    const node = walker.currentNode as Text;
    if (node.textContent?.toLowerCase().includes(q.toLowerCase())) {
      hits.push(node);
    }
  }
  hits.forEach((node) => {
    const text = node.textContent ?? "";
    const idx = text.toLowerCase().indexOf(q.toLowerCase());
    if (idx < 0) {
      return;
    }
    const before = text.slice(0, idx);
    const match = text.slice(idx, idx + q.length);
    const after = text.slice(idx + q.length);
    const span = document.createElement("span");
    span.append(document.createTextNode(before));
    const mark = document.createElement("mark");
    mark.className = "docs-search-hit";
    mark.textContent = match;
    span.append(mark, document.createTextNode(after));
    node.parentNode?.replaceChild(span, node);
  });
  const first = article.querySelector(".docs-search-hit");
  first?.scrollIntoView({ block: "nearest", behavior: "smooth" });
}

export async function renderDocs(root: HTMLElement, slug: string) {
  root.innerHTML = "<p class='viewer-loading'>Loading help…</p>";
  const [doc, toc] = await Promise.all([
    invoke<{ title: string; markdown: string }>("get_help_doc_markdown", { slug }),
    invoke<DocTocEntry[]>("get_help_doc_toc").catch(() => [] as DocTocEntry[]),
  ]);

  root.innerHTML = "";
  const shell = document.createElement("div");
  shell.className = "docs-viewer";

  const aside = document.createElement("aside");
  aside.className = "docs-toc";
  const tocTitle = document.createElement("h3");
  tocTitle.textContent = "Contents";
  aside.appendChild(tocTitle);
  const tocList = document.createElement("ul");
  tocList.className = "docs-toc-list";
  let lastSection = "";
  toc.forEach((entry) => {
    if (entry.section !== lastSection) {
      lastSection = entry.section;
      const sec = document.createElement("li");
      sec.className = "docs-toc-section";
      sec.textContent = entry.section;
      tocList.appendChild(sec);
    }
    const li = document.createElement("li");
    const link = document.createElement("button");
    link.type = "button";
    link.className = "docs-toc-link";
    if (entry.slug === slug) {
      link.classList.add("docs-toc-link--active");
    }
    link.textContent = entry.label;
    link.onclick = () => navigateDocs(entry.slug);
    li.appendChild(link);
    tocList.appendChild(li);
  });
  aside.appendChild(tocList);

  const main = document.createElement("div");
  main.className = "docs-main";

  const topNav = document.createElement("nav");
  topNav.className = "docs-top-nav";
  const back = document.createElement("button");
  back.type = "button";
  back.className = "docs-back-link";
  back.textContent = "← Documentation index";
  back.onclick = () => navigateDocs("index");
  topNav.appendChild(back);

  const searchRow = document.createElement("div");
  searchRow.className = "docs-search-row";
  const searchInput = document.createElement("input");
  searchInput.type = "search";
  searchInput.className = "docs-search-input";
  searchInput.placeholder = "Search in this page";
  searchInput.setAttribute("aria-label", "Search in this page");
  searchRow.appendChild(searchInput);

  const title = document.createElement("h2");
  title.className = "viewer-title";
  title.textContent = doc.title;

  const article = document.createElement("article");
  article.className = "docs-help-body";
  const parsed = marked.parse(doc.markdown, { async: false }) as string;
  article.innerHTML = parsed;
  article.dataset.sourceHtml = parsed;

  searchInput.addEventListener("input", () => {
    highlightSearch(article, searchInput.value);
  });

  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = `Source: docs/${slug === "index" ? "index.md" : `${slug}.md`} (dev checkout).`;

  main.append(topNav, searchRow, title, article, note);
  shell.append(aside, main);
  root.appendChild(shell);
}
