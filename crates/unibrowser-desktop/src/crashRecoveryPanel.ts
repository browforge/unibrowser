import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";

export interface CrashRecoveryBackupDto {
  backup_id: string;
  path: string;
  backend_id: string;
  saved_at_ms: number;
  content_bytes: number;
}

function formatWhen(ms: number): string {
  if (!ms) {
    return t("crashRecovery.unknownTime");
  }
  try {
    return new Date(ms).toLocaleString();
  } catch {
    return String(ms);
  }
}

function backendLabel(id: string): string {
  if (id.includes("svg")) {
    return "SVG";
  }
  if (id.includes("text")) {
    return "Text";
  }
  return id;
}

export async function appendCrashRecoveryPanel(
  root: HTMLElement,
  options?: { compact?: boolean },
) {
  const compact = options?.compact ?? false;
  const status = document.createElement("p");
  status.className = "crash-recovery-status viewer-note";
  status.hidden = true;

  const list = document.createElement("ul");
  list.className = "crash-recovery-list";

  const empty = document.createElement("p");
  empty.className = "viewer-note";
  empty.textContent = t("crashRecovery.empty");

  const refresh = async () => {
    list.replaceChildren();
    status.hidden = true;
    const backups = await invoke<CrashRecoveryBackupDto[]>("list_crash_recovery_backups");
    if (!backups.length) {
      empty.hidden = false;
      list.hidden = true;
      return;
    }
    empty.hidden = true;
    list.hidden = false;
    for (const row of backups) {
      const li = document.createElement("li");
      li.className = "crash-recovery-row";
      const meta = document.createElement("div");
      meta.className = "crash-recovery-meta";
      const pathEl = document.createElement("span");
      pathEl.className = "crash-recovery-path";
      pathEl.textContent = row.path;
      const detail = document.createElement("span");
      detail.className = "crash-recovery-detail";
      detail.textContent = t("crashRecovery.rowDetail", {
        kind: backendLabel(row.backend_id),
        when: formatWhen(row.saved_at_ms),
        bytes: row.content_bytes,
      });
      meta.appendChild(pathEl);
      meta.appendChild(detail);
      const actions = document.createElement("div");
      actions.className = "crash-recovery-actions";
      const restoreBtn = document.createElement("button");
      restoreBtn.type = "button";
      restoreBtn.textContent = t("crashRecovery.restore");
      restoreBtn.onclick = async () => {
        restoreBtn.disabled = true;
        try {
          const data = await invoke("restore_crash_recovery_backup", {
            backupId: row.backup_id,
          });
          status.textContent = t("crashRecovery.restored", { path: row.path });
          status.hidden = false;
          window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
          await refresh();
        } catch (err) {
          status.textContent =
            err instanceof Error ? err.message : t("toast.errorDefault");
          status.hidden = false;
        } finally {
          restoreBtn.disabled = false;
        }
      };
      const dismissBtn = document.createElement("button");
      dismissBtn.type = "button";
      dismissBtn.textContent = t("crashRecovery.dismiss");
      dismissBtn.onclick = async () => {
        dismissBtn.disabled = true;
        try {
          await invoke("dismiss_crash_recovery_backup", { backupId: row.backup_id });
          status.textContent = t("crashRecovery.dismissed");
          status.hidden = false;
          await refresh();
        } catch (err) {
          status.textContent =
            err instanceof Error ? err.message : t("toast.errorDefault");
          status.hidden = false;
        } finally {
          dismissBtn.disabled = false;
        }
      };
      actions.appendChild(restoreBtn);
      actions.appendChild(dismissBtn);
      li.appendChild(meta);
      li.appendChild(actions);
      list.appendChild(li);
    }
  };

  if (!compact) {
    const hint = document.createElement("p");
    hint.className = "viewer-note";
    hint.textContent = t("crashRecovery.hint");
    root.appendChild(hint);
  }
  root.appendChild(status);
  root.appendChild(empty);
  root.appendChild(list);
  await refresh();
}
