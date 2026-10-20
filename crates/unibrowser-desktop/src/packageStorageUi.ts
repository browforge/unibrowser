import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import type { StatusItem } from "./statusBar";
import { showActionToast } from "./toast";

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

let historyToastTab: string | null = null;

export function packageStorageStatusItems(
  session: PackageStorageSessionDto | null | undefined,
  saveLabel?: string,
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
  if (saveLabel) {
    left.push({
      id: "pkg-document-save",
      slot: "left",
      text: saveLabel,
      priority: 115,
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

function dismissPackageHistoryToast(): void {
  historyToastTab = null;
  document.querySelector("[data-pkg-history-toast]")?.remove();
}

export function syncPackageHistoryNotice(
  session: PackageStorageSessionDto | null | undefined,
  projectPath: string,
): void {
  if (
    !session ||
    session.mode === "view_only" ||
    session.history_notice_dismissed ||
    session.edited
  ) {
    dismissPackageHistoryToast();
    return;
  }
  if (historyToastTab === projectPath && document.querySelector("[data-pkg-history-toast]")) {
    return;
  }
  dismissPackageHistoryToast();
  historyToastTab = projectPath;
  const message = t("pkg.storage.historyWillBeSaved", { path: session.history_path });
  showActionToast(message, t("onboarding.dismiss"), () => {
    dismissPackageHistoryToast();
    void invoke("dismiss_package_history_notice", { projectPath });
  }, "info");
  const host = document.getElementById("toast-host");
  host?.setAttribute("data-pkg-history-toast", "1");
}

export function mountUnpackProgressOverlay(
  destinationDir: string,
  onCancel: () => void,
  percent = 65,
): HTMLElement {
  document.querySelector(".pkg-unpack-progress-overlay")?.remove();
  const overlay = document.createElement("div");
  overlay.className = "pkg-unpack-progress-overlay";
  overlay.dataset.pkgUnpackProgress = "1";
  const panel = document.createElement("div");
  panel.className = "first-save-panel pkg-unpack-progress-panel";
  const title = document.createElement("h2");
  title.textContent = t("pkg.unpack.title");
  const dest = document.createElement("p");
  dest.className = "pkg-unpack-dest viewer-note";
  dest.textContent = t("pkg.unpack.destination", { path: destinationDir });
  const bar = document.createElement("div");
  bar.className = "pkg-unpack-progress-bar";
  bar.setAttribute("role", "progressbar");
  bar.setAttribute("aria-valuenow", String(percent));
  const fill = document.createElement("div");
  fill.className = "pkg-unpack-progress-fill";
  fill.style.width = `${percent}%`;
  bar.appendChild(fill);
  const label = document.createElement("p");
  label.className = "viewer-note";
  label.textContent = t("pkg.unpack.progress", { percent });
  const actions = document.createElement("div");
  actions.className = "first-save-actions";
  const cancel = document.createElement("button");
  cancel.type = "button";
  cancel.className = "quiet-button";
  cancel.textContent = t("pkg.unpack.cancel");
  cancel.addEventListener("click", () => {
    overlay.remove();
    onCancel();
  });
  actions.appendChild(cancel);
  panel.append(title, dest, bar, label, actions);
  overlay.append(panel);
  document.body.appendChild(overlay);
  return overlay;
}

export async function showUnpackProgressCapture(destinationDir?: string): Promise<void> {
  let dest = destinationDir;
  if (!dest) {
    try {
      dest = await invoke<string>("pkg_capture_unpack_dest_path_cmd");
    } catch {
      dest = "~/Projects/UnibrowUnpack";
    }
  }
  mountUnpackProgressOverlay(dest, () => undefined, 65);
}

export async function runUnpackToFolder(projectPath: string): Promise<void> {
  const dest = await invoke<string | null>("pick_browse_folder");
  if (!dest) {
    return;
  }
  let cancelled = false;
  const overlay = mountUnpackProgressOverlay(dest, () => {
    cancelled = true;
    void invoke("cancel_unpack_unibrow", { projectPath });
  });
  try {
    await invoke("unpack_unibrow_to_folder", {
      projectPath,
      destinationDir: dest,
    });
  } catch (err) {
    if (!cancelled) {
      throw err;
    }
  } finally {
    overlay.remove();
  }
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

export async function showHistoryNoticeCapture(): Promise<void> {
  dismissPackageHistoryToast();
  const historyPath = await invoke<string>("pkg_capture_fixture_archive_path_cmd").catch(
    () => "~/Demo.unibrow",
  );
  syncPackageHistoryNotice(
    {
      bundle_root: "/tmp",
      mode: "archive",
      history_path: historyPath,
      edited: false,
      history_notice_dismissed: false,
      unpacked_bytes: 0,
      suggest_folder: false,
      unpack_in_progress: false,
      unpack_progress: 0,
    },
    historyPath,
  );
}
