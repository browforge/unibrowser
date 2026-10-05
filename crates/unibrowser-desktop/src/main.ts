import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";
import { commandManager } from "./commandManager";
import { textEditor, type TextDocumentDto, setCursorStatusListener } from "./textEditor";
import { documentViewer, adjustViewerZoom, type DocumentViewerDto } from "./documentViewer";
import { bindRovingTablist, setTabA11y } from "./rovingTabs";
import { cycleTheme, loadThemePreference, themeButtonLabel } from "./theme";
import { bindOmniboxAutocomplete } from "./omniboxUi";
import { invokeWithToast, showToast } from "./toast";
import { bindStatusBar, setPathCopyHandler, setZoomHandler, updateStatusBar } from "./statusBar";
import { copyText, readClipboardText } from "./clipboard";
import { t } from "./i18n";
import { renderNewTabPage, hideNewTabPage } from "./newTabPage";
import { setViewerZoomListener } from "./viewerZoom";
import { applyChromeI18n, modsLineText } from "./chromeI18n";
import { diffEditorView, type DiffViewDto } from "./diffEditor";
import { configureAutoSave } from "./autoSave";

interface TabDto {
  id: number;
  label: string;
  location_kind: string;
  is_directory: boolean;
  is_http: boolean;
  dirty: boolean;
}

interface TabSearchHit {
  tab_index: number;
  label: string;
  omnibox: string;
  score: number;
}

interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

interface ModSuggestionDto {
  mod_id: string;
  title: string;
  reason: string;
}

interface ModPortalDto {
  title: string;
  message: string;
  suggestions: ModSuggestionDto[];
  sideload_hint: string;
}

interface OpenVsxHitDto {
  extension_id: string;
  name: string;
  version: string;
  publisher: string;
  license: string | null;
  url: string;
}

interface InstalledVsixDto {
  extension_id: string;
  name: string;
  version: string;
  publisher: string;
  license: string | null;
  command_count: number;
  keybinding_count: number;
  enabled: boolean;
}

interface TabContentDto {
  tabs: TabDto[];
  active_index: number;
  omnibox: string;
  mods_loaded: string;
  status: string;
  directory_entries: DirEntry[] | null;
  document_status: string | null;
  text_document: TextDocumentDto | null;
  mod_portal: ModPortalDto | null;
  document_viewer: DocumentViewerDto | null;
  show_onboarding: boolean;
  recent_files: RecentFileDto[];
  split_view_enabled: boolean;
  split_index: number | null;
  split_text_document: TextDocumentDto | null;
  split_document_viewer: DocumentViewerDto | null;
  split_status: string | null;
  show_new_tab_page: boolean;
  browse_folder_path: string;
  diff_view: DiffViewDto | null;
  pinned_locations: PinnedLocationDto[];
}

interface PinnedLocationDto {
  id: string;
  label: string;
  location: string;
}

interface RecentFileDto {
  path: string;
  label: string;
}

interface PaletteEntryDto {
  command_id: string;
  title: string;
  enabled: boolean;
  reason: string | null;
}

const omnibox = document.getElementById("omnibox") as HTMLInputElement;
const tabStrip = document.getElementById("tab-strip")!;
const filesPanel = document.getElementById("files-panel")!;
const docPanel = document.getElementById("doc-panel")!;
const commandManagerTabs = document.getElementById("command-manager-tabs")!;
const commandManagerCommands = document.getElementById("command-manager-commands")!;
const commandManagerPin = document.getElementById("command-manager-pin")!;
const floatingToolbarHost = document.getElementById("floating-toolbar-host")!;
const statusBarEl = document.getElementById("status-bar")!;
const newTabPageHost = document.getElementById("new-tab-page-host")!;
let paletteChordHint = "Ctrl+Shift+P for command palette";
let webZoomPx = 16;
let lastViewerZoomLabel = "";
let editorLine = 1;
let editorCol = 1;
const modsLine = document.getElementById("mods-line")!;
const bookmarkBar = document.getElementById("bookmark-bar")!;
const diffEditorHost = document.getElementById("diff-editor-host")!;
const palette = document.getElementById("palette")!;
const paletteQuery = document.getElementById("palette-query") as HTMLInputElement;
const paletteResults = document.getElementById("palette-results")!;
const contentHost = document.getElementById("content-host")!;
const splitSecondary = document.getElementById("split-secondary")!;
const textEditorHost = document.getElementById("text-editor-host")!;
const documentViewerHost = document.getElementById("document-viewer-host")!;
const splitTextEditorHost = document.getElementById("split-text-editor-host")!;
const splitDocumentViewerHost = document.getElementById("split-document-viewer-host")!;
const splitDocPanel = document.getElementById("split-doc-panel")!;
const omniboxSuggestions = document.getElementById("omnibox-suggestions")!;
const btnSplit = document.getElementById("btn-split") as HTMLButtonElement;
const modPortalPanel = document.getElementById("mod-portal-panel")!;
const onboardingBanner = document.getElementById("onboarding-banner")!;
const onboardingDismiss = document.getElementById("onboarding-dismiss")!;
const recentPanel = document.getElementById("recent-panel")!;
const recentToggle = document.getElementById("btn-recent")!;
let recentPanelOpen = false;

