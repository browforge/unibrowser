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
    const restore = document.createElement("button");
    restore.type = "button";
    restore.textContent = t("recoveryBar.restore");
    restore.onclick = async () => {
      await invoke("restore_recovery_bar_item", { backupId: item.backup_id });
      onRefresh();
    };
    const show = document.createElement("button");
    show.type = "button";
    show.textContent = t("recoveryBar.showChanges");
    show.onclick = async () => {
      await invoke("show_recovery_changes", { backupId: item.backup_id });
      onRefresh();
    };
    const discard = document.createElement("button");
    discard.type = "button";
    discard.textContent = t("recoveryBar.discard");
    discard.onclick = async () => {
      await invoke("dismiss_recovery_bar_item", { backupId: item.backup_id });
      onRefresh();
    };
    row.appendChild(restore);
    row.appendChild(show);
    row.appendChild(discard);
    host.appendChild(row);
  }
}
