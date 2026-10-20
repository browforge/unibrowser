import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { createLucideIcon } from "./ui/lucideIcon";

export type PackageOpenMode = "view_only" | "archive" | "folder";

export type UnibrowOpenEstimate = {
  archive_path: string;
  unpacked_bytes: number;
  suggest_folder: boolean;
};

type OpenModeDialogOptions = {
  estimate: UnibrowOpenEstimate;
  captureOnly?: boolean;
};

function formatBytes(n: number): string {
  if (n < 1024 * 1024) {
    return `${Math.round(n / 1024)} KB`;
  }
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}

export function mountPackageOpenModeDialog(
  options: OpenModeDialogOptions,
): Promise<{ mode: PackageOpenMode; folderDest?: string } | null> {
  const { estimate, captureOnly = false } = options;
  return new Promise((resolve) => {
    document.querySelectorAll(".pkg-open-mode-overlay").forEach((el) => el.remove());
    const overlay = document.createElement("div");
    overlay.className = "first-save-overlay pkg-open-mode-overlay";
    overlay.dataset.pkgOpenMode = "1";
    const panel = document.createElement("div");
    panel.className = "first-save-panel pkg-open-mode-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");

    const title = document.createElement("h2");
    title.textContent = t("pkg.openMode.title");
    panel.appendChild(title);

    const lead = document.createElement("p");
    lead.className = "viewer-note first-save-lead";
    lead.textContent = t("pkg.openMode.lead");
    panel.appendChild(lead);

    if (estimate.suggest_folder) {
      const hint = document.createElement("p");
      hint.className = "pkg-folder-hint viewer-note";
      hint.textContent = t("pkg.openMode.largeHint", {
        size: formatBytes(estimate.unpacked_bytes),
      });
      panel.appendChild(hint);
    }

    let mode: PackageOpenMode = "archive";
    const modeGroup = document.createElement("div");
    modeGroup.className = "first-save-modes";
    const modes: {
      id: PackageOpenMode;
      label: string;
      hint: string;
      icon: "eye" | "package-open" | "folder";
    }[] = [
      {
        id: "view_only",
        label: t("pkg.storage.viewOnly"),
        hint: t("pkg.openMode.viewOnlyHint"),
        icon: "eye",
      },
      {
        id: "archive",
        label: t("pkg.storage.archive"),
        hint: t("pkg.openMode.archiveHint"),
        icon: "package-open",
      },
      {
        id: "folder",
        label: t("pkg.storage.folder"),
        hint: t("pkg.openMode.folderHint"),
        icon: "folder",
      },
    ];
    modes.forEach((entry) => {
      const row = document.createElement("label");
      row.className = "first-save-mode-card";
      if (entry.id === mode) {
        row.className += " first-save-mode-card--selected";
      }
      const radio = document.createElement("input");
      radio.type = "radio";
      radio.name = "pkg-open-mode";
      radio.value = entry.id;
      radio.checked = entry.id === mode;
      radio.addEventListener("change", () => {
        mode = entry.id;
        modeGroup.querySelectorAll(".first-save-mode-card").forEach((el) => {
          el.classList.toggle(
            "first-save-mode-card--selected",
            (el.querySelector("input") as HTMLInputElement)?.value === mode,
          );
        });
      });
      const icon = createLucideIcon(entry.icon, 20);
      const text = document.createElement("div");
      text.className = "first-save-mode-card-text";
      const label = document.createElement("strong");
      label.textContent = entry.label;
      const hintEl = document.createElement("span");
      hintEl.className = "first-save-mode-card-hint";
      hintEl.textContent = entry.hint;
      text.append(label, hintEl);
      row.append(radio, icon, text);
      modeGroup.appendChild(row);
    });
    panel.appendChild(modeGroup);

    const actions = document.createElement("div");
    actions.className = "first-save-actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.className = "quiet-button";
    cancel.textContent = t("project.cancel");
    const open = document.createElement("button");
    open.type = "button";
    open.className = "primary";
    open.textContent = t("pkg.openMode.open");
    actions.append(cancel, open);
    panel.appendChild(actions);
    overlay.append(panel);
    document.body.appendChild(overlay);

    const finish = async () => {
      if (mode === "folder" && !captureOnly) {
        const folder = await invoke<string | null>("pick_browse_folder");
        if (!folder) {
          return;
        }
        document.body.removeChild(overlay);
        resolve({ mode, folderDest: folder });
        return;
      }
      document.body.removeChild(overlay);
      resolve({ mode, folderDest: captureOnly ? "/tmp/UnibrowUnpack" : undefined });
    };

    open.addEventListener("click", () => {
      if (captureOnly) {
        return;
      }
      void finish();
    });
    cancel.addEventListener("click", () => {
      document.body.removeChild(overlay);
      resolve(null);
    });
    if (!captureOnly) {
      overlay.addEventListener("click", (e) => {
        if (e.target === overlay) {
          document.body.removeChild(overlay);
          resolve(null);
        }
      });
    }
  });
}

export async function promptUnibrowOpenMode(path: string): Promise<boolean> {
  const estimate = await invoke<UnibrowOpenEstimate>("query_unibrow_open_cmd", { path });
  const choice = await mountPackageOpenModeDialog({ estimate });
  if (!choice) {
    return false;
  }
  await invoke("open_unibrow_with_storage_mode", {
    path,
    mode: choice.mode,
    folderDest: choice.folderDest ?? null,
  });
  return true;
}

export function showPackageOpenModeCapture(estimate?: UnibrowOpenEstimate): void {
  void mountPackageOpenModeDialog({
    estimate: estimate ?? {
      archive_path: "/home/me/Demo.unibrow",
      unpacked_bytes: 120_000_000,
      suggest_folder: true,
    },
    captureOnly: true,
  });
}