const PALETTE_TOGGLE = "unibrowser.palette.toggle";

function refreshStatusBar(
  data: TabContentDto,
  active: TabDto | undefined,
  showText: boolean,
  showViewer: boolean,
) {
  const path =
    active?.is_http
      ? data.omnibox
      : data.text_document?.path ??
        data.document_viewer?.path ??
        (data.show_new_tab_page ? data.browse_folder_path : data.omnibox.replace(/^file:\/\//, ""));
  const encoding = showText && data.text_document ? data.text_document.encoding : "";
  const cursor = showText && data.text_document ? `Ln ${editorLine}, Col ${editorCol}` : "";
  let zoom = "";
  if (active?.is_http) {
    zoom = `Web ${webZoomPx}px`;
  } else if (showViewer && data.document_viewer) {
    zoom = lastViewerZoomLabel;
  }
  updateStatusBar({
    path,
    pathHint: path ? t("status.copyPath") : undefined,
    encoding,
    cursor,
    zoom,
    hint: data.status.includes("command palette") ? paletteChordHint : data.status,
  });
}

const tabSearchPanel = document.getElementById("tab-search")!;
const tabSearchQuery = document.getElementById("tab-search-query") as HTMLInputElement;
const tabSearchResults = document.getElementById("tab-search-results")!;
let tabSearchOpen = false;

let paletteOpen = false;
let paletteChordLabel = "Ctrl+Shift+P";

async function syncContentBounds() {
  const rect = contentHost.getBoundingClientRect();
  await invoke("set_content_bounds", {
    x: rect.left,
    y: rect.top,
    width: rect.width,
    height: rect.height,
  });
}

const tabButtons: HTMLButtonElement[] = [];

function renderTabs(data: TabContentDto) {
  tabStrip.innerHTML = "";
  tabButtons.length = 0;
  data.tabs.forEach((tab, index) => {
    const btn = document.createElement("button");
    btn.type = "button";
    const isSplit =
      data.split_view_enabled &&
      data.split_index !== null &&
      index === data.split_index;
    btn.className =
      "tab" +
      (index === data.active_index ? " active" : "") +
      (isSplit ? " split-pane-tab" : "");
    const dirtyMark = tab.dirty ? " •" : "";
    btn.textContent = tab.label + dirtyMark + (isSplit ? " |" : "");
    btn.title = isSplit
      ? `${tab.location_kind} (split pane; Shift+click to move split here)`
      : `${tab.location_kind} (Shift+click for split pane)`;
    setTabA11y(btn, index === data.active_index, `browser-tab-${index}`);
    btn.onclick = (e) => {
      if (e.shiftKey) {
        void invokeWithToast(
          () => invoke<TabContentDto>("set_split_tab", { index }),
          "Split tab failed",
        ).then((d) => d && applyState(d));
        return;
      }
      void selectTab(index);
    };
    const close = document.createElement("button");
    close.type = "button";
    close.className = "tab-close";
    close.setAttribute("aria-label", `Close ${tab.label}`);
    close.textContent = "×";
    close.onclick = (e) => {
      e.stopPropagation();
      void closeTabAt(index, tab.dirty);
    };
    const wrap = document.createElement("span");
    wrap.className = "tab-wrap";
    wrap.appendChild(btn);
    wrap.appendChild(close);
    tabStrip.appendChild(wrap);
    tabButtons.push(btn);
  });
  const plus = document.createElement("button");
  plus.type = "button";
  plus.className = "tab new-tab";
  plus.textContent = "+";
  plus.setAttribute("aria-label", "New tab");
  plus.tabIndex = -1;
  plus.onclick = () => invoke<TabContentDto>("new_tab").then(applyState);
  tabStrip.appendChild(plus);
  omnibox.value = data.omnibox;
}

function renderFiles(entries: DirEntry[] | null) {
  filesPanel.innerHTML = "";
  if (!entries) {
    filesPanel.classList.add("hidden");
    return;
  }
  filesPanel.classList.remove("hidden");
  const table = document.createElement("table");
  entries.forEach((entry) => {
    const row = document.createElement("tr");
    const cell = document.createElement("td");
    const link = document.createElement("button");
    link.className = "file-link";
    link.textContent = (entry.is_dir ? "[dir] " : "[file] ") + entry.name;
    link.onclick = () =>
      invoke<TabContentDto>("open_path", { path: entry.path }).then(applyState);
    cell.appendChild(link);
    row.appendChild(cell);
    table.appendChild(row);
  });
  filesPanel.appendChild(table);
}

function renderModPortal(portal: ModPortalDto | null) {
  modPortalPanel.innerHTML = "";
  if (!portal) {
    modPortalPanel.classList.add("hidden");
    return;
  }
  modPortalPanel.classList.remove("hidden");
  const h = document.createElement("h2");
  h.textContent = portal.title;
  modPortalPanel.appendChild(h);
  const p = document.createElement("p");
  p.textContent = portal.message;
  modPortalPanel.appendChild(p);
  portal.suggestions.forEach((s) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "mod-suggestion";
    btn.textContent = `${s.title} (${s.mod_id})`;
    btn.title = s.reason;
    btn.onclick = () => {
      showToast(`Install ${s.mod_id} — catalog stub (sideload always allowed)`, "info");
    };
    modPortalPanel.appendChild(btn);
  });
  const side = document.createElement("p");
  side.className = "sideload";
  side.textContent = portal.sideload_hint;
  modPortalPanel.appendChild(side);

  const searchRow = document.createElement("div");
  searchRow.className = "mod-portal-search";
  const optIn = document.createElement("input");
  optIn.type = "checkbox";
  optIn.id = "open-vsx-opt-in";
  const optLabel = document.createElement("label");
  optLabel.htmlFor = "open-vsx-opt-in";
  optLabel.textContent = "Allow network (Open VSX)";
  const searchInput = document.createElement("input");
  searchInput.type = "search";
  searchInput.placeholder = "Search Open VSX (FOSS catalog)";
  searchInput.className = "mod-portal-query";
  const searchBtn = document.createElement("button");
  searchBtn.type = "button";
  searchBtn.textContent = "Search";
  const searchStatus = document.createElement("p");
  searchStatus.className = "mod-portal-search-status viewer-note";
  const results = document.createElement("div");
  results.className = "mod-portal-results";
  searchRow.appendChild(optIn);
  searchRow.appendChild(optLabel);
  searchRow.appendChild(searchInput);
  searchRow.appendChild(searchBtn);
  modPortalPanel.appendChild(searchRow);
  modPortalPanel.appendChild(searchStatus);
  modPortalPanel.appendChild(results);

  const installedHeading = document.createElement("h3");
  installedHeading.textContent = "Installed VSIX";
  modPortalPanel.appendChild(installedHeading);
  const installedList = document.createElement("div");
  installedList.className = "mod-portal-installed";
  modPortalPanel.appendChild(installedList);

  const sideloadBtn = document.createElement("button");
  sideloadBtn.type = "button";
  sideloadBtn.className = "mod-suggestion";
  sideloadBtn.textContent = "Install from file (.vsix)…";
  sideloadBtn.onclick = async () => {
    try {
      const rec = await invoke<InstalledVsixDto>("pick_and_install_vsix");
      showToast(
        `Installed ${rec.extension_id} (${rec.command_count} commands, ${rec.keybinding_count} keybindings in metadata)`,
        "info",
      );
      await refreshInstalledVsix(installedList);
    } catch (err) {
      showToast(String(err), "error");
    }
  };
  modPortalPanel.appendChild(sideloadBtn);

  void refreshInstalledVsix(installedList);

  async function installFromOpenVsx(hit: OpenVsxHitDto) {
    showToast(`Downloading ${hit.extension_id}…`, "info");
    try {
      const rec = await invoke<InstalledVsixDto>("download_install_vsix", {
        url: hit.url,
        networkOptIn: optIn.checked,
        license: hit.license,
      });
      showToast(
        `Installed ${rec.extension_id} — palette lists V1 commands (extension host not run)`,
        "info",
      );
      await refreshInstalledVsix(installedList);
    } catch (err) {
      showToast(String(err), "error");
    }
  }

  searchBtn.onclick = async () => {
    results.innerHTML = "";
    searchStatus.textContent = "Searching…";
    try {
      const hits = await invoke<OpenVsxHitDto[]>("search_open_vsx_catalog", {
        query: searchInput.value,
        networkOptIn: optIn.checked,
      });
      searchStatus.textContent = hits.length
        ? `${hits.length} results (cached ~5 min)`
        : "No results";
      hits.forEach((hit) => {
        const row = document.createElement("button");
        row.type = "button";
        row.className = "mod-suggestion";
        row.textContent = `Install ${hit.name} v${hit.version} · ${hit.publisher}`;
        row.title = `${hit.extension_id}${hit.license ? ` · ${hit.license}` : ""}\n${hit.url}`;
        row.onclick = () => installFromOpenVsx(hit);
        results.appendChild(row);
      });
    } catch (err) {
      searchStatus.textContent = String(err);
    }
  };
}

async function refreshInstalledVsix(container: HTMLElement) {
  container.innerHTML = "";
  try {
    const list = await invoke<InstalledVsixDto[]>("list_installed_vsix");
    if (!list.length) {
      container.textContent = "No VSIX extensions installed yet.";
      return;
    }
    list.forEach((ext) => {
      const row = document.createElement("div");
      row.className = "vsix-installed-row";
      const label = document.createElement("span");
      label.textContent = `${ext.extension_id} v${ext.version} — ${ext.command_count} cmd / ${ext.keybinding_count} keys`;
      const toggle = document.createElement("button");
      toggle.type = "button";
      toggle.textContent = ext.enabled ? "Disable" : "Enable";
      toggle.onclick = async () => {
        await invoke("set_vsix_extension_enabled", {
          extensionId: ext.extension_id,
          enabled: !ext.enabled,
        });
        await refreshInstalledVsix(container);
      };
      const remove = document.createElement("button");
      remove.type = "button";
      remove.textContent = "Uninstall";
      remove.onclick = async () => {
        await invoke("uninstall_vsix_extension", { extensionId: ext.extension_id });
        showToast(`Removed ${ext.extension_id}`, "info");
        await refreshInstalledVsix(container);
      };
      row.appendChild(label);
      row.appendChild(toggle);
      row.appendChild(remove);
      container.appendChild(row);
    });
  } catch {
    container.textContent = "Could not list installed extensions.";
  }
}

function renderDoc(
  status: string | null,
  hasTextEditor: boolean,
  hasPortal: boolean,
  hasViewer: boolean,
) {
  docPanel.innerHTML = "";
  if (!status || hasTextEditor || hasPortal || hasViewer) {
    docPanel.classList.add("hidden");
    return;
  }
  docPanel.classList.remove("hidden");
  docPanel.textContent = status;
}

function renderOnboarding(show: boolean) {
  onboardingBanner.classList.toggle("hidden", !show);
}

async function renderSessionOptIn() {
  try {
    const prefs = await invoke<{ restore_session_on_launch: boolean }>("get_preferences");
    let row = document.getElementById("session-restore-row");
    if (!row) {
      row = document.createElement("div");
      row.id = "session-restore-row";
      row.className = "session-restore-row";
      recentPanel.appendChild(row);
    }
    row.innerHTML = "";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.id = "restore-session-opt-in";
    cb.checked = prefs.restore_session_on_launch;
    cb.onchange = async () => {
      await invoke("set_restore_session_on_launch", { enabled: cb.checked });
    };
    const label = document.createElement("label");
    label.htmlFor = cb.id;
    label.textContent = t("recent.restore");
    const autoRow = document.createElement("div");
    autoRow.className = "session-restore-row";
    const autoCb = document.createElement("input");
    autoCb.type = "checkbox";
    autoCb.id = "auto-save-opt-in";
    const autoLabel = document.createElement("label");
    autoLabel.htmlFor = autoCb.id;
    void invoke<{ auto_save_enabled: boolean; auto_save_interval_secs: number }>(
      "get_preferences",
    ).then((prefs) => {
      autoCb.checked = prefs.auto_save_enabled;
      autoLabel.textContent = t("autosave.label");
      autoCb.onchange = async () => {
        const next = await invoke<{ auto_save_enabled: boolean; auto_save_interval_secs: number }>(
          "set_auto_save_settings",
          { enabled: autoCb.checked, intervalSecs: prefs.auto_save_interval_secs },
        );
        configureAutoSave(next.auto_save_enabled, next.auto_save_interval_secs);
      };
      configureAutoSave(prefs.auto_save_enabled, prefs.auto_save_interval_secs);
    });
    autoRow.appendChild(autoCb);
    autoRow.appendChild(autoLabel);
    row.appendChild(cb);
    row.appendChild(label);
    row.appendChild(autoRow);
  } catch {
    /* optional */
  }
}

function renderBookmarkBar(pins: PinnedLocationDto[]) {
  bookmarkBar.innerHTML = "";
  pins.forEach((pin) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "bookmark-link";
    if (pin.id === "home") {
      btn.textContent = t("bookmark.home");
    } else if (pin.id === "downloads") {
      btn.textContent = t("bookmark.downloads");
    } else if (pin.id === "new-project") {
      btn.textContent = t("bookmark.project");
    } else {
      btn.textContent = pin.label;
    }
    btn.title = pin.location;
    btn.onclick = () => {
      if (pin.location === "unibrowser://action/new-project") {
        document.getElementById("btn-new-project")!.click();
        return;
      }
      omnibox.value = pin.location;
      navigateOmnibox();
    };
    bookmarkBar.appendChild(btn);
  });
}

