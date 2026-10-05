import { t } from "./i18n";

export function applyChromeI18n() {
  const set = (id: string, text: string, title?: string) => {
    const el = document.getElementById(id);
    if (!el) {
      return;
    }
    if (el instanceof HTMLInputElement) {
      el.placeholder = text;
    } else {
      el.textContent = text;
    }
    const label = title ?? text;
    if (label) {
      el.setAttribute("aria-label", label);
    }
    if (title) {
      el.title = title;
    }
  };

  set("btn-back", "←", t("chrome.back"));
  set("btn-forward", "→", t("chrome.forward"));
  set("btn-reload", "↻", t("chrome.reload"));
  set("omnibox", t("chrome.omniboxPlaceholder"));
  set("btn-tab-search", t("chrome.tabs"), t("chrome.tabsTitle"));
  set("btn-split", t("chrome.split"), t("chrome.splitTitle"));
  set("btn-mod-portal", t("chrome.mods"), t("chrome.modsTitle"));
  set("btn-new-project", t("chrome.newProject"), t("chrome.newProjectTitle"));
  set("btn-theme", t("chrome.theme"), t("chrome.themeTitle"));
  set("btn-recent", t("chrome.recent"), t("chrome.recentTitle"));
  set("btn-demo-download", t("chrome.demoDownload"), t("chrome.demoDownloadTitle"));
  set("btn-compare", t("chrome.compare"), t("chrome.compareTitle"));
  set("btn-pin", t("chrome.pin"), t("chrome.pinTitle"));

  const pin = document.getElementById("command-manager-pin");
  const pinLabel = document.getElementById("command-manager-pin-label");
  if (pinLabel) {
    pinLabel.textContent = t("commandManager.stateAutoFollowShort");
  }
  if (pin) {
    pin.textContent = t("commandManager.pinCommandsTab");
    pin.title = t("commandManager.pinCommandsTabTitle", { auto: "…" });
  }
  const cmExport = document.getElementById("command-manager-export");
  if (cmExport) {
    cmExport.textContent = t("commandManager.exportLayout");
    cmExport.title = t("commandManager.exportLayoutTitle");
  }
  const cmImport = document.getElementById("command-manager-import");
  if (cmImport) {
    cmImport.textContent = t("commandManager.importLayout");
    cmImport.title = t("commandManager.importLayoutTitle");
  }
  const dismiss = document.getElementById("onboarding-dismiss");
  if (dismiss) {
    dismiss.textContent = t("onboarding.dismiss");
  }
  const onboarding = document.querySelector("#onboarding-banner span");
  if (onboarding) {
    onboarding.textContent = t("onboarding.tip");
  }
  const tabSearch = document.getElementById("tab-search-query") as HTMLInputElement | null;
  if (tabSearch) {
    tabSearch.placeholder = t("tabSearch.placeholder");
  }
  const bookmarkBar = document.getElementById("bookmark-bar");
  if (bookmarkBar) {
    bookmarkBar.setAttribute("aria-label", t("bookmark.ariaLabel"));
  }

  const statusBar = document.getElementById("status-bar");
  if (statusBar) {
    statusBar.setAttribute("aria-label", t("status.ariaLabel"));
  }
  const statusDirty = document.getElementById("status-dirty");
  if (statusDirty) {
    statusDirty.setAttribute("aria-label", t("status.unsavedAria"));
  }
  const zoomOut = document.getElementById("status-zoom-out");
  if (zoomOut) {
    zoomOut.setAttribute("title", t("status.zoomOut"));
    zoomOut.setAttribute("aria-label", t("status.zoomOut"));
  }
  const zoomIn = document.getElementById("status-zoom-in");
  if (zoomIn) {
    zoomIn.setAttribute("title", t("status.zoomIn"));
    zoomIn.setAttribute("aria-label", t("status.zoomIn"));
  }
}

export function modsLineText(list: string): string {
  return t("chrome.modsLoaded", { list });
}
