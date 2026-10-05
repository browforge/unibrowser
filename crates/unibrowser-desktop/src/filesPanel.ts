import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { copyText } from "./clipboard";
import { t } from "./i18n";
import { showToast } from "./toast";
import type { DirEntry, FilesViewDto } from "./filesPanelTypes";

export type { DirEntry, FilesViewDto } from "./filesPanelTypes";

const THUMB_MAX_BYTES = 8_000_000;
const THUMB_MAX_ACTIVE = 24;
let thumbLoadCount = 0;

export interface FilesPanelDeps {
  entries: DirEntry[];
  filesView: FilesViewDto;
  browsePath: string;
  folderFavorites: string[];
  onApplyState: (data: unknown) => void | Promise<void>;
  persistFilesView: (view: FilesViewDto) => Promise<void>;
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function breadcrumbSegments(path: string): { label: string; path: string }[] {
  const normal = path.replace(/\\/g, "/").replace(/\/+$/, "");
  if (!normal) {
    return [];
  }
  const isAbs = normal.startsWith("/");
  const parts = normal.split("/").filter(Boolean);
  const out: { label: string; path: string }[] = [];
  let acc = "";
  for (const part of parts) {
    if (!acc) {
      acc = isAbs ? `/${part}` : part;
    } else {
      acc = `${acc}/${part}`;
    }
    out.push({ label: part, path: acc });
  }
  return out;
}

function parentPath(path: string): string | null {
  const normal = path.replace(/\\/g, "/").replace(/\/+$/, "");
  if (!normal || normal === "/") {
    return null;
  }
  const idx = normal.lastIndexOf("/");
  if (idx <= 0) {
    return "/";
  }
  return normal.slice(0, idx);
}

async function openPath(path: string, deps: FilesPanelDeps) {
  const data = await invoke("open_path", { path });
  if (data) {
    await deps.onApplyState(data);
  }
}

async function openPathNewTab(path: string, deps: FilesPanelDeps) {
  const data = await invoke("open_path_in_new_tab", { path });
  if (data) {
    await deps.onApplyState(data);
  }
}

function showContextMenu(
  x: number,
  y: number,
  entry: DirEntry,
  deps: FilesPanelDeps,
  onPeekArchive?: () => void,
) {
  const menu = document.createElement("div");
  menu.className = "files-context-menu";
  menu.style.left = `${x}px`;
  menu.style.top = `${y}px`;

  const addItem = (label: string, action: () => void) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.textContent = label;
    btn.onclick = () => {
      menu.remove();
      void action();
    };
    menu.appendChild(btn);
  };

  addItem(t("files.ctxOpen"), () => openPath(entry.path, deps));
  addItem(t("files.ctxOpenNewTab"), () => openPathNewTab(entry.path, deps));
  addItem(t("files.ctxCopyPath"), () => {
    void copyText(entry.path, () => showToast(t("status.pathCopied"), "info"));
  });
  addItem(t("files.ctxReveal"), async () => {
    try {
      await invoke("reveal_path_in_system", { path: entry.path });
    } catch (err) {
      showToast(String(err), "error");
    }
  });
  if (entry.is_archive && onPeekArchive) {
    addItem(t("files.ctxPeekArchive"), onPeekArchive);
  }

  const close = () => menu.remove();
  document.body.appendChild(menu);
  const onDoc = (ev: MouseEvent) => {
    if (!menu.contains(ev.target as Node)) {
      close();
      document.removeEventListener("mousedown", onDoc);
    }
  };
  setTimeout(() => document.addEventListener("mousedown", onDoc), 0);
}

