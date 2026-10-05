import { invoke } from "@tauri-apps/api/core";

const ALLOWED_PREFIXES = ["https://", "http://", "file://"];

function isAllowedAbsolute(href: string): boolean {
  const lower = href.toLowerCase();
  return ALLOWED_PREFIXES.some((p) => lower.startsWith(p));
}

export function bindPreviewLinks(container: HTMLElement, baseDocumentPath?: string) {
  container.addEventListener("click", (ev) => {
    const target = ev.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }
    const anchor = target.closest("a");
    if (!anchor || !container.contains(anchor)) {
      return;
    }
    const href = anchor.getAttribute("href");
    if (!href || href.startsWith("#")) {
      return;
    }
    ev.preventDefault();
    void openPreviewLink(href, baseDocumentPath);
  });
}

export async function openPreviewLink(href: string, baseDocumentPath?: string) {
  const trimmed = href.trim();
  if (!trimmed) {
    return;
  }
  const lower = trimmed.toLowerCase();
  if (lower.startsWith("javascript:") || lower.startsWith("data:")) {
    return;
  }
  let navigateTo = trimmed;
  if (isAllowedAbsolute(trimmed)) {
    navigateTo = trimmed;
  } else if (baseDocumentPath) {
    navigateTo = await invoke<string>("resolve_preview_link", {
      basePath: baseDocumentPath,
      href: trimmed,
    });
  } else {
    return;
  }
  window.dispatchEvent(new CustomEvent("unibrowser-navigate", { detail: navigateTo }));
}
