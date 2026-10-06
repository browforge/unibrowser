import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";

export interface RecoveryBarItem {
  backup_id: string;
  path: string;
  backend_id: string;
}

export function renderRecoveryBar(
  host: HTMLElement,
  items: RecoveryBarItem[],
  onRefresh: () => void,
) {
  host.innerHTML = "";
  if (!items.length) {
    host.classList.add("hidden");
    return;
  }
  host.classList.remove("hidden");
  for (const item of items) {
    const row = document.createElement("div");
    row.className = "recovery-bar-row";
    const label = document.createElement("span");
    label.textContent = t("recoveryBar.message", { path: item.path });
    row.appendChild(label);
    const view = document.createElement("button");
    view.type = "button";
    view.textContent = t("recoveryBar.viewOriginal");
    view.onclick = async () => {
      await invoke("reload_active_tab_from_disk");
      await invoke("dismiss_recovery_bar_item", { backupId: item.backup_id });
      onRefresh();
    };
    const discard = document.createElement("button");
    discard.type = "button";
    discard.textContent = t("recoveryBar.discard");
    discard.onclick = async () => {
      await invoke("dismiss_recovery_bar_item", { backupId: item.backup_id });
      onRefresh();
    };
    row.appendChild(view);
    row.appendChild(discard);
    host.appendChild(row);
  }
}
