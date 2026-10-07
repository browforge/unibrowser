import "./shellBootstrap";
import {
  hideShellFallback,
  reportSubsystemError,
  safeSubsystemInit,
} from "./shellBootstrap";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";
import "./ui/tokens.css";
import "./ui/compactChrome.css";
import "./ui/quietUtilityShell.css";
import { initCommandPalette } from "./ui/commandPalette";
import { initDockPanel } from "./ui/dockPanel";

/** WebDriver waits on this; set once shell chrome from index.html is in the DOM. */
function expectedCaptureContextLabel(launchPaths: string[]): string | null {
  const joined = launchPaths.join("|").toLowerCase();
  if (joined.includes(".pdf")) {
    return "PDF Annotation";
  }
  if (
    joined.includes(".svg") ||
    joined.includes(".png") ||
    joined.includes(".jpg") ||
    joined.includes(".jpeg")
  ) {
    return "Image";
  }
  return null;
}

function markUbReadyWhenDomChromePresent() {
  const mark = () => {
    if (document.getElementById("title-bar") && document.getElementById("omnibox")) {
      hideShellFallback();
    }
  };
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mark, { once: true });
  } else {
    mark();
  }
}
markUbReadyWhenDomChromePresent();
syncOpenVsxNetworkDataset();
exposeShellChromeAudit();
exposeChromeMetrics();
void invoke<{ scenarios: string[] } | null>("get_self_test_plan").then((plan) => {
  if (plan) {
    void invoke("self_test_submit_shell_chrome_audit", { errors: runShellChromeAudit() });
  }
});
void invoke<string | null>("take_doc_capture_path").then((docCapture) => {
  if (!docCapture) {
    return;
  }
  const pushAudit = () => {
    void invoke("self_test_submit_shell_chrome_audit", { errors: runShellChromeAudit() });
    window.setTimeout(pushAudit, 200);
  };
  pushAudit();
});

import { commandManager, getStatusContextMode } from "./commandManager";
import {
  textEditor,
  type TextDocumentDto,
  getActiveTextContent,
  setCursorStatusListener,
  setReadingStatsListener,
} from "./textEditor";
import { textReadingStats } from "./textStats";
import {
  armPdfHighlightMode,
  documentViewer,
  runViewerContextCommand,
  type DocumentViewerDto,
} from "./documentViewer";
import { bindRovingTablist, setTabA11y } from "./rovingTabs";
import { loadAccessibilityPreferences } from "./accessibility";
import { loadExperimentalSpellcheckPref } from "./experimentalSpellcheck";
import {
  loadProofingPrefs,
  PROOFING_NEXT,
  PROOFING_PREVIOUS,
  PROOFING_TOGGLE,
  toggleProofingSquiggles,
} from "./proofing";
import { markChecklistStep } from "./onboardingChecklist";
import { cycleTheme, loadThemePreference } from "./theme";
import { initMonacoEditorThemes } from "./monacoEditorTheme";
import { applyThemeFromDemoState } from "./theme";
import { setEditorThemePreference } from "./textEditor";
import { bindOmniboxAutocomplete } from "./omniboxUi";
import { isInsecureHttpNavigation } from "./omniboxSecurity";
import {
  exitContentFullscreen,
  initContentFullscreen,
  isContentFullscreen,
  toggleContentFullscreen,
} from "./fullscreenContent";
import { initNotificationCenter } from "./notifications";
import { tryMediaPictureInPicture } from "./mediaPip";
import { invokeWithToast, showActionToast, showErrorToast, showToast } from "./toast";
import { initProblemsPanel, toggleProblemsPanel } from "./problemsPanel";
import {
  bindStatusBar,
  patchStatusCursor,
  patchStatusReading,
  setFontResetHandler,
  setPathCopyHandler,
  setZoomHandler,
  updateStatusBar,
} from "./statusBar";
import { getEncodeProgress, setEncodeProgress } from "./encodeProgress";
import { runExportPrintPdf, runPrintSelection } from "./exportPdf";
import {
  isTabSleeping,
  pruneTabActivity,
  TAB_SLEEP_IDLE_MS,
  touchTabActivity,
} from "./tabSleep";
import { copyText, readClipboardText } from "./clipboard";
import { t } from "./i18n";
import { renderNewTabPage, hideNewTabPage } from "./newTabPage";
import { setViewerPdfPageListener, setViewerZoomListener } from "./viewerZoom";
import { clearImagePixelSample, getImagePixelSample } from "./viewerPixelSample";
import { applyChromeI18n } from "./chromeI18n";
import { initAppMenu } from "./appMenu";
import {
  initCompactTitleBar,
  syncTitleBar,
  updateTitleBarNavVisibility,
} from "./ui/compactTitleBar";
import { initActivityRail } from "./ui/activityRail";
import { exposeChromeMetrics } from "./ui/chromeMetrics";
import { createLucideIcon, tabIconForKind } from "./ui/lucideIcon";
import { diffEditorView, type DiffViewDto } from "./diffEditor";
import { initHttpLoadingListeners, syncHttpLoadingForTab } from "./httpLoadingOverlay";
import { configureAutoSave } from "./autoSave";
import { configureProjectAutoSave } from "./projectAutoSave";
import { maybePromptProjectRecovery } from "./projectRecoveryPrompt";
import { renderRecoveryBar, type RecoveryBarItem } from "./recoveryBar";
import { showInlineConfirm, showPickList } from "./inlineUi";
import { initLocatorNavigationListeners } from "./locatorListeners";
import { showOnboardingTour } from "./onboardingTour";
import {
  appendScriptOutput,
  initScriptOutputPanel,
  syncBottomPanelEmptyState,
} from "./scriptOutputPanel";
import { initTerminalPanel, toggleTerminalPanel } from "./terminalPanel";
import { initAiAssistantPanel, refreshAiAssistantChrome, setAiPanelVisible } from "./aiAssistantPanel";
import { syncContextDockPolicy } from "./contextDock";
import { applyDemoState } from "./demoState";
import {
  appendModCatalogCard,
  appendModSuggestionCard,
  renderModNetworkStatus,
  enableOpenVsxForSearch,
  type ModCatalogEntry,
} from "./modPortalUi";
import {
  isOpenVsxNetworkEnabled,
  setOpenVsxNetworkEnabled,
  syncOpenVsxNetworkDataset,
} from "./modsOpenVsx";
import {
  assertLightThemeForCapture,
  collectDocCaptureDiagnostics,
  validateCommandContextLabel,
  validateCommandManagerForCapture,
  validateProblemsForCapture,
  formatDiagnosticsForLog,
  validateEditorForCapture,
} from "./docCaptureAsserts";
import { validateShellDensityContrastForCapture } from "./ui/shellContrast";
import {
  editorReadyForCapture,
  launchPathsNeedEditor,
  waitForShellPaintReady,
} from "./shellPaintReady";
import { renderTitleBreadcrumb } from "./titleBreadcrumb";
import { exposeShellChromeAudit, runShellChromeAudit } from "./shellChromeAudit";
import { loadUiLocale } from "./i18n";
import { openNewProjectDialog } from "./newProjectDialog";
import { emptyStateBlock } from "./emptyState";
import {
  clearFilesPanelContext,
  handleFilesKeyDown,
  renderFilesPanel,
  runFilesCommand,
} from "./filesPanel";
import { pathsFromDataTransfer } from "./filesDrag";
import { initGlobalFind, toggleGlobalFind } from "./globalFind";
import {
  hideWorkspaceQuickOpen,
  initWorkspaceQuickOpen,
  toggleWorkspaceQuickOpen,
} from "./workspaceQuickOpen";
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
import { applyImageCompare, type ImageCompareViewDto } from "./imageCompare";
import { runCaptureVisibleTab } from "./tabCapture";
import { initToolbarOnlyShell } from "./toolbarOnly";
import { toggleLayoutEditMode } from "./layoutEditMode";

interface TabDto {
  id: number;
  label: string;
  location_kind: string;
  is_directory: boolean;
  is_http: boolean;
  dirty: boolean;
  pinned: boolean;
  browse_path?: string | null;
  muted: boolean;
  mute_available: boolean;
}

