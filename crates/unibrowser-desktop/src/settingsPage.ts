import { invoke } from "@tauri-apps/api/core";
import enNz from "./i18n/en-NZ.json";
import { t } from "./i18n";

type I18nKey = keyof typeof enNz;
import { applyThemeToDocument, loadThemePreference } from "./theme";
import { configureAutoSave } from "./autoSave";
import { configureProjectAutoSave } from "./projectAutoSave";
import { appendCrashRecoveryPanel } from "./crashRecoveryPanel";
import { appendWorkspaceLayoutPanel } from "./workspaceLayoutPanel";

interface NtpSectionsDto {
  show_hero: boolean;
  show_actions: boolean;
  show_browse: boolean;
  show_recent: boolean;
}

interface PrefsDto {
  theme: string;
  restore_session_on_launch: boolean;
  auto_save_enabled: boolean;
  auto_save_interval_secs: number;
  download_preference: string;
  board_snap_to_grid: boolean;
  midi_scrub_audio_enabled: boolean;
  project_autosave_enabled: boolean;
  project_autosave_interval_secs: number;
  crash_recovery_enabled: boolean;
  ntp_sections: NtpSectionsDto;
  advanced_webview_devtools: boolean;
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
  const projectAutoRow = document.createElement("div");
  projectAutoRow.className = "settings-row";
  const projectAutoCb = document.createElement("input");
  projectAutoCb.type = "checkbox";
  projectAutoCb.id = "settings-project-autosave";
  projectAutoCb.checked = prefs.project_autosave_enabled;
  const projectAutoLabel = document.createElement("label");
  projectAutoLabel.htmlFor = projectAutoCb.id;
  projectAutoLabel.textContent = t("settings.projectAutosave");
  const projectAutoInterval = document.createElement("select");
  projectAutoInterval.setAttribute("aria-label", t("settings.projectAutosave"));
  [60, 120, 300].forEach((secs) => {
    const opt = document.createElement("option");
    opt.value = String(secs);
    opt.textContent = t("autosave.interval", { seconds: String(secs) });
    projectAutoInterval.appendChild(opt);
  });
  projectAutoInterval.value = String(prefs.project_autosave_interval_secs);
  projectAutoInterval.disabled = !prefs.project_autosave_enabled;
  const syncProjectAuto = async () => {
    const next = await invoke<PrefsDto>("set_project_autosave_settings", {
      enabled: projectAutoCb.checked,
      intervalSecs: Number.parseInt(projectAutoInterval.value, 10) || 120,
    });
    projectAutoInterval.disabled = !next.project_autosave_enabled;
    configureProjectAutoSave(
      next.project_autosave_enabled,
      next.project_autosave_interval_secs,
    );
  };
  projectAutoCb.onchange = () => void syncProjectAuto();
  projectAutoInterval.onchange = () => void syncProjectAuto();
  projectAutoRow.appendChild(projectAutoCb);
  projectAutoRow.appendChild(projectAutoLabel);
  projectAutoRow.appendChild(projectAutoInterval);
  boardSection.appendChild(projectAutoRow);
  root.appendChild(boardSection);

  const mediaSection = document.createElement("section");
  mediaSection.className = "settings-section";
  const mediaHeading = document.createElement("h3");
  mediaHeading.textContent = t("settings.media");
  const midiRow = document.createElement("div");
  midiRow.className = "settings-row";
  const midiCb = document.createElement("input");
  midiCb.type = "checkbox";
  midiCb.id = "settings-midi-scrub-audio";
  midiCb.checked = prefs.midi_scrub_audio_enabled;
  midiCb.onchange = async () => {
    await invoke("set_midi_scrub_audio_enabled", { enabled: midiCb.checked });
  };
  const midiLabel = document.createElement("label");
  midiLabel.htmlFor = midiCb.id;
  midiLabel.textContent = t("settings.midiScrubAudio");
  midiRow.appendChild(midiCb);
  midiRow.appendChild(midiLabel);
  mediaSection.appendChild(mediaHeading);
  mediaSection.appendChild(midiRow);
  root.appendChild(mediaSection);

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

  const workspaceSection = document.createElement("section");
  workspaceSection.className = "settings-section";
  const workspaceHeading = document.createElement("h3");
  workspaceHeading.textContent = t("workspaceLayout.heading");
  const workspaceLead = document.createElement("p");
  workspaceLead.className = "viewer-note";
  workspaceLead.textContent = t("workspaceLayout.lead");
  workspaceSection.appendChild(workspaceHeading);
  workspaceSection.appendChild(workspaceLead);
  const workspaceBody = document.createElement("div");
  workspaceSection.appendChild(workspaceBody);
  void appendWorkspaceLayoutPanel(workspaceBody);
  root.appendChild(workspaceSection);