function renderRecentFiles(entries: RecentFileDto[]) {
  recentPanel.innerHTML = "";
  if (!recentPanelOpen) {
    recentPanel.classList.add("hidden");
    return;
  }
  recentPanel.classList.remove("hidden");
  if (!entries.length) {
    recentPanel.textContent = t("recent.empty");
    return;
  }
  const list = document.createElement("ul");
  list.className = "recent-list";
  entries.forEach((entry) => {
    const li = document.createElement("li");
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "recent-link";
    btn.textContent = entry.label;
    btn.title = entry.path;
    btn.onclick = () =>
      invoke<TabContentDto>("open_path", { path: entry.path }).then(applyState);
    li.appendChild(btn);
    list.appendChild(li);
  });
  recentPanel.appendChild(list);
  void renderSessionOptIn();
}

async function runUiCommand(commandId: string) {
  await textEditor.flush();
  const msg = await invoke<string>("execute_command", { commandId });
  showToast(String(msg), "info");
  await refreshCommandManager();
  await refreshState();
}

async function refreshCommandManager() {
  try {
    await commandManager.refresh(
      commandManagerTabs,
      commandManagerCommands,
      floatingToolbarHost,
      commandManagerPin,
      runUiCommand,
    );
  } catch {
    commandManagerCommands.innerHTML =
      "<span class='muted'>Commands unavailable</span>";
  }
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

async function applySplitPane(data: TabContentDto) {
  if (!data.split_view_enabled) {
    splitSecondary.classList.add("hidden");
    contentHost.classList.remove("split-active");
    btnSplit.classList.remove("active");
    return;
  }
  splitSecondary.classList.remove("hidden");
  contentHost.classList.add("split-active");
  btnSplit.classList.add("active");
  const showSplitViewer = Boolean(data.split_document_viewer);
  const showSplitText = Boolean(data.split_text_document) && !showSplitViewer;
  if (showSplitViewer && data.split_document_viewer) {
    splitTextEditorHost.classList.add("hidden");
    splitTextEditorHost.innerHTML = "";
    splitDocPanel.classList.add("hidden");
    await documentViewer.apply(data.split_document_viewer, splitDocumentViewerHost);
  } else if (showSplitText && data.split_text_document) {
    documentViewer.hide(splitDocumentViewerHost);
    splitDocPanel.classList.add("hidden");
    splitTextEditorHost.classList.remove("hidden");
    splitTextEditorHost.innerHTML = `<pre class="split-text-readonly">${escapeHtml(
      data.split_text_document.content,
    )}</pre>`;
  } else {
    documentViewer.hide(splitDocumentViewerHost);
    splitTextEditorHost.classList.add("hidden");
    splitTextEditorHost.innerHTML = "";
    splitDocPanel.classList.remove("hidden");
    splitDocPanel.textContent = data.split_status ?? "No preview for this tab.";
  }
}

async function applyState(data: TabContentDto) {
  const active = data.tabs[data.active_index];
  const showText = Boolean(data.text_document) && !active?.is_http;
  const showViewer = Boolean(data.document_viewer) && !active?.is_http;
  const showHome = data.show_new_tab_page && !active?.is_http;
  renderTabs(data);
  modsLine.textContent = modsLineText(data.mods_loaded);
  renderBookmarkBar(data.pinned_locations ?? []);
  const showDiff = Boolean(data.diff_view);
  if (showDiff && data.diff_view) {
    await diffEditorView.apply(data.diff_view, diffEditorHost);
    textEditor.hide(textEditorHost);
    documentViewer.hide(documentViewerHost);
  } else {
    diffEditorView.hide(diffEditorHost);
  }
  if (showHome) {
    filesPanel.classList.add("hidden");
    renderNewTabPage(newTabPageHost, {
      entries: data.directory_entries,
      recent: data.recent_files ?? [],
      browsePath: data.browse_folder_path,
      onOpenPath: (path) =>
        invoke<TabContentDto>("open_path", { path }).then((d) => d && applyState(d)),
      onNewProject: () => document.getElementById("btn-new-project")!.click(),
      onPickFolder: async () => {
        const folder = await invoke<string | null>("pick_browse_folder");
        if (folder) {
          await invoke<TabContentDto>("open_path", { path: folder }).then((d) => d && applyState(d));
        }
      },
    });
  } else {
    hideNewTabPage(newTabPageHost);
    if (showText || showViewer || active?.is_http) {
      filesPanel.classList.add("hidden");
    } else {
      renderFiles(data.directory_entries);
    }
  }
  renderModPortal(data.mod_portal);
  const hasPortal = Boolean(data.mod_portal);
  renderDoc(data.document_status, showText, hasPortal, showViewer);
  renderOnboarding(data.show_onboarding);
  renderRecentFiles(data.recent_files ?? []);
  if (!showDiff) {
    if (showText && data.text_document) {
      await textEditor.apply(data.text_document, textEditorHost);
      documentViewer.hide(documentViewerHost);
    } else {
      textEditor.hide(textEditorHost);
    }
    if (showViewer && data.document_viewer) {
      await documentViewer.apply(data.document_viewer, documentViewerHost);
    } else if (!showText) {
      documentViewer.hide(documentViewerHost);
    }
  }
  await applySplitPane(data);
  if (active?.is_http) {
    setZoomHandler("web", (delta) => {
      webZoomPx = Math.min(96, Math.max(8, webZoomPx + delta));
      void invoke("web_page_zoom", { deltaPx: delta });
      refreshStatusBar(data, active, showText, showViewer);
    });
  } else if (showViewer && data.document_viewer) {
    const kind = data.document_viewer.viewer;
    if (kind === "pdf" || kind === "image") {
      setZoomHandler("viewer", (delta) => adjustViewerZoom(delta));
    } else {
      setZoomHandler("hidden", null);
    }
  } else {
    setZoomHandler("hidden", null);
  }
  refreshStatusBar(data, active, showText, showViewer);
  await refreshCommandManager();
  await syncContentBounds();
  window.requestAnimationFrame(() => syncContentBounds());
}

async function refreshState() {
  try {
    const data = await invoke<TabContentDto>("get_browser_state");
    await applyState(data);
  } catch (err) {
    showToast(String(err), "error");
  }
}

async function selectTab(index: number) {
  await textEditor.flush();
  await invoke("select_tab", { index });
  await refreshState();
}

async function closeTabAt(index: number, dirty: boolean) {
  const force = dirty
    ? window.confirm("This tab has unsaved changes. Close anyway?")
    : true;
  if (!force) {
    return;
  }
  await textEditor.flush();
  const data = await invokeWithToast(
    () => invoke<TabContentDto>("close_tab", { index, force: true }),
    "Could not close tab",
  );
  if (data) {
    await applyState(data);
  }
}

function navigateOmnibox() {
  void invokeWithToast(
    () => invoke<TabContentDto>("navigate_omnibox", { input: omnibox.value }),
    "Could not navigate",
  ).then((data) => {
    if (data) {
      void applyState(data);
    }
  });
}

omnibox.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    navigateOmnibox();
    return;
  }
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "v") {
    e.preventDefault();
    void readClipboardText().then((text) => {
      if (text) {
        omnibox.value = text;
        navigateOmnibox();
      }
    });
  }
});

