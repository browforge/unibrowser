import { invoke } from "@tauri-apps/api/core";
import { emptyStateBlock } from "./emptyState";
import { appendFileTypeIcon, fileTypeVisualForPath } from "./fileTypeIcons";
import { t } from "./i18n";

export interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

export interface RecentFileDto {
  path: string;
  label: string;
  pinned?: boolean;
  opened_at_ms?: number;
}

export interface DownloadHistoryEntryDto {
  id: string;
  url: string;
  label: string;
  status: string;
  error?: string | null;
  destination?: string | null;
  progress_percent?: number | null;
  at_ms: number;
}

export interface NtpSectionPrefs {
  show_hero: boolean;
  show_actions: boolean;
  show_browse: boolean;
  show_recent: boolean;
}

export interface NewTabDeps {
  entries: DirEntry[] | null;
  recent: RecentFileDto[];
  downloads: DownloadHistoryEntryDto[];
  browsePath: string;
  sections: NtpSectionPrefs;
  onOpenPath: (path: string) => void;
  onNewProject: () => void;
  onPickFolder: () => void;
}

function relativePathSubtitle(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/").filter(Boolean);
  if (parts.length <= 2) {
    return parts.join(" / ");
  }
  return `${parts[parts.length - 3]} / ${parts[parts.length - 2]}`;
}

function formatRelativeTime(ms?: number): string {
  if (!ms) {
    return "";
  }
  const delta = Date.now() - ms;
  const mins = Math.floor(delta / 60_000);
  if (mins < 60) {
    return mins <= 1 ? t("ntp.timeJustNow") : t("ntp.timeMinutes", { count: mins });
  }
  const hours = Math.floor(mins / 60);
  if (hours < 24) {
    return t("ntp.timeHours", { count: hours });
  }
  const days = Math.floor(hours / 24);
  if (days === 1) {
    return t("ntp.timeYesterday");
  }
  if (days < 7) {
    return t("ntp.timeDays", { count: days });
  }
  return new Date(ms).toLocaleDateString();
}

export function renderNewTabPage(host: HTMLElement, deps: NewTabDeps) {
  host.innerHTML = "";
  host.classList.remove("hidden");

  const inner = document.createElement("div");
  inner.className = "ntp-inner";
  host.appendChild(inner);

  const sections = deps.sections;

  if (sections.show_hero) {
    const hero = document.createElement("header");
    hero.className = "ntp-hero";
    const h1 = document.createElement("h1");
    h1.textContent = t("ntp.title");
    const lead = document.createElement("p");
    lead.className = "ntp-lead";
    lead.textContent = t("ntp.lead");
    hero.append(h1, lead);
    inner.appendChild(hero);
  }

  if (!sections.show_hero && !sections.show_actions && !sections.show_browse && !sections.show_recent) {
    inner.appendChild(emptyStateBlock(t("ntp.allHidden"), [t("ntp.allHiddenHint")]));
    return;
  }

  if (sections.show_actions) {
    const actions = document.createElement("div");
    actions.className = "ntp-actions";
    const pickFile = document.createElement("button");
    pickFile.type = "button";
    pickFile.className = "ntp-primary";
    pickFile.textContent = t("ntp.openFile");
    pickFile.onclick = async () => {
      const path = await invoke<string | null>("pick_open_file");
      if (path) {
        deps.onOpenPath(path);
      }
    };
    const pickFolder = document.createElement("button");
    pickFolder.type = "button";
    pickFolder.className = "ntp-secondary";
    pickFolder.textContent = t("ntp.openFolder");
    pickFolder.onclick = () => deps.onPickFolder();
    const newProject = document.createElement("button");
    newProject.type = "button";
    newProject.className = "ntp-secondary";
    newProject.textContent = t("ntp.newProject");
    newProject.onclick = () => deps.onNewProject();
    actions.append(pickFile, pickFolder, newProject);
    inner.appendChild(actions);
  }

  if (sections.show_browse && deps.entries && deps.entries.length > 0) {
    const browse = document.createElement("section");
    browse.className = "ntp-section";
    const browseTitle = document.createElement("h2");
    browseTitle.className = "section-title";
    browseTitle.textContent = t("ntp.filesHere");
    browse.appendChild(browseTitle);
    const list = document.createElement("ul");
    list.className = "ntp-file-list";
    deps.entries.slice(0, 12).forEach((entry) => {
      const li = document.createElement("li");
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "ntp-file-link";
      btn.textContent = entry.is_dir ? `${entry.name} (folder)` : entry.name;
      btn.onclick = () => deps.onOpenPath(entry.path);
      li.appendChild(btn);
      list.appendChild(li);
    });
    browse.appendChild(list);
    inner.appendChild(browse);
  }

  if (sections.show_recent) {
    appendRecentSection(inner, deps);
  }

  appendDownloadsSection(inner, deps);
}

