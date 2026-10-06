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

type GrantRow = AiPrefs["grants"][number];

const PERMISSIONS = ["read", "write", "ask"] as const;
const SCOPES = ["global", "workspace", "file_type", "site"] as const;

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
    const grantsNote = document.createElement("p");
    grantsNote.className = "viewer-note";
    grantsNote.textContent = t("settings.aiGrantsNote");
    section.appendChild(grantsNote);

    const grantsTable = document.createElement("div");
    grantsTable.className = "ai-grants-table";
    section.appendChild(grantsTable);

    const renderGrants = () => {
      grantsTable.innerHTML = "";
      prefs.grants.forEach((g, index) => {
        const row = document.createElement("div");
        row.className = "ai-grants-row";

        const perm = document.createElement("select");
        for (const p of PERMISSIONS) {
          const opt = document.createElement("option");
          opt.value = p;
          opt.textContent = p;
          perm.appendChild(opt);
        }
        perm.value = g.permission;
        perm.onchange = () => {
          g.permission = perm.value;
        };

        const scope = document.createElement("select");
        for (const s of SCOPES) {
          const opt = document.createElement("option");
          opt.value = s;
          opt.textContent = s;
          scope.appendChild(opt);
        }
        scope.value = g.scope_kind;
        scope.onchange = () => {
          g.scope_kind = scope.value;
        };

        const value = document.createElement("input");
        value.type = "text";
        value.placeholder = t("settings.aiGrantScopeValue");
        value.value = g.scope_value ?? "";
        value.disabled = g.scope_kind === "global";
        value.oninput = () => {
          g.scope_value = value.value.trim() || null;
        };
        scope.onchange = () => {
          g.scope_kind = scope.value;
          value.disabled = g.scope_kind === "global";
          if (g.scope_kind === "global") {
            g.scope_value = null;
            value.value = "";
          }
        };

        const revoke = document.createElement("button");
        revoke.type = "button";
        revoke.textContent = t("settings.aiRevokeGrant");
        revoke.onclick = () => {
          prefs.grants.splice(index, 1);
          renderGrants();
        };

        row.appendChild(perm);
        row.appendChild(scope);
        row.appendChild(value);
        row.appendChild(revoke);
        grantsTable.appendChild(row);
      });
    };
    renderGrants();

    const addRow = document.createElement("div");
    addRow.className = "ai-grants-add-row";
    for (const p of PERMISSIONS) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = t("settings.aiAddGrant", { permission: p });
      btn.onclick = () => {
        const row: GrantRow = { permission: p, scope_kind: "global", scope_value: null };
        prefs.grants.push(row);
        renderGrants();
      };
      addRow.appendChild(btn);
    }
    section.appendChild(addRow);

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
