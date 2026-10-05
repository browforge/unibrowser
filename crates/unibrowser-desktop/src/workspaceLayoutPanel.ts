import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";

interface PresetSummary {
  id: string;
  name: string;
  saved_at_ms: number;
}

export async function appendWorkspaceLayoutPanel(root: HTMLElement) {
  const status = document.createElement("p");
  status.className = "viewer-note workspace-layout-status";
  status.hidden = true;

  const saveRow = document.createElement("div");
  saveRow.className = "settings-row";
  const nameInput = document.createElement("input");
  nameInput.type = "text";
  nameInput.className = "workspace-layout-name";
  nameInput.placeholder = t("workspaceLayout.namePlaceholder");
  const saveBtn = document.createElement("button");
  saveBtn.type = "button";
  saveBtn.textContent = t("workspaceLayout.save");
  saveRow.appendChild(nameInput);
  saveRow.appendChild(saveBtn);

  const list = document.createElement("ul");
  list.className = "workspace-layout-list";

  const empty = document.createElement("p");
  empty.className = "viewer-note";
  empty.textContent = t("workspaceLayout.empty");

  const refresh = async () => {
    list.replaceChildren();
    const presets = await invoke<PresetSummary[]>("list_workspace_layout_presets");
    if (!presets.length) {
      empty.hidden = false;
      list.hidden = true;
      return;
    }
    empty.hidden = true;
    list.hidden = false;
    for (const preset of presets) {
      const li = document.createElement("li");
      li.className = "workspace-layout-row";
      const label = document.createElement("span");
      label.textContent = preset.name;
      const actions = document.createElement("div");
      actions.className = "workspace-layout-actions";
      const loadBtn = document.createElement("button");
      loadBtn.type = "button";
      loadBtn.textContent = t("workspaceLayout.load");
      loadBtn.onclick = async () => {
        loadBtn.disabled = true;
        try {
          await invoke("load_workspace_layout_preset", { id: preset.id });
          status.textContent = t("workspaceLayout.loaded", { name: preset.name });
          status.hidden = false;
          window.dispatchEvent(new CustomEvent("unibrowser-workspace-layout-loaded"));
        } catch (err) {
          status.textContent =
            err instanceof Error ? err.message : t("toast.errorDefault");
          status.hidden = false;
        } finally {
          loadBtn.disabled = false;
        }
      };
      const delBtn = document.createElement("button");
      delBtn.type = "button";
      delBtn.textContent = t("workspaceLayout.delete");
      delBtn.onclick = async () => {
        delBtn.disabled = true;
        try {
          await invoke("delete_workspace_layout_preset", { id: preset.id });
          status.textContent = t("workspaceLayout.deleted", { name: preset.name });
          status.hidden = false;
          await refresh();
        } catch (err) {
          status.textContent =
            err instanceof Error ? err.message : t("toast.errorDefault");
          status.hidden = false;
        } finally {
          delBtn.disabled = false;
        }
      };
      actions.appendChild(loadBtn);
      actions.appendChild(delBtn);
      li.appendChild(label);
      li.appendChild(actions);
      list.appendChild(li);
    }
  };

  saveBtn.onclick = async () => {
    const name = nameInput.value.trim();
    if (!name) {
      status.textContent = t("workspaceLayout.nameRequired");
      status.hidden = false;
      return;
    }
    saveBtn.disabled = true;
    try {
      await invoke("save_workspace_layout_preset", { name });
      nameInput.value = "";
      status.textContent = t("workspaceLayout.saved", { name });
      status.hidden = false;
      await refresh();
    } catch (err) {
      status.textContent =
        err instanceof Error ? err.message : t("toast.errorDefault");
      status.hidden = false;
    } finally {
      saveBtn.disabled = false;
    }
  };

  root.appendChild(saveRow);
  root.appendChild(status);
  root.appendChild(empty);
  root.appendChild(list);
  await refresh();
}
