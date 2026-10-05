import { invoke } from "@tauri-apps/api/core";

interface EditorThemeSummary {
  id: string;
  label: string;
  source: string;
}

interface MonacoTokenRule {
  token: string;
  foreground?: string | null;
  background?: string | null;
  font_style?: string | null;
}

interface MonacoThemeDto {
  id: string;
  base: string;
  inherit: boolean;
  rules: MonacoTokenRule[];
  colors: Record<string, string>;
}

const registered = new Set<string>();

function resolveAutoThemeId(): string {
  return document.documentElement.dataset.theme === "light" ? "vs" : "vs-dark";
}

export function resolveEditorThemeId(prefId: string): string {
  if (prefId === "auto") {
    return resolveAutoThemeId();
  }
  return prefId;
}

async function ensureThemeDefined(themeId: string): Promise<void> {
  if (themeId === "vs" || themeId === "vs-dark" || registered.has(themeId)) {
    return;
  }
  const dto = await invoke<MonacoThemeDto | null>("get_editor_monaco_theme", {
    themeId,
  });
  if (!dto) {
    return;
  }
  const monaco = await import("monaco-editor");
  monaco.editor.defineTheme(dto.id, {
    base: dto.base as "vs" | "vs-dark" | "hc-black" | "hc-light",
    inherit: dto.inherit,
    rules: dto.rules.map((r) => ({
      token: r.token,
      foreground: r.foreground ?? undefined,
      background: r.background ?? undefined,
      fontStyle: r.font_style ?? undefined,
    })),
    colors: dto.colors,
  });
  registered.add(dto.id);
}

export async function applyMonacoEditorTheme(prefId: string): Promise<string> {
  const resolved = resolveEditorThemeId(prefId);
  await ensureThemeDefined(resolved);
  const monaco = await import("monaco-editor");
  monaco.editor.setTheme(resolved);
  document.documentElement.dataset.editorTheme = prefId;
  window.dispatchEvent(
    new CustomEvent("unibrowser-editor-theme", { detail: { themeId: resolved } }),
  );
  return resolved;
}

export async function initMonacoEditorThemes(prefId: string): Promise<void> {
  const themes = await invoke<EditorThemeSummary[]>("list_editor_themes_cmd");
  for (const t of themes) {
    if (t.id.startsWith("vsix:")) {
      await ensureThemeDefined(t.id);
    }
  }
  await applyMonacoEditorTheme(prefId);
}

export async function listEditorThemeOptions(): Promise<EditorThemeSummary[]> {
  return invoke<EditorThemeSummary[]>("list_editor_themes_cmd");
}
