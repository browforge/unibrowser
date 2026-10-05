import enNz from "./i18n/en-NZ.json";
import enUs from "./i18n/en-US.json";

const catalogues = { "en-NZ": enNz, "en-US": enUs } as const;

export type LocaleId = keyof typeof catalogues;

let locale: LocaleId = "en-NZ";

export function setLocale(id: LocaleId) {
  locale = id;
  document.documentElement.lang = id;
}

export function localeId(): LocaleId {
  return locale;
}

export function supportedLocales(): LocaleId[] {
  return Object.keys(catalogues) as LocaleId[];
}

export async function loadUiLocale(): Promise<LocaleId> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const prefs = await invoke<{ ui_locale: string }>("get_preferences");
    const id = prefs.ui_locale === "en-US" ? "en-US" : "en-NZ";
    setLocale(id);
    return id;
  } catch {
    setLocale("en-NZ");
    return "en-NZ";
  }
}

export function t(key: keyof typeof enNz, vars?: Record<string, string | number>): string {
  let text: string = catalogues[locale][key] ?? enNz[key] ?? key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      text = text.replace(`{${k}}`, String(v));
    }
  }
  return text;
}
