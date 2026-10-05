import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";
import { commandManager, getStatusContextMode } from "./commandManager";
import { textEditor, type TextDocumentDto, setCursorStatusListener } from "./textEditor";
import { documentViewer, type DocumentViewerDto } from "./documentViewer";
import { bindRovingTablist, setTabA11y } from "./rovingTabs";
import { loadAccessibilityPreferences } from "./accessibility";
import { loadExperimentalSpellcheckPref } from "./experimentalSpellcheck";
import { markChecklistStep } from "./onboardingChecklist";
import { cycleTheme, loadThemePreference, themeButtonLabel } from "./theme";
import { bindOmniboxAutocomplete } from "./omniboxUi";
import { invokeWithToast, showToast } from "./toast";
import {
  bindStatusBar,
  patchStatusCursor,
  setPathCopyHandler,
  setZoomHandler,
  updateStatusBar,
} from "./statusBar";
import { getEncodeProgress, setEncodeProgress } from "./encodeProgress";
import { runExportPrintPdf } from "./exportPdf";
import { copyText, readClipboardText } from "./clipboard";
import { t } from "./i18n";
import { renderNewTabPage, hideNewTabPage } from "./newTabPage";
import { setViewerZoomListener } from "./viewerZoom";
import { applyChromeI18n, modsLineText } from "./chromeI18n";
import { diffEditorView, type DiffViewDto } from "./diffEditor";
import { configureAutoSave } from "./autoSave";
import { configureProjectAutoSave } from "./projectAutoSave";
import { maybePromptProjectRecovery } from "./projectRecoveryPrompt";
import { showOnboardingTour } from "./onboardingTour";
import {
  appendScriptOutput,
  initScriptOutputPanel,
} from "./scriptOutputPanel";
import { loadUiLocale } from "./i18n";
import { openNewProjectDialog } from "./newProjectDialog";
import { emptyStateBlock } from "./emptyState";
import { clearFilesPanelContext, handleFilesKeyDown, renderFilesPanel } from "./filesPanel";
import { pathsFromDataTransfer } from "./filesDrag";
import { initGlobalFind, toggleGlobalFind } from "./globalFind";
import {
  closeDocumentFind,
  openDocumentFind,
  setDocumentFindSurface,
  toggleDocumentFind,
} from "./documentFind";
import type { DirEntry as FilesDirEntry, FilesViewDto } from "./filesPanelTypes";
import {
  closeKeysOverlay,
  initKeysOverlay,
  isKeysOverlayOpen,
  openKeysOverlay,
} from "./keysOverlay";

interface TabDto {
  id: number;
  label: string;
  location_kind: string;
  is_directory: boolean;
  is_http: boolean;
  dirty: boolean;
  pinned: boolean;
  browse_path?: string | null;
}

interface TabSearchHit {
  tab_index: number;
  label: string;
  omnibox: string;
  score: number;
}

type DirEntry = FilesDirEntry;

