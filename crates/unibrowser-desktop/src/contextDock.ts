import { invoke } from "@tauri-apps/api/core";
import { setAiPanelVisible } from "./aiAssistantPanel";

export type WorkDockContext = "ntp" | "pdf" | "image" | "document" | "browse";

let activeDockContext: WorkDockContext = "browse";

export function setActiveDockContext(ctx: WorkDockContext): void {
  activeDockContext = ctx;
}

export function getActiveDockContext(): WorkDockContext {
  return activeDockContext;
}

interface DockStateSlice {
  show_new_tab_page: boolean;
  text_document?: unknown;
  document_viewer?: { viewer?: string } | null;
}

const STORAGE_PREFIX = "ub-dock-ai-pref:";

function storageKey(ctx: WorkDockContext): string {
  return `${STORAGE_PREFIX}${ctx}`;
}

export function workDockContextFromState(data: DockStateSlice): WorkDockContext {
  if (data.show_new_tab_page) {
    return "ntp";
  }
  const viewer = data.document_viewer?.viewer;
  if (viewer === "pdf") {
    return "pdf";
  }
  if (viewer === "svg" || viewer === "png" || viewer === "image") {
    return "image";
  }
  if (data.text_document || data.document_viewer) {
    return "document";
  }
  return "browse";
}

function readStoredAiVisible(ctx: WorkDockContext): boolean | null {
  try {
    const raw = localStorage.getItem(storageKey(ctx));
    if (raw === "1") {
      return true;
    }
    if (raw === "0") {
      return false;
    }
  } catch {
    /* ignore */
  }
  return null;
}

export function rememberAiVisibleForContext(ctx: WorkDockContext, visible: boolean): void {
  try {
    localStorage.setItem(storageKey(ctx), visible ? "1" : "0");
  } catch {
    /* ignore */
  }
}

/** Apply scenario dock policy: primary task width first; AI only when the user chose it for this context. */
export async function syncContextDockPolicy(data: DockStateSlice): Promise<void> {
  const ctx = workDockContextFromState(data);
  setActiveDockContext(ctx);
  const stored = readStoredAiVisible(ctx);
  const prefs = await invoke<{ panel_visible: boolean }>("get_ai_assistant_prefs");
  const demoRaw = (document.body.dataset.demoState ?? "").toLowerCase();
  const demoForcesAi =
    demoRaw.includes("ai-sidebar") || demoRaw.includes("ai-open") || demoRaw.includes("ai,");

  if (ctx === "ntp" || ctx === "pdf" || ctx === "image") {
    if (demoForcesAi) {
      return;
    }
    const wantAi = stored === true;
    if (prefs.panel_visible !== wantAi) {
      await setAiPanelVisible(wantAi);
    }
    return;
  }

  if (stored !== null && prefs.panel_visible !== stored) {
    await setAiPanelVisible(stored);
  }
}
