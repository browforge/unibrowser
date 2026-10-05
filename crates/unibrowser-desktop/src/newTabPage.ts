import { invoke } from "@tauri-apps/api/core";
import { emptyStateBlock } from "./emptyState";
import { t } from "./i18n";

export interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

export interface RecentFileDto {
  path: string;
  label: string;
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
  browsePath: string;
  sections: NtpSectionPrefs;
  onOpenPath: (path: string) => void;
  onNewProject: () => void;
  onPickFolder: () => void;
}

export function renderNewTabPage(host: HTMLElement, deps: NewTabDeps) {
  host.innerHTML = "";
  host.classList.remove("hidden");

  const sections = deps.sections;

  if (sections.show_hero) {
    const hero = document.createElement("section");
    hero.className = "ntp-hero";
    const h1 = document.createElement("h1");
    h1.textContent = t("ntp.title");
    const lead = document.createElement("p");
    lead.className = "ntp-lead";
    lead.textContent = t("ntp.lead");
    hero.appendChild(h1);
    hero.appendChild(lead);
    host.appendChild(hero);
  }

  if (!sections.show_hero && !sections.show_actions && !sections.show_browse && !sections.show_recent) {
    host.appendChild(emptyStateBlock(t("ntp.allHidden"), [t("ntp.allHiddenHint")]));
    return;
  }

  const actions = document.createElement("div");
  actions.className = "ntp-actions";
  const pickFile = document.createElement("button");
  pickFile.type = "button";
  pickFile.className = "ntp-primary";
  pickFile.textContent = t("ntp.chooseFile");
  pickFile.onclick = async () => {
    const path = await invoke<string | null>("pick_open_file");
    if (path) {
      deps.onOpenPath(path);
    }
  };
  const pickFolder = document.createElement("button");
  pickFolder.type = "button";
  pickFolder.textContent = t("ntp.chooseFolder");
  pickFolder.onclick = () => deps.onPickFolder();
  const newProject = document.createElement("button");
  newProject.type = "button";
  newProject.textContent = t("ntp.newProject");
  newProject.onclick = () => deps.onNewProject();
  actions.appendChild(pickFile);
  actions.appendChild(pickFolder);
  actions.appendChild(newProject);
  if (sections.show_actions) {
    host.appendChild(actions);
  }

  if (!sections.show_browse) {
    if (sections.show_recent && deps.recent.length) {
      appendRecentSection(host, deps);
    }
    return;
  }

  const browse = document.createElement("section");
  browse.className = "ntp-section";
  const browseTitle = document.createElement("h2");
  browseTitle.textContent = t("ntp.filesHere");
  const browsePath = document.createElement("p");
  browsePath.className = "ntp-path";
  browsePath.textContent = deps.browsePath;
  browse.appendChild(browseTitle);
  browse.appendChild(browsePath);
  const list = document.createElement("ul");
  list.className = "ntp-file-list";
  const entries = deps.entries ?? [];
  if (!entries.length) {
    const emptyLi = document.createElement("li");
    emptyLi.className = "ntp-empty-wrap";
    emptyLi.appendChild(emptyStateBlock(t("ntp.emptyFolder"), []));
    list.appendChild(emptyLi);
  } else {
    entries.slice(0, 24).forEach((entry) => {
      const li = document.createElement("li");
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "ntp-file-link";
      btn.textContent = entry.is_dir ? `${entry.name} (folder)` : entry.name;
      btn.onclick = () => deps.onOpenPath(entry.path);
      li.appendChild(btn);
      list.appendChild(li);
    });
    if (entries.length > 24) {
      const more = document.createElement("li");
      more.className = "ntp-empty";
      more.textContent = t("ntp.moreFiles", { count: entries.length - 24 });
      list.appendChild(more);
    }
  }
  browse.appendChild(list);
  host.appendChild(browse);

  if (sections.show_recent && deps.recent.length) {
    appendRecentSection(host, deps);
  }
}

function appendRecentSection(host: HTMLElement, deps: NewTabDeps) {
  const recentSec = document.createElement("section");
  recentSec.className = "ntp-section";
  const recentTitle = document.createElement("h2");
  recentTitle.textContent = t("ntp.recent");
  recentSec.appendChild(recentTitle);
  const recentList = document.createElement("ul");
  recentList.className = "ntp-file-list";
  deps.recent.slice(0, 8).forEach((entry) => {
    const li = document.createElement("li");
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "ntp-file-link";
    btn.textContent = entry.label;
    btn.title = entry.path;
    btn.onclick = () => deps.onOpenPath(entry.path);
    li.appendChild(btn);
    recentList.appendChild(li);
  });
  recentSec.appendChild(recentList);
  host.appendChild(recentSec);
}

export function hideNewTabPage(host: HTMLElement) {
  host.classList.add("hidden");
  host.innerHTML = "";
}
