import { invoke } from "@tauri-apps/api/core";
import enNz from "./i18n/en-NZ.json";
import { setLocale, supportedLocales, t, type LocaleId } from "./i18n";

type I18nKey = keyof typeof enNz;
import {
  applyCommandManagerDensity,
  applyHighContrastToDocument,
  applyReduceMotionToDocument,
  setCommandManagerDense,
  setHighContrast,
  setReduceMotion,
  type ReduceMotionPref,
} from "./accessibility";
import { applyThemeToDocument, loadThemePreference } from "./theme";
import { appendOnboardingChecklistSection, markChecklistStep } from "./onboardingChecklist";
import { configureAutoSave } from "./autoSave";
import { configureProjectAutoSave } from "./projectAutoSave";
import { appendCrashRecoveryPanel } from "./crashRecoveryPanel";
import { appendWorkspaceLayoutPanel } from "./workspaceLayoutPanel";
import { setExperimentalSpellcheckEnabled } from "./experimentalSpellcheck";
import { showErrorToast, showToast } from "./toast";

interface NtpSectionsDto {
  show_hero: boolean;
  show_actions: boolean;
  show_browse: boolean;
  show_recent: boolean;
}

interface PrefsDto {
  ui_locale: string;
  onboarding_checklist_skip_forever: boolean;
  onboarding_checklist_completed: string[];
  theme: string;
  command_manager_dense: boolean;
  high_contrast: boolean;
  reduce_motion: string;
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
  advanced_experimental_spellcheck: boolean;
  open_downloads_in_system_browser: boolean;
  bookmark_bar_enabled: boolean;
  bookmark_quick_links: { label: string; url: string }[];
}

