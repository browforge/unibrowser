/** Shared Settings workbench layout helpers (UI-21 mockup parity). */

export function createSettingsFieldRow(
  label: string,
  helper: string | null,
  control: HTMLElement,
): HTMLElement {
  const row = document.createElement("div");
  row.className = "settings-field";
  const copy = document.createElement("div");
  copy.className = "settings-field-copy";
  const title = document.createElement("div");
  title.className = "settings-field-label";
  title.textContent = label;
  copy.appendChild(title);
  if (helper) {
    const hint = document.createElement("p");
    hint.className = "settings-field-helper";
    hint.textContent = helper;
    copy.appendChild(hint);
  }
  const slot = document.createElement("div");
  slot.className = "settings-field-control";
  slot.appendChild(control);
  row.append(copy, slot);
  return row;
}

function syncSwitchA11y(input: HTMLInputElement): void {
  input.setAttribute("aria-checked", input.checked ? "true" : "false");
}

export function createSettingsToggle(
  id: string,
  checked: boolean,
  onChange: () => void,
): HTMLInputElement {
  const input = document.createElement("input");
  input.type = "checkbox";
  input.id = id;
  input.className = "ub-switch settings-toggle-input";
  input.setAttribute("role", "switch");
  input.checked = checked;
  syncSwitchA11y(input);
  input.onchange = () => {
    syncSwitchA11y(input);
    onChange();
  };
  return input;
}

export interface SettingsCategoryMeta {
  id: string;
  label: string;
  subtitle: string;
}

export function applySettingsCategoryVisibility(
  root: HTMLElement,
  categoryId: string,
  pageTitle: HTMLElement,
  pageSubtitle: HTMLElement,
  meta: SettingsCategoryMeta,
  developerCallout: HTMLElement | null,
) {
  root.querySelectorAll<HTMLElement>("[data-settings-category]").forEach((node) => {
    const match = node.dataset.settingsCategory === categoryId;
    node.classList.toggle("settings-category-hidden", !match);
  });
  pageTitle.textContent = meta.label;
  pageSubtitle.textContent = meta.subtitle;
  if (developerCallout) {
    developerCallout.classList.toggle("hidden", categoryId !== "general");
  }
}
