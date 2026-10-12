import { invoke } from "@tauri-apps/api/core";
import { patchStatusItem } from "./statusBar";
import { patchPictureInPictureCommand } from "./mediaPip";
import { showToast } from "./toast";
import { t } from "./i18n";
import { bindCommandTooltip } from "./ui/commandTooltip";
import {
  familiesToRenderGroups,
  partitionGroupsByAvailableWidth,
  renderResolvedControl,
  renderResolvedGroups,
  type ResolvedFamilyDto,
} from "./ui/commandControls";
import { createLucideIcon, type LucideName } from "./ui/lucideIcon";
import { getPdfCommandManagerLabels } from "./viewerZoom";
import { announceCm } from "./ui/cmA11y";
import { bindCmRovingFocus } from "./ui/cmRovingFocus";
import { commandGroupBudgetFromViewport } from "./ui/commandOverflow";
import { isWebPlatform } from "./platform/isWeb";

function prioritizeWebMarkdownGroups(
  groups: ReturnType<typeof familiesToRenderGroups>,
): ReturnType<typeof familiesToRenderGroups> {
  if (!isWebPlatform() || groups.length === 0) {
    return groups;
  }
  const pick = (name: string) => groups.find((g) => g.name === name);
  const history = pick("History");
  const emphasis = pick("Emphasis");
  if (!history || !emphasis) {
    return groups;
  }
  const rest = groups.filter((g) => g.name !== "History" && g.name !== "Emphasis");
  return [history, emphasis, ...rest];
}

function layoutEditModeActive(): boolean {
  return document.body.classList.contains("layout-edit-mode");
}

const PDF_PAGE_CONTROL = "unibrowser.control.pdf.page";
const PDF_ZOOM_CONTROL = "unibrowser.control.pdf.viewZoom";

function patchPdfCommandFamilies(families: ResolvedFamilyDto[]): ResolvedFamilyDto[] {
  const { page, zoomPct } = getPdfCommandManagerLabels();
  if (!page && zoomPct == null) {
    return families;
  }
  return families.map((family) => ({
    ...family,
    groups: family.groups.map((group) => ({
      ...group,
      controls: group.controls.map((control) => {
        if (page && control.control_id === PDF_PAGE_CONTROL) {
          return {
            ...control,
            label: `${page.current} / ${page.total}`,
          };
        }
        if (zoomPct != null && control.control_id === PDF_ZOOM_CONTROL) {
          return { ...control, label: `${zoomPct}%` };
        }
        return control;
      }),
    })),
  }));
}

let chordByCommand = new Map<string, string>();

export async function loadCommandChords() {
  try {
    const bindings = await invoke<{ command_id: string; chord: string }[]>("get_keybindings");
    chordByCommand = new Map(bindings.map((b) => [b.command_id, b.chord]));
  } catch {
    chordByCommand = new Map();
  }
}

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
  families: ResolvedFamilyDto[];
  resolved_family_count: number;
  resolved_control_count: number;
}

export interface FloatingBarDto {
  id: string;
  label: string;
  anchor: string;
  x: number;
  y: number;
  coord_space?: string;
  presentation?: string;
  monitor_name?: string | null;
  monitor_edid?: string | null;
  commands: UiCommandDto[];
}

export interface ShellContributionsDto {
  panels: { mod_name: string; panel: { id: string; region: string; title: string } }[];
  views: { mod_name: string; view: { id: string; title: string } }[];
  status_items: {
    mod_name: string;
    item: { id: string; slot: string; priority: number; label: string };
  }[];
  services: { mod_name: string; service: { id: string; description: string } }[];
}

export interface CommandManagerDto {
  auto_context: string;
  selected_tab: string;
  context_display_label: string;
  context_pinned: boolean;
  tabs: ManagerTabDto[];
  floating_bars: FloatingBarDto[];
  diagnostics: string[];
  shell_contributions: ShellContributionsDto;
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
    coord_space?: string;
    presentation?: string;
    monitor_name?: string | null;
    monitor_edid?: string | null;
    commands: { command_id: string }[];
  }[];
}

function floatingBarViewportPosition(bar: FloatingBarDto): { left: number; top: number } {
  if (bar.anchor !== "floating") {
    return { left: bar.x, top: bar.y };
  }
  const space = bar.coord_space ?? "viewport";
  if (space === "screen") {
    return {
      left: bar.x - window.screenX,
      top: bar.y - window.screenY,
    };
  }
  return { left: bar.x, top: bar.y };
}

