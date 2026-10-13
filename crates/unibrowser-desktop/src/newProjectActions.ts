import { invoke } from "@tauri-apps/api/core";

export async function invokeNewEmptyProject(title?: string) {
  return invoke("new_empty_project", { title: title ?? null });
}

export async function invokeNewProjectFromTemplate(
  templateId: string,
  projectName: string,
  location: string,
) {
  return invoke("new_project_from_template", {
    templateId,
    projectName,
    location: location || null,
  });
}
