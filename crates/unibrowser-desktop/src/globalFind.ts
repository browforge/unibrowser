import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { showToast } from "./toast";

export interface FilenameHit {
  path: string;
  name: string;
  line?: number | null;
  match_kind?: string;
}

let panel: HTMLElement | null = null;
let queryInput: HTMLInputElement | null = null;
let resultsEl: HTMLElement | null = null;
let rootInput: HTMLInputElement | null = null;
let onOpenPath: ((path: string, line?: number) => void) | null = null;
let searchToken = 0;

export function initGlobalFind(opts: {
  onOpenPath: (path: string, line?: number) => void;
  defaultRoot: string;
}) {
  onOpenPath = opts.onOpenPath;
  panel = document.getElementById("filename-search");
  queryInput = document.getElementById("filename-search-query") as HTMLInputElement;
  rootInput = document.getElementById("filename-search-root") as HTMLInputElement;
  resultsEl = document.getElementById("filename-search-results");
  if (!panel || !queryInput || !rootInput || !resultsEl) {
    return;
  }
  rootInput.value = opts.defaultRoot;

  const cancelBtn = document.getElementById("filename-search-cancel");
  cancelBtn?.addEventListener("click", () => {
    void invoke("cancel_filename_search");
    searchToken += 1;
  });

  const closeBtn = document.getElementById("filename-search-close");
  closeBtn?.addEventListener("click", () => hideGlobalFind());

  let debounce: ReturnType<typeof setTimeout> | undefined;
  queryInput.addEventListener("input", () => {
    if (debounce) {
      clearTimeout(debounce);
    }
    debounce = setTimeout(() => void runSearch(), 280);
  });

  panel.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      hideGlobalFind();
    }
  });
}

export function toggleGlobalFind(defaultRoot?: string) {
  if (!panel || !queryInput) {
    return;
  }
  const open = panel.classList.contains("hidden");
  if (open) {
    panel.classList.remove("hidden");
    if (defaultRoot && rootInput) {
      rootInput.value = defaultRoot;
    }
    const limits = document.getElementById("filename-search-limits");
    if (limits) {
      limits.textContent = t("globalFind.limitsNote");
    }
    queryInput.focus();
    queryInput.select();
  } else {
    hideGlobalFind();
  }
}

export function hideGlobalFind() {
  if (!panel) {
    return;
  }
  panel.classList.add("hidden");
  void invoke("cancel_filename_search");
  searchToken += 1;
}

async function runSearch() {
  if (!queryInput || !rootInput || !resultsEl) {
    return;
  }
  const q = queryInput.value.trim();
  const root = rootInput.value.trim();
  resultsEl.innerHTML = "";
  if (!q || !root) {
    return;
  }
  const token = ++searchToken;
  resultsEl.innerHTML = `<p class="viewer-note">${t("globalFind.searching")}</p>`;
  try {
    const hits = await invoke<FilenameHit[]>("search_filenames", {
      root,
      query: q,
      maxDepth: 8,
      maxResults: 200,
    });
    if (token !== searchToken) {
      return;
    }
    resultsEl.innerHTML = "";
    if (!hits.length) {
      resultsEl.appendChild(
        Object.assign(document.createElement("p"), {
          className: "viewer-note",
          textContent: t("globalFind.noResults"),
        }),
      );
      return;
    }
    const ul = document.createElement("ul");
    ul.className = "filename-search-list";
    hits.forEach((hit) => {
      const li = document.createElement("li");
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "filename-search-hit";
      const lineLabel = hit.line ? `:${hit.line}` : "";
      btn.textContent = `${hit.name}${lineLabel}`;
      btn.title = `${hit.path}${hit.line ? ` (line ${hit.line})` : ""}`;
      btn.onclick = () => {
        hideGlobalFind();
        onOpenPath?.(hit.path, hit.line ?? undefined);
      };
      li.appendChild(btn);
      ul.appendChild(li);
    });
    resultsEl.appendChild(ul);
  } catch (err) {
    if (token !== searchToken) {
      return;
    }
    showToast(String(err), "error");
    resultsEl.textContent = String(err);
  }
}