function screenPositionFromPointer(e: PointerEvent, offsetX: number, offsetY: number): {
  screenX: number;
  screenY: number;
} {
  return {
    screenX: e.screenX - offsetX,
    screenY: e.screenY - offsetY,
  };
}

const DEFAULT_FLOAT_OFFSET = 96;

async function resetFloatingBarPosition(barId: string) {
  if (!lastSnapshot) {
    return;
  }
  const file = snapshotToFile(lastSnapshot);
  const barLayout = file.floating_bars.find((b) => b.id === barId);
  if (!barLayout) {
    return;
  }
  barLayout.anchor = "floating";
  barLayout.x = window.screenX + DEFAULT_FLOAT_OFFSET;
  barLayout.y = window.screenY + DEFAULT_FLOAT_OFFSET;
  barLayout.coord_space = "screen";
  await invoke("save_command_manager_layout", { layout: file });
}

function bindFloatingBarContextMenu(
  wrap: HTMLElement,
  bar: FloatingBarDto,
  onRefresh: () => Promise<void>,
) {
  wrap.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    const menu = document.createElement("div");
    menu.className = "floating-bar-context-menu";
    const reset = document.createElement("button");
    reset.type = "button";
    reset.textContent = t("commandManager.resetThisToolbar");
    reset.onclick = async () => {
      menu.remove();
      await resetFloatingBarPosition(bar.id);
      await onRefresh();
      showToast(t("commandManager.toolbarReset"), "info");
    };
    menu.appendChild(reset);
    const detached = bar.presentation === "detached_window";
    const detachBtn = document.createElement("button");
    detachBtn.type = "button";
    detachBtn.textContent = detached
      ? t("commandManager.dockToolbar")
      : t("commandManager.detachToolbar");
    detachBtn.onclick = async () => {
      menu.remove();
      if (detached) {
        await invoke("dock_floating_toolbar_window", { barId: bar.id });
      } else {
        await invoke("detach_floating_toolbar_window", { barId: bar.id });
      }
      await onRefresh();
    };
    menu.appendChild(detachBtn);
    const close = () => menu.remove();
    menu.addEventListener("pointerdown", (ev) => ev.stopPropagation());
    document.body.appendChild(menu);
    menu.style.left = `${e.clientX}px`;
    menu.style.top = `${e.clientY}px`;
    const onDoc = (ev: MouseEvent) => {
      if (!menu.contains(ev.target as Node)) {
        close();
        document.removeEventListener("click", onDoc);
      }
    };
    window.setTimeout(() => document.addEventListener("click", onDoc), 0);
  });
}

type RunCommand = (commandId: string) => Promise<void>;

let lastSnapshot: CommandManagerDto | null = null;

export function getStatusContextMode(): { tab: string; pinned: boolean } | null {
  const snapshot = lastSnapshot;
  if (!snapshot) {
    return null;
  }
  const tab = snapshot.tabs.find((t) => t.id === snapshot.selected_tab);
  const label = snapshot.context_pinned
    ? tab?.label
    : snapshot.context_display_label || tab?.label;
  if (!label) {
    return null;
  }
  return { tab: label, pinned: snapshot.context_pinned };
}
let dragCommandId: string | null = null;
type RenderGroup = ReturnType<typeof familiesToRenderGroups>[number];
let overflowGroupsCache: RenderGroup[] = [];
let cmRovingBound = false;

function lucideForRegistryIcon(icon: string): LucideName {
  const map: Record<string, LucideName> = {
    undo: "undo-2",
    redo: "redo-2",
    search: "search",
    export: "download",
    spellcheck: "spell-check",
    format: "pen-line",
    "font-size-up": "type",
    "font-size-down": "type",
    save: "save",
  };
  return map[icon] ?? "file";
}

