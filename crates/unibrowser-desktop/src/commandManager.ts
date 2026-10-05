import { invoke } from "@tauri-apps/api/core";
import { bindRovingTablist, setTabA11y } from "./rovingTabs";

export interface UiCommandDto {
  command_id: string;
  title: string;
  icon: string;
  enabled: boolean;
  reason: string | null;
}

export interface ManagerTabDto {
  id: string;
  label: string;
  phase: string;
  selected: boolean;
  commands: UiCommandDto[];
}

export interface FloatingBarDto {
  id: string;
  label: string;
  anchor: string;
  x: number;
  y: number;
  commands: UiCommandDto[];
}

export interface CommandManagerDto {
  auto_context: string;
  selected_tab: string;
  context_pinned: boolean;
  tabs: ManagerTabDto[];
  floating_bars: FloatingBarDto[];
}

export interface CommandManagerFile {
  selected_tab: string;
  context_pin?: string | null;
  tabs: {
    id: string;
    label: string;
    phase: string;
    commands: { command_id: string }[];
  }[];
  floating_bars: {
    id: string;
    label: string;
    anchor: string;
    x: number;
    y: number;
    commands: { command_id: string }[];
  }[];
}

type RunCommand = (commandId: string) => Promise<void>;

let lastSnapshot: CommandManagerDto | null = null;
let dragCommandId: string | null = null;
let cmTabButtons: HTMLButtonElement[] = [];

function renderCommandButtons(
  container: HTMLElement,
  commands: UiCommandDto[],
  onRun: RunCommand,
  draggable: boolean,
) {
  container.innerHTML = "";
  commands.forEach((cmd) => {
    const btn = document.createElement("button");
    btn.type = "button";
    const icon = document.createElement("span");
    icon.className = `cm-cmd-icon cm-icon-${cmd.icon || "extension"}`;
    icon.setAttribute("aria-hidden", "true");
    btn.appendChild(icon);
    const label = document.createElement("span");
    label.className = "cm-cmd-label";
    label.textContent = cmd.title;
    btn.appendChild(label);
    btn.title = cmd.reason ?? cmd.command_id;
    btn.disabled = !cmd.enabled;
    btn.className = cmd.enabled ? "ui-command" : "ui-command disabled";
    btn.dataset.commandId = cmd.command_id;
    if (draggable) {
      btn.draggable = true;
      btn.addEventListener("dragstart", (e) => {
        dragCommandId = cmd.command_id;
        e.dataTransfer?.setData("text/plain", cmd.command_id);
      });
      btn.addEventListener("dragover", (e) => {
        e.preventDefault();
      });
      btn.addEventListener("drop", (e) => {
        e.preventDefault();
        if (!dragCommandId || !lastSnapshot) {
          return;
        }
        const targetId = cmd.command_id;
        reorderActiveTabCommands(dragCommandId, targetId).catch(() => {});
        dragCommandId = null;
      });
    }
    btn.onclick = () => onRun(cmd.command_id);
    container.appendChild(btn);
  });
}

async function addCommandToFloatingBar(
  commandId: string,
  barId?: string,
  position?: { x: number; y: number },
) {
  if (!lastSnapshot) {
    return;
  }
  const file = snapshotToFile(lastSnapshot);
  if (barId) {
    const bar = file.floating_bars.find((b) => b.id === barId);
    if (bar && !bar.commands.some((c) => c.command_id === commandId)) {
      bar.commands.push({ command_id: commandId });
    }
  } else {
    const id = `float-${Date.now()}`;
    file.floating_bars.push({
      id,
      label: "Custom",
      anchor: "floating",
      x: position?.x ?? 96,
      y: position?.y ?? 96,
      commands: [{ command_id: commandId }],
    });
  }
  await invoke("save_command_manager_layout", { layout: file });
}

