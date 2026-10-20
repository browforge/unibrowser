import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { createLucideIcon } from "./ui/lucideIcon";
import {
  defaultFirstSaveFileName,
  defaultFirstSaveFolderDisplay,
  formatFirstSavePathPreview,
} from "./firstSaveDefaults";

export type FirstSaveMode = "archive" | "folder";

export type FirstSaveResult = {
  mode: FirstSaveMode;
  destPath: string;
  fileName: string;
};

type FirstSaveDialogOptions = {
  projectTitle?: string;
  captureOnly?: boolean;
};

function mountFirstSaveDialog(options: FirstSaveDialogOptions): Promise<FirstSaveResult | null> {
  const { projectTitle, captureOnly = false } = options;
  return new Promise((resolve) => {
    const overlay = document.createElement("div");
    overlay.className = "first-save-overlay";
    const panel = document.createElement("div");
    panel.className = "first-save-panel";
    panel.style.maxWidth = "min(480px, 92vw)";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");

    const title = document.createElement("h2");
    title.textContent = t("firstSave.title");
    panel.appendChild(title);

    const lead = document.createElement("p");
    lead.className = "viewer-note first-save-lead";
    lead.textContent = t("firstSave.lead");
    panel.appendChild(lead);

    const locationField = document.createElement("div");
    locationField.className = "first-save-field";
    const locationLabel = document.createElement("span");
    locationLabel.className = "first-save-field-label";
    locationLabel.textContent = t("firstSave.locationLabel");
    const locationRow = document.createElement("div");
    locationRow.className = "first-save-location-row";
    const locationInput = document.createElement("input");
    locationInput.type = "text";
    locationInput.className = "ub-form-control";
    locationInput.readOnly = true;
    locationInput.value = defaultFirstSaveFolderDisplay();
    const browse = document.createElement("button");
    browse.type = "button";
    browse.className = "quiet-button";
    browse.textContent = t("firstSave.browse");
    locationRow.append(locationInput, browse);
    locationField.append(locationLabel, locationRow);
    panel.appendChild(locationField);

    const nameField = document.createElement("div");
    nameField.className = "first-save-field";
    const nameLabel = document.createElement("span");
    nameLabel.className = "first-save-field-label";
    nameLabel.textContent = t("firstSave.nameLabel");
    const nameInput = document.createElement("input");
    nameInput.type = "text";
    nameInput.className = "ub-form-control";
    nameInput.value = defaultFirstSaveFileName(projectTitle).replace(/\.unibrow$/i, "");
    nameField.append(nameLabel, nameInput);
    panel.appendChild(nameField);

    let mode: FirstSaveMode = "archive";
    let locationPath = "";

    const preview = document.createElement("p");
    preview.className = "first-save-path-preview";
    const syncPreview = () => {
      const stem = nameInput.value.trim() || t("project.emptyDefaultTitle");
      const fileName = mode === "archive" ? `${stem}.unibrow` : stem;
      preview.textContent = formatFirstSavePathPreview(
        locationInput.value || locationPath || defaultFirstSaveFolderDisplay(),
        mode,
        fileName,
      );
    };

    const modeGroup = document.createElement("div");
    modeGroup.className = "first-save-modes";
    modeGroup.setAttribute("role", "radiogroup");
    modeGroup.setAttribute("aria-label", t("firstSave.modeLabel"));
    const modes: {
      id: FirstSaveMode;
      label: string;
      hint: string;
      icon: "package-open" | "folder";
    }[] = [
      {
        id: "archive",
        label: t("firstSave.modeArchive"),
        hint: t("firstSave.modeArchiveHint"),
        icon: "package-open",
      },
      {
        id: "folder",
        label: t("firstSave.modeFolder"),
        hint: t("firstSave.modeFolderHint"),
        icon: "folder",
      },
    ];
    modes.forEach((entry) => {
      const row = document.createElement("label");
      row.className = "first-save-mode-card";
      if (entry.id === mode) {
        row.classList.add("first-save-mode-card--selected");
      }
      const radio = document.createElement("input");
      radio.type = "radio";
      radio.name = "first-save-mode";
      radio.value = entry.id;
      radio.checked = entry.id === mode;
      radio.onchange = () => {
        mode = entry.id;
        modeGroup.querySelectorAll(".first-save-mode-card").forEach((el) => {
          el.classList.toggle(
            "first-save-mode-card--selected",
            el.querySelector<HTMLInputElement>("input[type=radio]")?.value === mode,
          );
        });
        syncPreview();
      };
      const icon = createLucideIcon(entry.icon, 20, "ub-icon svg-icon");
      const text = document.createElement("div");
      text.className = "first-save-mode-card-text";
      const strong = document.createElement("strong");
      strong.textContent = entry.label;
      const hint = document.createElement("span");
      hint.className = "first-save-mode-card-hint";
      hint.textContent = entry.hint;
      text.append(strong, hint);
      row.append(radio, icon, text);
      modeGroup.appendChild(row);
    });
    panel.appendChild(modeGroup);

    const previewLabel = document.createElement("span");
    previewLabel.className = "first-save-field-label";
    previewLabel.textContent = t("firstSave.pathPreviewLabel");
    panel.append(previewLabel, preview);

    const actions = document.createElement("div");
    actions.className = "first-save-actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.className = "quiet-button";
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
      if (captureOnly) {
        return;
      }
      const stem = nameInput.value.trim();
      if (!stem) {
        nameInput.focus();
        return;
      }
      const folder = (locationPath || locationInput.value).replace(/\/$/, "");
      close({
        mode,
        destPath: folder,
        fileName: mode === "archive" ? `${stem}.unibrow` : stem,
      });
    };
    browse.onclick = () => {
      if (captureOnly) {
        return;
      }
      void invoke<string | null>("pick_browse_folder").then((picked) => {
        if (picked) {
          locationPath = picked;
          locationInput.value = picked;
          syncPreview();
        }
      });
    };
    nameInput.addEventListener("input", syncPreview);
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay && !captureOnly) {
        close(null);
      }
    });
    actions.append(cancel, save);
    panel.appendChild(actions);
    overlay.appendChild(panel);
    document.body.appendChild(overlay);
    nameInput.focus();
    nameInput.select();

    void invoke<string>("get_home_dir")
      .then((home) => {
        locationPath = `${home.replace(/\/$/, "")}/Documents/Unibrowser`;
        locationInput.value = locationPath;
        syncPreview();
      })
      .catch(() => {
        syncPreview();
      });
  });
}

export function promptFirstSaveDialog(projectTitle?: string): Promise<FirstSaveResult | null> {
  return mountFirstSaveDialog({ projectTitle });
}

/** Doc capture: first-save dialog without native picker (NEW-09 evidence). */
export function showFirstSaveCaptureOverlay(projectTitle?: string): void {
  void mountFirstSaveDialog({ projectTitle, captureOnly: true });
}

export async function runFirstSaveFlow(
  projectTitle?: string,
  pickDest?: (mode: FirstSaveMode, fileName: string) => Promise<string | null>,
): Promise<unknown | null> {
  const draft = await promptFirstSaveDialog(projectTitle);
  if (!draft) {
    return null;
  }
  let dest = draft.destPath.trim();
  if (!dest) {
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
    const picked = await picker(draft.mode, draft.fileName);
    if (!picked) {
      return null;
    }
    dest = picked.replace(/\/$/, "");
  }
  const destPath =
    draft.mode === "folder"
      ? `${dest}/${draft.fileName}`
      : `${dest}/${draft.fileName}`;
  return invoke("save_memory_project", {
    input: { dest_path: destPath, mode: draft.mode },
  });
}