interface TabSearchHit {
  window_label: string;
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

interface UnknownFileBannerDto {
  path: string;
  extension: string;
  suggestions: ModSuggestionDto[];
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
  compatibility_level: string;
  compatibility_missing_apis: string[];
  theme_notes: string[];
  grammar_notes: string[];
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
  unknown_file_banner: UnknownFileBannerDto | null;
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
  image_compare_view?: ImageCompareViewDto | null;
  pinned_locations: PinnedLocationDto[];
  chrome_notice?: string | null;
  close_undo_available?: boolean;
  extension_tip?: {
    message: string;
    mod_id: string;
    mod_title: string;
  } | null;
  ntp_sections: {
    show_hero: boolean;
    show_actions: boolean;
    show_browse: boolean;
    show_recent: boolean;
  };
  files_view: FilesViewDto;
  folder_favorites: string[];
  workspace_root?: string | null;
  git_is_repo?: boolean;
  git_dirty_paths?: string[];
  download_history?: DownloadHistoryEntryDto[];
  bookmark_bar_enabled?: boolean;
  bookmark_quick_links?: BookmarkQuickLinkDto[];
  recovery_bar?: RecoveryBarItem[];
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
  opened_at_ms?: number;
}

const omnibox = document.getElementById("omnibox") as HTMLInputElement;
const tabStrip = document.getElementById("tab-strip")!;
const filesBrowserWrap = document.getElementById("files-browser-wrap")!;
const filesPanel = document.getElementById("files-panel")!;
const filesSidePanel = document.getElementById("files-side-panel")!;
const docPanel = document.getElementById("doc-panel")!;
const commandManagerGroups = document.getElementById("command-manager-groups")!;
const commandManagerContext = document.getElementById("command-manager-context")!;
const commandManagerOverflow = document.getElementById("command-manager-overflow");
const commandManagerPin = document.getElementById("command-manager-pin")!;
const commandManagerCollapse = document.getElementById("command-manager-collapse");
const floatingToolbarHost = document.getElementById("floating-toolbar-host")!;
const statusBarEl = document.getElementById("status-bar")!;
const newTabPageHost = document.getElementById("new-tab-page-host")!;
const contentHostEl = document.getElementById("content-host")!;
const splitPrimaryEl = document.getElementById("split-primary")!;
let paletteChordHint = "Ctrl+Shift+P for command palette";
let webZoomPercent = 100;
let lastAutosaveAtMs = 0;
let lastViewerZoomLabel = "";
let lastPdfPageLabel = "";
let lastPdfAnnotationCount = 0;

function formatAutosaveClock(ms: number): string {
  if (!ms) {
    return "";
  }
  return new Date(ms).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}
let editorLine = 1;
let editorCol = 1;
const bookmarkBar = document.getElementById("bookmark-bar")!;
const diffEditorHost = document.getElementById("diff-editor-host")!;
const imageCompareHost = document.getElementById("image-compare-host")!;
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
const modPortalPanel = document.getElementById("mod-portal-panel")!;
const onboardingBanner = document.getElementById("onboarding-banner")!;
const onboardingDismiss = document.getElementById("onboarding-dismiss")!;
const recentPanel = document.getElementById("recent-panel")!;
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
  const viewerKind = data.document_viewer?.viewer;
  let cursor = showText && data.text_document ? `Ln ${editorLine}, Col ${editorCol}` : "";
  if (!cursor && showViewer && viewerKind === "image") {
    const sample = getImagePixelSample();
    cursor = sample;
  }
  if (showViewer && viewerKind !== "image") {
    clearImagePixelSample();
  }
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
  let reading = "";
  if (showText && data.text_document) {
    const text = getActiveTextContent() ?? data.text_document.content;
    const stats = textReadingStats(text);
    if (stats.words > 0) {
      reading = t("status.reading", {
        words: String(stats.words),
        minutes: String(stats.minutes),
      });
    }
  } else if (showViewer && viewerKind === "docx") {
    const text = documentViewer.getReadableText(documentViewerHost);
    if (text) {
      const stats = textReadingStats(text);
      if (stats.words > 0) {
        reading = t("status.reading", {
          words: String(stats.words),
          minutes: String(stats.minutes),
        });
      }
    }
  } else if (showViewer && viewerKind === "pdf" && lastPdfPageLabel) {
    reading = lastPdfPageLabel;
  }
  const autosave =
    lastAutosaveAtMs > 0
      ? t("status.autosave", { time: formatAutosaveClock(lastAutosaveAtMs) })
      : "";
  const compact = document.body.classList.contains("ub-status-compact");
  let saved =
    compact
      ? autosave || (lastAutosaveAtMs > 0 ? `Saved ${formatAutosaveClock(lastAutosaveAtMs)}` : "Saved")
      : undefined;
  if (compact && showViewer && viewerKind === "pdf" && lastPdfAnnotationCount > 0) {
    saved = "Annotations saved";
  }
  const language =
    compact && showText && data.text_document
      ? data.text_document.language?.replace("typescript", "TypeScript") ?? "TypeScript"
      : undefined;
  let zoomOut = compact ? zoom || lastViewerZoomLabel || "100%" : zoom;
  if (compact && showViewer && viewerKind === "pdf" && lastPdfAnnotationCount > 0) {
    zoomOut = `${lastPdfAnnotationCount} annotation${lastPdfAnnotationCount === 1 ? "" : "s"} · ${zoomOut}`;
  }
  const useBreadcrumb =
    Boolean(path) &&
    !active?.is_http &&
    !active?.is_directory &&
    (showText || showViewer || Boolean(path?.includes(".")));
  renderTitleBreadcrumb(path ?? null, useBreadcrumb);
  updateStatusBar({
    path,
    pathHint: path ? t("status.copyPath") : undefined,
    context,
    dirty,
    encoding: compact ? encoding || "UTF-8" : encoding,
    cursor,
    font,
    zoom: zoomOut,
    encode,
    reading,
    autosave,
    saved,
    language,
    aiState: compact ? "AI: Read" : undefined,
    hint: data.status.includes("command palette") ? paletteChordHint : data.status,
  });
}

const tabSearchPanel = document.getElementById("tab-search")!;
const tabSearchQuery = document.getElementById("tab-search-query") as HTMLInputElement;
const tabSearchResults = document.getElementById("tab-search-results")!;
let tabSearchOpen = false;

let paletteOpen = false;
let paletteChordLabel = "Ctrl+Shift+P";
let paletteController: ReturnType<typeof initCommandPalette> | null = null;

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

let lastTabSnapshot: TabContentDto | null = null;

function applyTabSleepMarks() {
  if (!lastTabSnapshot) {
    return;
  }
  const data = lastTabSnapshot;
  data.tabs.forEach((tab, index) => {
    const btn = tabButtons[index];
    if (!btn) {
      return;
    }
    const sleeping =
      tab.is_http &&
      index !== data.active_index &&
      isTabSleeping(tab.id, tab.is_http, index === data.active_index);
    btn.classList.toggle("tab-sleeping", sleeping);
    const fullTitle = btn.dataset.fullTitle ?? tab.location_kind;
    btn.title = sleeping
      ? `${fullTitle} — ${t("tabs.sleepingHint", { minutes: Math.round(TAB_SLEEP_IDLE_MS / 60000) })}`
      : fullTitle;
  });
}

function renderTabs(data: TabContentDto) {
  lastTabSnapshot = data;
  pruneTabActivity(data.tabs.map((t) => t.id));
  const active = data.tabs[data.active_index];
  if (active) {
    touchTabActivity(active.id);
  }
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
      "tab title-tab" +
      (index === data.active_index ? " active" : "") +
      (isSplit ? " split-pane-tab" : "");
    btn.innerHTML = "";
    const tabIcon = createLucideIcon(
      tabIconForKind(tab.location_kind, tab.is_http, tab.is_directory),
      16,
      "ub-icon svg-icon",
    );
    btn.appendChild(tabIcon);
    const tabLabel = document.createElement("span");
    tabLabel.className = "tab-label";
    const dirtyMark = tab.dirty ? " *" : "";
    const pinMark = tab.pinned ? " [pinned]" : "";
    const muteMark = tab.mute_available && tab.muted ? " (muted)" : "";
    tabLabel.textContent = tab.label + pinMark + dirtyMark + muteMark + (isSplit ? " |" : "");
    btn.appendChild(tabLabel);
    btn.draggable = true;
    const fullTitle = isSplit
      ? t("tabs.titleSplit", { kind: tab.location_kind })
      : t("tabs.titleDefault", { kind: tab.location_kind });
    btn.dataset.fullTitle = fullTitle;
    btn.title = fullTitle;
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
      touchTabActivity(tab.id);
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
    if (tab.mute_available) {
      const muteBtn = document.createElement("button");
      muteBtn.type = "button";
      muteBtn.className = "tab-mute" + (tab.muted ? " tab-muted" : "");
      muteBtn.setAttribute(
        "aria-label",
        tab.muted ? `Unmute ${tab.label}` : `Mute ${tab.label}`,
      );
      muteBtn.title = tab.muted ? "Unmute tab" : "Mute tab";
      muteBtn.textContent = tab.muted ? "M" : "S";
      muteBtn.onclick = (e) => {
        e.stopPropagation();
        void invokeWithToast(
          () => invoke<TabContentDto>("toggle_tab_mute", { index }),
          "Could not change tab mute",
        ).then((d) => d && applyState(d));
      };
      wrap.appendChild(muteBtn);
    }
    wrap.appendChild(btn);
    wrap.appendChild(close);
    tabStrip.appendChild(wrap);
    tabButtons.push(btn);
  });
  const plus = document.createElement("button");
  plus.type = "button";
  plus.className = "tab new-tab title-icon";
  plus.innerHTML = "";
  plus.appendChild(createLucideIcon("plus", 16, "ub-icon svg-icon"));
  plus.setAttribute("aria-label", "New tab");
  plus.tabIndex = -1;
  plus.onclick = () => invoke<TabContentDto>("new_tab").then(applyState);
  tabStrip.appendChild(plus);
  omnibox.value = data.omnibox;
  applyTabSleepMarks();
}

