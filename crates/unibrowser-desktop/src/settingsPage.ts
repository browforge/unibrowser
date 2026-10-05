import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { applyThemeToDocument, loadThemePreference } from "./theme";
import { configureAutoSave } from "./autoSave";
import { appendCrashRecoveryPanel } from "./crashRecoveryPanel";

interface PrefsDto {
  theme: string;
  restore_session_on_launch: boolean;
  auto_save_enabled: boolean;
  auto_save_interval_secs: number;
  download_preference: string;
  board_snap_to_grid: boolean;
  crash_recovery_enabled: boolean;
}

export async function renderSettings(root: HTMLElement) {
  root.innerHTML = "";
  const intro = document.createElement("p");
  intro.className = "viewer-note";
  intro.textContent = t("settings.lead");
  root.appendChild(intro);

  const prefs = await invoke<PrefsDto>("get_preferences");

  const themeSection = document.createElement("section");
  themeSection.className = "settings-section";
  const themeHeading = document.createElement("h3");
  themeHeading.textContent = t("settings.theme");
  const themeSelect = document.createElement("select");
  const themeLabels: Record<string, string> = {
    light: t("settings.themeLight"),
    dark: t("settings.themeDark"),
    system: t("settings.themeSystem"),
  };
  ["light", "dark", "system"].forEach((value) => {
    const opt = document.createElement("option");
    opt.value = value;
    opt.textContent = themeLabels[value] ?? value;
    themeSelect.appendChild(opt);
  });
  themeSelect.value = prefs.theme.toLowerCase();
  themeSelect.onchange = async () => {
    await invoke("set_ui_theme", { theme: themeSelect.value });
    await loadThemePreference();
    applyThemeToDocument();
  };
  themeSection.appendChild(themeHeading);
  themeSection.appendChild(themeSelect);
  root.appendChild(themeSection);

  const dlSection = document.createElement("section");
  dlSection.className = "settings-section";
  const dlHeading = document.createElement("h3");
  dlHeading.textContent = t("download.preferenceLabel");
  const dlSelect = document.createElement("select");
  const openOpt = document.createElement("option");
  openOpt.value = "open_in_new_tab";
  openOpt.textContent = t("download.openInTab");
  const saveOpt = document.createElement("option");
  saveOpt.value = "save_to_disk";
  saveOpt.textContent = t("download.saveToDisk");
  dlSelect.appendChild(openOpt);
  dlSelect.appendChild(saveOpt);
  dlSelect.value = prefs.download_preference;
  dlSelect.onchange = async () => {
    await invoke("set_download_preference", { preference: dlSelect.value });
  };
  dlSection.appendChild(dlHeading);
  dlSection.appendChild(dlSelect);
  root.appendChild(dlSection);

  const autoSection = document.createElement("section");
  autoSection.className = "settings-section";
  const autoHeading = document.createElement("h3");
  autoHeading.textContent = t("settings.autosave");
  const autoRow = document.createElement("div");
  autoRow.className = "settings-row";
  const autoCb = document.createElement("input");
  autoCb.type = "checkbox";
  autoCb.id = "settings-autosave";
  autoCb.checked = prefs.auto_save_enabled;
  const autoLabel = document.createElement("label");
  autoLabel.htmlFor = autoCb.id;
  autoLabel.textContent = t("autosave.label");
  const intervalSelect = document.createElement("select");
  [30, 60, 120, 300, 600].forEach((secs) => {
    const opt = document.createElement("option");
    opt.value = String(secs);
    opt.textContent = t("autosave.interval", { seconds: secs });
    intervalSelect.appendChild(opt);
  });
  intervalSelect.value = String(prefs.auto_save_interval_secs);
  intervalSelect.disabled = !prefs.auto_save_enabled;
  const persistAuto = async () => {
    const next = await invoke<PrefsDto>("set_auto_save_settings", {
      enabled: autoCb.checked,
      intervalSecs: Number.parseInt(intervalSelect.value, 10),
    });
    intervalSelect.disabled = !next.auto_save_enabled;
    configureAutoSave(next.auto_save_enabled, next.auto_save_interval_secs);
  };
  autoCb.onchange = () => void persistAuto();
  intervalSelect.onchange = () => void persistAuto();
  autoRow.appendChild(autoCb);
  autoRow.appendChild(autoLabel);
  autoRow.appendChild(intervalSelect);
  autoSection.appendChild(autoHeading);
  autoSection.appendChild(autoRow);
  root.appendChild(autoSection);

  const sessionSection = document.createElement("section");
  sessionSection.className = "settings-section";
  const sessionHeading = document.createElement("h3");
  sessionHeading.textContent = t("settings.session");
  const sessionRow = document.createElement("div");
  sessionRow.className = "settings-row";
  const sessionCb = document.createElement("input");
  sessionCb.type = "checkbox";
  sessionCb.id = "settings-restore-session";
  sessionCb.checked = prefs.restore_session_on_launch;
  sessionCb.onchange = async () => {
    await invoke("set_restore_session_on_launch", { enabled: sessionCb.checked });
  };
  const sessionLabel = document.createElement("label");
  sessionLabel.htmlFor = sessionCb.id;
  sessionLabel.textContent = t("recent.restore");
  sessionRow.appendChild(sessionCb);
  sessionRow.appendChild(sessionLabel);
  sessionSection.appendChild(sessionHeading);
  sessionSection.appendChild(sessionRow);
  root.appendChild(sessionSection);

  const boardSection = document.createElement("section");
  boardSection.className = "settings-section";
  const boardHeading = document.createElement("h3");
  boardHeading.textContent = t("settings.board");
  const boardRow = document.createElement("div");
  boardRow.className = "settings-row";
  const snapCb = document.createElement("input");
  snapCb.type = "checkbox";
  snapCb.id = "settings-board-snap";
  snapCb.checked = prefs.board_snap_to_grid;
  snapCb.onchange = async () => {
    await invoke("set_board_snap_to_grid", { enabled: snapCb.checked });
  };
  const snapLabel = document.createElement("label");
  snapLabel.htmlFor = snapCb.id;
  snapLabel.textContent = t("settings.boardSnap");
  boardRow.appendChild(snapCb);
  boardRow.appendChild(snapLabel);
  boardSection.appendChild(boardHeading);
  boardSection.appendChild(boardRow);
  root.appendChild(boardSection);

  const recoverySection = document.createElement("section");
  recoverySection.className = "settings-section";
  const recoveryHeading = document.createElement("h3");
  recoveryHeading.textContent = t("settings.crashRecovery");
  const recoveryRow = document.createElement("div");
  recoveryRow.className = "settings-row";
  const recoveryCb = document.createElement("input");
  recoveryCb.type = "checkbox";
  recoveryCb.id = "settings-crash-recovery";
  recoveryCb.checked = prefs.crash_recovery_enabled;
  recoveryCb.onchange = async () => {
    await invoke("set_crash_recovery_enabled", { enabled: recoveryCb.checked });
  };
  const recoveryLabel = document.createElement("label");
  recoveryLabel.htmlFor = recoveryCb.id;
  recoveryLabel.textContent = t("settings.crashRecoveryLabel");
  recoveryRow.appendChild(recoveryCb);
  recoveryRow.appendChild(recoveryLabel);
  recoverySection.appendChild(recoveryHeading);
  recoverySection.appendChild(recoveryRow);
  const backupsWrap = document.createElement("div");
  backupsWrap.className = "settings-crash-backups";
  recoverySection.appendChild(backupsWrap);
  void appendCrashRecoveryPanel(backupsWrap);
  root.appendChild(recoverySection);

  const aboutLink = document.createElement("p");
  aboutLink.className = "viewer-note";
  const link = document.createElement("button");
  link.type = "button";
  link.className = "recent-link";
  link.textContent = t("about.open");
  link.onclick = () => {
    window.dispatchEvent(
      new CustomEvent("unibrowser-navigate", { detail: "unibrowser://about" }),
    );
  };
  aboutLink.appendChild(link);
  root.appendChild(aboutLink);
}
