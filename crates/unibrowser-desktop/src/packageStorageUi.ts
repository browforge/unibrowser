import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import type { StatusItem } from "./statusBar";

export type PackageStorageSessionDto = {
  archive_path?: string | null;
  bundle_root: string;
  mode: string;
  history_path: string;
  edited: boolean;
  history_notice_dismissed: boolean;
  unpacked_bytes: number;
  suggest_folder: boolean;
  unpack_in_progress: boolean;
  unpack_progress: number;
};

export function packageStorageStatusItems(
  session: PackageStorageSessionDto | null | undefined,
): StatusItem[] {
  if (!session) {
    return [];
  }
  const left: StatusItem[] = [];
  const right: StatusItem[] = [];
  const modeLabel =
    session.mode === "view_only"
      ? t("pkg.storage.viewOnly")
      : session.mode === "archive"
        ? t("pkg.storage.archive")
        : t("pkg.storage.folder");
  const modeTip =
    session.mode === "view_only"
      ? t("pkg.storage.viewOnlyTip")
      : session.mode === "archive"
        ? t("pkg.storage.archiveTip")
        : t("pkg.storage.folderTip");
  left.push({
    id: "pkg-storage-mode",
    slot: "left",
    text: modeLabel,
    title: modeTip,
    priority: 120,
  });
  if (session.mode !== "view_only" && !session.history_notice_dismissed) {
    left.push({
      id: "pkg-history-path",
      slot: "left",
      text: t("pkg.storage.historyWillBeSaved", { path: session.history_path }),
      priority: 110,
    });
  }
  if (session.suggest_folder && session.mode === "archive" && !session.unpack_in_progress) {
    right.push({
      id: "pkg-large-hint",
      slot: "right",
      text: t("pkg.storage.largeHint"),
      kind: "warning",
      priority: 90,
      title: t("pkg.storage.largeHintAction"),
    });
  }
  if (session.unpack_in_progress) {
    left.push({
      id: "pkg-unpack-progress",
      slot: "left",
      text: t("pkg.unpack.progress", { percent: session.unpack_progress }),
      priority: 130,
    });
  }
  return [...left, ...right];
}

export function mountHistoryLocationNotice(
  session: PackageStorageSessionDto,
  onDismiss: () => void,
): HTMLElement | null {
  if (session.mode === "view_only" || session.history_notice_dismissed) {
    return null;
  }
  const bar = document.createElement("div");
  bar.className = "pkg-history-notice";
  bar.dataset.pkgHistoryNotice = "1";
  const text = document.createElement("span");
  text.textContent = t("pkg.storage.historyWillBeSaved", { path: session.history_path });
  const dismiss = document.createElement("button");
  dismiss.type = "button";
  dismiss.className = "quiet-button";
  dismiss.textContent = t("onboarding.dismiss");
  dismiss.addEventListener("click", () => {
    bar.remove();
    void onDismiss();
  });
  bar.append(text, dismiss);
  return bar;
}

export function showUnpackProgressCapture(): void {
  document.querySelector(".pkg-unpack-progress-overlay")?.remove();
  const overlay = document.createElement("div");
  overlay.className = "pkg-unpack-progress-overlay";
  overlay.dataset.pkgUnpackProgress = "1";
  const panel = document.createElement("div");
  panel.className = "first-save-panel pkg-unpack-progress-panel";
  const title = document.createElement("h2");
  title.textContent = t("pkg.unpack.title");
  const bar = document.createElement("div");
  bar.className = "pkg-unpack-progress-bar";
  bar.setAttribute("role", "progressbar");
  bar.setAttribute("aria-valuenow", "65");
  const fill = document.createElement("div");
  fill.className = "pkg-unpack-progress-fill";
  fill.style.width = "65%";
  bar.appendChild(fill);
  const label = document.createElement("p");
  label.className = "viewer-note";
  label.textContent = t("pkg.unpack.progress", { percent: 65 });
  panel.append(title, bar, label);
  overlay.append(panel);
  document.body.appendChild(overlay);
}

export async function runUnpackToFolder(projectPath: string): Promise<void> {
  const dest = await invoke<string | null>("pick_browse_folder");
  if (!dest) {
    return;
  }
  await invoke("unpack_unibrow_to_folder", {
    projectPath,
    destinationDir: dest,
  });
}

export async function runPackToUnibrow(projectPath: string): Promise<void> {
  const dest = await invoke<string | null>("pick_save_unibrow_project_path");
  if (!dest) {
    return;
  }
  await invoke("pack_unibrow_to_archive", {
    projectPath,
    destZip: dest,
  });
}

export function showHistoryNoticeCapture(): void {
  document.querySelector("[data-pkg-history-notice]")?.remove();
  const notice = mountHistoryLocationNotice(
    {
      bundle_root: "/tmp",
      mode: "archive",
      history_path: "/home/me/Projects/Demo.unibrow",
      edited: false,
      history_notice_dismissed: false,
      unpacked_bytes: 0,
      suggest_folder: false,
      unpack_in_progress: false,
      unpack_progress: 0,
    },
    () => undefined,
  );
  if (notice) {
    document.body.prepend(notice);
  }
}
