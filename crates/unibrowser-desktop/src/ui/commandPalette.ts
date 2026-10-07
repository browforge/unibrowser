import { invoke } from "@tauri-apps/api/core";
import { paletteEntryHint } from "./commandTooltip";
import { announceCm } from "./cmA11y";

export interface PaletteEntryDto {
  command_id: string;
  title: string;
  chord: string | null;
  enabled: boolean;
  reason: string | null;
  open_path?: string | null;
}

type PaletteSection = "Recent" | "Commands" | "Open" | "Settings" | "Help";

function sectionForEntry(entry: PaletteEntryDto): PaletteSection {
  const title = entry.title;
  const id = entry.command_id;
  if (title.startsWith("Recent:")) {
    return "Recent";
  }
  if (
    title.startsWith("Open recent:") ||
    title.startsWith("Open folder:") ||
    title.startsWith("Place:")
  ) {
    return "Open";
  }
  if (id.includes("settings") || title.toLowerCase().includes("settings")) {
    return "Settings";
  }
  if (id.includes("help") || title.toLowerCase().includes("help")) {
    return "Help";
  }
  return "Commands";
}

const SECTION_ORDER: PaletteSection[] = ["Recent", "Commands", "Open", "Settings", "Help"];

export interface CommandPaletteBindings {
  palette: HTMLElement;
  query: HTMLInputElement;
  results: HTMLElement;
  onClose: () => void;
  onExecute: (entry: PaletteEntryDto) => void;
}

export function initCommandPalette(bind: CommandPaletteBindings) {
  const { palette, query, results, onClose, onExecute } = bind;
  palette.classList.add("command-palette");

  let activeIndex = -1;
  let flatRows: HTMLButtonElement[] = [];
  let lastFocus: HTMLElement | null = null;

  const setActive = (idx: number) => {
    flatRows.forEach((row, i) => {
      row.classList.toggle("is-active", i === idx);
      if (i === idx) {
        row.scrollIntoView({ block: "nearest" });
      }
    });
    activeIndex = idx;
  };

  const render = (entries: PaletteEntryDto[]) => {
    results.innerHTML = "";
    flatRows = [];
    const grouped = new Map<PaletteSection, PaletteEntryDto[]>();
    for (const e of entries) {
      const sec = sectionForEntry(e);
      const list = grouped.get(sec) ?? [];
      list.push(e);
      grouped.set(sec, list);
    }
    for (const sec of SECTION_ORDER) {
      const list = grouped.get(sec);
      if (!list?.length) {
        continue;
      }
      const label = document.createElement("div");
      label.className = "palette-section-label";
      label.textContent = sec;
      results.appendChild(label);
      for (const entry of list) {
        const row = document.createElement("button");
        row.type = "button";
        row.className = "palette-item-row";
        if (!entry.enabled) {
          row.classList.add("disabled");
          row.setAttribute("aria-disabled", "true");
        }
        const title = document.createElement("span");
        title.className = "palette-item-title";
        title.textContent = paletteEntryHint({
          title: entry.title,
          chord: entry.chord,
          reason: entry.reason,
          enabled: entry.enabled,
        });
        const meta = document.createElement("span");
        meta.className = "palette-item-meta";
        meta.textContent = entry.reason && !entry.enabled ? entry.reason : (entry.reason ?? "");
        const chord = document.createElement("span");
        chord.className = "palette-item-chord";
        chord.textContent = entry.chord ?? "";
        row.appendChild(title);
        row.appendChild(meta);
        row.appendChild(chord);
        row.title = entry.enabled ? "" : (entry.reason ?? "");
        row.onclick = () => {
          if (!entry.enabled) {
            announceCm(
              entry.reason
                ? `${entry.title}, unavailable, ${entry.reason}`
                : `${entry.title}, unavailable`,
            );
            return;
          }
          onExecute(entry);
        };
        results.appendChild(row);
        flatRows.push(row);
      }
    }
    setActive(flatRows.length ? 0 : -1);
  };

  const refresh = async () => {
    const entries = await invoke<PaletteEntryDto[]>("palette_search", {
      query: query.value,
    });
    render(entries);
  };

  query.addEventListener("input", () => {
    void refresh();
  });

  query.addEventListener("keydown", (e) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (!flatRows.length) {
        return;
      }
      setActive(Math.min(activeIndex + 1, flatRows.length - 1));
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (!flatRows.length) {
        return;
      }
      setActive(Math.max(activeIndex - 1, 0));
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      const row = flatRows[activeIndex];
      if (row) {
        row.click();
      }
      return;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  });

  return {
    open() {
      lastFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      void refresh();
    },
    close() {
      if (lastFocus && document.contains(lastFocus)) {
        lastFocus.focus();
      }
      lastFocus = null;
    },
    refresh,
  };
}