window.setInterval(() => applyTabSleepMarks(), 30_000);

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
  workspaceRoot: string | null | undefined,
  gitIsRepo: boolean,
  gitDirtyPaths: string[],
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
    workspaceRoot: workspaceRoot ?? null,
    gitIsRepo,
    gitDirtyPaths: new Set(gitDirtyPaths),
    recentFiles,
    pinnedPlaces,
    sidePanel: filesSidePanel,
    onApplyState: (data) => applyState(data as TabContentDto),
    persistFilesView,
    onRefresh: refreshState,
  });
}

function renderUnknownFileBanner(banner: UnknownFileBannerDto | null) {
  const el = document.getElementById("unknown-file-banner");
  const splitPrimary = document.getElementById("split-primary");
  const workArea = document.getElementById("work-area");
  const onboarding = document.getElementById("onboarding-banner");
  if (!el) {
    return;
  }
  el.innerHTML = "";
  if (!banner) {
    el.classList.add("hidden");
    el.classList.remove("unknown-support-panel");
    splitPrimary?.classList.remove("unknown-support-active");
    if (workArea && el.parentElement !== workArea) {
      workArea.insertBefore(el, onboarding);
    }
    return;
  }
  splitPrimary?.classList.add("unknown-support-active");
  el.classList.add("unknown-support-panel");
  if (splitPrimary && el.parentElement !== splitPrimary) {
    splitPrimary.appendChild(el);
  }
  el.classList.remove("hidden");
  const heading = document.createElement("h2");
  heading.textContent = "Open support";
  el.appendChild(heading);
  const lead = document.createElement("p");
  lead.textContent = t("unknownFile.lead", { ext: banner.extension });
  el.appendChild(lead);
  const actions = document.createElement("div");
  actions.className = "unknown-file-banner-actions";
  const textBtn = document.createElement("button");
  textBtn.type = "button";
  textBtn.textContent = t("unknownFile.openAsText");
  textBtn.onclick = () => {
    void invoke<TabContentDto>("open_active_file_as_text").then((d) => d && applyState(d));
  };
  const hexBtn = document.createElement("button");
  hexBtn.type = "button";
  hexBtn.textContent = t("unknownFile.openAsHex");
  hexBtn.onclick = () => {
    void invoke<TabContentDto>("open_active_file_as_hex").then((d) => d && applyState(d));
  };
  actions.appendChild(textBtn);
  actions.appendChild(hexBtn);
  banner.suggestions.slice(0, 4).forEach((s) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "mod-suggestion";
    btn.textContent = s.title;
    btn.title = s.reason;
    btn.onclick = () => {
      if (s.open_vsx_id) {
        void invoke<TabContentDto>("open_mod_portal", { ext: banner.extension.replace(/^\./, "") }).then(
          (d) => d && applyState(d),
        );
        return;
      }
      showToast(t("mods.installStub", { modId: s.mod_id }), "info");
    };
    actions.appendChild(btn);
  });
  const modsBtn = document.createElement("button");
  modsBtn.type = "button";
  modsBtn.textContent = t("unknownFile.browseMods");
  modsBtn.onclick = () => {
    void invoke<TabContentDto>("open_mod_portal", {
      ext: banner.extension.replace(/^\./, ""),
    }).then((d) => d && applyState(d));
  };
  actions.appendChild(modsBtn);
  el.appendChild(actions);
}

function setModPortalTab(index: number) {
  modPortalPanel.querySelectorAll<HTMLButtonElement>(".mod-portal-tab").forEach((tab, i) => {
    tab.setAttribute("aria-current", i === index ? "page" : "false");
  });
}

async function runModPortalOpenVsxSearch() {
  const searchBtn = modPortalPanel.querySelector<HTMLButtonElement>(
    ".mod-portal-search button:last-of-type",
  );
  searchBtn?.click();
}