async function reorderActiveTabCommands(fromId: string, toId: string) {
  if (!lastSnapshot) {
    return;
  }
  const tab = lastSnapshot.tabs.find((t) => t.selected);
  if (!tab) {
    return;
  }
  const ids = tab.commands.map((c) => c.command_id);
  const from = ids.indexOf(fromId);
  const to = ids.indexOf(toId);
  if (from < 0 || to < 0 || from === to) {
    return;
  }
  ids.splice(from, 1);
  ids.splice(to, 0, fromId);
  const file = snapshotToFile(lastSnapshot, ids);
  await invoke("save_command_manager_layout", { layout: file });
}

function contextLabel(id: string, tabs: ManagerTabDto[]): string {
  const tab = tabs.find((t) => t.id === id);
  return tab?.label ?? id;
}

function snapshotToFile(snapshot: CommandManagerDto, activeTabCommandIds?: string[]): CommandManagerFile {
  return {
    selected_tab: snapshot.selected_tab,
    context_pin: snapshot.context_pinned ? snapshot.selected_tab : null,
    tabs: snapshot.tabs.map((tab) => ({
      id: tab.id,
      label: tab.label,
      phase: tab.phase,
      commands: (tab.selected && activeTabCommandIds
        ? activeTabCommandIds
        : tab.commands.map((c) => c.command_id)
      ).map((id) => ({ command_id: id })),
    })),
    floating_bars: snapshot.floating_bars.map((bar) => ({
      id: bar.id,
      label: bar.label,
      anchor: bar.anchor,
      x: bar.x,
      y: bar.y,
      commands: bar.commands.map((c) => ({ command_id: c.command_id })),
    })),
  };
}

function updatePinControl(pinEl: HTMLElement, snapshot: CommandManagerDto) {
  pinEl.classList.toggle("pinned", snapshot.context_pinned);
  pinEl.classList.toggle("auto-follow", !snapshot.context_pinned);
  if (snapshot.context_pinned) {
    pinEl.textContent = "Follow context";
    pinEl.title =
      `Pinned to ${contextLabel(snapshot.selected_tab, snapshot.tabs)}. Click to auto-follow document context (${contextLabel(snapshot.auto_context, snapshot.tabs)}).`;
  } else {
    pinEl.textContent = "Pin tab";
    pinEl.title =
      `Auto-following ${contextLabel(snapshot.auto_context, snapshot.tabs)}. Click to keep the current tab while you switch documents.`;
  }
}

