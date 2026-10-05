import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { copyText } from "./clipboard";
import { t } from "./i18n";
import { showToast } from "./toast";
import type { DirEntry, FilesViewDto } from "./filesPanelTypes";

export type { DirEntry, FilesViewDto } from "./filesPanelTypes";

const THUMB_MAX_BYTES = 8_000_000;
const THUMB_MAX_ACTIVE = 24;
const SEARCH_DEBOUNCE_MS = 250;
let thumbLoadCount = 0;

const selectedPaths = new Set<string>();
let lastBrowsePath = "";
let searchFilter = "";
let selectionAnchorIndex = -1;

export interface FilesPanelDeps {
  entries: DirEntry[];
  filesView: FilesViewDto;
  browsePath: string;
  folderFavorites: string[];
  sidePanel: HTMLElement;
  onApplyState: (data: unknown) => void | Promise<void>;
  persistFilesView: (view: FilesViewDto) => Promise<void>;
  onRefresh?: () => Promise<void>;
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

function clearSidePanel(sidePanel: HTMLElement) {
  sidePanel.classList.add("hidden");
  sidePanel.innerHTML = "";
}

function filterEntries(entries: DirEntry[]): DirEntry[] {
  if (!searchFilter) {
    return entries;
  }
  const q = searchFilter.toLowerCase();
  return entries.filter((e) => e.name.toLowerCase().includes(q));
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

function toggleSelection(
  entry: DirEntry,
  index: number,
  visible: DirEntry[],
  opts: { range?: boolean; toggle?: boolean },
) {
  if (opts.range && selectionAnchorIndex >= 0) {
    const from = Math.min(selectionAnchorIndex, index);
    const to = Math.max(selectionAnchorIndex, index);
    for (let i = from; i <= to; i++) {
      selectedPaths.add(visible[i].path);
    }
    return;
  }
  if (opts.toggle) {
    if (selectedPaths.has(entry.path)) {
      selectedPaths.delete(entry.path);
    } else {
      selectedPaths.add(entry.path);
    }
    selectionAnchorIndex = index;
    return;
  }
  selectedPaths.clear();
  selectedPaths.add(entry.path);
  selectionAnchorIndex = index;
}

function showDeleteConfirm(paths: string[], deps: FilesPanelDeps, onDone: () => void) {
  const panel = deps.sidePanel;
  panel.classList.remove("hidden");
  panel.innerHTML = "";

  const title = document.createElement("h3");
  title.className = "files-side-title";
  title.textContent = t("files.deleteTitle");
  panel.appendChild(title);

  const prompt = document.createElement("p");
  prompt.className = "files-side-note";
  prompt.textContent = t("files.deletePrompt");
  panel.appendChild(prompt);

  const ul = document.createElement("ul");
  ul.className = "files-side-list";
  paths.forEach((p) => {
    const li = document.createElement("li");
    li.textContent = p.split("/").filter(Boolean).pop() ?? p;
    li.title = p;
    ul.appendChild(li);
  });
  panel.appendChild(ul);

  const actions = document.createElement("div");
  actions.className = "files-side-actions";

  const cancel = document.createElement("button");
  cancel.type = "button";
  cancel.textContent = t("files.deleteCancel");
  cancel.onclick = () => {
    clearSidePanel(panel);
    onDone();
  };

  const confirm = document.createElement("button");
  confirm.type = "button";
  confirm.className = "files-side-danger";
  confirm.textContent = t("files.deleteConfirm");
  confirm.onclick = () => {
    void (async () => {
      try {
        const data = await invoke("delete_paths", { paths });
        selectedPaths.clear();
        clearSidePanel(panel);
        if (data) {
          await deps.onApplyState(data);
        } else {
          await deps.onRefresh?.();
        }
      } catch (err) {
        showToast(String(err), "error");
      } finally {
        onDone();
      }
    })();
  };

  actions.appendChild(cancel);
  actions.appendChild(confirm);
  panel.appendChild(actions);
}

async function showArchivePeek(
  peekHost: HTMLElement,
  archivePath: string,
  archiveName: string,
  deps: FilesPanelDeps,
) {
  peekHost.classList.remove("hidden");
  peekHost.innerHTML = `<p class="viewer-note">${t("files.peekLoading")}</p>`;
  try {
    const items = await invoke<{ path: string; is_dir: boolean; size: number }[]>(
      "peek_archive_entries",
      { path: archivePath, maxEntries: 120 },
    );
    peekHost.innerHTML = "";
    const head = document.createElement("p");
    head.className = "viewer-note";
    head.textContent = t("files.peekTitle", { name: archiveName });
    peekHost.appendChild(head);
    const hint = document.createElement("p");
    hint.className = "viewer-note files-peek-hint";
    hint.textContent = t("files.peekOpenHint");
    peekHost.appendChild(hint);
    const ul = document.createElement("ul");
    ul.className = "files-peek-list";
    items.forEach((item) => {
      const li = document.createElement("li");
      if (item.is_dir) {
        li.textContent = `[dir] ${item.path}`;
        li.className = "files-peek-dir";
      } else {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "files-peek-entry";
        btn.textContent = `[file] ${item.path} (${formatFileSize(item.size)})`;
        btn.onclick = () => {
          void (async () => {
            try {
              const data = await invoke("open_archive_entry", {
                archivePath,
                entryPath: item.path,
              });
              if (data) {
                await deps.onApplyState(data);
              }
            } catch (err) {
              showToast(String(err), "error");
            }
          })();
        };
        li.appendChild(btn);
      }
      ul.appendChild(li);
    });
    peekHost.appendChild(ul);
  } catch (err) {
    peekHost.textContent = String(err);
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

  if (deps.browsePath !== lastBrowsePath) {
    selectedPaths.clear();
    selectionAnchorIndex = -1;
    lastBrowsePath = deps.browsePath;
    clearSidePanel(deps.sidePanel);
  }

  const visibleEntries = filterEntries(deps.entries);

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
    await invoke("toggle_folder_favorite", { path: deps.browsePath });
    await deps.onRefresh?.();
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

  const searchRow = document.createElement("div");
  searchRow.className = "files-search-row";
  const searchLabel = document.createElement("label");
  searchLabel.className = "files-search-label";
  searchLabel.textContent = t("files.search");
  const searchInput = document.createElement("input");
  searchInput.type = "search";
  searchInput.className = "files-search-input";
  searchInput.placeholder = t("files.searchPlaceholder");
  searchInput.value = searchFilter;
  searchInput.oninput = () => {
    const value = searchInput.value;
    window.clearTimeout((searchInput as unknown as { _debounce?: number })._debounce);
    (searchInput as unknown as { _debounce?: number })._debounce = window.setTimeout(() => {
      searchFilter = value.trim();
      renderFilesPanel(host, deps);
    }, SEARCH_DEBOUNCE_MS);
  };
  searchLabel.appendChild(searchInput);
  searchRow.appendChild(searchLabel);
  host.appendChild(searchRow);

  const bulkBar = document.createElement("div");
  bulkBar.className = "files-bulk-bar" + (selectedPaths.size ? "" : " hidden");
  const bulkLabel = document.createElement("span");
  bulkLabel.textContent = t("files.selectedCount", { count: String(selectedPaths.size) });
  bulkBar.appendChild(bulkLabel);

  const openSel = document.createElement("button");
  openSel.type = "button";
  openSel.textContent = t("files.openSelected");
  openSel.onclick = () => {
    const paths = [...selectedPaths];
    void (async () => {
      for (let i = 0; i < paths.length; i++) {
        if (i === 0) {
          await openPath(paths[i], deps);
        } else {
          await openPathNewTab(paths[i], deps);
        }
      }
    })();
  };
  bulkBar.appendChild(openSel);

  const copySel = document.createElement("button");
  copySel.type = "button";
  copySel.textContent = t("files.copySelectedPaths");
  copySel.onclick = () => {
    void copyText([...selectedPaths].join("\n"), () =>
      showToast(t("status.pathCopied"), "info"),
    );
  };
  bulkBar.appendChild(copySel);

  const delSel = document.createElement("button");
  delSel.type = "button";
  delSel.className = "files-bulk-delete";
  delSel.textContent = t("files.deleteSelected");
  delSel.onclick = () => {
    showDeleteConfirm([...selectedPaths], deps, () => {
      renderFilesPanel(host, deps);
    });
  };
  bulkBar.appendChild(delSel);
  host.appendChild(bulkBar);

  const peekHost = document.createElement("div");
  peekHost.className = "files-archive-peek hidden";
  host.appendChild(peekHost);

  const table = document.createElement("table");
  table.className = "files-table";
  const thead = document.createElement("thead");
  const headRow = document.createElement("tr");
  const selectAllTh = document.createElement("th");
  selectAllTh.className = "files-check-col";
  const selectAll = document.createElement("input");
  selectAll.type = "checkbox";
  selectAll.checked =
    visibleEntries.length > 0 && visibleEntries.every((e) => selectedPaths.has(e.path));
  selectAll.onchange = () => {
    if (selectAll.checked) {
      visibleEntries.forEach((e) => selectedPaths.add(e.path));
    } else {
      visibleEntries.forEach((e) => selectedPaths.delete(e.path));
    }
    renderFilesPanel(host, deps);
  };
  selectAllTh.appendChild(selectAll);
  headRow.appendChild(selectAllTh);
  ["", t("files.sortName"), ""].forEach((label) => {
    const th = document.createElement("th");
    th.textContent = label;
    headRow.appendChild(th);
  });
  thead.appendChild(headRow);
  table.appendChild(thead);

  const tbody = document.createElement("tbody");
  visibleEntries.forEach((entry, index) => {
    const row = document.createElement("tr");
    if (selectedPaths.has(entry.path)) {
      row.classList.add("files-row-selected");
    }
    row.oncontextmenu = (ev) => {
      ev.preventDefault();
      const peek = entry.is_archive
        ? () => void showArchivePeek(peekHost, entry.path, entry.name, deps)
        : undefined;
      showContextMenu(ev.clientX, ev.clientY, entry, deps, peek);
    };

    const checkCell = document.createElement("td");
    checkCell.className = "files-check-col";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = selectedPaths.has(entry.path);
    cb.onclick = (e) => e.stopPropagation();
    cb.onchange = () => {
      if (cb.checked) {
        selectedPaths.add(entry.path);
        selectionAnchorIndex = index;
      } else {
        selectedPaths.delete(entry.path);
      }
      renderFilesPanel(host, deps);
    };
    checkCell.appendChild(cb);
    row.appendChild(checkCell);

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
    link.onclick = (e) => {
      e.stopPropagation();
      void openPath(entry.path, deps);
    };
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
        await invoke("toggle_folder_favorite", { path: entry.path });
        await deps.onRefresh?.();
      };
      nameCell.appendChild(star);
    }

    row.onclick = (e) => {
      const target = e.target as HTMLElement;
      if (target.closest("button, input, a")) {
        return;
      }
      if (e.shiftKey) {
        toggleSelection(entry, index, visibleEntries, { range: true });
      } else if (e.ctrlKey || e.metaKey) {
        toggleSelection(entry, index, visibleEntries, { toggle: true });
      } else {
        toggleSelection(entry, index, visibleEntries, {});
      }
      renderFilesPanel(host, deps);
    };

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
    tbody.appendChild(row);
  });
  table.appendChild(tbody);
  host.appendChild(table);
}
