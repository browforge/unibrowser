import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { showToast } from "./toast";
import { emptyStateBlock } from "./emptyState";

export interface DownloadHistoryEntryDto {
  id: string;
  url: string;
  label: string;
  status: string;
  error?: string | null;
  destination?: string | null;
  at_ms: number;
}

interface TabContentDto {
  tabs: unknown[];
  active_index: number;
  omnibox: string;
}

export async function renderDownloadsManager(
  root: HTMLElement,
  entries: DownloadHistoryEntryDto[],
  onRefresh: () => void | Promise<void>,
) {
  root.innerHTML = "";
  const head = document.createElement("div");
  head.className = "downloads-manager-head";
  const title = document.createElement("h2");
  title.textContent = t("downloads.managerTitle");
  const clearBtn = document.createElement("button");
  clearBtn.type = "button";
  clearBtn.textContent = t("downloads.clearList");
  clearBtn.onclick = async () => {
    await invoke("clear_download_history");
    showToast(t("downloads.cleared"), "info");
    await onRefresh();
  };
  const folderBtn = document.createElement("button");
  folderBtn.type = "button";
  folderBtn.textContent = t("downloads.openFolder");
  folderBtn.onclick = async () => {
    const path = await invoke<string>("get_downloads_dir");
    await invoke<TabContentDto>("open_path", { path });
    await onRefresh();
  };
  head.appendChild(title);
  head.appendChild(clearBtn);
  head.appendChild(folderBtn);
  root.appendChild(head);

  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = t("downloads.managerNote");
  root.appendChild(note);

  if (!entries.length) {
    root.appendChild(
      emptyStateBlock(t("downloads.empty"), [t("downloads.emptyHint")]),
    );
    return;
  }

  const list = document.createElement("ul");
  list.className = "downloads-manager-list";
  entries.forEach((entry) => {
    const li = document.createElement("li");
    li.className = `downloads-item downloads-item--${entry.status}`;
    const main = document.createElement("button");
    main.type = "button";
    main.className = "downloads-item-open";
    const statusLabel =
      entry.status === "failed"
        ? t("downloads.statusFailed")
        : entry.status === "saved"
          ? t("downloads.statusSaved")
          : t("downloads.statusOpenedTab");
    main.textContent = `${entry.label} · ${statusLabel}`;
    main.title = entry.url;
    main.onclick = async () => {
      if (entry.status === "failed") {
        return;
      }
      if (entry.url.startsWith("file://") || entry.url.startsWith("/")) {
        const path = entry.url.replace(/^file:\/\//, "");
        await invoke<TabContentDto>("open_path", { path });
      } else {
        await invoke<TabContentDto>("open_path", { path: entry.url });
      }
      await onRefresh();
    };
    li.appendChild(main);
    if (entry.error) {
      const err = document.createElement("span");
      err.className = "downloads-item-error";
      err.textContent = entry.error;
      li.appendChild(err);
    }
    if (entry.status === "failed") {
      const retry = document.createElement("button");
      retry.type = "button";
      retry.className = "downloads-item-retry";
      retry.textContent = t("downloads.retry");
      retry.onclick = async () => {
        try {
          await invoke<TabContentDto>("retry_download", { url: entry.url });
          showToast(t("downloads.retryOk"), "info");
          await onRefresh();
        } catch (e) {
          showToast(String(e), "error");
        }
      };
      li.appendChild(retry);
    }
    list.appendChild(li);
  });
  root.appendChild(list);
}
