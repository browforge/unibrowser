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
  document.body.dataset.cmDensity = dense ? "compact" : "large";
}

export function applyChromeFocusMode(enabled: boolean) {
  document.body.classList.toggle("chrome-focus-mode", enabled);
}

export function applyLargerHitTargets(enabled: boolean) {
  document.documentElement.dataset.largeTargets = enabled ? "true" : "false";
}

export async function loadAccessibilityPreferences() {
  try {
    const prefs = await invoke<{
      high_contrast: boolean;
      reduce_motion: string;
      command_manager_dense: boolean;
      chrome_focus_mode: boolean;
      larger_hit_targets: boolean;
    }>("get_preferences");
    applyHighContrastToDocument(prefs.high_contrast);
    const rm = prefs.reduce_motion?.toLowerCase();
    if (rm === "on" || rm === "off" || rm === "system") {
      reduceMotion = rm;
    }
    applyReduceMotionToDocument();
    applyCommandManagerDensity(prefs.command_manager_dense);
    applyChromeFocusMode(prefs.chrome_focus_mode);
    applyLargerHitTargets(prefs.larger_hit_targets);
  } catch {
    applyReduceMotionToDocument();
    applyCommandManagerDensity(true);
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

export async function setChromeFocusMode(enabled: boolean) {
  await invoke("set_chrome_focus_mode", { enabled });
  applyChromeFocusMode(enabled);
}

export async function setLargerHitTargets(enabled: boolean) {
  await invoke("set_larger_hit_targets", { enabled });
  applyLargerHitTargets(enabled);
}