function renderCommandButtons(
  container: HTMLElement,
  commands: UiCommandDto[],
  onRun: RunCommand,
  draggable: boolean,
) {
  container.innerHTML = "";
  commands.forEach((cmd) => {
    const patched = patchPictureInPictureCommand(cmd);
    const btn = document.createElement("button");
    btn.type = "button";
    btn.appendChild(createLucideIcon(lucideForRegistryIcon(cmd.icon), 16, "ub-icon svg-icon"));
    const label = document.createElement("span");
    label.className = "cm-cmd-label";
    label.textContent = patched.title;
    btn.appendChild(label);
    btn.disabled = !patched.enabled;
    btn.className = patched.enabled ? "ui-command" : "ui-command disabled";
    btn.dataset.commandId = patched.command_id;
    const chord = chordByCommand.get(patched.command_id) ?? null;
    bindCommandTooltip(btn, {
      title: patched.title,
      reason: patched.enabled ? null : patched.reason,
      chord,
      enabled: patched.enabled,
    });
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
    if (patched.enabled) {
      btn.onclick = () => onRun(patched.command_id);
    }
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
    const px = position?.x ?? 96;
    const py = position?.y ?? 96;
    file.floating_bars.push({
      id,
      label: t("commandManager.floatingCustomLabel"),
      anchor: "floating",
      x: window.screenX + px,
      y: window.screenY + py,
      coord_space: "screen",
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
      coord_space: bar.coord_space ?? "viewport",
      presentation: bar.presentation ?? "in_window",
      monitor_name: bar.monitor_name ?? null,
      monitor_edid: bar.monitor_edid ?? null,
      commands: bar.commands.map((c) => ({ command_id: c.command_id })),
    })),
  };
}

function openContextTabMenu(
  anchor: HTMLElement,
  snapshot: CommandManagerDto,
  onPick: (tabId: string) => void,
) {
  const menu = document.createElement("div");
  menu.className = "context-menu-panel";
  menu.setAttribute("role", "menu");
  snapshot.tabs.forEach((tab) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.setAttribute("role", "menuitem");
    const preview = isPreviewingTab(snapshot, tab.id);
    const autoMatch = !snapshot.context_pinned && tab.id === snapshot.auto_context;
    btn.textContent =
      autoMatch ? `${tab.label} *` : preview ? `${tab.label} ...` : tab.label;
    if (tab.selected) {
      btn.setAttribute("aria-current", "true");
    }
    btn.onclick = () => {
      menu.remove();
      onPick(tab.id);
    };
    menu.appendChild(btn);
  });
  const rect = anchor.getBoundingClientRect();
  menu.style.left = `${rect.left}px`;
  menu.style.top = `${rect.bottom + 4}px`;
  document.body.appendChild(menu);
  const close = (ev: MouseEvent) => {
    if (!menu.contains(ev.target as Node)) {
      menu.remove();
      document.removeEventListener("click", close);
    }
  };
  window.setTimeout(() => document.addEventListener("click", close), 0);
}

function openOverflowGroupsMenu(
  anchor: HTMLElement,
  groups: RenderGroup[],
  onRun: RunCommand,
) {
  const menu = document.createElement("div");
  menu.className = "cm-overflow-menu";
  groups.forEach((group) => {
    const heading = document.createElement("div");
    heading.className = "palette-section-label";
    heading.textContent = group.name;
    menu.appendChild(heading);
    for (const control of group.resolved.controls) {
      const el = renderResolvedControl(control, onRun, chordByCommand, false);
      el.onclick = (e) => {
        e.stopPropagation();
        menu.remove();
      };
      menu.appendChild(el);
    }
  });
  const rect = anchor.getBoundingClientRect();
  menu.style.left = `${rect.left}px`;
  menu.style.top = `${rect.bottom + 4}px`;
  document.body.appendChild(menu);
  const close = (ev: MouseEvent) => {
    if (!menu.contains(ev.target as Node)) {
      menu.remove();
      document.removeEventListener("click", close);
    }
  };
  window.setTimeout(() => document.addEventListener("click", close), 0);
}

function updatePinControl(
  pinEl: HTMLElement,
  labelEl: HTMLElement | null,
  snapshot: CommandManagerDto,
) {
  const pinnedTab = contextLabel(snapshot.selected_tab, snapshot.tabs);
  const autoTab =
    snapshot.context_display_label || contextLabel(snapshot.auto_context, snapshot.tabs);
  pinEl.classList.toggle("pinned", snapshot.context_pinned);
  pinEl.classList.toggle("auto-follow", !snapshot.context_pinned);
  if (snapshot.context_pinned) {
    pinEl.textContent = t("commandManager.followDocument");
    pinEl.title = t("commandManager.followDocumentTitle", { tab: pinnedTab, auto: autoTab });
    if (labelEl) {
      labelEl.textContent = t("commandManager.statePinned", { tab: pinnedTab });
    }
  } else {
    pinEl.textContent = t("commandManager.pinCommandsTab");
    pinEl.title = t("commandManager.pinCommandsTabTitle", { auto: autoTab });
    if (labelEl) {
      labelEl.textContent = t("commandManager.stateAutoFollow", { auto: autoTab });
    }
  }
}