bindOmniboxAutocomplete(omnibox, omniboxSuggestions, () => navigateOmnibox());

document.getElementById("btn-tab-search")!.onclick = () => toggleTabSearch();
tabSearchQuery.addEventListener("input", () => void refreshTabSearchResults());

listen("browser-state-updated", () => {
  void refreshState();
});

btnSplit.onclick = () => {
  void invokeWithToast(() => invoke<TabContentDto>("toggle_split_view"), "Split view failed").then(
    (data) => data && applyState(data),
  );
};

document.getElementById("btn-back")!.onclick = () =>
  invoke<TabContentDto>("go_back").then(applyState);
document.getElementById("btn-forward")!.onclick = () =>
  invoke<TabContentDto>("go_forward").then(applyState);
document.getElementById("btn-reload")!.onclick = () =>
  invoke<TabContentDto>("reload_tab").then(applyState);

document.getElementById("btn-demo-download")!.onclick = () =>
  invoke<TabContentDto>("handle_download_url", {
    url: "https://example.com/",
  }).then(applyState);

document.getElementById("btn-compare")!.onclick = () => {
  void invokeWithToast(
    () => invoke<TabContentDto>("pick_and_compare"),
    t("compare.needFile"),
  ).then((data) => {
    if (data) {
      showToast(t("compare.started"), "info");
      void applyState(data);
    }
  });
};

