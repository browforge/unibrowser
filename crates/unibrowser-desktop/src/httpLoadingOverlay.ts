import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { t } from "./i18n";

const HTTP_LOAD_TIMEOUT_MS = 30_000;

type HttpPageLoadPayload = {
  windowLabel: string;
  url: string;
  phase: string;
};

type HttpPageTitlePayload = {
  windowLabel: string;
  title: string;
};

let overlayEl: HTMLDivElement | null = null;
let spinnerEl: HTMLElement | null = null;
let messageEl: HTMLElement | null = null;
let timeoutId: ReturnType<typeof setTimeout> | null = null;
let activeUrl: string | null = null;
let loading = false;

function ensureOverlay(host: HTMLElement) {
  if (overlayEl && host.contains(overlayEl)) {
    return;
  }
  overlayEl = document.createElement("div");
  overlayEl.id = "http-loading-overlay";
  overlayEl.className = "http-loading-overlay hidden";
  overlayEl.setAttribute("role", "status");
  overlayEl.setAttribute("aria-live", "polite");
  spinnerEl = document.createElement("div");
  spinnerEl.className = "http-loading-spinner";
  spinnerEl.setAttribute("aria-hidden", "true");
  messageEl = document.createElement("p");
  messageEl.className = "http-loading-message";
  overlayEl.appendChild(spinnerEl);
  overlayEl.appendChild(messageEl);
  host.appendChild(overlayEl);
}

function clearTimeoutTimer() {
  if (timeoutId !== null) {
    clearTimeout(timeoutId);
    timeoutId = null;
  }
}

function hideOverlay() {
  loading = false;
  activeUrl = null;
  clearTimeoutTimer();
  if (!overlayEl) {
    return;
  }
  overlayEl.classList.add("hidden");
  if (messageEl) {
    messageEl.textContent = "";
  }
  if (spinnerEl) {
    spinnerEl.classList.remove("hidden");
  }
}

function showSlowMessage() {
  if (!loading || !messageEl || !spinnerEl) {
    return;
  }
  spinnerEl.classList.add("hidden");
  messageEl.textContent = t("httpLoad.timeout");
}

function showLoading(host: HTMLElement, url: string) {
  ensureOverlay(host);
  loading = true;
  activeUrl = url;
  clearTimeoutTimer();
  if (messageEl) {
    messageEl.textContent = "";
  }
  if (spinnerEl) {
    spinnerEl.classList.remove("hidden");
  }
  overlayEl!.classList.remove("hidden");
  timeoutId = setTimeout(showSlowMessage, HTTP_LOAD_TIMEOUT_MS);
}

function isThisWindow(windowLabel: string) {
  return windowLabel === getCurrentWebviewWindow().label;
}

function maybeEndLoad(url: string) {
  if (!loading || !activeUrl) {
    return;
  }
  if (url !== activeUrl) {
    return;
  }
  hideOverlay();
}

function maybeEndOnTitle(title: string) {
  if (!loading) {
    return;
  }
  const trimmed = title.trim();
  if (!trimmed || trimmed === "about:blank") {
    return;
  }
  hideOverlay();
}

export function bindHttpLoadingOverlay(contentHost: HTMLElement) {
  ensureOverlay(contentHost);
}

export function syncHttpLoadingForTab(isHttp: boolean, url?: string) {
  if (!isHttp || !url || url === "about:blank") {
    hideOverlay();
    return;
  }
  const host = document.getElementById("content-host");
  if (!host) {
    return;
  }
  if (activeUrl === url && loading) {
    return;
  }
  showLoading(host, url);
}

export function initHttpLoadingListeners(contentHost: HTMLElement) {
  bindHttpLoadingOverlay(contentHost);
  void listen<HttpPageLoadPayload>("unibrowser-http-page-load", (event) => {
    const { windowLabel, url, phase } = event.payload;
    if (!isThisWindow(windowLabel)) {
      return;
    }
    if (phase === "started") {
      showLoading(contentHost, url);
      return;
    }
    if (phase === "finished") {
      maybeEndLoad(url);
    }
  });
  void listen<HttpPageTitlePayload>("unibrowser-http-page-title", (event) => {
    const { windowLabel, title } = event.payload;
    if (!isThisWindow(windowLabel)) {
      return;
    }
    maybeEndOnTitle(title);
  });
}
