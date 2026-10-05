import { invoke } from "@tauri-apps/api/core";

export type ReduceMotionPref = "system" | "on" | "off";

let reduceMotion: ReduceMotionPref = "system";
let motionMedia: MediaQueryList | null = null;

function systemPrefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function applyReduceMotionToDocument() {
  const active =
    reduceMotion === "on" || (reduceMotion === "system" && systemPrefersReducedMotion());
  document.documentElement.dataset.reduceMotion = active ? "true" : "false";
}

export function applyHighContrastToDocument(enabled: boolean) {
  document.documentElement.dataset.highContrast = enabled ? "true" : "false";
}

export function applyCommandManagerDensity(dense: boolean) {
  const el = document.getElementById("command-manager");
  if (el) {
    el.classList.toggle("command-manager-dense", dense);
  }
}

export async function loadAccessibilityPreferences() {
  try {
    const prefs = await invoke<{
      high_contrast: boolean;
      reduce_motion: string;
      command_manager_dense: boolean;
    }>("get_preferences");
    applyHighContrastToDocument(prefs.high_contrast);
    const rm = prefs.reduce_motion?.toLowerCase();
    if (rm === "on" || rm === "off" || rm === "system") {
      reduceMotion = rm;
    }
    applyReduceMotionToDocument();
    applyCommandManagerDensity(prefs.command_manager_dense);
  } catch {
    applyReduceMotionToDocument();
  }
  if (!motionMedia) {
    motionMedia = window.matchMedia("(prefers-reduced-motion: reduce)");
    motionMedia.addEventListener("change", () => {
      if (reduceMotion === "system") {
        applyReduceMotionToDocument();
      }
    });
  }
}

export async function setHighContrast(enabled: boolean) {
  await invoke("set_accessibility_prefs", { high_contrast: enabled, reduce_motion: null });
  applyHighContrastToDocument(enabled);
}

export async function setReduceMotion(pref: ReduceMotionPref) {
  await invoke("set_accessibility_prefs", { highContrast: null, reduceMotion: pref });
  reduceMotion = pref;
  applyReduceMotionToDocument();
}

export async function setCommandManagerDense(dense: boolean) {
  await invoke("set_command_manager_dense", { dense });
  applyCommandManagerDensity(dense);
}
