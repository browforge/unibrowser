import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { showInlineConfirm } from "./inlineUi";

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

/** Non-modal launch flow when debounced project.json backups exist. */
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
    const restore = await showInlineConfirm(message, t("recoveryBar.viewOriginal"), "Not now");
    if (restore) {
      const data = await invoke("restore_project_recovery_backup", {
        backupId: latest.backup_id,
      });
      onRestored(data);
      return;
    }
    const dismiss = await showInlineConfirm(t("projectRecovery.dismissAllConfirm"));
    if (dismiss) {
      for (const row of backups) {
        await invoke("dismiss_project_recovery_backup", { backupId: row.backup_id });
      }
    }
  } catch {
    /* non-fatal */
  }
}