document.getElementById("btn-pin")!.onclick = () => {
  void invoke<PinnedLocationDto[]>("pin_current_location").then((pins) => {
    renderBookmarkBar(pins);
  });
};

diffEditorHost.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;
  if (target.dataset.action === "close-diff") {
    void invoke<TabContentDto>("close_file_compare").then((d) => d && applyState(d));
  }
});

document.getElementById("btn-mod-portal")!.onclick = () =>
  invoke<TabContentDto>("open_mod_portal", { ext: null }).then(applyState);

const themeBtn = document.getElementById("btn-theme") as HTMLButtonElement;
recentToggle.onclick = () => {
  recentPanelOpen = !recentPanelOpen;
  recentToggle.setAttribute("aria-expanded", recentPanelOpen ? "true" : "false");
  void refreshState();
};

themeBtn.onclick = async () => {
  await cycleTheme();
  themeBtn.textContent = themeButtonLabel();
};

document.getElementById("btn-new-project")!.onclick = async () => {
  const name = window.prompt("Project name", "MyProject");
  if (!name) {
    return;
  }
  const home = await invoke<string>("get_home_dir").catch(() => "/tmp");
  await invoke<TabContentDto>("create_unibrow_project", {
    parentDir: home,
    name,
  }).then(applyState);
};