export const commandManager = {
  async refresh(
    tabsEl: HTMLElement,
    commandsEl: HTMLElement,
    floatingEl: HTMLElement,
    pinEl: HTMLElement,
    onRun: RunCommand,
  ) {
    const snapshot = await invoke<CommandManagerDto>("get_command_manager");
    lastSnapshot = snapshot;
    updatePinControl(pinEl, snapshot);
    tabsEl.innerHTML = "";
    cmTabButtons = [];
    snapshot.tabs.forEach((tab) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className =
        "cm-tab" +
        (tab.selected ? " active" : "") +
        (tab.phase === "planned" ? " planned" : "");
      setTabA11y(btn, tab.selected, `cm-tab-${tab.id}`, "command-manager-commands");
      const autoMatch =
        !snapshot.context_pinned && tab.id === snapshot.auto_context;
      btn.textContent = autoMatch ? `${tab.label} *` : tab.label;
      btn.title =
        tab.phase === "planned"
          ? `${tab.label} (coming soon)`
          : autoMatch
            ? `${tab.label} (matches document context)`
            : tab.selected && snapshot.context_pinned
              ? `${tab.label} (pinned)`
              : tab.label;
      btn.onclick = async () => {
        await invoke("select_command_manager_tab", { tabId: tab.id });
        await commandManager.refresh(tabsEl, commandsEl, floatingEl, pinEl, onRun);
      };
      tabsEl.appendChild(btn);
      cmTabButtons.push(btn);
    });
    if (!tabsEl.dataset.rovingBound) {
      tabsEl.dataset.rovingBound = "1";
      bindRovingTablist(tabsEl, () => cmTabButtons);
    }
    commandsEl.id = "command-manager-commands";
    commandsEl.setAttribute("role", "tabpanel");
    const active = snapshot.tabs.find((t) => t.selected) ?? snapshot.tabs[0];
    if (active) {
      renderCommandButtons(commandsEl, active.commands, onRun, true);
    } else {
      commandsEl.innerHTML = "";
    }
    floatingEl.innerHTML = "";
    floatingEl.classList.add("floating-toolbar-drop-host");
    floatingEl.ondragover = (e) => {
      if (dragCommandId) {
        e.preventDefault();
      }
    };
    floatingEl.ondrop = (e) => {
      e.preventDefault();
      const id =
        dragCommandId ?? e.dataTransfer?.getData("text/plain") ?? "";
      dragCommandId = null;
      if (!id) {
        return;
      }
      addCommandToFloatingBar(id, undefined, { x: e.clientX - 48, y: e.clientY - 24 }).catch(
        () => {},
      );
    };
    snapshot.floating_bars.forEach((bar) => {
      const wrap = document.createElement("div");
      wrap.className =
        "floating-bar" + (bar.anchor === "floating" ? " floating" : " docked");
      if (bar.anchor === "floating") {
        wrap.style.left = `${bar.x}px`;
        wrap.style.top = `${bar.y}px`;
      }
      const label = document.createElement("span");
      label.className = "floating-bar-label";
      label.textContent = bar.label;
      wrap.appendChild(label);
      const row = document.createElement("div");
      row.className = "floating-bar-commands";
      row.ondragover = (e) => {
        if (dragCommandId) {
          e.preventDefault();
        }
      };
      row.ondrop = (e) => {
        e.preventDefault();
        const id =
          dragCommandId ?? e.dataTransfer?.getData("text/plain") ?? "";
        dragCommandId = null;
        if (!id) {
          return;
        }
        addCommandToFloatingBar(id, bar.id).catch(() => {});
      };
      renderCommandButtons(row, bar.commands, onRun, false);
      wrap.appendChild(row);
      if (bar.anchor === "floating") {
        const handle = document.createElement("span");
        handle.className = "drag-handle";
        handle.textContent = "⋮⋮";
        handle.title = "Drag to another monitor or position";
        wrap.insertBefore(handle, wrap.firstChild);
        let dragOffsetX = 0;
        let dragOffsetY = 0;
        const onMove = async (e: PointerEvent) => {
          if (!lastSnapshot) {
            return;
          }
          const x = Math.max(0, e.clientX - dragOffsetX);
          const y = Math.max(0, e.clientY - dragOffsetY);
          wrap.style.left = `${x}px`;
          wrap.style.top = `${y}px`;
        };
        const onUp = async (e: PointerEvent) => {
          window.removeEventListener("pointermove", onMove);
          window.removeEventListener("pointerup", onUp);
          if (!lastSnapshot) {
            return;
          }
          const x = Math.max(0, e.clientX - dragOffsetX);
          const y = Math.max(0, e.clientY - dragOffsetY);
          const file = snapshotToFile(lastSnapshot);
          const barLayout = file.floating_bars.find((b) => b.id === bar.id);
          if (barLayout) {
            barLayout.anchor = "floating";
            barLayout.x = x;
            barLayout.y = y;
            await invoke("save_command_manager_layout", { layout: file });
          }
        };
        handle.addEventListener("pointerdown", (e) => {
          e.preventDefault();
          const rect = wrap.getBoundingClientRect();
          dragOffsetX = e.clientX - rect.left;
          dragOffsetY = e.clientY - rect.top;
          window.addEventListener("pointermove", onMove);
          window.addEventListener("pointerup", onUp);
        });
      }
      floatingEl.appendChild(wrap);
    });
  },

  bindPinControl(
    pinEl: HTMLElement,
    tabsEl: HTMLElement,
    commandsEl: HTMLElement,
    floatingEl: HTMLElement,
    onRun: RunCommand,
  ) {
    pinEl.addEventListener("click", async () => {
      if (!lastSnapshot) {
        return;
      }
      if (lastSnapshot.context_pinned) {
        await invoke("clear_command_manager_context_pin");
      } else {
        await invoke("pin_command_manager_context");
      }
      await commandManager.refresh(tabsEl, commandsEl, floatingEl, pinEl, onRun);
    });
  },
};