  const ntpSection = document.createElement("section");
  ntpSection.className = "settings-section";
  const ntpHeading = document.createElement("h3");
  ntpHeading.textContent = t("settings.ntp");
  ntpSection.appendChild(ntpHeading);
  const ntpLead = document.createElement("p");
  ntpLead.className = "viewer-note";
  ntpLead.textContent = t("settings.ntpLead");
  ntpSection.appendChild(ntpLead);
  const ntpRows: { id: string; key: keyof NtpSectionsDto; labelKey: I18nKey }[] = [
    { id: "ntp-hero", key: "show_hero", labelKey: "settings.ntpHero" as I18nKey },
    { id: "ntp-actions", key: "show_actions", labelKey: "settings.ntpActions" as I18nKey },
    { id: "ntp-browse", key: "show_browse", labelKey: "settings.ntpBrowse" as I18nKey },
    { id: "ntp-recent", key: "show_recent", labelKey: "settings.ntpRecent" as I18nKey },
  ];
  const ntpState: NtpSectionsDto = { ...prefs.ntp_sections };
  const persistNtp = async () => {
    const next = await invoke<PrefsDto>("set_ntp_sections", { sections: ntpState });
    Object.assign(ntpState, next.ntp_sections);
  };
  ntpRows.forEach((row) => {
    const line = document.createElement("div");
    line.className = "settings-row";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.id = row.id;
    cb.checked = ntpState[row.key];
    cb.onchange = () => {
      ntpState[row.key] = cb.checked;
      void persistNtp();
    };
    const label = document.createElement("label");
    label.htmlFor = row.id;
    label.textContent = t(row.labelKey);
    line.appendChild(cb);
    line.appendChild(label);
    ntpSection.appendChild(line);
  });
  root.appendChild(ntpSection);

  const advancedSection = document.createElement("details");
  advancedSection.className = "settings-section settings-advanced";
  const advancedSummary = document.createElement("summary");
  advancedSummary.textContent = t("settings.advanced");
  advancedSection.appendChild(advancedSummary);
  const devtoolsRow = document.createElement("div");
  devtoolsRow.className = "settings-row";
  const devtoolsCb = document.createElement("input");
  devtoolsCb.type = "checkbox";
  devtoolsCb.id = "settings-webview-devtools";
  devtoolsCb.checked = prefs.advanced_webview_devtools;
  devtoolsCb.onchange = async () => {
    await invoke<PrefsDto>("set_advanced_webview_devtools", { enabled: devtoolsCb.checked });
  };
  const devtoolsLabel = document.createElement("label");
  devtoolsLabel.htmlFor = devtoolsCb.id;
  devtoolsLabel.textContent = t("settings.webviewDevtools");
  const devtoolsNote = document.createElement("p");
  devtoolsNote.className = "viewer-note";
  devtoolsNote.textContent = t("settings.webviewDevtoolsNote");
  devtoolsRow.appendChild(devtoolsCb);
  devtoolsRow.appendChild(devtoolsLabel);
  advancedSection.appendChild(devtoolsRow);
  advancedSection.appendChild(devtoolsNote);
  root.appendChild(advancedSection);

  const keysLink = document.createElement("p");
  keysLink.className = "viewer-note";
  const keysBtn = document.createElement("button");
  keysBtn.type = "button";
  keysBtn.className = "recent-link";
  keysBtn.textContent = t("keys.open");
  keysBtn.onclick = () => {
    window.dispatchEvent(
      new CustomEvent("unibrowser-navigate", { detail: "unibrowser://keys" }),
    );
  };
  keysLink.appendChild(keysBtn);
  root.appendChild(keysLink);

  const windowSection = document.createElement("section");
  windowSection.className = "settings-section";
  const windowHeading = document.createElement("h3");
  windowHeading.textContent = t("settings.windows");
  const windowRow = document.createElement("div");
  windowRow.className = "settings-row";
  const newWindowBtn = document.createElement("button");
  newWindowBtn.type = "button";
  newWindowBtn.textContent = t("settings.newWindow");
  newWindowBtn.onclick = async () => {
    try {
      await invoke("open_new_browser_window");
    } catch (err) {
      console.error(err);
    }
  };
  const windowNote = document.createElement("p");
  windowNote.className = "viewer-note";
  windowNote.textContent = t("settings.newWindowNote");
  windowRow.appendChild(newWindowBtn);
  windowSection.appendChild(windowHeading);
  windowSection.appendChild(windowRow);
  windowSection.appendChild(windowNote);
  root.appendChild(windowSection);

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