onboardingDismiss.onclick = async () => {
  await invoke("dismiss_onboarding");
  onboardingBanner.classList.add("hidden");
};

let geometryTimer: number | undefined;
function scheduleGeometrySave() {
  window.clearTimeout(geometryTimer);
  geometryTimer = window.setTimeout(() => {
    invoke("save_window_geometry", {
      width: window.outerWidth,
      height: window.outerHeight,
      x: window.screenX,
      y: window.screenY,
    }).catch(() => {});
  }, 400);
}

window.addEventListener("resize", scheduleGeometrySave);
window.addEventListener("move", scheduleGeometrySave);

function togglePalette() {
  paletteOpen = !paletteOpen;
  palette.classList.toggle("hidden", !paletteOpen);
  if (paletteOpen) {
    tabSearchOpen = false;
    tabSearchPanel.classList.add("hidden");
    paletteQuery.placeholder = `Search commands (${paletteChordLabel})`;
    paletteQuery.focus();
  }
}

function toggleTabSearch() {
  tabSearchOpen = !tabSearchOpen;
  tabSearchPanel.classList.toggle("hidden", !tabSearchOpen);
  if (tabSearchOpen) {
    paletteOpen = false;
    palette.classList.add("hidden");
    tabSearchQuery.value = "";
    tabSearchQuery.focus();
    void refreshTabSearchResults();
  }
}

