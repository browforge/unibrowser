import enNz from "./i18n/en-NZ.json";

const catalogues = { "en-NZ": enNz } as const;

export type LocaleId = keyof typeof catalogues;

let locale: LocaleId = "en-NZ";

export function setLocale(id: LocaleId) {
  locale = id;
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
