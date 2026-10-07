import { invoke } from "@tauri-apps/api/core";

export type ResolvedTheme = "light" | "dark";

let storedPreference: "light" | "dark" | "system" = "system";
let media: MediaQueryList | null = null;
let userChoseTheme = false;

function systemTheme(): ResolvedTheme {
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

export function resolveTheme(): ResolvedTheme {
  if (storedPreference === "light" || storedPreference === "dark") {
    return storedPreference;
  }
  return systemTheme();
}

export function applyThemeToDocument() {
  const resolved = resolveTheme();
  document.documentElement.dataset.theme = resolved;
  document.documentElement.style.colorScheme = resolved;
  window.dispatchEvent(new CustomEvent("unibrowser-theme", { detail: resolved }));
}

export function applyThemeOverride(theme: "light" | "dark") {
  if (userChoseTheme) {
    return;
  }
  storedPreference = theme;
  applyThemeToDocument();
}

/** Apply a user-chosen theme without re-reading demo-state overrides. */
export function applyUserThemePreference(theme: "light" | "dark" | "system") {
  userChoseTheme = true;
  storedPreference = theme;
  applyThemeToDocument();
}

export function applyThemeFromDemoState(raw: string) {
  if (userChoseTheme) {
    return false;
  }
  const lower = raw.toLowerCase();
  if (lower.includes("theme-light")) {
    applyThemeOverride("light");
    return true;
  }
  if (lower.includes("theme-dark")) {
    applyThemeOverride("dark");
    return true;
  }
  return false;
}

export async function loadThemePreference() {
  try {
    if (!userChoseTheme) {
      const demo = await invoke<string | null>("get_demo_state");
      if (demo && applyThemeFromDemoState(demo)) {
        return;
      }
    }
    const prefs = await invoke<{ theme: string }>("get_preferences");
    const t = prefs.theme?.toLowerCase();
    if (t === "light" || t === "dark" || t === "system") {
      storedPreference = t;
    }
  } catch {
    storedPreference = "system";
  }
  if (!media) {
    media = window.matchMedia("(prefers-color-scheme: light)");
    media.addEventListener("change", () => {
      if (storedPreference === "system") {
        applyThemeToDocument();
      }
    });
  }
  applyThemeToDocument();
}

export async function cycleTheme(): Promise<string> {
  const order: ("light" | "dark" | "system")[] = ["light", "dark", "system"];
  const idx = order.indexOf(storedPreference);
  storedPreference = order[(idx + 1) % order.length];
  userChoseTheme = true;
  await invoke("set_ui_theme", { theme: storedPreference });
  applyThemeToDocument();
  return storedPreference;
}

export function themeButtonLabel(): string {
  if (storedPreference === "system") {
    return `Theme: system (${resolveTheme()})`;
  }
  return `Theme: ${storedPreference}`;
}