async function refreshTabSearchResults() {
  const hits = await invoke<TabSearchHit[]>("search_open_tabs", {
    query: tabSearchQuery.value,
  });
  tabSearchResults.innerHTML = "";
  hits.forEach((hit) => {
    const row = document.createElement("button");
    row.type = "button";
    row.className = "palette-item";
    row.textContent = hit.label;
    row.title = hit.omnibox;
    row.onclick = () => {
      toggleTabSearch();
      void selectTab(hit.tab_index);
    };
    tabSearchResults.appendChild(row);
  });
}

function chordFromKeyboardEvent(e: KeyboardEvent) {
  let key = e.key;
  if (key.length === 1) {
    key = key.toLowerCase();
  } else {
    key = key.toLowerCase();
  }
  if (key === "+") {
    key = "=";
  }
  return {
    ctrl: e.ctrlKey,
    alt: e.altKey,
    shift: e.shiftKey,
    meta: e.metaKey,
    key,
  };
}

async function runKeybindingCommand(commandId: string) {
  if (commandId === PALETTE_TOGGLE) {
    togglePalette();
    return;
  }
  await textEditor.flush();
  const msg = await invoke<string>("execute_command", { commandId });
  showToast(String(msg), "info");
  await refreshCommandManager();
  await refreshState();
}

