import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
export type FirstSaveMode = "archive" | "folder";

export type FirstSaveResult = {
  mode: FirstSaveMode;
  destPath: string;
  fileName: string;
};

export function defaultFirstSaveFileName(projectTitle?: string): string {
  const base = (projectTitle ?? t("project.emptyDefaultTitle")).trim() || t("project.emptyDefaultTitle");
  return `${base}.unibrow`;
}

export function promptFirstSaveDialog(projectTitle?: string): Promise<FirstSaveResult | null> {
  return new Promise((resolve) => {
    const overlay = document.createElement("div");
    overlay.className = "first-save-overlay";
    const panel = document.createElement("div");
    panel.className = "first-save-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");

    const title = document.createElement("h2");
    title.textContent = t("firstSave.title");
    panel.appendChild(title);

    const lead = document.createElement("p");
    lead.className = "viewer-note";
    lead.textContent = t("firstSave.lead");
    panel.appendChild(lead);

    const nameLabel = document.createElement("label");
    nameLabel.textContent = t("firstSave.nameLabel");
    const nameInput = document.createElement("input");
    nameInput.type = "text";
    nameInput.className = "ub-form-control";
    nameInput.value = defaultFirstSaveFileName(projectTitle).replace(/\.unibrow$/i, "");
    nameLabel.appendChild(nameInput);
    panel.appendChild(nameLabel);

    let mode: FirstSaveMode = "archive";
    const modeGroup = document.createElement("div");
    modeGroup.className = "first-save-modes";
    modeGroup.setAttribute("role", "radiogroup");
    modeGroup.setAttribute("aria-label", t("firstSave.modeLabel"));
    const modes: { id: FirstSaveMode; label: string; hint: string }[] = [
      { id: "archive", label: t("firstSave.modeArchive"), hint: t("firstSave.modeArchiveHint") },
      { id: "folder", label: t("firstSave.modeFolder"), hint: t("firstSave.modeFolderHint") },
    ];
    modes.forEach((entry) => {
      const row = document.createElement("label");
      row.className = "first-save-mode-row";
      const radio = document.createElement("input");
      radio.type = "radio";
      radio.name = "first-save-mode";
      radio.value = entry.id;
      radio.checked = entry.id === mode;
      radio.onchange = () => {
        mode = entry.id;
      };
      const strong = document.createElement("strong");
      strong.textContent = entry.label;
      const hint = document.createElement("span");
      hint.textContent = ` — ${entry.hint}`;
      row.append(radio, strong, hint);
      modeGroup.appendChild(row);
    });
    panel.appendChild(modeGroup);

    const actions = document.createElement("div");
    actions.className = "first-save-actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = t("project.cancel");
    const save = document.createElement("button");
    save.type = "button";
    save.className = "primary";
    save.textContent = t("firstSave.save");
    const close = (result: FirstSaveResult | null) => {
      overlay.remove();
      resolve(result);
    };
    cancel.onclick = () => close(null);
    save.onclick = () => {
      const stem = nameInput.value.trim();
      if (!stem) {
        nameInput.focus();
        return;
      }
      close({
        mode,
        destPath: "",
        fileName: mode === "archive" ? `${stem}.unibrow` : stem,
      });
    };
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) {
        close(null);
      }
    });
    actions.append(cancel, save);
    panel.appendChild(actions);
    overlay.appendChild(panel);
    document.body.appendChild(overlay);
    nameInput.focus();
    nameInput.select();
  });
}

/** Doc capture: first-save dialog without native picker (NEW-09 evidence). */
export function showFirstSaveCaptureOverlay(projectTitle?: string): void {
  const overlay = document.createElement("div");
  overlay.className = "first-save-overlay";
  const panel = document.createElement("div");
  panel.className = "first-save-panel";
  panel.setAttribute("role", "dialog");
  const title = document.createElement("h2");
  title.textContent = t("firstSave.title");
  const lead = document.createElement("p");
  lead.className = "viewer-note";
  lead.textContent = t("firstSave.lead");
  const nameLabel = document.createElement("label");
  nameLabel.textContent = t("firstSave.nameLabel");
  const nameInput = document.createElement("input");
  nameInput.type = "text";
  nameInput.className = "ub-form-control";
  nameInput.value = defaultFirstSaveFileName(projectTitle).replace(/\.unibrow$/i, "");
  nameLabel.appendChild(nameInput);
  const archiveRow = document.createElement("label");
  archiveRow.className = "first-save-mode-row";
  const archiveRadio = document.createElement("input");
  archiveRadio.type = "radio";
  archiveRadio.name = "first-save-mode";
  archiveRadio.checked = true;
  archiveRow.append(
    archiveRadio,
    document.createTextNode(` ${t("firstSave.modeArchive")} — ${t("firstSave.modeArchiveHint")}`),
  );
  const folderRow = document.createElement("label");
  folderRow.className = "first-save-mode-row";
  const folderRadio = document.createElement("input");
  folderRadio.type = "radio";
  folderRadio.name = "first-save-mode";
  folderRow.append(
    folderRadio,
    document.createTextNode(` ${t("firstSave.modeFolder")} — ${t("firstSave.modeFolderHint")}`),
  );
  const actions = document.createElement("div");
  actions.className = "first-save-actions";
  const save = document.createElement("button");
  save.type = "button";
  save.className = "primary";
  save.textContent = t("firstSave.save");
  actions.appendChild(save);
  panel.append(title, lead, nameLabel, archiveRow, folderRow, actions);
  overlay.appendChild(panel);
  document.body.appendChild(overlay);
}

export async function runFirstSaveFlow(
  projectTitle?: string,
  pickDest?: (mode: FirstSaveMode, fileName: string) => Promise<string | null>,
): Promise<unknown | null> {
  const draft = await promptFirstSaveDialog(projectTitle);
  if (!draft) {
    return null;
  }
  const picker =
    pickDest ??
    (async (mode, fileName) => {
      if (mode === "folder") {
        return invoke<string | null>("pick_browse_folder");
      }
      return invoke<string | null>("pick_save_unibrow_project_path", {
        suggestedName: fileName,
      });
    });
  const dest = await picker(draft.mode, draft.fileName);
  if (!dest) {
    return null;
  }
  const destPath =
    draft.mode === "folder" ? `${dest.replace(/\/$/, "")}/${draft.fileName}` : dest;
  return invoke("save_memory_project", {
    input: { dest_path: destPath, mode: draft.mode },
  });
}
