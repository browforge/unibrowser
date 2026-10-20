import { t } from "./i18n";

export type HistoryReplayStatus = {
  blocked: boolean;
  read_only: boolean;
  applied_seq: number;
  plugin_id: string | null;
  required_version: string | null;
  installed_version: string | null;
};

let bannerEl: HTMLElement | null = null;
let dismissed = false;
let activePath: string | null = null;

function ensureBanner(hostParent: HTMLElement): HTMLElement {
  if (!bannerEl) {
    bannerEl = document.createElement("div");
    bannerEl.className = "history-plugin-banner hidden";
    bannerEl.setAttribute("role", "status");
    bannerEl.innerHTML = `
      <span class="history-plugin-banner-text"></span>
      <div class="history-plugin-banner-actions">
        <button type="button" class="history-plugin-banner-install">${t("historyPluginBanner.install")}</button>
        <button type="button" class="history-plugin-banner-continue">${t("historyPluginBanner.continueReadOnly")}</button>
      </div>`;
    hostParent.insertBefore(bannerEl, hostParent.firstChild);
    bannerEl.querySelector(".history-plugin-banner-install")?.addEventListener("click", () => {
      void import("@tauri-apps/api/core").then(({ invoke }) =>
        invoke("open_mod_portal", { ext: null }).catch(() => undefined),
      );
    });
    bannerEl.querySelector(".history-plugin-banner-continue")?.addEventListener("click", () => {
      dismissed = true;
      bannerEl?.classList.add("hidden");
      document.body.dataset.historyReadOnly = "1";
    });
  } else if (bannerEl.parentElement !== hostParent) {
    hostParent.insertBefore(bannerEl, hostParent.firstChild);
  }
  return bannerEl;
}

export function hideHistoryPluginBanner(): void {
  bannerEl?.classList.add("hidden");
  delete document.body.dataset.historyReadOnly;
  dismissed = false;
  activePath = null;
}

function resolveBannerParent(hostParent: HTMLElement | null): HTMLElement | null {
  if (!hostParent) {
    return document.getElementById("document-card-shell");
  }
  return document.getElementById("document-card-shell") ?? hostParent;
}

export async function refreshHistoryPluginBanner(
  hostParent: HTMLElement | null,
  path: string,
): Promise<HistoryReplayStatus | null> {
  const parent = resolveBannerParent(hostParent);
  if (!parent || !path) {
    hideHistoryPluginBanner();
    return null;
  }
  if (activePath !== path) {
    dismissed = false;
    activePath = path;
  }
  const { invoke } = await import("@tauri-apps/api/core");
  const status = await invoke<HistoryReplayStatus>("document_history_replay_status", { path });
  if (!status.blocked || dismissed) {
    bannerEl?.classList.add("hidden");
    if (!status.read_only) {
      delete document.body.dataset.historyReadOnly;
    }
    return status;
  }
  const banner = ensureBanner(parent);
  const text = banner.querySelector(".history-plugin-banner-text");
  if (text) {
    const plugin = status.plugin_id ?? "plugin";
    const required = status.required_version ?? "?";
    text.textContent = t("historyPluginBanner.lead", { plugin, version: required });
  }
  banner.classList.remove("hidden");
  document.body.dataset.historyReadOnly = "1";
  document.body.dataset.historyPluginBannerVisible = "1";
  document.body.dataset.historyMissingPlugin = status.plugin_id ?? "";
  document.body.dataset.historyMissingVersion = status.required_version ?? "";
  return status;
}