document.addEventListener(
  "keydown",
  async (e) => {
  if (e.repeat) {
    return;
  }
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "t") {
    e.preventDefault();
    toggleTabSearch();
    return;
  }
  if (textEditor.isActive() && e.ctrlKey && e.key.toLowerCase() === "f") {
    return;
  }
  if (e.ctrlKey && e.altKey && e.key.toLowerCase() === "z") {
    e.preventDefault();
    void invokeWithToast(() => invoke<TabContentDto>("document_undo"), "Nothing to undo").then(
      (data) => data && applyState(data),
    );
    return;
  }
  if (e.ctrlKey && e.altKey && e.key.toLowerCase() === "y") {
    e.preventDefault();
    void invokeWithToast(() => invoke<TabContentDto>("document_redo"), "Nothing to redo").then(
      (data) => data && applyState(data),
    );
    return;
  }
  const chord = chordFromKeyboardEvent(e);
  const commandId = await invoke<string | null>("dispatch_key_chord", { chord });
  if (!commandId) {
    return;
  }
  e.preventDefault();
  await runKeybindingCommand(commandId);
},
  { capture: true },
);

async function loadKeybindingHints() {
  try {
    const bindings = await invoke<{ command_id: string; chord: string }[]>(
      "get_keybindings",
    );
    const palette = bindings.find((b) => b.command_id === PALETTE_TOGGLE);
    if (palette) {
      paletteChordLabel = palette.chord;
      paletteChordHint = `${paletteChordLabel} for command palette`;
      paletteQuery.placeholder = `Search commands (${paletteChordLabel})`;
    }
  } catch {
    /* defaults */
  }
}

paletteQuery.addEventListener("input", async () => {
  const entries = await invoke<PaletteEntryDto[]>("palette_search", {
    query: paletteQuery.value,
  });
  paletteResults.innerHTML = "";
  entries.forEach((entry) => {
    const row = document.createElement("button");
    row.className = entry.enabled ? "palette-item" : "palette-item disabled";
    row.textContent = entry.title + (entry.enabled ? "" : " (greyed)");
    row.disabled = !entry.enabled;
    row.onclick = () => {
      invoke<string>("execute_command", { commandId: entry.command_id }).then((msg) => {
        showToast(String(msg), "info");
        togglePalette();
        refreshCommandManager();
        refreshState();
      });
    };
    paletteResults.appendChild(row);
  });
});

window.addEventListener("resize", () => syncContentBounds());

listen<string>("unibrowser-download", (event) => {
  void invokeWithToast(
    () => invoke<TabContentDto>("handle_download_url", { url: event.payload }),
    t("toast.errorDefault"),
  ).then((data) => {
    if (data) {
      showToast(t("download.openedTab"), "info");
      void applyState(data);
    }
  });
});

window.addEventListener("unibrowser-refresh", (event) => {
  const detail = (event as CustomEvent<TabContentDto>).detail;
  if (detail) {
    void applyState(detail);
  } else {
    void refreshState();
  }
});

commandManager.bindPinControl(
  commandManagerPin,
  commandManagerTabs,
  commandManagerCommands,
  floatingToolbarHost,
  runUiCommand,
);

bindRovingTablist(tabStrip, () => tabButtons);

bindStatusBar(statusBarEl);
setPathCopyHandler((path) => {
  void copyText(path, () => showToast(t("status.pathCopied"), "info"));
});
setCursorStatusListener((pos) => {
  editorLine = pos.line;
  editorCol = pos.column;
});
setViewerZoomListener((_scale, label) => {
  lastViewerZoomLabel = label;
  const zoomEl = document.getElementById("status-zoom-label");
  if (zoomEl && lastViewerZoomLabel) {
    zoomEl.textContent = lastViewerZoomLabel;
    zoomEl.classList.remove("hidden");
  }
});

applyChromeI18n();
loadKeybindingHints();
void loadThemePreference().then(() => {
  themeBtn.textContent = themeButtonLabel();
});
void invoke<{ auto_save_enabled: boolean; auto_save_interval_secs: number }>("get_preferences").then(
  (prefs) => configureAutoSave(prefs.auto_save_enabled, prefs.auto_save_interval_secs),
);
refreshState();
