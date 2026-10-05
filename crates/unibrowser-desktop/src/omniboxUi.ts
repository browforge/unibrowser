import { invoke } from "@tauri-apps/api/core";

export interface OmniboxSuggestion {
  label: string;
  value: string;
  kind: string;
  score: number;
}

export interface OmniboxAutocompleteHandle {
  hide: () => void;
  applyActiveSuggestion: () => boolean;
  hasVisibleSuggestions: () => boolean;
}

export function bindOmniboxAutocomplete(
  input: HTMLInputElement,
  listEl: HTMLElement,
  onPick: (value: string) => void,
): OmniboxAutocompleteHandle {
  let active = -1;
  let suggestions: OmniboxSuggestion[] = [];

  const hide = () => {
    listEl.classList.add("hidden");
    listEl.innerHTML = "";
    active = -1;
    suggestions = [];
  };

  const hasVisibleSuggestions = () =>
    suggestions.length > 0 && !listEl.classList.contains("hidden");

  const render = () => {
    listEl.innerHTML = "";
    if (!suggestions.length) {
      hide();
      return;
    }
    listEl.classList.remove("hidden");
    suggestions.forEach((s, i) => {
      const row = document.createElement("button");
      row.type = "button";
      row.className = "omnibox-suggestion" + (i === active ? " active" : "");
      const kind =
        s.kind === "place"
          ? "Place"
          : s.kind === "recent"
            ? "Recent"
            : s.kind === "url"
              ? "URL"
              : s.kind === "dir"
                ? "Folder"
                : "File";
      row.textContent = `${kind}: ${s.label}`;
      row.title = s.value;
      row.onclick = () => {
        input.value = s.value;
        hide();
        onPick(s.value);
      };
      listEl.appendChild(row);
    });
  };

  const refresh = async () => {
    suggestions = await invoke<OmniboxSuggestion[]>("omnibox_completions", {
      query: input.value,
    });
    active = suggestions.length ? 0 : -1;
    render();
  };

  const applyActiveSuggestion = (): boolean => {
    if (!hasVisibleSuggestions() || active < 0) {
      return false;
    }
    input.value = suggestions[active].value;
    hide();
    return true;
  };

  input.addEventListener("input", () => {
    void refresh();
  });
  input.addEventListener("focus", () => {
    void refresh();
  });
  input.addEventListener("blur", () => {
    window.setTimeout(hide, 150);
  });
  input.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      hide();
      return;
    }
    if (!hasVisibleSuggestions()) {
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      active = (active + 1) % suggestions.length;
      render();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      active = (active - 1 + suggestions.length) % suggestions.length;
      render();
    } else if (e.key === "Tab" && active >= 0) {
      e.preventDefault();
      applyActiveSuggestion();
    }
  });

  return { hide, applyActiveSuggestion, hasVisibleSuggestions };
}