interface ModSuggestionDto {
  mod_id: string;
  title: string;
  reason: string;
  open_vsx_id?: string | null;
  ubmod_id?: string | null;
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

interface InstalledNativeModDto {
  name: string;
  title: string;
  version: string;
  enabled: boolean;
  library_present: boolean;
  signature_ok: boolean;
  load_block_reason: string | null;
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

interface DownloadHistoryEntryDto {
  id: string;
  url: string;
  label: string;
  status: string;
  error?: string | null;
  destination?: string | null;
  at_ms: number;
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
  show_onboarding_tour: boolean;
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
  chrome_notice?: string | null;
  ntp_sections: {
    show_hero: boolean;
    show_actions: boolean;
    show_browse: boolean;
    show_recent: boolean;
  };
  files_view: FilesViewDto;
  folder_favorites: string[];
  download_history?: DownloadHistoryEntryDto[];
  bookmark_bar_enabled?: boolean;
  bookmark_quick_links?: BookmarkQuickLinkDto[];
}

interface BookmarkQuickLinkDto {
  label: string;
  url: string;
}

interface DownloadEventPayload {
  url: string;
  destination?: string | null;
}

interface PinnedLocationDto {
  id: string;
  label: string;
  location: string;
}

interface RecentFileDto {
  path: string;
  label: string;
  pinned: boolean;
}

interface PaletteEntryDto {
  command_id: string;
  title: string;
  chord: string | null;
  enabled: boolean;
  reason: string | null;
  open_path?: string | null;
}

const omnibox = document.getElementById("omnibox") as HTMLInputElement;
const tabStrip = document.getElementById("tab-strip")!;
const filesBrowserWrap = document.getElementById("files-browser-wrap")!;
const filesPanel = document.getElementById("files-panel")!;
const filesSidePanel = document.getElementById("files-side-panel")!;
const docPanel = document.getElementById("doc-panel")!;
const commandManagerTabs = document.getElementById("command-manager-tabs")!;
const commandManagerCommands = document.getElementById("command-manager-commands")!;
const commandManagerPin = document.getElementById("command-manager-pin")!;
const floatingToolbarHost = document.getElementById("floating-toolbar-host")!;
const statusBarEl = document.getElementById("status-bar")!;
const newTabPageHost = document.getElementById("new-tab-page-host")!;
let paletteChordHint = "Ctrl+Shift+P for command palette";
let webZoomPercent = 100;
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
    zoom = t("status.webZoom", { percent: String(webZoomPercent) });
  } else if (showViewer && data.document_viewer) {
    zoom = lastViewerZoomLabel;
  }
  let font = "";
  if (showText && data.text_document) {
    font = t("status.fontPt", { pt: String(data.text_document.font_size_pt) });
  } else if (showViewer && data.document_viewer?.reader_font_pt) {
    font = t("status.fontPt", { pt: String(data.document_viewer.reader_font_pt) });
  }
  const ctx = getStatusContextMode();
  const context = ctx
    ? t("status.commandContext", {
        tab: ctx.tab,
        mode: ctx.pinned ? t("status.contextPinned") : t("status.contextAuto"),
      })
    : "";
  const dirty =
    active?.dirty && !active.is_directory ? t("status.unsaved") : "";
  const enc = getEncodeProgress();
  const encode =
    enc.active
      ? t("status.encode", { percent: String(enc.percent), stage: enc.stage })
      : "";
  updateStatusBar({
    path,
    pathHint: path ? t("status.copyPath") : undefined,
    context,
    dirty,
    encoding,
    cursor,
    font,
    zoom,
    encode,
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
    windowLabel: getCurrentWebviewWindow().label,
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
    const pinMark = tab.pinned ? " ★" : "";
    btn.textContent = tab.label + pinMark + dirtyMark + (isSplit ? " |" : "");
    btn.draggable = true;
    btn.title = isSplit
      ? `${tab.location_kind} (split pane; Shift+click to move split here; Ctrl+click pin)`
      : `${tab.location_kind} (Shift+click split; Ctrl+click pin; drag to reorder)`;
    setTabA11y(btn, index === data.active_index, `browser-tab-${index}`);
    btn.addEventListener("dragstart", (e) => {
      e.dataTransfer?.setData("text/tab-index", String(index));
      e.dataTransfer!.effectAllowed = "move";
      wrap.classList.add("tab-dragging");
    });
    btn.addEventListener("dragend", () => wrap.classList.remove("tab-dragging"));
    btn.addEventListener("auxclick", (e) => {
      if (e.button !== 1) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      if (tab.pinned) {
        showToast(t("tabs.pinnedNoClose"), "info");
        return;
      }
      void closeTabAt(index, tab.dirty);
    });
    btn.onclick = (e) => {
      if (e.ctrlKey && !e.shiftKey) {
        e.preventDefault();
        void invokeWithToast(
          () => invoke<TabContentDto>("toggle_tab_pin", { index }),
          t("tabs.pinFailed"),
        ).then((d) => d && applyState(d));
        return;
      }
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
    wrap.className = "tab-wrap" + (tab.pinned ? " tab-pinned" : "");
    wrap.addEventListener("dragover", (e) => {
      if (e.dataTransfer?.types.includes("text/tab-index")) {
        e.preventDefault();
        wrap.classList.add("tab-drop-target");
      }
    });
    wrap.addEventListener("dragleave", () => wrap.classList.remove("tab-drop-target"));
    wrap.addEventListener("drop", (e) => {
      wrap.classList.remove("tab-drop-target");
      const raw = e.dataTransfer?.getData("text/tab-index");
      if (!raw) {
        return;
      }
      const from = Number.parseInt(raw, 10);
      if (!Number.isFinite(from) || from === index) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      void invokeWithToast(
        () => invoke<TabContentDto>("reorder_tabs", { fromIndex: from, toIndex: index }),
        t("tabs.reorderFailed"),
      ).then((d) => d && applyState(d));
    });
    if (tab.browse_path) {
      wrap.dataset.dropFolder = tab.browse_path;
      wrap.addEventListener("dragover", (e) => {
        if (pathsFromDataTransfer(e.dataTransfer).length) {
          e.preventDefault();
          wrap.classList.add("tab-drop-target");
        }
      });
      wrap.addEventListener("dragleave", () => wrap.classList.remove("tab-drop-target"));
      wrap.addEventListener("drop", (e) => {
        wrap.classList.remove("tab-drop-target");
        const paths = pathsFromDataTransfer(e.dataTransfer);
        if (!paths.length) {
          return;
        }
        e.preventDefault();
        const mode = e.shiftKey ? "move" : "copy";
        void invoke<TabContentDto>("drop_paths_on_tab", {
          tabIndex: index,
          paths,
          mode,
        }).then((d) => d && applyState(d));
      });
    }
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

async function persistFilesView(view: FilesViewDto) {
  await invoke("set_files_view_prefs", {
    input: {
      sort: view.sort,
      sortAscending: view.sort_ascending,
      showHidden: view.show_hidden,
    },
  });
  await refreshState();
}

const folderFavoritesCache: string[] = [];
let lastBrowseFolderPath = "";

function renderFiles(
  entries: DirEntry[] | null,
  filesView: FilesViewDto,
  browsePath: string,
  favorites: string[],
  recentFiles: RecentFileDto[],
  pinnedPlaces: PinnedLocationDto[],
) {
  filesPanel.innerHTML = "";
  if (!entries) {
    filesBrowserWrap.classList.add("hidden");
    filesSidePanel.classList.add("hidden");
    filesSidePanel.innerHTML = "";
    clearFilesPanelContext();
    return;
  }
  filesBrowserWrap.classList.remove("hidden");
  folderFavoritesCache.length = 0;
  folderFavoritesCache.push(...favorites);
  renderFilesPanel(filesPanel, {
    entries,
    filesView,
    browsePath,
    folderFavorites: folderFavoritesCache,
    recentFiles,
    pinnedPlaces,
    sidePanel: filesSidePanel,
    onApplyState: (data) => applyState(data as TabContentDto),
    persistFilesView,
    onRefresh: refreshState,
  });
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
    const ubmod = s.ubmod_id ? ` · .ubmod ${s.ubmod_id}` : "";
    btn.textContent = `${s.title} (${s.mod_id}${ubmod})`;
    btn.title = s.reason;
    btn.onclick = () => {
      if (s.open_vsx_id) {
        const searchInput = modPortalPanel.querySelector<HTMLInputElement>(".mod-portal-query");
        const optIn = modPortalPanel.querySelector<HTMLInputElement>("#open-vsx-opt-in");
        if (searchInput) {
          searchInput.value = s.open_vsx_id;
          searchInput.focus();
        }
        if (optIn) {
          optIn.checked = true;
        }
        showToast(t("mods.installStub", { modId: s.mod_id }), "info");
        return;
      }
      if (s.ubmod_id) {
        showToast(t("mods.ubmodStub", { modId: s.ubmod_id }), "info");
        return;
      }
      showToast(t("mods.installStub", { modId: s.mod_id }), "info");
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
  optLabel.textContent = t("mods.networkOptIn");
  const searchInput = document.createElement("input");
  searchInput.type = "search";
  searchInput.placeholder = t("mods.searchPlaceholder");
  searchInput.className = "mod-portal-query";
  const searchBtn = document.createElement("button");
  searchBtn.type = "button";
  searchBtn.textContent = t("mods.search");
  const searchStatus = document.createElement("p");
  searchStatus.className = "mod-portal-search-status viewer-note";
  const catalogResults = document.createElement("div");
  catalogResults.className = "mod-portal-catalog";
  const results = document.createElement("div");
  results.className = "mod-portal-results";
  searchRow.appendChild(optIn);
  searchRow.appendChild(optLabel);
  searchRow.appendChild(searchInput);
  searchRow.appendChild(searchBtn);
  modPortalPanel.appendChild(searchRow);
  const catalogNote = document.createElement("p");
  catalogNote.className = "viewer-note";
  catalogNote.textContent = t("mods.catalogNote");
  modPortalPanel.appendChild(catalogNote);
  modPortalPanel.appendChild(catalogResults);
  modPortalPanel.appendChild(searchStatus);
  modPortalPanel.appendChild(results);

  const installedHeading = document.createElement("h3");
  installedHeading.textContent = t("mods.installedHeading");
  modPortalPanel.appendChild(installedHeading);

  const nativeHeading = document.createElement("h4");
  nativeHeading.textContent = t("mods.nativeHeading");
  modPortalPanel.appendChild(nativeHeading);
  const nativeList = document.createElement("div");
  nativeList.className = "mod-portal-installed mod-portal-native";
  modPortalPanel.appendChild(nativeList);

  const vsixHeading = document.createElement("h4");
  vsixHeading.textContent = t("mods.vsixHeading");
  modPortalPanel.appendChild(vsixHeading);
  const installedList = document.createElement("div");
  installedList.className = "mod-portal-installed mod-portal-vsix";
  modPortalPanel.appendChild(installedList);

  const sideloadBtn = document.createElement("button");
  sideloadBtn.type = "button";
  sideloadBtn.className = "mod-suggestion";
  sideloadBtn.textContent = t("mods.sideloadButton");
  sideloadBtn.onclick = async () => {
    try {
      const rec = await invoke<InstalledVsixDto>("pick_and_install_vsix");
      showToast(
        t("mods.installedMeta", {
          extensionId: rec.extension_id,
          commandCount: rec.command_count,
          keybindingCount: rec.keybinding_count,
        }),
        "info",
      );
      await refreshInstalledExtensions(nativeList, installedList);
    } catch (err) {
      showToast(String(err), "error");
    }
  };
  modPortalPanel.appendChild(sideloadBtn);

  void refreshInstalledExtensions(nativeList, installedList);

  async function installFromOpenVsx(hit: OpenVsxHitDto) {
    showToast(t("mods.downloading", { extensionId: hit.extension_id }), "info");
    try {
      const rec = await invoke<InstalledVsixDto>("download_install_vsix", {
        url: hit.url,
        networkOptIn: optIn.checked,
        license: hit.license,
      });
      showToast(t("mods.installedPaletteNote", { extensionId: rec.extension_id }), "info");
      await refreshInstalledExtensions(nativeList, installedList);
    } catch (err) {
      showToast(String(err), "error");
    }
  }

  async function renderCatalog() {
    const entries = await invoke<
      {
        id: string;
        title: string;
        kind: string;
        blurb: string;
        open_vsx_id?: string | null;
        ubmod_id?: string | null;
      }[]
    >("list_mod_catalog", { query: searchInput.value });
    catalogResults.innerHTML = "";
    entries.forEach((entry) => {
      const row = document.createElement("button");
      row.type = "button";
      row.className = "mod-suggestion mod-catalog-row";
      row.textContent = `${entry.title} (${entry.kind})`;
      row.title = entry.blurb;
      row.onclick = () => {
        if (entry.open_vsx_id) {
          searchInput.value = entry.open_vsx_id;
          optIn.checked = true;
          showToast(t("mods.catalogOpenVsx"), "info");
        } else if (entry.ubmod_id) {
          showToast(t("mods.ubmodStub", { modId: entry.ubmod_id }), "info");
        }
      };
      catalogResults.appendChild(row);
    });
  }

  searchInput.addEventListener("input", () => {
    void renderCatalog();
  });
  void renderCatalog();

  searchBtn.onclick = async () => {
    results.innerHTML = "";
    searchStatus.textContent = t("mods.searching");
    try {
      const hits = await invoke<OpenVsxHitDto[]>("search_open_vsx_catalog", {
        query: searchInput.value,
        networkOptIn: optIn.checked,
      });
      searchStatus.textContent = hits.length
        ? t("mods.resultsCount", { count: hits.length })
        : t("mods.noResults");
      hits.forEach((hit) => {
        const row = document.createElement("button");
        row.type = "button";
        row.className = "mod-suggestion";
        row.textContent = t("mods.installRow", {
          name: hit.name,
          version: hit.version,
          publisher: hit.publisher,
        });
        row.title = `${hit.extension_id}${hit.license ? ` · ${hit.license}` : ""}\n${hit.url}`;
        row.onclick = () => installFromOpenVsx(hit);
        results.appendChild(row);
      });
    } catch (err) {
      searchStatus.textContent = String(err);
    }
  };
}

async function refreshInstalledExtensions(
  nativeContainer: HTMLElement,
  vsixContainer: HTMLElement,
) {
  nativeContainer.innerHTML = "";
  vsixContainer.innerHTML = "";
  try {
    const native = await invoke<InstalledNativeModDto[]>("list_installed_native_mods");
    if (!native.length) {
      nativeContainer.textContent = t("mods.noNative");
    } else {
      native.forEach((mod) => {
        const row = document.createElement("div");
        row.className = "vsix-installed-row";
        const label = document.createElement("span");
        label.textContent = t("mods.nativeRow", {
          title: mod.title,
          name: mod.name,
          version: mod.version,
        });
        if (mod.load_block_reason) {
          const block = document.createElement("span");
          block.className = "viewer-note";
          block.textContent = t("mods.nativeBlocked", { reason: mod.load_block_reason });
          label.appendChild(document.createElement("br"));
          label.appendChild(block);
        }
        const toggle = document.createElement("button");
        toggle.type = "button";
        toggle.textContent = mod.enabled ? t("mods.disable") : t("mods.enable");
        toggle.onclick = async () => {
          await invoke("set_native_mod_enabled", {
            name: mod.name,
            enabled: !mod.enabled,
          });
          await refreshInstalledExtensions(nativeContainer, vsixContainer);
        };
        row.appendChild(label);
        row.appendChild(toggle);
        nativeContainer.appendChild(row);
      });
    }
  } catch {
    nativeContainer.textContent = t("mods.nativeListFailed");
  }
  try {
    const list = await invoke<InstalledVsixDto[]>("list_installed_vsix");
    if (!list.length) {
      vsixContainer.textContent = t("mods.noInstalled");
      return;
    }
    list.forEach((ext) => {
      const row = document.createElement("div");
      row.className = "vsix-installed-row";
      const label = document.createElement("span");
      label.textContent = `${ext.extension_id} v${ext.version} — ${ext.command_count} cmd / ${ext.keybinding_count} keys`;
      const toggle = document.createElement("button");
      toggle.type = "button";
      toggle.textContent = ext.enabled ? t("mods.disable") : t("mods.enable");
      toggle.onclick = async () => {
        await invoke("set_vsix_extension_enabled", {
          extensionId: ext.extension_id,
          enabled: !ext.enabled,
        });
        await refreshInstalledExtensions(nativeContainer, vsixContainer);
      };
      const remove = document.createElement("button");
      remove.type = "button";
      remove.textContent = t("mods.uninstall");
      remove.onclick = async () => {
        await invoke("uninstall_vsix_extension", { extensionId: ext.extension_id });
        showToast(t("mods.removed", { extensionId: ext.extension_id }), "info");
        await refreshInstalledExtensions(nativeContainer, vsixContainer);
      };
      row.appendChild(label);
      row.appendChild(toggle);
      row.appendChild(remove);
      vsixContainer.appendChild(row);
    });
  } catch {
    vsixContainer.textContent = t("mods.listFailed");
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

async function renderSessionOptIn() {
  try {
    let row = document.getElementById("session-restore-row");
    if (!row) {
      row = document.createElement("div");
      row.id = "session-restore-row";
      row.className = "session-restore-row";
      recentPanel.appendChild(row);
    }
    row.innerHTML = "";
    const settingsBtn = document.createElement("button");
    settingsBtn.type = "button";
    settingsBtn.className = "recent-link";
    settingsBtn.textContent = t("settings.open");
    settingsBtn.onclick = () => {
      omnibox.value = "unibrowser://settings";
      void navigateOmnibox();
    };
    row.appendChild(settingsBtn);
  } catch {
    /* optional */
  }
}

function renderBookmarkBar(
  enabled: boolean,
  pins: PinnedLocationDto[],
  quickLinks: BookmarkQuickLinkDto[] = [],
) {
  bookmarkBar.innerHTML = "";
  if (!enabled) {
    bookmarkBar.classList.add("hidden");
    return;
  }
  bookmarkBar.classList.remove("hidden");
  pins.forEach((pin) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "bookmark-link";
    if (pin.id === "home") {
      btn.textContent = t("bookmark.home");
    } else if (pin.id === "download-manager") {
      btn.textContent = t("bookmark.downloadManager");
    } else if (pin.id === "downloads") {
      btn.textContent = t("bookmark.downloads");
    } else if (pin.id === "new-project") {
      btn.textContent = t("bookmark.project");
    } else if (pin.id === "settings") {
      btn.textContent = t("bookmark.settings");
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
  quickLinks.forEach((link) => {
    const url = link.url.trim();
    if (!url) {
      return;
    }
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "bookmark-link quick-link";
    const label = link.label.trim();
    btn.textContent = label || url;
    btn.title = url;
    btn.onclick = () => {
      omnibox.value = url;
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
    recentPanel.appendChild(
      emptyStateBlock(t("recent.empty"), [t("recent.emptyHint")]),
    );
    return;
  }
  const head = document.createElement("div");
  head.className = "recent-head";
  const clearBtn = document.createElement("button");
  clearBtn.type = "button";
  clearBtn.className = "recent-clear";
  clearBtn.textContent = t("recent.clear");
  clearBtn.onclick = () =>
    invoke<TabContentDto>("clear_recent_files").then((d) => d && applyState(d));
  head.appendChild(clearBtn);
  recentPanel.appendChild(head);

  const list = document.createElement("ul");
  list.className = "recent-list";
  entries.forEach((entry) => {
    const li = document.createElement("li");
    li.className = entry.pinned ? "recent-item pinned" : "recent-item";
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "recent-link";
    btn.textContent = (entry.pinned ? "★ " : "") + entry.label;
    btn.title = entry.path;
    btn.onclick = () =>
      invoke<TabContentDto>("open_path", { path: entry.path }).then(applyState);
    const pin = document.createElement("button");
    pin.type = "button";
    pin.className = "recent-pin";
    pin.textContent = entry.pinned ? t("recent.unpin") : t("recent.pin");
    pin.onclick = () =>
      invoke<TabContentDto>("toggle_recent_pin", { path: entry.path }).then(
        (d) => d && applyState(d),
      );
    li.appendChild(btn);
    li.appendChild(pin);
    list.appendChild(li);
  });
  recentPanel.appendChild(list);
  void renderSessionOptIn();
}

async function runPageZoom(commandId: string) {
  await textEditor.flush();
  const msg = await invoke<string>("execute_command", { commandId });
  const pct = msg.match(/(\d+)%/);
  if (pct) {
    webZoomPercent = Number(pct[1]);
  }
  const toast = await handleShellCommandMessage(msg);
  if (toast) {
    showToast(String(msg), "info");
  }
  await refreshCommandManager();
  await refreshState();
}

async function handleShellCommandMessage(msg: string): Promise<boolean> {
  if (msg === "document-find") {
    openDocumentFind();
    return false;
  }
  if (msg === "export-print-pdf") {
    await runExportPrintPdf(documentViewerHost);
    return false;
  }
  if (msg === "devtools-open" || msg === "devtools-close") {
    return false;
  }
  if (msg === "keys-overlay") {
    await openKeysOverlay();
    return false;
  }
  if (msg === "reader-on") {
    showToast(t("reader.on"), "info");
    return false;
  }
  if (msg === "reader-off") {
    showToast(t("reader.off"), "info");
    return false;
  }
  const pct = msg.match(/(\d+)%/);
  if (pct) {
    webZoomPercent = Number(pct[1]);
  }
  return true;
}

async function processCommandResult(msg: string, commandId?: string) {
  if (commandId === "unibrowser.script.demo.hello") {
    appendScriptOutput(String(msg));
    return;
  }
  if (msg === "palette") {
    togglePalette();
    return;
  }
  const toast = await handleShellCommandMessage(msg);
  if (toast && msg !== "Cancelled" && commandId !== "unibrowser.script.demo.hello") {
    showToast(String(msg), "info");
  }
}

async function runUiCommand(commandId: string) {
  await textEditor.flush();
  const msg = await invoke<string>("execute_command", { commandId });
  await processCommandResult(msg, commandId);
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
    await documentViewer.apply(data.split_document_viewer, splitDocumentViewerHost, {
      downloadHistory: data.download_history ?? [],
      onRefreshState: refreshState,
    });
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
  if (data.chrome_notice) {
    showToast(data.chrome_notice, "info");
  }
  onboardingBanner.classList.toggle("hidden", !data.show_onboarding);
  showOnboardingTour(data.show_onboarding_tour);
  if (data.browse_folder_path) {
    lastBrowseFolderPath = data.browse_folder_path;
  }
  const active = data.tabs[data.active_index];
  const showText = Boolean(data.text_document) && !active?.is_http;
  const showViewer = Boolean(data.document_viewer) && !active?.is_http;
  const showHome = data.show_new_tab_page && !active?.is_http;
  renderTabs(data);
  const remoteHint = active?.is_http ? ` · ${t("trust.remoteWeb")}` : "";
  modsLine.textContent = modsLineText(data.mods_loaded) + remoteHint;
  renderBookmarkBar(
    data.bookmark_bar_enabled ?? true,
    data.pinned_locations ?? [],
    data.bookmark_quick_links ?? [],
  );
  const showDiff = Boolean(data.diff_view);
  if (showDiff && data.diff_view) {
    await diffEditorView.apply(data.diff_view, diffEditorHost);
    textEditor.hide(textEditorHost);
    documentViewer.hide(documentViewerHost);
  } else {
    diffEditorView.hide(diffEditorHost);
  }
  if (showHome) {
    filesBrowserWrap.classList.add("hidden");
    renderNewTabPage(newTabPageHost, {
      entries: data.directory_entries,
      recent: data.recent_files ?? [],
      browsePath: data.browse_folder_path,
      sections: data.ntp_sections ?? {
        show_hero: true,
        show_actions: true,
        show_browse: true,
        show_recent: true,
      },
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
      filesBrowserWrap.classList.add("hidden");
      filesSidePanel.classList.add("hidden");
    } else {
      renderFiles(
        data.directory_entries,
        data.files_view ?? { sort: "name", sort_ascending: true, show_hidden: false },
        data.browse_folder_path,
        data.folder_favorites ?? [],
        data.recent_files ?? [],
        data.pinned_locations ?? [],
      );
    }
  }
  renderModPortal(data.mod_portal);
  const hasPortal = Boolean(data.mod_portal);
  renderDoc(data.document_status, showText, hasPortal, showViewer);
  if (data.directory_entries && data.directory_entries.length > 0) {
    void markChecklistStep("browse_folder");
  }
  if (data.text_document || data.document_viewer) {
    void markChecklistStep("open_file");
  }
  renderRecentFiles(data.recent_files ?? []);
  if (!showDiff) {
    if (showText && data.text_document) {
      await textEditor.apply(data.text_document, textEditorHost);
      documentViewer.hide(documentViewerHost);
    } else {
      textEditor.hide(textEditorHost);
    }
    if (showViewer && data.document_viewer) {
      await documentViewer.apply(data.document_viewer, documentViewerHost, {
        downloadHistory: data.download_history ?? [],
        onRefreshState: refreshState,
      });
    } else if (!showText) {
      documentViewer.hide(documentViewerHost);
    }
  }
  await applySplitPane(data);
  const viewerKind = data.document_viewer?.viewer;
  if (showText) {
    setDocumentFindSurface("monaco");
  } else if (showViewer && (viewerKind === "docx" || viewerKind === "epub")) {
    setDocumentFindSurface("viewer");
  } else {
    setDocumentFindSurface("none");
    closeDocumentFind();
  }
  if (active?.is_http) {
    setZoomHandler("web", (delta) => {
      const cmd =
        delta > 0
          ? "unibrowser.page.zoom.increase"
          : "unibrowser.page.zoom.decrease";
      void runPageZoom(cmd);
    });
  } else if (showViewer && data.document_viewer) {
    const kind = data.document_viewer.viewer;
    if (kind === "pdf" || kind === "image") {
      setZoomHandler("viewer", (delta) => {
        const cmd =
          delta > 0
            ? "unibrowser.page.zoom.increase"
            : "unibrowser.page.zoom.decrease";
        void runPageZoom(cmd);
      });
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
    e.preventDefault();
    omniboxAutocomplete.applyActiveSuggestion();
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

const omniboxAutocomplete = bindOmniboxAutocomplete(omnibox, omniboxSuggestions, () =>
  navigateOmnibox(),
);

document.getElementById("btn-tab-search")!.onclick = () => toggleTabSearch();
tabSearchQuery.addEventListener("input", () => void refreshTabSearchResults());

listen("browser-state-updated", () => {
  void refreshState();
});

btnSplit.onclick = () => {
  void invokeWithToast(() => invoke<TabContentDto>("toggle_split_view"), t("chrome.splitFailed")).then(
    (data) => {
      if (data) {
        void markChecklistStep("split_view");
        applyState(data);
      }
    },
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
    destination: null,
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
  void refreshState();
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
  const choice = await openNewProjectDialog();
  if (!choice) {
    return;
  }
  const home = await invoke<string>("get_home_dir").catch(() => "/tmp");
  await invoke<TabContentDto>("create_unibrow_project", {
    parentDir: home,
    name: choice.name,
    template: choice.template,
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

function closePalette() {
  paletteOpen = false;
  palette.classList.add("hidden");
}

function togglePalette() {
  paletteOpen = !paletteOpen;
  palette.classList.toggle("hidden", !paletteOpen);
  if (paletteOpen) {
    tabSearchOpen = false;
    tabSearchPanel.classList.add("hidden");
    paletteQuery.placeholder = t("palette.placeholder", { chord: paletteChordLabel });
    paletteQuery.focus();
    void markChecklistStep("palette");
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
  await processCommandResult(msg, commandId);
  await refreshCommandManager();
  await refreshState();
}

document.addEventListener(
  "keydown",
  async (e) => {
  if (e.repeat) {
    return;
  }
  if (e.key === "Escape") {
    if (isKeysOverlayOpen()) {
      e.preventDefault();
      closeKeysOverlay();
      return;
    }
    if (paletteOpen) {
      e.preventDefault();
      closePalette();
      return;
    }
    if (tabSearchOpen) {
      e.preventDefault();
      toggleTabSearch();
      return;
    }
  }
  if (!filesBrowserWrap.classList.contains("hidden") && handleFilesKeyDown(e)) {
    return;
  }
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "f") {
    e.preventDefault();
    toggleGlobalFind(lastBrowseFolderPath);
    return;
  }
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "t") {
    e.preventDefault();
    toggleTabSearch();
    return;
  }
  if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "f") {
    e.preventDefault();
    toggleDocumentFind();
    return;
  }
  if (e.ctrlKey && e.altKey && e.key.toLowerCase() === "z") {
    e.preventDefault();
    void invokeWithToast(() => invoke<TabContentDto>("document_undo"), t("editor.nothingToUndo")).then(
      (data) => data && applyState(data),
    );
    return;
  }
  if (e.ctrlKey && e.altKey && e.key.toLowerCase() === "y") {
    e.preventDefault();
    void invokeWithToast(() => invoke<TabContentDto>("document_redo"), t("editor.nothingToRedo")).then(
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
      paletteQuery.placeholder = t("palette.placeholder", { chord: paletteChordLabel });
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
    const chord = entry.chord ? `  ·  ${entry.chord}` : "";
    row.textContent = entry.title + chord + (entry.enabled ? "" : " (greyed)");
    row.disabled = !entry.enabled;
    row.onclick = () => {
      if (entry.open_path) {
        void invoke<TabContentDto>("open_path", { path: entry.open_path }).then((d) => {
          if (d) {
            void applyState(d);
          }
          togglePalette();
        });
        return;
      }
      void invoke<string>("execute_command", { commandId: entry.command_id }).then(async (msg) => {
        togglePalette();
        await processCommandResult(msg, entry.command_id);
        await refreshCommandManager();
        await refreshState();
      });
    };
    paletteResults.appendChild(row);
  });
});

window.addEventListener("resize", () => syncContentBounds());

window.addEventListener("unibrowser-navigate", (event) => {
  const detail = (event as CustomEvent<string>).detail;
  if (detail) {
    omnibox.value = detail;
    void navigateOmnibox();
  }
});

listen<DownloadEventPayload | string>("unibrowser-download", (event) => {
  const payload =
    typeof event.payload === "string"
      ? { url: event.payload, destination: null }
      : event.payload;
  void (async () => {
    const prefs = await invoke<{ open_downloads_in_system_browser: boolean }>("get_preferences");
    const url = payload.url.trim();
    if (
      prefs.open_downloads_in_system_browser &&
      (url.startsWith("http://") || url.startsWith("https://"))
    ) {
      try {
        await invoke("open_url_externally", { url });
        showToast(t("download.openedSystemBrowser"), "info");
      } catch (err) {
        showToast(String(err), "error");
      }
      return;
    }
    const data = await invokeWithToast(
      () =>
        invoke<TabContentDto>("handle_download_url", {
          url: payload.url,
          destination: payload.destination ?? null,
        }),
      t("toast.errorDefault"),
    );
    if (!data) {
      await invoke("record_download_failure", {
        url: payload.url,
        error: t("toast.errorDefault"),
      });
      return;
    }
    if (!data.chrome_notice) {
      const prefs = await invoke<{ download_preference: string }>("get_preferences");
      if (prefs.download_preference === "open_in_new_tab") {
        showToast(t("download.openedTab"), "info");
      }
    }
    void applyState(data);
  })();
});

listen<{ percent: number; stage: string }>("unibrowser-encode-progress", (event) => {
  const { percent, stage } = event.payload;
  if (percent >= 100) {
    setEncodeProgress(null);
    showToast(t("encode.done"), "info");
  } else {
    setEncodeProgress(percent, stage);
  }
  void refreshState();
});

window.addEventListener("unibrowser-refresh", (event) => {
  const detail = (event as CustomEvent<TabContentDto>).detail;
  if (detail) {
    void applyState(detail);
  } else {
    void refreshState();
  }
});

bindRovingTablist(tabStrip, () => tabButtons);

bindStatusBar(statusBarEl);
initGlobalFind({
  defaultRoot: lastBrowseFolderPath,
  onOpenPath: (path) => {
    void invoke<TabContentDto>("open_path", { path }).then((d) => d && applyState(d));
  },
});
setPathCopyHandler((path) => {
  void copyText(path, () => showToast(t("status.pathCopied"), "info"));
});
setCursorStatusListener((pos) => {
  editorLine = pos.line;
  editorCol = pos.column;
  patchStatusCursor(`Ln ${editorLine}, Col ${editorCol}`);
});
setViewerZoomListener((_scale, label) => {
  lastViewerZoomLabel = label;
  const zoomEl = document.getElementById("status-zoom-label");
  if (zoomEl && lastViewerZoomLabel) {
    zoomEl.textContent = lastViewerZoomLabel;
    zoomEl.classList.remove("hidden");
  }
});

function deferNonCriticalInit(fn: () => void) {
  const run = () => {
    if (typeof requestIdleCallback === "function") {
      requestIdleCallback(fn, { timeout: 2500 });
    } else {
      window.setTimeout(fn, 1);
    }
  };
  if (document.readyState === "complete") {
    run();
  } else {
    window.addEventListener("load", run, { once: true });
  }
}

initScriptOutputPanel();
initKeysOverlay();
window.addEventListener("unibrowser-prefs-changed", () => {
  void refreshState();
});
listen<string>("unibrowser-script-output", (event) => {
  appendScriptOutput(String(event.payload));
});
applyChromeI18n();
void loadUiLocale().then(() => applyChromeI18n());
void loadThemePreference().then(() => {
  themeBtn.textContent = themeButtonLabel();
});
void loadAccessibilityPreferences();
void loadExperimentalSpellcheckPref();
void refreshState().then(() => {
  void maybePromptProjectRecovery((data) => {
    void applyState(data as TabContentDto);
  });
});
window.addEventListener("focus", () => {
  void invoke("reload_keybindings").then(() => loadKeybindingHints());
});

deferNonCriticalInit(() => {
  commandManager.bindPinControl(
    commandManagerPin,
    commandManagerTabs,
    commandManagerCommands,
    floatingToolbarHost,
    runUiCommand,
  );
  document.getElementById("command-manager-export")!.onclick = async () => {
    const path = await invoke<string | null>("export_command_manager_layout_file");
    if (path) {
      showToast(t("commandManager.exported", { path }), "info");
    }
  };
  window.addEventListener("unibrowser-workspace-layout-loaded", () => {
    void refreshCommandManager();
  });
  document.getElementById("command-manager-import")!.onclick = async () => {
    try {
      await invoke("import_command_manager_layout_file");
      showToast(t("commandManager.imported"), "info");
      await refreshCommandManager();
    } catch (err) {
      showToast(String(err), "error");
    }
  };
  void loadKeybindingHints();
  void invoke<{
    auto_save_enabled: boolean;
    auto_save_interval_secs: number;
    project_autosave_enabled: boolean;
    project_autosave_interval_secs: number;
  }>("get_preferences").then((prefs) => {
    configureAutoSave(prefs.auto_save_enabled, prefs.auto_save_interval_secs);
    configureProjectAutoSave(
      prefs.project_autosave_enabled,
      prefs.project_autosave_interval_secs,
    );
  });
});
