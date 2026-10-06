import { invoke } from "@tauri-apps/api/core";
import { showErrorToast } from "./toast";
import { t } from "./i18n";

interface AiPrefs {
  base_url?: string | null;
  model?: string | null;
  api_key?: string | null;
  grants: { permission: string; scope_kind: string; scope_value?: string | null }[];
  panel_visible: boolean;
  panel_x: number;
  panel_y: number;
  panel_docked: boolean;
}

export function renderAiAssistantSettings(root: HTMLElement) {
  const section = document.createElement("section");
  section.className = "settings-section";
  section.dataset.settingsSection = "ai-assistant";
  const heading = document.createElement("h3");
  heading.textContent = t("settings.aiAssistantHeading");
  section.appendChild(heading);

  void invoke<AiPrefs>("get_ai_assistant_prefs").then((prefs) => {
    const urlRow = document.createElement("label");
    urlRow.textContent = t("settings.aiBaseUrl");
    const urlInput = document.createElement("input");
    urlInput.type = "url";
    urlInput.placeholder = "http://127.0.0.1:11434/v1";
    urlInput.value = prefs.base_url ?? "";
    section.appendChild(urlRow);
    section.appendChild(urlInput);

    const modelRow = document.createElement("label");
    modelRow.textContent = t("settings.aiModel");
    const modelInput = document.createElement("input");
    modelInput.type = "text";
    modelInput.value = prefs.model ?? "";
    section.appendChild(modelRow);
    section.appendChild(modelInput);

    const keyRow = document.createElement("label");
    keyRow.textContent = t("settings.aiApiKey");
    const keyInput = document.createElement("input");
    keyInput.type = "password";
    keyInput.autocomplete = "off";
    keyInput.value = prefs.api_key ?? "";
    section.appendChild(keyRow);
    section.appendChild(keyInput);

    const grantsHead = document.createElement("h4");
    grantsHead.textContent = t("settings.aiGrants");
    section.appendChild(grantsHead);
    const grantsList = document.createElement("div");
    grantsList.className = "ai-grants-list";
    const renderGrants = () => {
      grantsList.innerHTML = "";
      for (const g of prefs.grants) {
        const row = document.createElement("div");
        row.textContent = `${g.permission} · ${g.scope_kind}${g.scope_value ? ` (${g.scope_value})` : ""}`;
        grantsList.appendChild(row);
      }
    };
    renderGrants();
    section.appendChild(grantsList);

    const addGrant = document.createElement("button");
    addGrant.type = "button";
    addGrant.textContent = t("settings.aiAddReadGlobal");
    addGrant.onclick = () => {
      prefs.grants.push({ permission: "read", scope_kind: "global", scope_value: null });
      renderGrants();
    };
    section.appendChild(addGrant);

    const saveBtn = document.createElement("button");
    saveBtn.type = "button";
    saveBtn.textContent = t("settings.aiSave");
    saveBtn.onclick = async () => {
      prefs.base_url = urlInput.value.trim() || null;
      prefs.model = modelInput.value.trim() || null;
      prefs.api_key = keyInput.value.trim() || null;
      try {
        await invoke("set_ai_assistant_prefs", { prefs });
      } catch (e) {
        showErrorToast(e);
      }
    };
    section.appendChild(saveBtn);

    const testBtn = document.createElement("button");
    testBtn.type = "button";
    testBtn.textContent = t("settings.aiTestConnection");
    testBtn.onclick = async () => {
      try {
        const reply = await invoke<string>("test_ai_connection", {
          baseUrl: urlInput.value.trim() || null,
          model: modelInput.value.trim() || null,
          apiKey: keyInput.value.trim() || null,
        });
        testBtn.textContent = reply.slice(0, 80);
      } catch (e) {
        showErrorToast(e);
      }
    };
    section.appendChild(testBtn);
  });

  root.appendChild(section);
}
