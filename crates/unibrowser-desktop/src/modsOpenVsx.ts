const STORAGE_KEY = "unibrowser.openVsxNetwork";

export function isOpenVsxNetworkEnabled(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

export function setOpenVsxNetworkEnabled(enabled: boolean): void {
  try {
    localStorage.setItem(STORAGE_KEY, enabled ? "1" : "0");
  } catch {
    /* ignore */
  }
  document.documentElement.dataset.openVsxNetwork = enabled ? "on" : "off";
  window.dispatchEvent(
    new CustomEvent("unibrowser-open-vsx-network", { detail: enabled }),
  );
}

export function syncOpenVsxNetworkDataset(): void {
  document.documentElement.dataset.openVsxNetwork = isOpenVsxNetworkEnabled() ? "on" : "off";
}
