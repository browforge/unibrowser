import { invoke } from "@tauri-apps/api/core";

export interface OmniboxSuggestion {
  label: string;
  value: string;
  kind: string;
  score: number;
}

export function bindOmniboxAutocomplete(
  input: HTMLInputElement,
  listEl: HTMLElement,
  onPick: (value: string) => void,
) {
  let active = -1;
  let suggestions: OmniboxSuggestion[] = [];

  const hide = () => {
    listEl.classList.add("hidden");
    listEl.innerHTML = "";
    active = -1;
    suggestions = [];
  };

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
      row.textContent = s.label;
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
    if (!suggestions.length || listEl.classList.contains("hidden")) {
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
      input.value = suggestions[active].value;
      hide();
    }
  });

  return { hide };
}
