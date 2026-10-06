import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { showErrorToast, showToast } from "./toast";

export async function appendDeveloperSettingsSection(root: HTMLElement) {
  const section = document.createElement("details");
  section.className = "settings-section settings-developer";
  section.setAttribute("data-settings-section", "developer");
  const summary = document.createElement("summary");
  summary.textContent = t("settings.developer");
  section.appendChild(summary);

  const lead = document.createElement("p");
  lead.className = "viewer-note";
  lead.textContent = t("settings.developerLead");
  section.appendChild(lead);

  const modsLine = document.createElement("p");
  modsLine.className = "settings-developer-mods";
  section.appendChild(modsLine);

  try {
    const info = await invoke<{ mods_loaded: string; mod_catalog_count: number }>(
      "get_diagnostics_info",
    );
    modsLine.textContent = t("settings.developerMods", {
      list: info.mods_loaded,
      count: String(info.mod_catalog_count),
    });
  } catch {
    modsLine.textContent = t("settings.developerModsUnavailable");
  }

  const row = document.createElement("div");
  row.className = "settings-row settings-button-row";

  const demoBtn = document.createElement("button");
  demoBtn.type = "button";
  demoBtn.textContent = t("chrome.demoDownload");
  demoBtn.onclick = async () => {
    try {
      await invoke("handle_download_url", {
        url: "https://example.com/",
        destination: null,
      });
      showToast(t("settings.developerDemoStarted"), "info");
    } catch (err) {
      showErrorToast(err);
    }
  };

  const exportBtn = document.createElement("button");
  exportBtn.type = "button";
  exportBtn.textContent = t("commandManager.exportLayout");
  exportBtn.onclick = async () => {
    const path = await invoke<string | null>("export_command_manager_layout_file");
    if (path) {
      showToast(t("commandManager.exported", { path }), "info");
    }
  };

  const importBtn = document.createElement("button");
  importBtn.type = "button";
  importBtn.textContent = t("commandManager.importLayout");
  importBtn.onclick = async () => {
    try {
      await invoke("import_command_manager_layout_file");
      showToast(t("commandManager.imported"), "info");
      window.dispatchEvent(new Event("unibrowser-workspace-layout-loaded"));
    } catch (err) {
      showErrorToast(err);
    }
  };

  const captureBtn = document.createElement("button");
  captureBtn.type = "button";
  captureBtn.textContent = t("settings.developerCaptureTab");
  captureBtn.onclick = async () => {
    try {
      const msg = await invoke<string>("execute_command", {
        commandId: "unibrowser.capture.visibleTab",
      });
      if (msg === "capture-visible") {
        showToast(t("settings.developerCaptureStarted"), "info");
      }
    } catch (err) {
      showErrorToast(err);
    }
  };

  row.appendChild(demoBtn);
  row.appendChild(exportBtn);
  row.appendChild(importBtn);
  row.appendChild(captureBtn);
  section.appendChild(row);
  root.appendChild(section);
}
