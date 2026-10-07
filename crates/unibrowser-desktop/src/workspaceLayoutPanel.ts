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
          const data = await invoke("load_workspace_layout_preset", { id: preset.id });
          status.textContent = t("workspaceLayout.loaded", { name: preset.name });
          status.hidden = false;
          window.dispatchEvent(new CustomEvent("unibrowser-workspace-layout-loaded"));
          window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
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

  const transferRow = document.createElement("div");
  transferRow.className = "workspace-layout-actions workspace-layout-transfer";
  const importBtn = document.createElement("button");
  importBtn.type = "button";
  importBtn.textContent = "Import layouts";
  importBtn.onclick = async () => {
    try {
      const count = await invoke<number>("import_workspace_layouts_file");
      status.textContent = `Imported ${count} saved layout${count === 1 ? "" : "s"}.`;
      status.hidden = false;
      await refresh();
    } catch (err) {
      status.textContent = err instanceof Error ? err.message : String(err);
      status.hidden = false;
    }
  };
  const exportBtn = document.createElement("button");
  exportBtn.type = "button";
  exportBtn.textContent = "Export layouts";
  exportBtn.onclick = async () => {
    try {
      const path = await invoke<string | null>("export_workspace_layouts_file");
      if (!path) {
        return;
      }
      status.textContent = `Exported saved layouts to ${path}.`;
      status.hidden = false;
    } catch (err) {
      status.textContent = err instanceof Error ? err.message : String(err);
      status.hidden = false;
    }
  };
  transferRow.append(importBtn, exportBtn);

  const personaSection = document.createElement("div");
  personaSection.className = "workspace-personas";
  const personaHeading = document.createElement("h4");
  personaHeading.textContent = t("workspacePersona.heading");
  const personaLead = document.createElement("p");
  personaLead.className = "viewer-note";
  personaLead.textContent = t("workspacePersona.lead");
  personaSection.appendChild(personaHeading);
  personaSection.appendChild(personaLead);
  const personas = await invoke<Array<{ id: string; name: string; description: string }>>(
    "list_workspace_personas",
  );
  const personaList = document.createElement("ul");
  personaList.className = "workspace-persona-list";
  for (const p of personas) {
    const li = document.createElement("li");
    const label = document.createElement("span");
    label.textContent = `${p.name} - ${p.description}`;
    const applyBtn = document.createElement("button");
    applyBtn.type = "button";
    applyBtn.textContent = t("workspacePersona.apply");
    applyBtn.onclick = async () => {
      applyBtn.disabled = true;
      try {
        await invoke("apply_workspace_persona", { personaId: p.id });
        status.textContent = t("workspaceLayout.loaded", { name: p.name });
        status.hidden = false;
        window.dispatchEvent(new CustomEvent("unibrowser-workspace-layout-loaded"));
      } catch (err) {
        status.textContent =
          err instanceof Error ? err.message : t("toast.errorDefault");
        status.hidden = false;
      } finally {
        applyBtn.disabled = false;
      }
    };
    li.appendChild(label);
    li.appendChild(applyBtn);
    personaList.appendChild(li);
  }
  personaSection.appendChild(personaList);

  root.appendChild(personaSection);
  root.appendChild(saveRow);
  root.appendChild(transferRow);
  root.appendChild(status);
  root.appendChild(empty);
  root.appendChild(list);
  await refresh();
}
