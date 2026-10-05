import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";

export interface WorkspaceQuickHit {
  path: string;
  name: string;
  score: number;
}

let panel: HTMLElement | null = null;
let queryInput: HTMLInputElement | null = null;
let resultsEl: HTMLElement | null = null;
let hintEl: HTMLElement | null = null;
let onOpenPath: ((path: string) => void) | null = null;
let searchToken = 0;

export function initWorkspaceQuickOpen(opts: { onOpenPath: (path: string) => void }) {
  onOpenPath = opts.onOpenPath;
  panel = document.getElementById("workspace-quick-open");
  queryInput = document.getElementById("workspace-quick-open-query") as HTMLInputElement;
  resultsEl = document.getElementById("workspace-quick-open-results");
  hintEl = document.getElementById("workspace-quick-open-hint");
  if (!panel || !queryInput || !resultsEl) {
    return;
  }

  const closeBtn = document.getElementById("workspace-quick-open-close");
  closeBtn?.addEventListener("click", () => hideWorkspaceQuickOpen());

  let debounce: ReturnType<typeof setTimeout> | undefined;
  queryInput.addEventListener("input", () => {
    if (debounce) {
      clearTimeout(debounce);
    }
    debounce = setTimeout(() => void runSearch(), 180);
  });

  panel.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      hideWorkspaceQuickOpen();
    }
  });
}

export function toggleWorkspaceQuickOpen() {
  if (!panel || !queryInput) {
    return;
  }
  const open = panel.classList.contains("hidden");
  if (open) {
    panel.classList.remove("hidden");
    if (hintEl) {
      hintEl.textContent = t("workspaceQuickOpen.hint");
    }
    queryInput.focus();
    queryInput.select();
    void runSearch();
  } else {
    hideWorkspaceQuickOpen();
  }
}

export function hideWorkspaceQuickOpen() {
  if (!panel) {
    return;
  }
  panel.classList.add("hidden");
  searchToken += 1;
}

async function runSearch() {
  if (!queryInput || !resultsEl) {
    return;
  }
  const token = ++searchToken;
  const q = queryInput.value.trim();
  if (!q) {
    resultsEl.innerHTML = `<p class="viewer-note">${t("workspaceQuickOpen.typeToSearch")}</p>`;
    return;
  }
  resultsEl.innerHTML = `<p class="viewer-note">${t("workspaceQuickOpen.searching")}</p>`;
  try {
    const hits = await invoke<WorkspaceQuickHit[]>("search_workspace_quick_files", {
      query: q,
      maxResults: 40,
    });
    if (token !== searchToken) {
      return;
    }
    if (hits.length === 0) {
      resultsEl.innerHTML = "";
      const p = document.createElement("p");
      p.className = "viewer-note";
      p.textContent = t("workspaceQuickOpen.noResults");
      resultsEl.appendChild(p);
      return;
    }
    resultsEl.innerHTML = "";
    const ul = document.createElement("ul");
    ul.className = "filename-search-list";
    for (const hit of hits) {
      const li = document.createElement("li");
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "filename-search-hit";
      btn.textContent = hit.name;
      btn.title = hit.path;
      btn.onclick = () => {
        hideWorkspaceQuickOpen();
        onOpenPath?.(hit.path);
      };
      li.appendChild(btn);
      ul.appendChild(li);
    }
    resultsEl.appendChild(ul);
  } catch (err) {
    if (token !== searchToken) {
      return;
    }
    resultsEl.innerHTML = "";
    const p = document.createElement("p");
    p.className = "viewer-note";
    p.textContent = String(err);
    resultsEl.appendChild(p);
  }
}
