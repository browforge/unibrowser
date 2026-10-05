import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";

export interface ProjectRecoveryBackupDto {
  backup_id: string;
  project_path: string;
  package_root: string;
  saved_at_ms: number;
  bytes: number;
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

/** Simple launch prompt when debounced project.json backups exist. */
export async function maybePromptProjectRecovery(
  onRestored: (data: unknown) => void,
): Promise<void> {
  try {
    const backups = await invoke<ProjectRecoveryBackupDto[]>("list_project_recovery_backups");
    if (!backups.length) {
      return;
    }
    const latest = backups[0];
    const when = formatWhen(latest.saved_at_ms);
    const message = t("projectRecovery.launchPrompt", {
      path: latest.project_path,
      when,
    });
    const restore = window.confirm(message);
    if (restore) {
      const data = await invoke("restore_project_recovery_backup", {
        backupId: latest.backup_id,
      });
      onRestored(data);
      return;
    }
    const dismiss = window.confirm(t("projectRecovery.dismissAllConfirm"));
    if (dismiss) {
      for (const row of backups) {
        await invoke("dismiss_project_recovery_backup", { backupId: row.backup_id });
      }
    }
  } catch {
    /* non-fatal */
  }
}
