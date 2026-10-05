import { invoke } from "@tauri-apps/api/core";

export interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

export interface RecentFileDto {
  path: string;
  label: string;
}

export interface NewTabDeps {
  entries: DirEntry[] | null;
  recent: RecentFileDto[];
  browsePath: string;
  onOpenPath: (path: string) => void;
  onNewProject: () => void;
  onPickFolder: () => void;
}

export function renderNewTabPage(host: HTMLElement, deps: NewTabDeps) {
  host.innerHTML = "";
  host.classList.remove("hidden");

  const hero = document.createElement("section");
  hero.className = "ntp-hero";
  const h1 = document.createElement("h1");
  h1.textContent = "What would you like to open?";
  const lead = document.createElement("p");
  lead.className = "ntp-lead";
  lead.textContent = "Pick a file, a folder, something you opened before, or start a small project.";
  hero.appendChild(h1);
  hero.appendChild(lead);
  host.appendChild(hero);

  const actions = document.createElement("div");
  actions.className = "ntp-actions";
  const pickFile = document.createElement("button");
  pickFile.type = "button";
  pickFile.className = "ntp-primary";
  pickFile.textContent = "Choose a file…";
  pickFile.onclick = async () => {
    const path = await invoke<string | null>("pick_open_file");
    if (path) {
      deps.onOpenPath(path);
    }
  };
  const pickFolder = document.createElement("button");
  pickFolder.type = "button";
  pickFolder.textContent = "Choose a folder…";
  pickFolder.onclick = () => deps.onPickFolder();
  const newProject = document.createElement("button");
  newProject.type = "button";
  newProject.textContent = "Start a new project";
  newProject.onclick = () => deps.onNewProject();
  actions.appendChild(pickFile);
  actions.appendChild(pickFolder);
  actions.appendChild(newProject);
  host.appendChild(actions);

  const browse = document.createElement("section");
  browse.className = "ntp-section";
  const browseTitle = document.createElement("h2");
  browseTitle.textContent = "Files here";
  const browsePath = document.createElement("p");
  browsePath.className = "ntp-path";
  browsePath.textContent = deps.browsePath;
  browse.appendChild(browseTitle);
  browse.appendChild(browsePath);
  const list = document.createElement("ul");
  list.className = "ntp-file-list";
  const entries = deps.entries ?? [];
  if (!entries.length) {
    const empty = document.createElement("li");
    empty.className = "ntp-empty";
    empty.textContent = "This folder is empty.";
    list.appendChild(empty);
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
      more.textContent = `… and ${entries.length - 24} more. Use “Choose a folder…” to browse elsewhere.`;
      list.appendChild(more);
    }
  }
  browse.appendChild(list);
  host.appendChild(browse);

  if (deps.recent.length) {
    const recentSec = document.createElement("section");
    recentSec.className = "ntp-section";
    const recentTitle = document.createElement("h2");
    recentTitle.textContent = "Opened recently";
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
}

export function hideNewTabPage(host: HTMLElement) {
  host.classList.add("hidden");
  host.innerHTML = "";
}
