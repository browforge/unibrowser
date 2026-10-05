import { invoke } from "@tauri-apps/api/core";
import enNz from "./i18n/en-NZ.json";
import { t } from "./i18n";

type I18nKey = keyof typeof enNz;

interface KeyboardShortcutRow {
  command_id: string;
  chord: string;
  title: string;
}

const STATIC_HINTS = [
  { chord: "Ctrl+Shift+P", labelKey: "keys.palette" as I18nKey },
  { chord: "Ctrl+Shift+A", labelKey: "keys.goToTab" as I18nKey },
  { chord: "Ctrl+T", labelKey: "keys.newTab" as I18nKey },
  { chord: "Ctrl+W", labelKey: "keys.closeTab" as I18nKey },
  { chord: "Ctrl+L", labelKey: "keys.focusOmnibox" as I18nKey },
  { chord: "Ctrl+Tab", labelKey: "keys.nextTab" as I18nKey },
  { chord: "Ctrl+Shift+Tab", labelKey: "keys.prevTab" as I18nKey },
];

export async function renderKeys(root: HTMLElement) {
  root.innerHTML = "";
  const intro = document.createElement("p");
  intro.className = "viewer-note";
  intro.textContent = t("keys.lead");
  root.appendChild(intro);

  const staticSec = document.createElement("section");
  staticSec.className = "settings-section";
  const staticHeading = document.createElement("h3");
  staticHeading.textContent = t("keys.common");
  staticSec.appendChild(staticHeading);
  const staticTable = document.createElement("table");
  staticTable.className = "keys-table";
  STATIC_HINTS.forEach((row) => {
    const tr = document.createElement("tr");
    const tdChord = document.createElement("td");
    tdChord.className = "keys-chord";
    tdChord.textContent = row.chord;
    const tdLabel = document.createElement("td");
    tdLabel.textContent = t(row.labelKey);
    tr.appendChild(tdChord);
    tr.appendChild(tdLabel);
    staticTable.appendChild(tr);
  });
  staticSec.appendChild(staticTable);
  root.appendChild(staticSec);

  const boundSec = document.createElement("section");
  boundSec.className = "settings-section";
  const boundHeading = document.createElement("h3");
  boundHeading.textContent = t("keys.bound");
  boundSec.appendChild(boundHeading);
  const boundTable = document.createElement("table");
  boundTable.className = "keys-table";
  try {
    const bindings = await invoke<KeyboardShortcutRow[]>("get_keyboard_shortcuts");
    bindings.forEach((row) => {
        const tr = document.createElement("tr");
        const tdChord = document.createElement("td");
        tdChord.className = "keys-chord";
        tdChord.textContent = row.chord;
        const tdLabel = document.createElement("td");
        tdLabel.textContent = row.title;
        tr.appendChild(tdChord);
        tr.appendChild(tdLabel);
        boundTable.appendChild(tr);
      });
  } catch {
    const err = document.createElement("p");
    err.className = "viewer-note";
    err.textContent = t("keys.loadError");
    boundSec.appendChild(err);
  }
  boundSec.appendChild(boundTable);
  root.appendChild(boundSec);
}