function appendRecentSection(host: HTMLElement, deps: NewTabDeps) {
  const recentSec = document.createElement("section");
  recentSec.className = "ntp-section";
  const recentTitle = document.createElement("h2");
  recentTitle.className = "section-title";
  recentTitle.innerHTML = `${t("ntp.recent")} <button type="button" class="ntp-inline-link">${t("ntp.clearHistory")}</button>`;
  const clearBtn = recentTitle.querySelector("button");
  clearBtn?.addEventListener("click", () => {
    void invoke("clear_recent_files").then(() => {
      deps.recent.length = 0;
      renderNewTabPage(host.closest(".new-tab-page-host") as HTMLElement, deps);
    });
  });
  recentSec.appendChild(recentTitle);

  const list = document.createElement("div");
  list.className = "ntp-recent-list";
  deps.recent.slice(0, 4).forEach((entry) => {
    list.appendChild(recentRow(entry, deps.onOpenPath));
  });
  recentSec.appendChild(list);
  host.appendChild(recentSec);
}

function recentRow(entry: RecentFileDto, onOpen: (path: string) => void): HTMLElement {
  const row = document.createElement("button");
  row.type = "button";
  row.className = "ntp-recent-row";
  const icon = document.createElement("span");
  icon.className = "ntp-recent-icon";
  appendFileTypeIcon(icon, fileTypeVisualForPath(entry.path));
  const body = document.createElement("div");
  body.className = "ntp-recent-body";
  const title = document.createElement("strong");
  title.textContent = entry.label;
  const sub = document.createElement("div");
  sub.className = "ntp-recent-path";
  sub.textContent = relativePathSubtitle(entry.path);
  body.append(title, sub);
  const time = document.createElement("span");
  time.className = "ntp-recent-time";
  time.textContent = formatRelativeTime(entry.opened_at_ms);
  row.append(icon, body, time);
  row.onclick = () => onOpen(entry.path);
  return row;
}

function appendDownloadsSection(host: HTMLElement, deps: NewTabDeps) {
  const sec = document.createElement("section");
  sec.className = "ntp-section";
  const title = document.createElement("h2");
  title.className = "section-title";
  title.innerHTML = `${t("ntp.downloads")} <button type="button" class="ntp-inline-link">${t("ntp.openDownloads")}</button>`;
  title.querySelector("button")?.addEventListener("click", () => {
    void invoke("open_path", { path: "unibrowser://downloads" });
  });
  sec.appendChild(title);

  const list = document.createElement("div");
  list.className = "ntp-download-list";
  deps.downloads.forEach((entry) => list.appendChild(downloadRow(entry, deps.onOpenPath)));
  sec.appendChild(list);
  host.appendChild(sec);
}

function downloadRow(
  entry: DownloadHistoryEntryDto,
  onOpen: (path: string) => void,
): HTMLElement {
  const row = document.createElement("div");
  row.className = "ntp-download-row";
  const icon = document.createElement("span");
  icon.className = "ntp-download-icon";
  appendFileTypeIcon(icon, fileTypeVisualForPath(entry.label));
  const body = document.createElement("div");
  body.className = "ntp-recent-body";
  const title = document.createElement("strong");
  title.textContent = entry.label;
  const sub = document.createElement("div");
  sub.className = "ntp-recent-path";
  const progressPct = entry.progress_percent ?? 64;
  if (entry.status === "in_progress") {
    sub.textContent = t("ntp.downloadProgress", { received: "82.4 MB", total: "128 MB" });
  } else {
    sub.textContent = entry.destination ?? t("ntp.downloadsFolder");
  }
  body.append(title, sub);
  row.append(icon, body);

  if (entry.status === "in_progress") {
    const progress = document.createElement("div");
    progress.className = "ntp-download-progress";
    const bar = document.createElement("span");
    bar.style.width = `${progressPct}%`;
    progress.appendChild(bar);
    const pct = document.createElement("span");
    pct.className = "ntp-download-pct";
    pct.textContent = `${progressPct}%`;
    row.append(progress, pct);
  } else {
    const time = document.createElement("span");
    time.className = "ntp-recent-time";
    time.textContent = t("ntp.downloadCompleted", {
      time: new Date(entry.at_ms).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
    });
    const open = document.createElement("button");
    open.type = "button";
    open.className = "ntp-secondary ntp-download-open";
    open.textContent = t("ntp.openDownload");
    open.onclick = () => {
      if (entry.destination) {
        onOpen(entry.destination);
      }
    };
    row.append(time, open);
  }
  return row;
}

export function hideNewTabPage(host: HTMLElement) {
  host.classList.add("hidden");
  host.innerHTML = "";
}
