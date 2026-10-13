import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { showToast } from "./toast";

/** File → Save as template… */
export async function runSaveProjectAsTemplate(projectPath?: string | null): Promise<void> {
  const state = await invoke<{ omnibox: string }>("get_browser_state");
  const path = (projectPath ?? state.omnibox).replace(/^file:\/\//, "");
  if (!path.includes("unibrowser-memory://") && !path.toLowerCase().endsWith(".unibrow")) {
    showToast(t("saveAsTemplate.needProject"), "info");
    return;
  }
  const suggested = "MyTemplate.ubtemplate";
  const dest = await invoke<string | null>("pick_save_ubtemplate_path", {
    suggestedName: suggested,
  });
  if (!dest) {
    return;
  }
  await invoke("save_project_as_template", {
    projectPath: path,
    destPath: dest,
    title: null,
    description: null,
  });
  showToast(t("saveAsTemplate.saved", { path: dest }), "info");
}

/** Doc capture overlay (no native file dialog). */
export function showSaveAsTemplateCaptureOverlay(): void {
  const overlay = document.createElement("div");
  overlay.className = "save-template-overlay";
  const panel = document.createElement("div");
  panel.className = "save-template-panel";
  panel.setAttribute("role", "dialog");
  const title = document.createElement("h2");
  title.textContent = t("saveAsTemplate.title");
  const label = document.createElement("label");
  label.textContent = t("saveAsTemplate.fileName");
  const input = document.createElement("input");
  input.type = "text";
  input.className = "ub-form-control";
  input.value = "Studio-starter.ubtemplate";
  label.appendChild(input);
  const actions = document.createElement("div");
  actions.className = "save-template-actions";
  const save = document.createElement("button");
  save.type = "button";
  save.className = "primary";
  save.textContent = t("saveAsTemplate.save");
  actions.appendChild(save);
  panel.append(title, label, actions);
  overlay.appendChild(panel);
  document.body.appendChild(overlay);
}