export async function renderSettings(root: HTMLElement) {
  root.innerHTML = "";
  const intro = document.createElement("p");
  intro.className = "viewer-note";
  intro.textContent = t("settings.lead");
  root.appendChild(intro);

  const searchWrap = document.createElement("div");
  searchWrap.className = "settings-search-wrap";
  const searchInput = document.createElement("input");
  searchInput.type = "search";
  searchInput.className = "settings-search";
  searchInput.placeholder = t("settings.searchPlaceholder");
  searchInput.setAttribute("aria-label", t("settings.searchPlaceholder"));
  searchWrap.appendChild(searchInput);
  root.appendChild(searchWrap);

  const filterSettings = () => {
    const q = searchInput.value.trim().toLowerCase();
    root.querySelectorAll(".settings-section, .settings-advanced").forEach((node) => {
      const sec = node as HTMLElement;
      if (!q) {
        sec.classList.remove("settings-filtered-out");
        return;
      }
      const hay = sec.innerText.toLowerCase();
      sec.classList.toggle("settings-filtered-out", !hay.includes(q));
    });
  };
  searchInput.addEventListener("input", filterSettings);

  const prefs = await invoke<PrefsDto>("get_preferences");

  const langSection = document.createElement("section");
  langSection.className = "settings-section";
  const langHeading = document.createElement("h3");
  langHeading.textContent = t("settings.language");
  const langLead = document.createElement("p");
  langLead.className = "viewer-note";
  langLead.textContent = t("settings.languageLead");
  const langSelect = document.createElement("select");
  supportedLocales().forEach((loc) => {
    const opt = document.createElement("option");
    opt.value = loc;
    opt.textContent = loc;
    langSelect.appendChild(opt);
  });
  langSelect.value = prefs.ui_locale === "en-US" ? "en-US" : "en-NZ";
  langSelect.onchange = async () => {
    const next = await invoke<PrefsDto>("set_ui_locale", { locale: langSelect.value });
    setLocale(next.ui_locale as LocaleId);
    const { applyChromeI18n } = await import("./chromeI18n");
    applyChromeI18n();
  };
  langSection.appendChild(langHeading);
  langSection.appendChild(langLead);
  langSection.appendChild(langSelect);
  root.appendChild(langSection);

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

  const bookmarkSection = document.createElement("section");
  bookmarkSection.className = "settings-section";
  const bookmarkHeading = document.createElement("h3");
  bookmarkHeading.textContent = t("settings.bookmarkBar");
  const bookmarkLead = document.createElement("p");
  bookmarkLead.className = "viewer-note";
  bookmarkLead.textContent = t("settings.bookmarkBarLead");
  bookmarkSection.appendChild(bookmarkHeading);
  bookmarkSection.appendChild(bookmarkLead);
  const bookmarkEnableRow = document.createElement("div");
  bookmarkEnableRow.className = "settings-row";
  const bookmarkCb = document.createElement("input");
  bookmarkCb.type = "checkbox";
  bookmarkCb.id = "settings-bookmark-bar";
  bookmarkCb.checked = prefs.bookmark_bar_enabled;
  bookmarkCb.onchange = async () => {
    await invoke<PrefsDto>("set_bookmark_bar_enabled", { enabled: bookmarkCb.checked });
    window.dispatchEvent(new Event("unibrowser-prefs-changed"));
  };
  const bookmarkEnableLabel = document.createElement("label");
  bookmarkEnableLabel.htmlFor = bookmarkCb.id;
  bookmarkEnableLabel.textContent = t("settings.bookmarkBarShow");
  bookmarkEnableRow.appendChild(bookmarkCb);
  bookmarkEnableRow.appendChild(bookmarkEnableLabel);
  bookmarkSection.appendChild(bookmarkEnableRow);

  const quickLead = document.createElement("p");
  quickLead.className = "viewer-note";
  quickLead.textContent = t("settings.bookmarkQuickLead");
  bookmarkSection.appendChild(quickLead);
  const quickList = document.createElement("div");
  quickList.className = "settings-quick-links";
  const quickState = prefs.bookmark_quick_links.map((l) => ({
    label: l.label,
    url: l.url,
  }));
  const persistQuickLinks = async () => {
    await invoke<PrefsDto>("set_bookmark_quick_links", { links: quickState });
    window.dispatchEvent(new Event("unibrowser-prefs-changed"));
  };
  const renderQuickRows = () => {
    quickList.innerHTML = "";
    quickState.forEach((link, index) => {
      const row = document.createElement("div");
      row.className = "settings-quick-link-row";
      const labelInput = document.createElement("input");
      labelInput.type = "text";
      labelInput.placeholder = t("settings.bookmarkQuickLabel");
      labelInput.value = link.label;
      labelInput.onchange = () => {
        quickState[index].label = labelInput.value;
        void persistQuickLinks();
      };
      const urlInput = document.createElement("input");
      urlInput.type = "url";
      urlInput.placeholder = "https://";
      urlInput.value = link.url;
      urlInput.onchange = () => {
        quickState[index].url = urlInput.value;
        void persistQuickLinks();
      };
      const removeBtn = document.createElement("button");
      removeBtn.type = "button";
      removeBtn.textContent = t("settings.bookmarkQuickRemove");
      removeBtn.onclick = () => {
        quickState.splice(index, 1);
        void persistQuickLinks().then(renderQuickRows);
      };
      row.appendChild(labelInput);
      row.appendChild(urlInput);
      row.appendChild(removeBtn);
      quickList.appendChild(row);
    });
  };
  renderQuickRows();
  const addQuickBtn = document.createElement("button");
  addQuickBtn.type = "button";
  addQuickBtn.textContent = t("settings.bookmarkQuickAdd");
  addQuickBtn.onclick = () => {
    if (quickState.length >= 8) {
      showToast(t("settings.bookmarkQuickLimit"), "info");
      return;
    }
    quickState.push({ label: "", url: "" });
    renderQuickRows();
  };
  bookmarkSection.appendChild(quickList);
  bookmarkSection.appendChild(addQuickBtn);
  root.appendChild(bookmarkSection);

  const a11ySection = document.createElement("section");
  a11ySection.className = "settings-section";
  const a11yHeading = document.createElement("h3");
  a11yHeading.textContent = t("settings.accessibility");
  a11ySection.appendChild(a11yHeading);

  const hcRow = document.createElement("div");
  hcRow.className = "settings-row";
  const hcCb = document.createElement("input");
  hcCb.type = "checkbox";
  hcCb.id = "settings-high-contrast";
  hcCb.checked = prefs.high_contrast;
  const hcLabel = document.createElement("label");
  hcLabel.htmlFor = hcCb.id;
  hcLabel.textContent = t("settings.highContrast");
  hcCb.onchange = async () => {
    await setHighContrast(hcCb.checked);
    applyHighContrastToDocument(hcCb.checked);
  };
  hcRow.appendChild(hcCb);
  hcRow.appendChild(hcLabel);
  a11ySection.appendChild(hcRow);

  const motionLabel = document.createElement("label");
  motionLabel.textContent = t("settings.reduceMotion");
  const motionSelect = document.createElement("select");
  (["system", "on", "off"] as ReduceMotionPref[]).forEach((value) => {
    const opt = document.createElement("option");
    opt.value = value;
    opt.textContent = t(`settings.reduceMotion.${value}` as I18nKey);
    motionSelect.appendChild(opt);
  });
  motionSelect.value = prefs.reduce_motion?.toLowerCase() ?? "system";
  motionSelect.onchange = async () => {
    const v = motionSelect.value as ReduceMotionPref;
    await setReduceMotion(v);
    applyReduceMotionToDocument();
  };
  a11ySection.appendChild(motionLabel);
  a11ySection.appendChild(motionSelect);
  root.appendChild(a11ySection);

  const cmSection = document.createElement("section");
  cmSection.className = "settings-section";
  const cmHeading = document.createElement("h3");
  cmHeading.textContent = t("settings.commandManager");
  const denseRow = document.createElement("div");
  denseRow.className = "settings-row";
  const denseCb = document.createElement("input");
  denseCb.type = "checkbox";
  denseCb.id = "settings-cm-dense";
  denseCb.checked = prefs.command_manager_dense;
  const denseLabel = document.createElement("label");
  denseLabel.htmlFor = denseCb.id;
  denseLabel.textContent = t("settings.commandManagerDense");
  denseCb.onchange = async () => {
    await setCommandManagerDense(denseCb.checked);
    applyCommandManagerDensity(denseCb.checked);
  };
  denseRow.appendChild(denseCb);
  denseRow.appendChild(denseLabel);
  const cmTip = document.createElement("p");
  cmTip.className = "settings-hint";
  cmTip.textContent = t("settings.commandManagerPinTip");
  cmSection.appendChild(cmHeading);
  cmSection.appendChild(denseRow);
  cmSection.appendChild(cmTip);
  root.appendChild(cmSection);

  appendOnboardingChecklistSection(root, {
    onboarding_checklist_skip_forever: prefs.onboarding_checklist_skip_forever,
    onboarding_checklist_completed: prefs.onboarding_checklist_completed,
  });
  void markChecklistStep("settings_tour");

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

  const backupSection = document.createElement("section");
  backupSection.className = "settings-section";
  const backupHeading = document.createElement("h3");
  backupHeading.textContent = t("settings.grandmaBackup");
  const backupLead = document.createElement("p");
  backupLead.className = "viewer-note";
  backupLead.textContent = t("settings.grandmaBackupLead");
  const backupRow = document.createElement("div");
  backupRow.className = "settings-row settings-button-row";
  const exportBackupBtn = document.createElement("button");
  exportBackupBtn.type = "button";
  exportBackupBtn.textContent = t("settings.grandmaExport");
  exportBackupBtn.onclick = async () => {
    try {
      const path = await invoke<string | null>("export_grandma_backup_file");
      if (path) {
        showToast(t("settings.grandmaExported", { path }), "info");
      }
    } catch (err) {
      showErrorToast(err);
    }
  };
  const importBackupBtn = document.createElement("button");
  importBackupBtn.type = "button";
  importBackupBtn.textContent = t("settings.grandmaImport");
  importBackupBtn.onclick = async () => {
    try {
      await invoke("import_grandma_backup_file");
      showToast(t("settings.grandmaImported"), "info");
      const next = await invoke<PrefsDto>("get_preferences");
      langSelect.value = next.ui_locale === "en-US" ? "en-US" : "en-NZ";
      setLocale(next.ui_locale as LocaleId);
      const { applyChromeI18n } = await import("./chromeI18n");
      applyChromeI18n();
      await loadThemePreference();
      applyThemeToDocument();
      setCommandManagerDense(next.command_manager_dense);
      applyCommandManagerDensity(next.command_manager_dense);
      setHighContrast(next.high_contrast);
      applyHighContrastToDocument(next.high_contrast);
      setReduceMotion(next.reduce_motion as ReduceMotionPref);
      applyReduceMotionToDocument();
      void invoke("reload_keybindings");
    } catch (err) {
      showErrorToast(err);
    }
  };
  backupRow.appendChild(exportBackupBtn);
  backupRow.appendChild(importBackupBtn);
  backupSection.appendChild(backupHeading);
  backupSection.appendChild(backupLead);
  backupSection.appendChild(backupRow);
  root.appendChild(backupSection);

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
  const spellRow = document.createElement("div");
  spellRow.className = "settings-row";
  const spellCb = document.createElement("input");
  spellCb.type = "checkbox";
  spellCb.id = "settings-experimental-spellcheck";
  spellCb.checked = prefs.advanced_experimental_spellcheck;
  spellCb.onchange = async () => {
    await invoke<PrefsDto>("set_advanced_experimental_spellcheck", {
      enabled: spellCb.checked,
    });
    setExperimentalSpellcheckEnabled(spellCb.checked);
  };
  const spellLabel = document.createElement("label");
  spellLabel.htmlFor = spellCb.id;
  spellLabel.textContent = t("settings.experimentalSpellcheck");
  const spellNote = document.createElement("p");
  spellNote.className = "viewer-note";
  spellNote.textContent = t("settings.experimentalSpellcheckNote");
  spellRow.appendChild(spellCb);
  spellRow.appendChild(spellLabel);
  advancedSection.appendChild(spellRow);
  advancedSection.appendChild(spellNote);
  const dlBrowserRow = document.createElement("div");
  dlBrowserRow.className = "settings-row";
  const dlBrowserCb = document.createElement("input");
  dlBrowserCb.type = "checkbox";
  dlBrowserCb.id = "settings-downloads-system-browser";
  dlBrowserCb.checked = prefs.open_downloads_in_system_browser;
  dlBrowserCb.onchange = async () => {
    await invoke<PrefsDto>("set_open_downloads_in_system_browser", {
      enabled: dlBrowserCb.checked,
    });
  };
  const dlBrowserLabel = document.createElement("label");
  dlBrowserLabel.htmlFor = dlBrowserCb.id;
  dlBrowserLabel.textContent = t("settings.openDownloadsSystemBrowser");
  const dlBrowserNote = document.createElement("p");
  dlBrowserNote.className = "viewer-note";
  dlBrowserNote.textContent = t("settings.openDownloadsSystemBrowserNote");
  dlBrowserRow.appendChild(dlBrowserCb);
  dlBrowserRow.appendChild(dlBrowserLabel);
  advancedSection.appendChild(dlBrowserRow);
  advancedSection.appendChild(dlBrowserNote);
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

  const diagLink = document.createElement("p");
  diagLink.className = "viewer-note";
  const diagBtn = document.createElement("button");
  diagBtn.type = "button";
  diagBtn.className = "recent-link";
  diagBtn.textContent = t("diagnostics.open");
  diagBtn.onclick = () => {
    window.dispatchEvent(
      new CustomEvent("unibrowser-navigate", {
        detail: "unibrowser://diagnostics",
      }),
    );
  };
  diagLink.appendChild(diagBtn);
  root.appendChild(diagLink);
}