function applyModShellContributions(shell: ShellContributionsDto): void {
  for (const entry of shell.status_items) {
    patchStatusItem(entry.item.id, entry.item.label);
  }
}

function isPreviewingTab(snapshot: CommandManagerDto, tabId: string): boolean {
  return (
    !snapshot.context_pinned &&
    snapshot.selected_tab === tabId &&
    snapshot.selected_tab !== snapshot.auto_context
  );
}

export const commandManager = {
  async refresh(
    groupsEl: HTMLElement,
    floatingEl: HTMLElement,
    pinEl: HTMLElement,
    contextEl: HTMLElement,
    overflowEl: HTMLElement | null,
    onRun: RunCommand,
  ) {
    const labelEl = document.getElementById("command-manager-pin-label");
    if (chordByCommand.size === 0) {
      await loadCommandChords();
    }
    const snapshot = await invoke<CommandManagerDto>("get_command_manager");
    lastSnapshot = snapshot;
    applyModShellContributions(
      snapshot.shell_contributions ?? {
        panels: [],
        views: [],
        status_items: [],
        services: [],
      },
    );
    updatePinControl(pinEl, labelEl, snapshot);
    const commandTabId = snapshot.context_pinned
      ? snapshot.selected_tab
      : snapshot.auto_context;
    const active =
      snapshot.tabs.find((tab) => tab.id === commandTabId) ??
      snapshot.tabs.find((tab) => tab.selected) ??
      snapshot.tabs[0];
    const contextText = snapshot.context_pinned
      ? (active?.label ?? contextLabel(snapshot.selected_tab, snapshot.tabs))
      : (snapshot.context_display_label ||
        active?.label ||
        contextLabel(snapshot.auto_context, snapshot.tabs));
    contextEl.textContent = contextText;
    document.body.dataset.cmAutoContext = snapshot.auto_context;
    contextEl.setAttribute("aria-label", `${contextText} commands`);
    const diagnosticText = snapshot.diagnostics.join("\n");
    contextEl.title = diagnosticText || `${contextText} commands`;
    if (diagnosticText) {
      contextEl.setAttribute("aria-description", diagnosticText);
    } else {
      contextEl.removeAttribute("aria-description");
    }
    if (!contextEl.dataset.bound) {
      contextEl.dataset.bound = "1";
      contextEl.addEventListener("click", (e) => {
        e.stopPropagation();
        if (!lastSnapshot) {
          return;
        }
        openContextTabMenu(contextEl, lastSnapshot, async (tabId) => {
          await invoke("select_command_manager_tab", { tabId });
          await commandManager.refresh(
            groupsEl,
            floatingEl,
            pinEl,
            contextEl,
            overflowEl,
            onRun,
          );
        });
      });
    }
    if (active) {
      const patchedFamilies =
        active.families?.length > 0 ? patchPdfCommandFamilies(active.families) : [];
      let allGroups =
        patchedFamilies.length > 0 ? familiesToRenderGroups(patchedFamilies) : [];
      allGroups = prioritizeWebMarkdownGroups(allGroups);
      let widthBudget = commandGroupBudgetFromViewport(
        window.innerWidth,
        groupsEl.clientWidth,
      );
      if (isWebPlatform()) {
        widthBudget += 160;
      }
      let { visible, overflow } = partitionGroupsByAvailableWidth(allGroups, widthBudget);
      const setOverflowState = () => {
        const hasOverflow = overflow.length > 0;
        groupsEl.classList.toggle("cm-overflow-active", hasOverflow);
        if (overflowEl) {
          overflowEl.classList.toggle("visible", hasOverflow);
          overflowEl.classList.toggle("hidden", !hasOverflow);
        }
      };
      setOverflowState();
      if (visible.length > 0) {
        renderResolvedGroups(groupsEl, visible, onRun, chordByCommand);
        let correctedBudget = widthBudget;
        for (let attempt = 0; attempt <= allGroups.length; attempt += 1) {
          const clippedWidth = Math.ceil(groupsEl.scrollWidth - groupsEl.clientWidth);
          if (clippedWidth <= 0) {
            break;
          }
          correctedBudget = Math.max(0, correctedBudget - clippedWidth - 8);
          ({ visible, overflow } = partitionGroupsByAvailableWidth(
            allGroups,
            correctedBudget,
          ));
          setOverflowState();
          renderResolvedGroups(groupsEl, visible, onRun, chordByCommand);
        }
      } else {
        renderCommandButtons(groupsEl, active.commands, onRun, layoutEditModeActive());
      }
      overflowGroupsCache = overflow;
      if (!cmRovingBound) {
        cmRovingBound = true;
        bindCmRovingFocus({
          groupsHost: groupsEl,
          overflowButton: overflowEl,
          onOpenOverflow: () => {
            if (overflowEl) {
              openOverflowGroupsMenu(overflowEl, overflowGroupsCache, onRun);
            }
          },
          onAnnounce: announceCm,
        });
      }
      const frontendResolvedControlCount = allGroups.reduce(
        (count, group) => count + group.resolved.controls.length,
        0,
      );
      const frontendVisibleModelCount = visible.reduce(
        (count, group) => count + group.resolved.controls.length,
        0,
      );
      document.documentElement.dataset.cmPipeline = JSON.stringify({
        activeTabId: active.id,
        rustFamilyCount: active.resolved_family_count,
        rustControlCount: active.resolved_control_count,
        ipcFamilyCount: active.families.length,
        ipcControlCount: active.families.reduce(
          (count, family) =>
            count +
            family.groups.reduce(
              (groupCount, group) => groupCount + group.controls.length,
              0,
            ),
          0,
        ),
        frontendGroupCount: allGroups.length,
        frontendResolvedControlCount,
        frontendVisibleModelCount,
        frontendOverflowModelCount: frontendResolvedControlCount - frontendVisibleModelCount,
        groupsWidth: groupsEl.clientWidth,
      });
      if (overflowEl) {
        if (!overflowEl.dataset.bound) {
          overflowEl.dataset.bound = "1";
          overflowEl.innerHTML = "";
          overflowEl.appendChild(createLucideIcon("chevron-down", 16, "ub-icon svg-icon"));
          overflowEl.addEventListener("click", (e) => {
            e.stopPropagation();
            openOverflowGroupsMenu(overflowEl, overflowGroupsCache, onRun);
          });
        }
      }
    } else {
      groupsEl.innerHTML = "";
    }
    floatingEl.innerHTML = "";
    floatingEl.classList.add("floating-toolbar-drop-host");
    floatingEl.ondragover = (e) => {
      if (layoutEditModeActive() && dragCommandId) {
        e.preventDefault();
      }
    };
    floatingEl.ondrop = (e) => {
      if (!layoutEditModeActive()) {
        return;
      }
      e.preventDefault();
      const id =
        dragCommandId ?? e.dataTransfer?.getData("text/plain") ?? "";
      dragCommandId = null;
      if (!id) {
        return;
      }
      addCommandToFloatingBar(id, undefined, {
        x: e.clientX - 48,
        y: e.clientY - 24,
      }).catch(() => {});
    };
    const visibleBars = snapshot.floating_bars.filter(
      (bar) => bar.id !== "quick-text" && bar.commands.length > 0,
    );
    visibleBars.forEach((bar) => {
      const wrap = document.createElement("div");
      wrap.className =
        "floating-bar" + (bar.anchor === "floating" ? " floating" : " docked");
      if (bar.anchor === "floating" && layoutEditModeActive()) {
        const pos = floatingBarViewportPosition(bar);
        wrap.style.left = `${pos.left}px`;
        wrap.style.top = `${pos.top}px`;
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
        const handle = document.createElement("button");
        handle.type = "button";
        handle.className = "drag-handle";
        handle.textContent = "⋮⋮";
        handle.title = t("commandManager.dragToolbarTitle");
        handle.setAttribute("aria-label", t("commandManager.dragToolbarKeyboard"));
        wrap.insertBefore(handle, wrap.firstChild);
        let dragOffsetX = 0;
        let dragOffsetY = 0;
        const onMove = async (e: PointerEvent) => {
          if (!lastSnapshot) {
            return;
          }
          const { screenX, screenY } = screenPositionFromPointer(
            e,
            dragOffsetX,
            dragOffsetY,
          );
          wrap.style.left = `${screenX - window.screenX}px`;
          wrap.style.top = `${screenY - window.screenY}px`;
        };
        const onUp = async (e: PointerEvent) => {
          window.removeEventListener("pointermove", onMove);
          window.removeEventListener("pointerup", onUp);
          if (!lastSnapshot) {
            return;
          }
          const { screenX, screenY } = screenPositionFromPointer(
            e,
            dragOffsetX,
            dragOffsetY,
          );
          const file = snapshotToFile(lastSnapshot);
          const barLayout = file.floating_bars.find((b) => b.id === bar.id);
          if (barLayout) {
            barLayout.anchor = "floating";
            barLayout.x = screenX;
            barLayout.y = screenY;
            barLayout.coord_space = "screen";
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
        const nudgeToolbar = async (dx: number, dy: number) => {
          const left = Number.parseFloat(wrap.style.left) || 0;
          const top = Number.parseFloat(wrap.style.top) || 0;
          wrap.style.left = `${Math.max(0, left + dx)}px`;
          wrap.style.top = `${Math.max(0, top + dy)}px`;
          if (!lastSnapshot) {
            return;
          }
          const file = snapshotToFile(lastSnapshot);
          const barLayout = file.floating_bars.find((b) => b.id === bar.id);
          if (!barLayout) {
            return;
          }
          const rect = wrap.getBoundingClientRect();
          barLayout.anchor = "floating";
          barLayout.x = window.screenX + rect.left;
          barLayout.y = window.screenY + rect.top;
          barLayout.coord_space = "screen";
          await invoke("save_command_manager_layout", { layout: file });
        };
        handle.addEventListener("keydown", (e) => {
          const step = e.shiftKey ? 32 : 8;
          if (e.key === "ArrowLeft") {
            e.preventDefault();
            void nudgeToolbar(-step, 0);
          } else if (e.key === "ArrowRight") {
            e.preventDefault();
            void nudgeToolbar(step, 0);
          } else if (e.key === "ArrowUp") {
            e.preventDefault();
            void nudgeToolbar(0, -step);
          } else if (e.key === "ArrowDown") {
            e.preventDefault();
            void nudgeToolbar(0, step);
          }
        });
      }
      bindFloatingBarContextMenu(wrap, bar, async () => {
        await commandManager.refresh(
          groupsEl,
          floatingEl,
          pinEl,
          contextEl,
          overflowEl,
          onRun,
        );
      });
      floatingEl.appendChild(wrap);
    });
    const floatHost = document.getElementById("floating-toolbar-host");
    if (floatHost) {
      floatHost.classList.toggle("hidden", visibleBars.length === 0);
    }
  },

  bindPinControl(
    pinEl: HTMLElement,
    groupsEl: HTMLElement,
    floatingEl: HTMLElement,
    contextEl: HTMLElement,
    overflowEl: HTMLElement | null,
    onRun: RunCommand,
  ) {
    pinEl.addEventListener("click", async () => {
      if (!lastSnapshot) {
        return;
      }
      if (lastSnapshot.context_pinned) {
        await invoke("clear_command_manager_context_pin");
      } else {
        const prefs = await invoke<{ cm_pin_tip_seen: boolean }>("get_preferences");
        await invoke("pin_command_manager_context");
        if (!prefs.cm_pin_tip_seen) {
          showToast(t("commandManager.pinTip"), "info");
          await invoke("dismiss_cm_pin_tip");
        }
      }
      await commandManager.refresh(
        groupsEl,
        floatingEl,
        pinEl,
        contextEl,
        overflowEl,
        onRun,
      );
    });
  },

  bindCollapseControl(collapseEl: HTMLElement | null) {
    if (!collapseEl || collapseEl.dataset.bound) {
      return;
    }
    collapseEl.dataset.bound = "1";
    collapseEl.innerHTML = "";
    collapseEl.appendChild(createLucideIcon("chevron-down", 16, "ub-icon svg-icon"));
    collapseEl.addEventListener("click", () => {
      document.body.classList.toggle("cm-collapsed");
    });
  },
};
