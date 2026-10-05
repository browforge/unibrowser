import { invoke } from "@tauri-apps/api/core";
import { runViewerFind } from "./documentViewer";
import { openTextFind, runTextFind } from "./textEditor";

let bar: HTMLElement | null = null;
let input: HTMLInputElement | null = null;
let open = false;

export type DocumentFindSurface = "monaco" | "viewer" | "web" | "none";

let surface: DocumentFindSurface = "none";

export function setDocumentFindSurface(next: DocumentFindSurface) {
  surface = next;
}

function ensureBar() {
  if (bar) {
    return;
  }
  bar = document.getElementById("document-find-bar");
  input = document.getElementById("document-find-input") as HTMLInputElement | null;
  if (!bar || !input) {
    return;
  }
  document.getElementById("document-find-close")?.addEventListener("click", () => {
    closeDocumentFind();
  });
  document.getElementById("document-find-prev")?.addEventListener("click", () => {
    runFind(false);
  });
  document.getElementById("document-find-next")?.addEventListener("click", () => {
    runFind(true);
  });
  input.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      closeDocumentFind();
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      runFind(!e.shiftKey);
    }
  });
}

function runFind(forward: boolean) {
  const query = input?.value ?? "";
  if (surface === "monaco") {
    if (!query.trim()) {
      openTextFind();
      return;
    }
    runTextFind(query, forward);
    return;
  }
  if (surface === "viewer") {
    runViewerFind(query, forward);
    return;
  }
  if (surface === "web") {
    if (!query.trim()) {
      return;
    }
    void invoke("web_find_in_page", { query, forward });
  }
}

export function openDocumentFind() {
  ensureBar();
  if (!bar || !input) {
    return;
  }
  open = true;
  bar.classList.remove("hidden");
  input.focus();
  input.select();
  if (surface === "monaco" && !input.value.trim()) {
    openTextFind();
  }
}

export function closeDocumentFind() {
  if (!bar) {
    return;
  }
  open = false;
  bar.classList.add("hidden");
}

export function toggleDocumentFind() {
  if (open) {
    closeDocumentFind();
  } else {
    openDocumentFind();
  }
}

export function isDocumentFindOpen() {
  return open;
}
