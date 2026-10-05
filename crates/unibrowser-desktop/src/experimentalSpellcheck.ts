import { invoke } from "@tauri-apps/api/core";

let enabled = false;
let pane: HTMLTextAreaElement | null = null;

export async function loadExperimentalSpellcheckPref(): Promise<boolean> {
  try {
    const prefs = await invoke<{ advanced_experimental_spellcheck: boolean }>("get_preferences");
    enabled = prefs.advanced_experimental_spellcheck;
  } catch {
    enabled = false;
  }
  return enabled;
}

export function isExperimentalSpellcheckEnabled(): boolean {
  return enabled;
}

export function setExperimentalSpellcheckEnabled(value: boolean): void {
  enabled = value;
  window.dispatchEvent(new CustomEvent("unibrowser-spellcheck-pref", { detail: value }));
  if (!value) {
    detachSpellcheckPane();
  }
}

export function attachSpellcheckPane(hostParent: HTMLElement | null): void {
  if (!enabled || !hostParent) {
    detachSpellcheckPane();
    return;
  }
  if (pane && pane.parentElement === hostParent) {
    return;
  }
  detachSpellcheckPane();
  pane = document.createElement("textarea");
  pane.className = "spellcheck-experiment-pane";
  pane.readOnly = true;
  pane.spellcheck = true;
  pane.setAttribute("lang", "en-NZ");
  pane.setAttribute("aria-label", "Browser spellcheck mirror (experimental)");
  hostParent.appendChild(pane);
}

export function syncSpellcheckPane(text: string): void {
  if (!pane) {
    return;
  }
  if (pane.value !== text) {
    pane.value = text;
  }
}

export function detachSpellcheckPane(): void {
  pane?.remove();
  pane = null;
}

export function applySpellcheckToEditable(el: HTMLElement | null, on: boolean): void {
  if (!el) {
    return;
  }
  if (on && enabled) {
    el.spellcheck = true;
    if (el instanceof HTMLTextAreaElement) {
      return;
    }
    if (el.isContentEditable) {
      return;
    }
    el.setAttribute("spellcheck", "true");
    return;
  }
  el.spellcheck = false;
  el.removeAttribute("spellcheck");
}
