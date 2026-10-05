import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";

interface KeyboardShortcutRow {
  chord: string;
  title: string;
  command_id: string;
}

let overlay: HTMLElement;
let queryInput: HTMLInputElement;
let resultsHost: HTMLElement;
let cachedRows: KeyboardShortcutRow[] = [];
let open = false;

function fuzzyIncludes(haystack: string, needle: string): boolean {
  const n = needle.trim().toLowerCase();
  if (!n) {
    return true;
  }
  const h = haystack.toLowerCase();
  let i = 0;
  for (const ch of n) {
    const idx = h.indexOf(ch, i);
    if (idx < 0) {
      return false;
    }
    i = idx + 1;
  }
  return true;
}

function filterRows(q: string): KeyboardShortcutRow[] {
  const needle = q.trim();
  if (!needle) {
    return cachedRows;
  }
  return cachedRows.filter(
    (row) =>
      fuzzyIncludes(row.chord, needle) ||
      fuzzyIncludes(row.title, needle) ||
      fuzzyIncludes(row.command_id, needle),
  );
}

function renderResults(rows: KeyboardShortcutRow[]) {
  resultsHost.innerHTML = "";
  if (!rows.length) {
    const empty = document.createElement("p");
    empty.className = "viewer-note";
    empty.textContent = t("keys.overlayEmpty");
    resultsHost.appendChild(empty);
    return;
  }
  rows.forEach((row) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "palette-item keys-shortcut-row";
    const chord = document.createElement("span");
    chord.className = "keys-shortcut-chord";
    chord.textContent = row.chord;
    const title = document.createElement("span");
    title.className = "keys-shortcut-title";
    title.textContent = row.title;
    btn.appendChild(chord);
    btn.appendChild(title);
    btn.title = row.command_id;
    resultsHost.appendChild(btn);
  });
}

async function refreshList() {
  try {
    cachedRows = await invoke<KeyboardShortcutRow[]>("get_keyboard_shortcuts");
  } catch {
    cachedRows = [];
  }
  renderResults(filterRows(queryInput.value));
}

export function initKeysOverlay() {
  overlay = document.getElementById("keys-shortcuts-overlay")!;
  queryInput = document.getElementById("keys-shortcuts-query") as HTMLInputElement;
  resultsHost = document.getElementById("keys-shortcuts-results")!;
  queryInput.placeholder = t("keys.overlayPlaceholder");
  queryInput.addEventListener("input", () => {
    renderResults(filterRows(queryInput.value));
  });
}

export function isKeysOverlayOpen(): boolean {
  return open;
}

export async function openKeysOverlay() {
  open = true;
  overlay.classList.remove("hidden");
  queryInput.value = "";
  await refreshList();
  queryInput.focus();
}

export function closeKeysOverlay() {
  open = false;
  overlay.classList.add("hidden");
  queryInput.value = "";
  resultsHost.innerHTML = "";
}