function attachLazyThumb(img: HTMLImageElement, path: string, sizeBytes: number) {
  const load = async () => {
    if (thumbLoadCount >= THUMB_MAX_ACTIVE) {
      return;
    }
    if (sizeBytes > THUMB_MAX_BYTES) {
      return;
    }
    thumbLoadCount += 1;
    try {
      await invoke("allow_asset_path", { path });
      img.src = convertFileSrc(path);
      img.classList.add("loaded");
    } catch {
      img.remove();
    }
  };
  if (typeof IntersectionObserver === "undefined") {
    void load();
    return;
  }
  const obs = new IntersectionObserver(
    (entries) => {
      if (entries.some((e) => e.isIntersecting)) {
        obs.disconnect();
        void load();
      }
    },
    { rootMargin: "80px" },
  );
  obs.observe(img);
}

export function renderFilesPanel(host: HTMLElement, deps: FilesPanelDeps) {
  host.innerHTML = "";
  thumbLoadCount = 0;

  const nav = document.createElement("div");
  nav.className = "files-nav";
  const parent = parentPath(deps.browsePath);
  if (parent) {
    const up = document.createElement("button");
    up.type = "button";
    up.className = "files-up";
    up.textContent = t("files.parent");
    up.title = parent;
    up.onclick = () => void openPath(parent, deps);
    nav.appendChild(up);
  }
  const crumbs = document.createElement("nav");
  crumbs.className = "files-breadcrumbs";
  crumbs.setAttribute("aria-label", t("files.breadcrumbs"));
  breadcrumbSegments(deps.browsePath).forEach((seg) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "files-crumb";
    btn.textContent = seg.label;
    btn.onclick = () => void openPath(seg.path, deps);
    crumbs.appendChild(btn);
  });
  nav.appendChild(crumbs);
  const favCurrent = document.createElement("button");
  favCurrent.type = "button";
  favCurrent.className = "files-fav-star";
  const isFav = deps.folderFavorites.includes(deps.browsePath);
  favCurrent.textContent = isFav ? "★" : "☆";
  favCurrent.title = t("files.toggleFavorite");
  favCurrent.onclick = async () => {
    const next = await invoke<string[]>("toggle_folder_favorite", { path: deps.browsePath });
    deps.folderFavorites.length = 0;
    deps.folderFavorites.push(...next);
    favCurrent.textContent = next.includes(deps.browsePath) ? "★" : "☆";
  };
  nav.appendChild(favCurrent);
  host.appendChild(nav);

  if (deps.folderFavorites.length) {
    const favRow = document.createElement("div");
    favRow.className = "files-favorites";
    deps.folderFavorites.forEach((p) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "files-fav-link";
      const label = p.split("/").filter(Boolean).pop() ?? p;
      btn.textContent = `★ ${label}`;
      btn.title = p;
      btn.onclick = () => void openPath(p, deps);
      favRow.appendChild(btn);
    });
    host.appendChild(favRow);
  }

  const toolbar = document.createElement("div");
  toolbar.className = "files-toolbar";
  const sortLabel = document.createElement("label");
  sortLabel.textContent = t("files.sort");
  const sortSelect = document.createElement("select");
  sortSelect.className = "files-sort";
  [
    { value: "name", label: t("files.sortName") },
    { value: "date", label: t("files.sortDate") },
    { value: "size", label: t("files.sortSize") },
  ].forEach((opt) => {
    const o = document.createElement("option");
    o.value = opt.value;
    o.textContent = opt.label;
    sortSelect.appendChild(o);
  });
  sortSelect.value = deps.filesView.sort;
  const orderBtn = document.createElement("button");
  orderBtn.type = "button";
  orderBtn.className = "files-order";
  orderBtn.textContent = deps.filesView.sort_ascending ? t("files.ascending") : t("files.descending");
  const hiddenCb = document.createElement("input");
  hiddenCb.type = "checkbox";
  hiddenCb.id = "files-show-hidden";
  hiddenCb.checked = deps.filesView.show_hidden;
  const hiddenLabel = document.createElement("label");
  hiddenLabel.htmlFor = hiddenCb.id;
  hiddenLabel.textContent = t("files.showHidden");
  sortSelect.onchange = () => {
    void deps.persistFilesView({ ...deps.filesView, sort: sortSelect.value });
  };
  orderBtn.onclick = () => {
    void deps.persistFilesView({
      ...deps.filesView,
      sort_ascending: !deps.filesView.sort_ascending,
    });
  };
  hiddenCb.onchange = () => {
    void deps.persistFilesView({ ...deps.filesView, show_hidden: hiddenCb.checked });
  };
  toolbar.appendChild(sortLabel);
  toolbar.appendChild(sortSelect);
  toolbar.appendChild(orderBtn);
  toolbar.appendChild(hiddenCb);
  toolbar.appendChild(hiddenLabel);
  host.appendChild(toolbar);

  const peekHost = document.createElement("div");
  peekHost.className = "files-archive-peek hidden";
  host.appendChild(peekHost);

  const table = document.createElement("table");
  table.className = "files-table";
  deps.entries.forEach((entry) => {
    const row = document.createElement("tr");
    row.oncontextmenu = (ev) => {
      ev.preventDefault();
      const peek = entry.is_archive
        ? () => {
            void (async () => {
              peekHost.classList.remove("hidden");
              peekHost.innerHTML = `<p class="viewer-note">${t("files.peekLoading")}</p>`;
              try {
                const items = await invoke<{ path: string; is_dir: boolean; size: number }[]>(
                  "peek_archive_entries",
                  { path: entry.path, maxEntries: 120 },
                );
                peekHost.innerHTML = "";
                const head = document.createElement("p");
                head.className = "viewer-note";
                head.textContent = t("files.peekTitle", { name: entry.name });
                peekHost.appendChild(head);
                const ul = document.createElement("ul");
                ul.className = "files-peek-list";
                items.forEach((item) => {
                  const li = document.createElement("li");
                  li.textContent = `${item.is_dir ? "[dir]" : "[file]"} ${item.path}${
                    item.is_dir ? "" : ` (${formatFileSize(item.size)})`
                  }`;
                  ul.appendChild(li);
                });
                peekHost.appendChild(ul);
              } catch (err) {
                peekHost.textContent = String(err);
              }
            })();
          }
        : undefined;
      showContextMenu(ev.clientX, ev.clientY, entry, deps, peek);
    };

    const thumbCell = document.createElement("td");
    thumbCell.className = "files-thumb-cell";
    if (entry.is_image) {
      const img = document.createElement("img");
      img.className = "files-thumb";
      img.alt = "";
      img.loading = "lazy";
      attachLazyThumb(img, entry.path, entry.size_bytes ?? 0);
      thumbCell.appendChild(img);
    }

    const nameCell = document.createElement("td");
    const link = document.createElement("button");
    link.className = "file-link";
    link.textContent = (entry.is_dir ? "[dir] " : "[file] ") + entry.name;
    link.onclick = () => void openPath(entry.path, deps);
    nameCell.appendChild(link);
    if (entry.is_dir) {
      const star = document.createElement("button");
      star.type = "button";
      star.className = "files-fav-star inline";
      const fav = deps.folderFavorites.includes(entry.path);
      star.textContent = fav ? "★" : "☆";
      star.title = t("files.toggleFavorite");
      star.onclick = async (e) => {
        e.stopPropagation();
        const next = await invoke<string[]>("toggle_folder_favorite", { path: entry.path });
        deps.folderFavorites.length = 0;
        deps.folderFavorites.push(...next);
        star.textContent = next.includes(entry.path) ? "★" : "☆";
      };
      nameCell.appendChild(star);
    }

    const metaCell = document.createElement("td");
    metaCell.className = "files-meta";
    if (entry.is_dir) {
      metaCell.textContent = "—";
    } else {
      const parts: string[] = [];
      if (entry.size_bytes != null) {
        parts.push(formatFileSize(entry.size_bytes));
      }
      if (entry.modified_unix) {
        parts.push(new Date(entry.modified_unix * 1000).toLocaleString());
      }
      metaCell.textContent = parts.join(" · ") || "";
    }
    row.appendChild(thumbCell);
    row.appendChild(nameCell);
    row.appendChild(metaCell);
    table.appendChild(row);
  });
  host.appendChild(table);
}