function runModsCommand(commandId: string): boolean {
  if (!commandId.startsWith("unibrowser.mods.")) {
    return false;
  }
  if (modPortalPanel.classList.contains("hidden")) {
    showToast(t("menu.mods"), "info");
    return true;
  }
  switch (commandId) {
    case "unibrowser.mods.search":
      modPortalPanel.querySelector<HTMLInputElement>(".mod-portal-query")?.focus();
      void runModPortalOpenVsxSearch();
      return true;
    case "unibrowser.mods.install": {
      const first = modPortalPanel.querySelector<HTMLButtonElement>(".mod-card-action");
      if (first) {
        first.focus();
        first.click();
      } else {
        showToast(t("mods.noResults"), "info");
      }
      return true;
    }
    case "unibrowser.mods.updates":
      setModPortalTab(2);
      showToast("Updates checks built-in and VSIX extensions.", "info");
      return true;
    case "unibrowser.mods.permissions":
      showToast("Review licence and permission reminders on each card before installing.", "info");
      return true;
    case "unibrowser.mods.openVsx": {
      const next = !isOpenVsxNetworkEnabled();
      setOpenVsxNetworkEnabled(next);
      showToast(
        next ? t("mods.catalogOpenVsx") : t("mods.networkOptInRequired"),
        "info",
      );
      void refreshCommandManager();
      return true;
    }
    default:
      return false;
  }
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
  const tabs = document.createElement("nav");
  tabs.className = "mod-portal-tabs";
  tabs.setAttribute("aria-label", "Mod Portal sections");
  ["Installed", "Discover", "Updates"].forEach((label, index) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "mod-portal-tab";
    button.textContent = label;
    button.setAttribute("aria-current", index === 1 ? "page" : "false");
    button.onclick = () => setModPortalTab(index);
    tabs.appendChild(button);
  });
  modPortalPanel.appendChild(tabs);
  const p = document.createElement("p");
  p.textContent = portal.message;
  modPortalPanel.appendChild(p);
  const searchRow = document.createElement("div");
  searchRow.className = "mod-portal-search";
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
  searchRow.append(searchInput, searchBtn);
  modPortalPanel.appendChild(searchRow);
  renderModNetworkStatus(modPortalPanel);
  portal.suggestions.forEach((s) => {
    appendModSuggestionCard(modPortalPanel, s, (suggestion) => {
      if (suggestion.open_vsx_id) {
        searchInput.value = suggestion.open_vsx_id;
        enableOpenVsxForSearch();
        searchInput.focus();
        showToast(t("mods.installStub", { modId: suggestion.mod_id }), "info");
        return;
      }
      if (suggestion.ubmod_id) {
        showToast(t("mods.ubmodStub", { modId: suggestion.ubmod_id }), "info");
        return;
      }
      showToast(t("mods.installStub", { modId: suggestion.mod_id }), "info");
    });
  });
  const catalogNote = document.createElement("p");
  catalogNote.className = "viewer-note";
  catalogNote.textContent = t("mods.catalogNote");
  modPortalPanel.appendChild(catalogNote);
  void invoke<{ offline_mode: boolean }>("get_preferences").then((netPrefs) => {
    if (!netPrefs.offline_mode) {
      return;
    }
    setOpenVsxNetworkEnabled(false);
    searchBtn.disabled = true;
    const offlineNote = document.createElement("p");
    offlineNote.className = "viewer-note";
    offlineNote.textContent = t("mods.offlineModeActive");
    modPortalPanel.insertBefore(offlineNote, catalogNote);
  });
  modPortalPanel.appendChild(catalogResults);
  modPortalPanel.appendChild(searchStatus);
  modPortalPanel.appendChild(results);

  const installedHeading = document.createElement("h3");
  installedHeading.textContent = t("mods.installedHeading");
  modPortalPanel.appendChild(installedHeading);

  const nativeHeading = document.createElement("h4");
  nativeHeading.textContent = t("mods.nativeHeading");
  modPortalPanel.appendChild(nativeHeading);
  const userModNote = document.createElement("p");
  userModNote.className = "viewer-note";
  userModNote.textContent = t("mods.perModToggleNote");
  modPortalPanel.appendChild(userModNote);

  const nativeList = document.createElement("div");
  nativeList.className = "mod-portal-installed mod-portal-native";
  modPortalPanel.appendChild(nativeList);

  const vsixHeading = document.createElement("h4");
  vsixHeading.textContent = t("mods.vsixHeading");
  modPortalPanel.appendChild(vsixHeading);
  const installedList = document.createElement("div");
  installedList.className = "mod-portal-installed mod-portal-vsix";
  modPortalPanel.appendChild(installedList);

  const developer = document.createElement("details");
  developer.className = "mod-portal-developer";
  const developerSummary = document.createElement("summary");
  developerSummary.textContent = "Developer";
  const side = document.createElement("p");
  side.className = "sideload";
  side.textContent = portal.sideload_hint;
  const sideloadBtn = document.createElement("button");
  sideloadBtn.type = "button";
  sideloadBtn.className = "mod-sideload-button";
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
      const { reloadVsixEditorGrammars } = await import("./textEditor");
      await reloadVsixEditorGrammars().catch(() => {});
    } catch (err) {
      showToast(String(err), "error");
    }
  };
  developer.append(developerSummary, side, sideloadBtn);
  modPortalPanel.appendChild(developer);

  void refreshInstalledExtensions(nativeList, installedList);

  async function installFromOpenVsx(hit: OpenVsxHitDto) {
    showToast(t("mods.downloading", { extensionId: hit.extension_id }), "info");
    try {
      const rec = await invoke<InstalledVsixDto>("download_install_vsix", {
        url: hit.url,
        networkOptIn: isOpenVsxNetworkEnabled(),
        license: hit.license,
      });
      showToast(t("mods.installedPaletteNote", { extensionId: rec.extension_id }), "info");
      await refreshInstalledExtensions(nativeList, installedList);
      const { reloadVsixEditorGrammars } = await import("./textEditor");
      await reloadVsixEditorGrammars().catch(() => {});
    } catch (err) {
      showToast(String(err), "error");
    }
  }

  async function installedSets(): Promise<{ native: Set<string>; vsix: Set<string> }> {
    const native = new Set<string>();
    const vsix = new Set<string>();
    try {
      const mods = await invoke<InstalledNativeModDto[]>("list_installed_native_mods");
      mods.forEach((m) => native.add(m.name));
    } catch {
      /* ignore */
    }
    try {
      const list = await invoke<InstalledVsixDto[]>("list_installed_vsix");
      list.forEach((v) => vsix.add(v.extension_id));
    } catch {
      /* ignore */
    }
    return { native, vsix };
  }

  async function renderCatalog() {
    const entries = await invoke<ModCatalogEntry[]>("list_mod_catalog", {
      query: searchInput.value,
    });
    const { native, vsix } = await installedSets();
    catalogResults.innerHTML = "";
    entries.forEach((entry) => {
      appendModCatalogCard(catalogResults, entry, native, vsix, (picked) => {
        if (picked.open_vsx_id) {
          searchInput.value = picked.open_vsx_id;
          enableOpenVsxForSearch();
          void runModPortalOpenVsxSearch();
          return;
        }
        if (picked.ubmod_id) {
          showToast(t("mods.ubmodStub", { modId: picked.ubmod_id }), "info");
          return;
        }
        showToast(t("mods.installStub", { modId: picked.id }), "info");
      });
    });
  }

  searchInput.addEventListener("input", () => {
    void renderCatalog();
  });
  void renderCatalog();

  searchBtn.onclick = async () => {
    results.innerHTML = "";
    const netPrefs = await invoke<{ offline_mode: boolean }>("get_preferences");
    if (netPrefs.offline_mode) {
      searchStatus.textContent = t("mods.offlineModeActive");
      return;
    }
    if (!isOpenVsxNetworkEnabled()) {
      searchStatus.textContent = t("mods.networkOptInRequired");
      return;
    }
    searchStatus.textContent = t("mods.searching");
    try {
      const hits = await invoke<OpenVsxHitDto[]>("search_open_vsx_catalog", {
        query: searchInput.value,
        networkOptIn: isOpenVsxNetworkEnabled(),
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

function showVsixTrustBar(
  row: HTMLElement,
  extensionId: string,
  nativeContainer: HTMLElement,
  vsixContainer: HTMLElement,
) {
  row.querySelector(".vsix-trust-bar")?.remove();
  const bar = document.createElement("div");
  bar.className = "vsix-trust-bar";
  const msg = document.createElement("p");
  msg.textContent = t("mods.vsixTrustBar", { extensionId });
  bar.appendChild(msg);
  const actions = document.createElement("div");
  actions.className = "vsix-trust-bar-actions";
  const trustBtn = document.createElement("button");
  trustBtn.type = "button";
  trustBtn.className = "vsix-trust-confirm";
  trustBtn.textContent = t("mods.vsixTrustConfirm");
  const cancelBtn = document.createElement("button");
  cancelBtn.type = "button";
  cancelBtn.textContent = t("mods.vsixTrustCancel");
  cancelBtn.onclick = () => bar.remove();
  trustBtn.onclick = async () => {
    try {
      await invoke("trust_vsix_extension", { extensionId });
      bar.remove();
      const result = await invoke<{ restart_recommended: boolean; needs_trust?: boolean }>(
        "set_vsix_extension_enabled",
        { extensionId, enabled: true },
      );
      showToast(
        t(result.restart_recommended ? "mods.toggleRestartIfStale" : "mods.toggleAppliedVsix"),
        "info",
      );
      await refreshInstalledExtensions(nativeContainer, vsixContainer);
      await refreshState();
    } catch (err) {
      showToast(String(err), "error");
    }
  };
  actions.appendChild(trustBtn);
  actions.appendChild(cancelBtn);
  bar.appendChild(actions);
  row.insertBefore(bar, row.firstChild);
}

async function refreshInstalledExtensions(
  nativeContainer: HTMLElement,
  vsixContainer: HTMLElement,
) {
  nativeContainer.innerHTML = "";
  vsixContainer.innerHTML = "";
  const prefs = await invoke<{
    disable_user_mods: boolean;
    safe_mode_env: boolean;
    advanced_node_extension_host: boolean;
    vsix_trusted_extensions: string[];
  }>("get_preferences");
  const userModsBlocked = prefs.disable_user_mods || prefs.safe_mode_env;
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
        toggle.disabled = userModsBlocked;
        toggle.title = userModsBlocked ? t("mods.perModBlockedBySafeMode") : "";
        toggle.onclick = async () => {
          const result = await invoke<{ restart_recommended: boolean }>("set_native_mod_enabled", {
            name: mod.name,
            enabled: !mod.enabled,
          });
          showToast(
            t(result.restart_recommended ? "mods.toggleRestartNative" : "mods.toggleApplied"),
            "info",
          );
          await refreshInstalledExtensions(nativeContainer, vsixContainer);
          await refreshState();
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
      const compatKey =
        ext.compatibility_level === "works"
          ? "mods.vsixCompatWorks"
          : ext.compatibility_level === "partial"
            ? "mods.vsixCompatPartial"
            : "mods.vsixCompatUnsupported";
      const badge = t(compatKey);
      label.textContent = `${ext.extension_id} v${ext.version} — ${badge} — ${ext.command_count} cmd / ${ext.keybinding_count} keys`;
      if (ext.compatibility_missing_apis.length) {
        label.title = t("mods.vsixCompatMissing", {
          apis: ext.compatibility_missing_apis.join(", "),
        });
      }
      const assets = [...ext.theme_notes, ...ext.grammar_notes];
      if (assets.length) {
        const note = document.createElement("p");
        note.className = "viewer-note";
        note.textContent = assets.join("; ");
        row.appendChild(note);
      }
      const toggle = document.createElement("button");
      toggle.type = "button";
      toggle.textContent = ext.enabled ? t("mods.disable") : t("mods.enable");
      toggle.onclick = async () => {
        const enabling = !ext.enabled;
        if (
          enabling &&
          prefs.advanced_node_extension_host &&
          !prefs.vsix_trusted_extensions.includes(ext.extension_id)
        ) {
          showVsixTrustBar(row, ext.extension_id, nativeContainer, vsixContainer);
          return;
        }
        try {
          const result = await invoke<{ restart_recommended: boolean; needs_trust?: boolean }>(
            "set_vsix_extension_enabled",
            { extensionId: ext.extension_id, enabled: enabling },
          );
          if (result.needs_trust) {
            showVsixTrustBar(row, ext.extension_id, nativeContainer, vsixContainer);
            return;
          }
          showToast(
            t(result.restart_recommended ? "mods.toggleRestartIfStale" : "mods.toggleAppliedVsix"),
            "info",
          );
          await refreshInstalledExtensions(nativeContainer, vsixContainer);
          await refreshState();
        } catch (err) {
          showToast(String(err), "error");
        }
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
  _enabled: boolean,
  _pins: PinnedLocationDto[],
  _quickLinks: BookmarkQuickLinkDto[] = [],
) {
  bookmarkBar.innerHTML = "";
  bookmarkBar.classList.add("hidden");
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

async function resetPageZoomFromStatusBar() {
  await textEditor.flush();
  const data = await invokeWithToast(
    () => invoke<TabContentDto>("reset_active_page_zoom"),
    t("status.zoomResetFailed"),
  );
  if (data) {
    webZoomPercent = 100;
    await applyState(data);
  }
}

async function resetContextFontFromStatusBar() {
  await textEditor.flush();
  const data = await invokeWithToast(
    () => invoke<TabContentDto>("reset_active_context_font"),
    t("status.fontResetFailed"),
  );
  if (data) {
    await applyState(data);
  }
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
  if (msg === "document-find" || msg === "web-find") {
    openDocumentFind();
    return false;
  }
  if (msg === "export-print-pdf") {
    await runExportPrintPdf(documentViewerHost);
    return false;
  }
  if (msg === "export-print-selection") {
    await runPrintSelection(documentViewerHost);
    return false;
  }
  if (msg === "devtools-open" || msg === "devtools-close") {
    return false;
  }
  if (msg === "terminal-toggle") {
    toggleTerminalPanel();
    return false;
  }
  if (msg.startsWith("ocr-copy:")) {
    const text = msg.slice("ocr-copy:".length);
    try {
      await navigator.clipboard.writeText(text);
      showToast(t("image.ocrCopied"), "info");
    } catch {
      showToast(text.slice(0, 120), "info");
    }
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
  if (msg === "capture-visible") {
    await runCaptureVisibleTab();
    return false;
  }
  if (msg === "image-compare-pick") {
    const data = await invokeWithToast(
      () => invoke<TabContentDto>("pick_and_compare_images"),
      t("imageCompare.failed"),
    );
    if (data) {
      await applyState(data);
    }
    return false;
  }
  if (msg === "text-compare-pick") {
    const data = await invokeWithToast(
      () => invoke<TabContentDto>("pick_and_compare"),
      t("compare.needFile"),
    );
    if (data) {
      showToast(t("compare.started"), "info");
      await applyState(data);
    }
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
  if (msg === "tab-search") {
    toggleTabSearch();
    return;
  }
  if (msg === "autosave-restore-prompt") {
    void openAutosaveRestorePicker();
    return;
  }
  if (msg === "problems-panel-toggle") {
    toggleProblemsPanel();
    return;
  }
  if (msg === "workspace-quick-open") {
    toggleWorkspaceQuickOpen();
    return;
  }
  if (msg === "start-open-file") {
    const path = await invoke<string | null>("pick_open_file");
    if (path) {
      const data = await invoke<TabContentDto>("open_path", { path });
      if (data) {
        await applyState(data);
      }
    }
    return;
  }
  if (msg === "start-open-folder") {
    const folder = await invoke<string | null>("pick_browse_folder");
    if (folder) {
      const data = await invoke<TabContentDto>("open_path", { path: folder });
      if (data) {
        await applyState(data);
      }
    }
    return;
  }
  if (msg === "start-new-project") {
    document.getElementById("btn-new-project")?.click();
    return;
  }
  if (msg === "start-recent") {
    document.getElementById("new-tab-page-host")?.scrollIntoView({ behavior: "smooth", block: "start" });
    return;
  }
  if (msg === "find-files-deep") {
    toggleGlobalFind(lastBrowseFolderPath);
    return;
  }
  if (msg === "media-pip") {
    void tryMediaPictureInPicture().catch((err) => showErrorToast(err));
    return;
  }
  if (msg === "context-help-opened") {
    return;
  }
  if (msg === "pdf-highlight-armed") {
    armPdfHighlightMode();
    showToast(t("pdf.highlightArmed"), "info");
    return;
  }
  const toast = await handleShellCommandMessage(msg);
  if (toast && msg !== "Cancelled" && commandId !== "unibrowser.script.demo.hello") {
    showToast(String(msg), "info");
  }
}

async function runUiCommand(commandId: string) {
  if (await runFilesCommand(commandId)) {
    return;
  }
  if (runViewerContextCommand(commandId)) {
    return;
  }
  if (runModsCommand(commandId)) {
    return;
  }
  if (commandId === "unibrowser.settings.search") {
    document.querySelector<HTMLInputElement>(".settings-search")?.focus();
    return;
  }
  if (
    commandId === "unibrowser.settings.keybindings" ||
    commandId === "unibrowser.settings.diagnostics"
  ) {
    const path = commandId.endsWith("keybindings")
      ? "unibrowser://keys"
      : "unibrowser://diagnostics";
    window.dispatchEvent(new CustomEvent("unibrowser-navigate", { detail: path }));
    return;
  }
  if (commandId === "ai-assistant.toggle") {
    const prefs = await invoke<{ panel_visible: boolean }>("get_ai_assistant_prefs");
    await setAiPanelVisible(!prefs.panel_visible);
    return;
  }
  if (commandId === PROOFING_TOGGLE) {
    await toggleProofingSquiggles();
    await textEditor.refreshProofingMarkers();
    return;
  }
  if (commandId === PROOFING_NEXT) {
    textEditor.focusNextProofingInEditor();
    return;
  }
  if (commandId === PROOFING_PREVIOUS) {
    textEditor.focusPreviousProofingInEditor();
    return;
  }
  await textEditor.flush();
  const msg = await invoke<string>("execute_command", { commandId });
  await processCommandResult(msg, commandId);
  await refreshCommandManager();
  await refreshState();
}

async function refreshCommandManager() {
  try {
    await commandManager.refresh(
      commandManagerGroups,
      floatingToolbarHost,
      commandManagerPin,
      commandManagerContext,
      commandManagerOverflow,
      runUiCommand,
    );
  } catch {
    commandManagerGroups.innerHTML =
      "<span class='muted'>Commands unavailable</span>";
  }
}

window.addEventListener("unibrowser-demo-cm-refresh", () => {
  void refreshCommandManager();
});
window.addEventListener("unibrowser-layout-edit-changed", () => {
  void refreshCommandManager();
});

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
    return;
  }
  splitSecondary.classList.remove("hidden");
  contentHost.classList.add("split-active");
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

const extensionTipEl = document.getElementById("extension-tip")!;
const extensionTipMessage = document.getElementById("extension-tip-message")!;
document.getElementById("extension-tip-mods")!.onclick = () => {
  void invoke<TabContentDto>("open_mod_portal", { ext: "js" }).then(applyState);
};
document.getElementById("extension-tip-dismiss")!.onclick = () => {
  void invoke("dismiss_scripting_extension_tip").then(() => {
    extensionTipEl.classList.add("hidden");
  });
};

function renderExtensionTip(tip: TabContentDto["extension_tip"]) {
  if (!tip) {
    extensionTipEl.classList.add("hidden");
    return;
  }
  extensionTipMessage.textContent = tip.message;
  extensionTipEl.classList.remove("hidden");
}

async function applyState(data: TabContentDto) {
  if (data.chrome_notice) {
    showToast(data.chrome_notice, "info");
  }
  renderExtensionTip(data.extension_tip ?? null);
  const activeTab = data.tabs[data.active_index];
  const winTitle = activeTab?.label ? `Unibrowser — ${activeTab.label}` : "Unibrowser";
  document.title = winTitle;
  void getCurrentWebviewWindow().setTitle(winTitle);
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
  const activeTabForNav = data.tabs[data.active_index];
  updateTitleBarNavVisibility(activeTabForNav);
  syncTitleBar(data);
  const showBookmarkBar =
    (data.bookmark_bar_enabled ?? false) &&
    !showHome &&
    !showText &&
    !showViewer &&
    !active?.is_http &&
    Boolean(active?.is_directory);
  renderBookmarkBar(
    showBookmarkBar,
    data.pinned_locations ?? [],
    data.bookmark_quick_links ?? [],
  );
  const showDiff = Boolean(data.diff_view);
  const showImageCompare = Boolean(data.image_compare_view);
  if (showImageCompare && data.image_compare_view) {
    diffEditorView.hide(diffEditorHost);
    await applyImageCompare(data.image_compare_view, imageCompareHost);
    textEditor.hide(textEditorHost);
    documentViewer.hide(documentViewerHost);
  } else {
    applyImageCompare(null, imageCompareHost);
    if (showDiff && data.diff_view) {
      await diffEditorView.apply(data.diff_view, diffEditorHost);
      textEditor.hide(textEditorHost);
      documentViewer.hide(documentViewerHost);
    } else {
      diffEditorView.hide(diffEditorHost);
    }
  }
  if (showHome) {
    contentHostEl.classList.add("content-host--start");
    splitPrimaryEl.classList.add("hidden");
    filesBrowserWrap.classList.add("hidden");
    renderNewTabPage(newTabPageHost, {
      entries: data.directory_entries,
      recent: data.recent_files ?? [],
      downloads: data.download_history ?? [],
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
    contentHostEl.classList.remove("content-host--start");
    splitPrimaryEl.classList.remove("hidden");
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
        data.workspace_root,
        data.git_is_repo ?? false,
        data.git_dirty_paths ?? [],
      );
    }
  }
  renderUnknownFileBanner(data.unknown_file_banner ?? null);
  renderModPortal(data.mod_portal);
  const hasPortal = Boolean(data.mod_portal);
  renderDoc(data.document_status, showText, hasPortal, showViewer);
  if (data.directory_entries && data.directory_entries.length > 0) {
    void markChecklistStep("browse_folder");
  }
  if (data.text_document || data.document_viewer) {
    void markChecklistStep("open_file");
  }
  if (!showHome) {
    renderRecentFiles(data.recent_files ?? []);
  } else {
    recentPanel.classList.add("hidden");
  }
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
  } else if (active?.is_http) {
    setDocumentFindSurface("web");
  } else if (showViewer && (viewerKind === "docx" || viewerKind === "epub")) {
    setDocumentFindSurface("viewer");
  } else {
    setDocumentFindSurface("none");
    closeDocumentFind();
  }
  const zoomReset = () => void resetPageZoomFromStatusBar();
  if (active?.is_http) {
    syncHttpLoadingForTab(true, data.omnibox);
  } else {
    syncHttpLoadingForTab(false);
  }
  if (active?.is_http) {
    setZoomHandler(
      "web",
      (delta) => {
        const cmd =
          delta > 0
            ? "unibrowser.page.zoom.increase"
            : "unibrowser.page.zoom.decrease";
        void runPageZoom(cmd);
      },
      zoomReset,
    );
  } else if (showViewer && data.document_viewer) {
    const kind = data.document_viewer.viewer;
    if (kind === "pdf" || kind === "image") {
      setZoomHandler("readout", null, null);
    } else {
      setZoomHandler("hidden", null, null);
    }
  } else {
    setZoomHandler("hidden", null, null);
  }
  const fontResetable =
    (showText && data.text_document) ||
    (showViewer &&
      data.document_viewer &&
      (data.document_viewer.reader_font_pt ||
        data.document_viewer.viewer === "svg"));
  setFontResetHandler(fontResetable ? () => void resetContextFontFromStatusBar() : null);
  const zoomLabel = document.getElementById("status-zoom-label");
  if (zoomLabel) {
    zoomLabel.title =
      active?.is_http || (showViewer && data.document_viewer && (data.document_viewer.viewer === "pdf" || data.document_viewer.viewer === "image"))
        ? t("status.zoomResetTitle")
        : "";
  }
  const fontLabel = document.getElementById("status-font");
  if (fontLabel) {
    fontLabel.title = fontResetable ? t("status.fontResetTitle") : "";
  }
  refreshStatusBar(data, active, showText, showViewer);
  const recoveryHost = document.getElementById("recovery-bar-host");
  if (recoveryHost) {
    renderRecoveryBar(recoveryHost, data.recovery_bar ?? [], () => {
      void refreshState();
    });
  }
  await syncContextDockPolicy(data);
  await refreshAiAssistantChrome();
  await refreshCommandManager();
  syncBottomPanelEmptyState();
  await syncContentBounds();
  window.requestAnimationFrame(() => syncContentBounds());
}

(window as unknown as { __selfTestApplyState?: (d: TabContentDto) => Promise<void> }).__selfTestApplyState =
  applyState;

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

async function openAutosaveRestorePicker() {
  const slots = await invoke<{ path: string; saved_at_ms: number }[]>(
    "list_autosave_slots_for_active_tab",
  ).catch(() => []);
  if (slots.length === 0) {
    showToast(t("autosave.noSlots"), "info");
    return;
  }
  const items = slots.map((s, i) => {
    const when = new Date(s.saved_at_ms).toLocaleString();
    const name = s.path.split("/").pop() ?? s.path;
    return { id: String(i), label: `${when} — ${name}`, detail: s.path };
  });
  const picked = await showPickList(t("autosave.pickSlot"), items);
  if (!picked) {
    return;
  }
  const idx = Number.parseInt(picked.id, 10);
  const slot = slots[Number.isNaN(idx) ? 0 : idx] ?? slots[0];
  const data = await invoke<TabContentDto>("restore_autosave_from_slot", {
    slotPath: slot.path,
  }).catch((e) => {
    showErrorToast(e);
    return undefined;
  });
  if (data) {
    await applyState(data);
    showToast(t("autosave.restoreDone"), "info");
  }
}

async function activateTabHit(hit: TabSearchHit) {
  await textEditor.flush();
  await invoke("activate_tab_in_window", {
    windowLabel: hit.window_label,
    tabIndex: hit.tab_index,
  });
  await refreshState();
}

async function closeTabAt(index: number, _dirty: boolean) {
  await textEditor.flush();
  try {
    await invoke("auto_save_active_document");
  } catch {
    /* best-effort slot before close */
  }
  const data = await invokeWithToast(
    () => invoke<TabContentDto>("close_tab", { index, force: true }),
    "Could not close tab",
  );
  if (data) {
    if (data.close_undo_available) {
      showActionToast(t("tabs.closedUndo"), t("tabs.undo"), () => {
        void invoke<TabContentDto>("undo_close_tab").then((d) => d && applyState(d));
      });
    }
    await applyState(data);
  }
}

async function confirmInsecureHttpNavigation(input: string): Promise<boolean> {
  const prefs = await invoke<{ warn_insecure_http: boolean }>("get_preferences");
  if (!prefs.warn_insecure_http || !isInsecureHttpNavigation(input)) {
    return true;
  }
  return showInlineConfirm(t("omnibox.insecureHttpConfirm"));
}

async function navigateOmnibox() {
  const input = omnibox.value;
  if (!(await confirmInsecureHttpNavigation(input))) {
    return;
  }
  void invokeWithToast(
    () => invoke<TabContentDto>("navigate_omnibox", { input }),
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

tabSearchQuery.addEventListener("input", () => void refreshTabSearchResults());

listen("browser-state-updated", () => {
  void refreshState();
});

listen<{ message: string; severity?: string }>("unibrowser-extension-toast", (event) => {
  const severity = event.payload.severity ?? "info";
  showToast(event.payload.message, severity === "error" ? "error" : "info");
});

async function openNewProjectFromNav() {
  const choice = await openNewProjectDialog();
  if (!choice) {
    return;
  }
  const home = await invoke<string>("get_home_dir").catch(() => "/tmp");
  const data = await invoke<TabContentDto>("create_unibrow_project", {
    parentDir: home,
    name: choice.name,
    template: choice.template,
  });
  void applyState(data);
}

diffEditorHost.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;
  if (target.dataset.action === "close-diff") {
    void invoke<TabContentDto>("close_file_compare").then((d) => d && applyState(d));
  }
  if (target.dataset.action === "toggle-diff-layout") {
    diffEditorView.toggleLayout(diffEditorHost);
  }
});

imageCompareHost.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;
  if (target.dataset.action === "close-image-compare") {
    void invoke<TabContentDto>("close_image_compare").then((d) => d && applyState(d));
  }
});

function openTitleOverflow(anchor: HTMLElement) {
  const menu = document.createElement("div");
  menu.className = "nav-overflow-menu";
  const items: { label: string; run: () => void }[] = [
    {
      label: t("menu.findInDocument"),
      run: () => void runKeybindingCommand("unibrowser.find.inDocument"),
    },
    {
      label: t("bookmark.downloadManager"),
      run: () => {
        omnibox.value = "unibrowser://downloads";
        void navigateOmnibox();
      },
    },
    {
      label: t("chrome.tabs"),
      run: () => toggleTabSearch(),
    },
    {
      label: t("chrome.split"),
      run: () => {
        void invokeWithToast(
          () => invoke<TabContentDto>("toggle_split_view"),
          t("chrome.splitFailed"),
        ).then((data) => data && applyState(data));
      },
    },
    {
      label: t("chrome.mods"),
      run: () => {
        void invoke<TabContentDto>("open_mod_portal", { ext: null }).then(applyState);
      },
    },
    {
      label: t("chrome.newProject"),
      run: () => void openNewProjectFromNav(),
    },
    {
      label: t("chrome.theme"),
      run: () => void cycleTheme(),
    },
    {
      label: t("chrome.compare"),
      run: () => {
        void invokeWithToast(
          () => invoke<TabContentDto>("pick_and_compare"),
          t("compare.needFile"),
        ).then((data) => data && applyState(data));
      },
    },
    {
      label: t("chrome.recent"),
      run: () => {
        recentPanelOpen = !recentPanelOpen;
        void refreshState();
      },
    },
    {
      label: t("settings.open"),
      run: () => {
        omnibox.value = "unibrowser://settings";
        void navigateOmnibox();
      },
    },
  ];
  for (const item of items) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "nav-overflow-item";
    btn.textContent = item.label;
    btn.onclick = () => {
      menu.remove();
      item.run();
    };
    menu.appendChild(btn);
  }
  const rect = anchor.getBoundingClientRect();
  menu.style.left = `${rect.right - 180}px`;
  menu.style.top = `${rect.bottom + 4}px`;
  document.body.appendChild(menu);
  const close = (ev: MouseEvent) => {
    if (!menu.contains(ev.target as Node)) {
      menu.remove();
      document.removeEventListener("click", close);
    }
  };
  window.setTimeout(() => document.addEventListener("click", close), 0);
}

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
  paletteController?.close();
}

function togglePalette() {
  paletteOpen = !paletteOpen;
  palette.classList.toggle("hidden", !paletteOpen);
  if (paletteOpen) {
    tabSearchOpen = false;
    tabSearchPanel.classList.add("hidden");
    paletteQuery.placeholder = t("palette.placeholder", { chord: paletteChordLabel });
    paletteQuery.value = "";
    paletteQuery.focus();
    paletteController?.open();
    void markChecklistStep("palette");
  } else {
    paletteController?.close();
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
    const winTag = hit.window_label === "main" ? "" : `[${hit.window_label}] `;
    row.textContent = `${winTag}${hit.label}`;
    row.title = `${hit.omnibox} (${hit.window_label})`;
    row.onclick = () => {
      toggleTabSearch();
      void activateTabHit(hit);
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
  if (commandId === "unibrowser.workspace.editLayout") {
    toggleLayoutEditMode();
    return;
  }
  if (commandId === "ai-assistant.toggle") {
    const prefs = await invoke<{ panel_visible: boolean }>("get_ai_assistant_prefs");
    await setAiPanelVisible(!prefs.panel_visible);
    return;
  }
  if (commandId === PALETTE_TOGGLE) {
    togglePalette();
    return;
  }
  if (
    commandId === PROOFING_TOGGLE ||
    commandId === PROOFING_NEXT ||
    commandId === PROOFING_PREVIOUS
  ) {
    await runUiCommand(commandId);
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
    if (isContentFullscreen()) {
      e.preventDefault();
      exitContentFullscreen();
      return;
    }
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
    const wsQuick = document.getElementById("workspace-quick-open");
    if (wsQuick && !wsQuick.classList.contains("hidden")) {
      e.preventDefault();
      hideWorkspaceQuickOpen();
      return;
    }
  }
  if (!filesBrowserWrap.classList.contains("hidden") && handleFilesKeyDown(e)) {
    return;
  }
  if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "p") {
    e.preventDefault();
    toggleWorkspaceQuickOpen();
    return;
  }
  if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "f") {
    e.preventDefault();
    toggleGlobalFind(lastBrowseFolderPath);
    return;
  }
  if (e.key === "F11") {
    e.preventDefault();
    toggleContentFullscreen();
    return;
  }
  if (e.ctrlKey && e.key === "F1") {
    e.preventDefault();
    document.body.classList.toggle("cm-collapsed");
    return;
  }
  if (e.ctrlKey && e.shiftKey && (e.key.toLowerCase() === "t" || e.key.toLowerCase() === "a")) {
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

paletteController = initCommandPalette({
  palette,
  query: paletteQuery,
  results: paletteResults,
  onClose: closePalette,
  onExecute: (entry) => {
    if (entry.open_path) {
      void invoke<TabContentDto>("open_path", { path: entry.open_path }).then((d) => {
        if (d) {
          void applyState(d);
        }
        closePalette();
      });
      return;
    }
    void invoke<string>("execute_command", { commandId: entry.command_id }).then(async (msg) => {
      closePalette();
      await processCommandResult(msg, entry.command_id);
      await refreshCommandManager();
      await refreshState();
    });
  },
});

window.addEventListener("resize", () => syncContentBounds());

window.addEventListener("unibrowser-navigate", (event) => {
  const detail = (event as CustomEvent<string>).detail;
  if (detail) {
    omnibox.value = detail;
    void (async () => {
      if (!(await confirmInsecureHttpNavigation(detail))) {
        return;
      }
      void invokeWithToast(
        () => invoke<TabContentDto>("navigate_omnibox", { input: detail }),
        "Could not navigate",
      ).then((data) => data && applyState(data));
    })();
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
    let data: TabContentDto | undefined;
    try {
      data = await invoke<TabContentDto>("handle_download_url", {
        url: payload.url,
        destination: payload.destination ?? null,
      });
    } catch (err) {
      const detail = err instanceof Error ? err.message : String(err);
      showErrorToast(err, t("download.failed"));
      await invoke("record_download_failure", {
        url: payload.url,
        error: detail,
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
initNotificationCenter(statusBarEl);
initContentFullscreen(() => {
  void syncContentBounds();
});
initGlobalFind({
  defaultRoot: lastBrowseFolderPath,
  onOpenPath: (path, line) => {
    void invoke<TabContentDto>("open_path", { path, line: line ?? null }).then((d) => d && applyState(d));
  },
});
initWorkspaceQuickOpen({
  onOpenPath: (path) => {
    void invoke<TabContentDto>("open_path", { path, line: null }).then((d) => d && applyState(d));
  },
});
setPathCopyHandler((path) => {
  void copyText(path, () => showToast(t("status.pathCopied"), "info"));
});
setFontResetHandler(null);
setCursorStatusListener((pos) => {
  editorLine = pos.line;
  editorCol = pos.column;
  patchStatusCursor(`Ln ${editorLine}, Col ${editorCol}`);
});
setReadingStatsListener((label) => patchStatusReading(label));
setViewerZoomListener((_scale, label) => {
  lastViewerZoomLabel = label;
  const zoomEl = document.getElementById("status-zoom-label");
  if (zoomEl && lastViewerZoomLabel) {
    zoomEl.textContent = lastViewerZoomLabel;
    zoomEl.classList.remove("hidden");
  }
  if (lastViewerZoomLabel.endsWith("%") && !lastViewerZoomLabel.includes("Image")) {
    void refreshCommandManager();
  }
});
setViewerPdfPageListener((page, total, annotationCount) => {
  lastPdfPageLabel = total > 0 ? `Page ${page} of ${total}` : "";
  lastPdfAnnotationCount = annotationCount;
  const readingEl = document.getElementById("status-reading");
  if (readingEl && lastPdfPageLabel) {
    readingEl.textContent = lastPdfPageLabel;
    readingEl.classList.remove("hidden");
  }
  void refreshCommandManager();
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

safeSubsystemInit("script-output", () => initScriptOutputPanel());
safeSubsystemInit("terminal", () => initTerminalPanel());
safeSubsystemInit("problems", () => initProblemsPanel());
safeSubsystemInit("dock-panel", () => initDockPanel());
safeSubsystemInit("ai-assistant", () => initAiAssistantPanel());
safeSubsystemInit("keys-overlay", () => initKeysOverlay());
window.addEventListener("unibrowser-prefs-changed", () => {
  void refreshState();
});
listen<string>("unibrowser-script-output", (event) => {
  appendScriptOutput(String(event.payload));
});
initHttpLoadingListeners(contentHost);
applyChromeI18n();
void loadUiLocale().then(() => applyChromeI18n());
void loadThemePreference();
void invoke<{ editor_theme_id: string }>("get_preferences").then((prefs) => {
  setEditorThemePreference(prefs.editor_theme_id);
  void initMonacoEditorThemes(prefs.editor_theme_id);
});
void loadAccessibilityPreferences();
void loadExperimentalSpellcheckPref();
void loadProofingPrefs();
initLocatorNavigationListeners();
window.addEventListener("focus", () => {
  void invoke("reload_keybindings").then(() => loadKeybindingHints());
});

void initToolbarOnlyShell(refreshCommandManager).then((toolbarOnly) => {
  if (!toolbarOnly) {
    hideShellFallback();
    void refreshState().then(async () => {
      await loadThemePreference();
      const launchPaths = await invoke<string[]>("take_launch_open_paths");
      const demo = await invoke<string | null>("get_demo_state");
      if (demo) {
        applyThemeFromDemoState(demo);
        if (demo.toLowerCase().includes("theme-light") || demo.toLowerCase().includes("theme-dark")) {
          const { applyMonacoEditorTheme } = await import("./monacoEditorTheme");
          await applyMonacoEditorTheme("auto");
        }
      }
      for (const p of launchPaths) {
        try {
          const opened = await invoke<TabContentDto>("open_path", { path: p, line: null });
          if (opened) {
            await applyState(opened);
          }
        } catch {
          /* argv open failures surface via toast on next interaction */
        }
      }
      if (launchPaths.length > 0) {
        await refreshState();
        await new Promise((resolve) => setTimeout(resolve, 350));
        await refreshState();
      }
      if (demo) {
        try {
          await applyDemoState(demo);
        } catch (err) {
          reportSubsystemError("demo-state", err);
        }
      }
      const docCapture = await invoke<string | null>("take_doc_capture_path");
      if (docCapture) {
        const demoAgain = await invoke<string | null>("get_demo_state");
        if (demoAgain?.toLowerCase().includes("cmd-proving-detached")) {
          return;
        }
        if (demoAgain) {
          try {
            await applyDemoState(demoAgain);
            await refreshState();
            const demoTokens = demoAgain.toLowerCase();
            const proofControl = demoTokens.includes("cmd11-plain-proof")
              ? "unibrowser.control.text.strikethrough"
              : demoTokens.includes("cmd11-markdown-proof")
                ? "unibrowser.control.text.subscript"
                : null;
            if (proofControl) {
              await new Promise((resolve) => window.setTimeout(resolve, 50));
              document
                .querySelector<HTMLElement>(`[data-control-id="${proofControl}"]`)
                ?.focus();
            }
            if (demoTokens.includes("ui19-focus-visible")) {
              await new Promise((resolve) => window.setTimeout(resolve, 50));
              document
                .querySelector<HTMLElement>(
                  '[data-control-id="unibrowser.control.find"]',
                )
                ?.focus();
            }
          } catch (err) {
            reportSubsystemError("demo-state", err);
          }
        }
        try {
          const painted = await waitForShellPaintReady(demoAgain, launchPaths, 90_000);
          if (!painted) {
            const timeoutDiag = formatDiagnosticsForLog(collectDocCaptureDiagnostics());
            const editorLengths = {
              document: textEditorHost.dataset.documentContentLength ?? null,
              model: textEditorHost.dataset.modelContentLength ?? null,
            };
            await invoke("abort_doc_capture", {
              reason: `Shell paint readiness timed out; editorLengths=${JSON.stringify(editorLengths)}; ${timeoutDiag}`,
            });
            return;
          }
          const auditErrors = runShellChromeAudit();
          await invoke("self_test_submit_shell_chrome_audit", { errors: auditErrors });
          if (auditErrors.length > 0) {
            await invoke("abort_doc_capture", {
              reason: `shell chrome audit: ${auditErrors.join("; ")}`,
            });
            return;
          }
          const diag = collectDocCaptureDiagnostics();
          const diagJson = formatDiagnosticsForLog(diag);
          console.error(`[doc-capture] diagnostics ${diagJson}`);
          void invoke("log_shell_ui_event", {
            level: "error",
            message: "doc-capture-diagnostics",
            detail: diagJson,
          });
          if (demoAgain?.toLowerCase().includes("theme-light")) {
            const themeErr = assertLightThemeForCapture();
            if (themeErr) {
              await invoke("abort_doc_capture", { reason: themeErr });
              return;
            }
          }
          const cmErr = validateCommandManagerForCapture();
          if (cmErr) {
            await invoke("abort_doc_capture", {
              reason: `${cmErr}; ${formatDiagnosticsForLog(diag)}`,
            });
            return;
          }
          const shellContrastErr = validateShellDensityContrastForCapture();
          if (shellContrastErr) {
            await invoke("abort_doc_capture", {
              reason: `${shellContrastErr}; ${formatDiagnosticsForLog(diag)}`,
            });
            return;
          }
          const expectedContext = expectedCaptureContextLabel(launchPaths);
          if (expectedContext) {
            const ctxErr = validateCommandContextLabel(expectedContext);
            if (ctxErr) {
              await invoke("abort_doc_capture", {
                reason: `${ctxErr}; ${formatDiagnosticsForLog(diag)}`,
              });
              return;
            }
          }
          if (
            demoAgain?.toLowerCase().includes("problems") ||
            demoAgain?.toLowerCase().includes("document-mockup")
          ) {
            const problemsErr = validateProblemsForCapture();
            if (problemsErr) {
              await invoke("abort_doc_capture", {
                reason: `${problemsErr}; ${formatDiagnosticsForLog(diag)}`,
              });
              return;
            }
          }
          const captureUsesEditor =
            launchPathsNeedEditor(launchPaths) &&
            !demoAgain?.toLowerCase().includes("settings");
          const editorErr = validateEditorForCapture();
          if (captureUsesEditor && editorErr) {
            await invoke("abort_doc_capture", {
              reason: `${editorErr}; ${formatDiagnosticsForLog(diag)}`,
            });
            return;
          }
          if (
            captureUsesEditor &&
            !editorReadyForCapture()
          ) {
            await invoke("abort_doc_capture", {
              reason: `Monaco editor is blank after paint wait; ${formatDiagnosticsForLog(diag)}`,
            });
            return;
          }
          let editorRegion: {
            x: number;
            y: number;
            width: number;
            height: number;
            viewportWidth: number;
            viewportHeight: number;
            theme: string;
          } | null = null;
          if (captureUsesEditor) {
            const monacoRoot = textEditorHost.querySelector(".monaco-editor") as HTMLElement | null;
            const editorRect = (monacoRoot ?? textEditorHost).getBoundingClientRect();
            if (editorRect.width >= 80 && editorRect.height >= 40) {
              editorRegion = {
                x: editorRect.x,
                y: editorRect.y,
                width: editorRect.width,
                height: editorRect.height,
                viewportWidth: window.innerWidth,
                viewportHeight: window.innerHeight,
                theme: document.documentElement.dataset.theme ?? "dark",
              };
            }
          }
          await invoke("finish_doc_capture_and_exit", { editorRegion });
        } catch (err) {
          const msg = err instanceof Error ? err.message : String(err);
          await invoke("abort_doc_capture", { reason: msg }).catch(() => undefined);
        }
        return;
      }
      void maybePromptProjectRecovery((data) => {
        void applyState(data as TabContentDto);
      });
    });
  }
  if (toolbarOnly) {
    return;
  }
  deferNonCriticalInit(() => {
  initCompactTitleBar({
    runCommand: (commandId) => runKeybindingCommand(commandId),
    openTitleOverflow,
    onBack: () => void invoke<TabContentDto>("go_back").then(applyState),
    onForward: () => void invoke<TabContentDto>("go_forward").then(applyState),
    onReload: () => void invoke<TabContentDto>("reload_tab").then(applyState),
    navigateOmnibox: () => void navigateOmnibox(),
  });
  const workAreaRow = document.querySelector(".ub-work-area-row") as HTMLElement;
  initActivityRail({
    workAreaRow,
    onDestination: (id) => {
      if (id === "files") {
        filesBrowserWrap.classList.toggle("hidden");
        if (!filesBrowserWrap.classList.contains("hidden")) {
          void refreshState();
        }
        return;
      }
      if (id === "search") {
        toggleGlobalFind(lastBrowseFolderPath);
        return;
      }
      if (id === "settings") {
        omnibox.value = "unibrowser://settings";
        void navigateOmnibox();
        return;
      }
      if (id === "project") {
        omnibox.value = "unibrowser://action/new-project";
        void navigateOmnibox();
        return;
      }
      if (id === "assets") {
        omnibox.value = "unibrowser://mods";
        void navigateOmnibox();
      }
    },
  });
  commandManager.bindCollapseControl(commandManagerCollapse);
  commandManager.bindPinControl(
    commandManagerPin,
    commandManagerGroups,
    floatingToolbarHost,
    commandManagerContext,
    commandManagerOverflow,
    runUiCommand,
  );
  window.addEventListener("unibrowser-workspace-layout-loaded", () => {
    void refreshCommandManager();
  });
  window.addEventListener("unibrowser-shell-layout-changed", () => {
    void syncContentBounds();
  });
  initAppMenu(
    (commandId) => runKeybindingCommand(commandId),
    (path) => {
      omnibox.value = path;
      void navigateOmnibox();
    },
    { menuButtonId: "title-menu-btn" },
  );
  void loadKeybindingHints();
  window.addEventListener("unibrowser-autosave-tick", (event) => {
    const detail = (event as CustomEvent<number>).detail;
    if (typeof detail === "number" && detail > 0) {
      lastAutosaveAtMs = detail;
      void refreshState();
    }
  });
  void invoke<{ last_autosave_at_ms: number }>("get_autosave_status").then((st) => {
    if (st.last_autosave_at_ms > 0) {
      lastAutosaveAtMs = st.last_autosave_at_ms;
    }
  });
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
});
