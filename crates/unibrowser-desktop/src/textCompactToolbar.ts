import {
  familiesToRenderGroups,
  partitionGroupsByAvailableWidth,
  renderResolvedControl,
  capabilityReason,
  controlIconUsesFallback,
  lucideForControlIcon,
  type ResolvedControlDto,
  type ResolvedFamilyDto,
} from "./ui/commandControls";
import { createLucideIcon } from "./ui/lucideIcon";
import { bindCommandTooltip } from "./ui/commandTooltip";
import { controlA11yState } from "./ui/controlA11y";
import { uiCommandChromeClassList } from "./ui/cmControlClasses";
import { syncTextDocumentCard } from "./ui/textDocumentCard";
import { inferSemanticBlockStyleAtCursor } from "./textEditor";
import {
  blockPickerProbeBelowTrigger,
  collectBlockPickerCaptureMetrics,
} from "./blockPickerCaptureMetrics";
import type { FloatingBarDto, ManagerTabDto } from "./commandManager";

export const BLOCK_PICKER_MENU_LABELS = [
  "Paragraph",
  "Heading 1",
  "Heading 2",
  "Heading 3",
  "Heading 4",
  "Heading 5",
  "Heading 6",
  "Block quote",
  "Code block",
] as const;

const BLOCK_COMMANDS: { id: string; label: string }[] = [
  { id: "unibrowser.semantic.block.paragraph", label: "Paragraph" },
  { id: "unibrowser.semantic.block.heading_1", label: "Heading 1" },
  { id: "unibrowser.semantic.block.heading_2", label: "Heading 2" },
  { id: "unibrowser.semantic.block.heading_3", label: "Heading 3" },
  { id: "unibrowser.semantic.block.heading_4", label: "Heading 4" },
  { id: "unibrowser.semantic.block.heading_5", label: "Heading 5" },
  { id: "unibrowser.semantic.block.heading_6", label: "Heading 6" },
  { id: "unibrowser.semantic.block.blockquote", label: "Block quote" },
  { id: "unibrowser.semantic.block.code_block", label: "Code block" },
];

const BLOCK_STYLE_CONTROL = "unibrowser.control.semantic.block_style";
const BLOCK_PICKER_POPOVER_HOST_ID = "ub-block-picker-popover-host";

let blockPickerDismissListener: ((ev: MouseEvent) => void) | null = null;

function ensureBlockPickerPopoverHost(): HTMLElement {
  let popoverHost = document.getElementById(BLOCK_PICKER_POPOVER_HOST_ID);
  if (!popoverHost) {
    popoverHost = document.createElement("div");
    popoverHost.id = BLOCK_PICKER_POPOVER_HOST_ID;
    popoverHost.setAttribute("aria-hidden", "true");
    document.body.appendChild(popoverHost);
  }
  return popoverHost;
}

export function findBlockPickerMenu(): HTMLDivElement | null {
  return (
    document.querySelector<HTMLDivElement>(
      `#${BLOCK_PICKER_POPOVER_HOST_ID} .text-compact-block-menu:not(.hidden)`,
    ) ??
    document.querySelector<HTMLDivElement>(
      "#text-compact-toolbar-host .text-compact-block-menu:not(.hidden)",
    )
  );
}

function positionBlockPickerMenu(trigger: HTMLElement, menu: HTMLElement): void {
  const rect = trigger.getBoundingClientRect();
  menu.style.position = "fixed";
  menu.style.left = `${Math.max(4, rect.left)}px`;
  menu.style.top = `${rect.bottom + 4}px`;
  menu.style.minWidth = `${Math.max(180, rect.width)}px`;
  menu.style.maxWidth = "240px";
  menu.style.zIndex = "10050";
}

function bindBlockPickerDismiss(trigger: HTMLElement, menu: HTMLElement, wrap: HTMLElement): void {
  if (blockPickerDismissListener) {
    document.removeEventListener("click", blockPickerDismissListener, true);
    blockPickerDismissListener = null;
  }
  blockPickerDismissListener = (ev: MouseEvent) => {
    if (menu.dataset.ubCapturePinned === "1") {
      return;
    }
    const target = ev.target;
    if (
      target instanceof Node &&
      (wrap.contains(target) || menu.contains(target))
    ) {
      return;
    }
    collapseBlockPickerMenu(trigger, menu, wrap);
  };
  window.setTimeout(() => {
    if (blockPickerDismissListener) {
      document.addEventListener("click", blockPickerDismissListener, true);
    }
  }, 0);
}

function collapseBlockPickerMenu(
  trigger: HTMLElement,
  menu: HTMLElement,
  wrap: HTMLElement,
): void {
  menu.classList.add("hidden");
  menu.classList.remove("text-compact-block-menu--popover");
  menu.style.position = "";
  menu.style.left = "";
  menu.style.top = "";
  menu.style.minWidth = "";
  menu.style.maxWidth = "";
  menu.style.zIndex = "";
  delete menu.dataset.ubCapturePinned;
  if (menu.parentElement?.id === BLOCK_PICKER_POPOVER_HOST_ID) {
    wrap.appendChild(menu);
  }
  document.getElementById(BLOCK_PICKER_POPOVER_HOST_ID)?.setAttribute("aria-hidden", "true");
  document.documentElement.classList.remove("ub-block-picker-open");
  trigger.setAttribute("aria-expanded", "false");
  if (blockPickerDismissListener) {
    document.removeEventListener("click", blockPickerDismissListener, true);
    blockPickerDismissListener = null;
  }
}

export function openBlockPickerMenuPortal(
  trigger: HTMLElement,
  menu: HTMLDivElement,
  wrap: HTMLElement,
  options?: { capturePinned?: boolean },
): void {
  const popoverHost = ensureBlockPickerPopoverHost();
  popoverHost.setAttribute("aria-hidden", "false");
  if (menu.parentElement !== popoverHost) {
    popoverHost.appendChild(menu);
  }
  menu.classList.remove("hidden");
  menu.classList.add("text-compact-block-menu--popover");
  if (options?.capturePinned) {
    menu.dataset.ubCapturePinned = "1";
  } else {
    delete menu.dataset.ubCapturePinned;
  }
  positionBlockPickerMenu(trigger, menu);
  document.documentElement.classList.add("ub-block-picker-open");
  trigger.setAttribute("aria-expanded", "true");
  bindBlockPickerDismiss(trigger, menu, wrap);
}

export function resetBlockPickerPopoverHost(): void {
  document.getElementById(BLOCK_PICKER_POPOVER_HOST_ID)?.replaceChildren();
  document.documentElement.classList.remove("ub-block-picker-open");
}

function compactWidthBudget(host: HTMLElement): number {
  const cardMount = document.querySelector<HTMLElement>(".document-card-toolbar-mount");
  if (document.body.classList.contains("text-document-card") && cardMount) {
    const mountWidth = cardMount.clientWidth;
    if (mountWidth > 120) {
      return mountWidth - 28;
    }
  }
  const width = host.clientWidth || window.innerWidth;
  if (width < 400) {
    return Math.max(120, width - 48);
  }
  if (width < 560) {
    return width - 80;
  }
  if (width < 720) {
    return width - 64;
  }
  return width - 32;
}

function renderBlockStylePicker(
  control: ResolvedControlDto,
  onRun: (commandId: string) => Promise<void>,
  chordByCommand: Map<string, string>,
  selectedCommandId?: string,
): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "text-compact-block-picker";
  const a11y = controlA11yState(control);
  const emulated = control.capability.state === "emulated";
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = [
    ...uiCommandChromeClassList({
      controlType: "button",
      capabilityState: control.capability.state,
      actionable: a11y.actionable,
      emulated,
      locked: control.capability.state === "locked",
      mixed: a11y.mixed,
      modControl: false,
    }),
    "split-button",
    "text-compact-btn",
    "text-compact-block-trigger",
  ].join(" ");
  btn.dataset.controlId = control.control_id;
  btn.setAttribute("aria-haspopup", "listbox");
  btn.setAttribute("aria-expanded", "false");
  const cardMode = document.body.classList.contains("text-document-card");
  if (!cardMode) {
    const icon = createLucideIcon("pilcrow", 16, "ub-icon svg-icon");
    btn.appendChild(icon);
  }
  const inferred = inferSemanticBlockStyleAtCursor();
  const activeCommandId = selectedCommandId ?? inferred.commandId;
  const displayLabel = inferred.label;
  const label = document.createElement("span");
  label.className = "text-compact-block-label";
  label.textContent = displayLabel;
  btn.appendChild(label);
  if (emulated) {
    const reason = capabilityReason(control.capability) ?? "Approximated style";
    const mark = document.createElement("span");
    mark.className = "cm-emulation-mark block-emulation-badge";
    mark.textContent = "≈";
    mark.setAttribute("role", "img");
    mark.setAttribute("aria-label", `Emulated: ${reason}`);
    mark.title = reason;
    btn.appendChild(mark);
  }
  const chevron = createLucideIcon("chevron-down", 14, "ub-icon svg-icon text-compact-chevron");
  btn.appendChild(chevron);
  bindCommandTooltip(btn, {
    title: "Block style",
    reason: capabilityReason(control.capability),
    chord: chordByCommand.get(BLOCK_COMMANDS[2].id) ?? null,
    enabled: true,
  });
  const menu = document.createElement("div");
  menu.className = "text-compact-block-menu hidden";
  menu.setAttribute("role", "listbox");
  for (const item of BLOCK_COMMANDS) {
    const row = document.createElement("button");
    row.type = "button";
    row.className = "text-compact-block-item";
    row.dataset.commandId = item.id;
    row.textContent = item.label;
    row.setAttribute("role", "option");
    if (item.id === activeCommandId) {
      row.setAttribute("aria-selected", "true");
    }
    row.onclick = (e) => {
      e.stopPropagation();
      collapseBlockPickerMenu(btn, menu, wrap);
      void onRun(item.id);
    };
    menu.appendChild(row);
  }
  btn.onclick = (e) => {
    e.stopPropagation();
    if (menu.classList.contains("hidden")) {
      openBlockPickerMenuPortal(btn, menu, wrap);
    } else {
      collapseBlockPickerMenu(btn, menu, wrap);
    }
  };
  wrap.append(btn, menu);
  return wrap;
}

function renderCompactControl(
  control: ResolvedControlDto,
  onRun: (commandId: string) => Promise<void>,
  chordByCommand: Map<string, string>,
): HTMLElement {
  if (control.control_id === BLOCK_STYLE_CONTROL) {
    return renderBlockStylePicker(control, onRun, chordByCommand);
  }
  const el = renderResolvedControl(control, onRun, chordByCommand, false);
  el.classList.add("text-compact-btn");
  const label = el.querySelector(".cm-cmd-label");
  if (label) {
    label.remove();
  }
  return el;
}

function appendModDockedSection(
  tools: HTMLElement,
  bars: FloatingBarDto[],
  onRun: (commandId: string) => Promise<void>,
  chordByCommand: Map<string, string>,
) {
  for (const bar of bars) {
    if (bar.commands.length === 0) {
      continue;
    }
    const div = document.createElement("span");
    div.className = "cm-divider";
    div.setAttribute("aria-hidden", "true");
    tools.appendChild(div);
    const dock = document.createElement("div");
    dock.className = "text-compact-mod-dock";
    dock.dataset.toolbarId = bar.id;
    const name = document.createElement("span");
    name.className = "text-compact-mod-label";
    name.textContent = bar.label;
    dock.appendChild(name);
    for (const cmd of bar.commands) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "ui-command cm-mod-control cm-cap-supported text-compact-btn";
      btn.dataset.commandId = cmd.command_id;
      btn.setAttribute("aria-label", cmd.title);
      const iconName = lucideForControlIcon(cmd.icon);
      if (controlIconUsesFallback(cmd.icon) || iconName === "file") {
        btn.dataset.iconFallback = "1";
      }
      btn.appendChild(createLucideIcon(iconName, 16, "ub-icon svg-icon"));
      bindCommandTooltip(btn, {
        title: cmd.title,
        reason: cmd.reason,
        chord: chordByCommand.get(cmd.command_id) ?? null,
        enabled: cmd.enabled,
      });
      if (cmd.enabled) {
        btn.onclick = () => void onRun(cmd.command_id);
      } else {
        btn.setAttribute("aria-disabled", "true");
      }
      dock.appendChild(btn);
    }
    tools.appendChild(dock);
  }
}

function renderCompactRow(
  container: HTMLElement,
  families: ResolvedFamilyDto[],
  compactDockedBars: FloatingBarDto[],
  showModDock: boolean,
  onRun: (commandId: string) => Promise<void>,
  chordByCommand: Map<string, string>,
) {
  resetBlockPickerPopoverHost();
  container.innerHTML = "";
  const groups = familiesToRenderGroups(families);
  const budget = compactWidthBudget(container);
  let { visible, overflow } = document.body.classList.contains("text-document-card")
    ? { visible: groups, overflow: [] as typeof groups }
    : partitionGroupsByAvailableWidth(groups, budget);
  const tools = document.createElement("div");
  tools.className = "text-compact-tools";
  tools.setAttribute("role", "toolbar");
  tools.setAttribute("aria-label", "Text formatting");
  const pushControls = (list: typeof visible) => {
    list.forEach((group, index) => {
      if (index > 0 || tools.childElementCount > 0) {
        const div = document.createElement("span");
        div.className = "cm-divider";
        div.setAttribute("aria-hidden", "true");
        tools.appendChild(div);
      }
      for (const control of group.resolved.controls) {
        tools.appendChild(renderCompactControl(control, onRun, chordByCommand));
      }
    });
  };
  pushControls(visible);
  if (showModDock && compactDockedBars.length > 0) {
    appendModDockedSection(tools, compactDockedBars, onRun, chordByCommand);
  }
  if (overflow.length > 0) {
    const overflowBtn = document.createElement("button");
    overflowBtn.type = "button";
    overflowBtn.className = "ui-command cm-cap-supported text-compact-btn";
    overflowBtn.setAttribute("aria-label", "More formatting");
    overflowBtn.appendChild(createLucideIcon("ellipsis", 16, "ub-icon svg-icon"));
    overflowBtn.onclick = (e) => {
      e.stopPropagation();
      const menu = document.createElement("div");
      menu.className = "text-compact-overflow-menu";
      for (const group of overflow) {
        for (const control of group.resolved.controls) {
          const item = document.createElement("button");
          item.type = "button";
          item.textContent = control.label;
          item.onclick = () => {
            menu.remove();
            void onRun(control.command_ids[0] ?? control.control_id);
          };
          menu.appendChild(item);
        }
      }
      document.body.appendChild(menu);
      const rect = overflowBtn.getBoundingClientRect();
      menu.style.left = `${rect.left}px`;
      menu.style.top = `${rect.bottom + 4}px`;
      const close = () => {
        menu.remove();
        document.removeEventListener("click", close);
      };
      window.setTimeout(() => document.addEventListener("click", close), 0);
    };
    tools.appendChild(overflowBtn);
  }
  container.appendChild(tools);
  const fileLabel =
    document.querySelector<HTMLElement>(".title-tabs .tab.active .tab-label")?.textContent ??
    document.querySelector<HTMLElement>("#title-breadcrumb strong")?.textContent ??
    null;
  syncTextDocumentCard(true, fileLabel);
}

export function refreshTextCompactToolbar(
  primaryHost: HTMLElement,
  modHost: HTMLElement,
  tab: ManagerTabDto | undefined,
  autoContext: string,
  compactDockedBars: FloatingBarDto[],
  onRun: (commandId: string) => Promise<void>,
  chordByCommand: Map<string, string>,
) {
  const show =
    tab &&
    (tab.compact_toolbar_families?.length ?? 0) > 0 &&
    (autoContext === "text" || autoContext === "office");
  primaryHost.classList.toggle("hidden", !show);
  modHost.classList.add("hidden");
  modHost.innerHTML = "";
  if (!show || !tab) {
    primaryHost.innerHTML = "";
    document.body.classList.remove("text-compact-visible");
    document.body.classList.remove("cm-collapsed");
    syncTextDocumentCard(false);
    return;
  }
  document.body.classList.add("text-compact-visible");
  document.body.classList.add("cm-collapsed");
  const showModDock = document.documentElement.dataset.ubModToolbarDocked === "1";
  renderCompactRow(
    primaryHost,
    tab.compact_toolbar_families,
    compactDockedBars,
    showModDock,
    onRun,
    chordByCommand,
  );
}

export function refreshCompactBlockPickerLabel(): void {
  if (!document.body.classList.contains("text-compact-visible")) {
    return;
  }
  const host = document.getElementById("text-compact-toolbar-host");
  if (!host) {
    return;
  }
  const inferred = inferSemanticBlockStyleAtCursor();
  const label = host.querySelector<HTMLElement>(".text-compact-block-label");
  if (label) {
    label.textContent = inferred.label;
  }
  const menu =
    findBlockPickerMenu() ??
    host.querySelector<HTMLElement>(".text-compact-block-menu");
  if (menu) {
    const triggerLabel = label?.textContent?.trim();
    const fromLabel = triggerLabel
      ? BLOCK_COMMANDS.find((item) => item.label === triggerLabel)?.id
      : undefined;
    const activeId = fromLabel ?? inferred.commandId;
    menu.querySelectorAll<HTMLElement>(".text-compact-block-item").forEach((row) => {
      row.toggleAttribute("aria-selected", row.dataset.commandId === activeId);
    });
  }
}

export function closeBlockPickerMenuForCapture(host: HTMLElement): void {
  const trigger = host.querySelector<HTMLButtonElement>(".text-compact-block-trigger");
  const wrap = host.querySelector<HTMLElement>(".text-compact-block-picker");
  const menu =
    host.querySelector<HTMLDivElement>(".text-compact-block-menu") ?? findBlockPickerMenu();
  if (trigger && menu && wrap) {
    collapseBlockPickerMenu(trigger, menu, wrap);
  }
}

export function openBlockStyleMenuForCapture(host: HTMLElement): void {
  refreshCompactBlockPickerLabel();
  const trigger = host.querySelector<HTMLButtonElement>(".text-compact-block-trigger");
  const wrap = host.querySelector<HTMLElement>(".text-compact-block-picker");
  const menu =
    host.querySelector<HTMLDivElement>(".text-compact-block-menu") ??
    findBlockPickerMenu();
  if (trigger && menu && wrap) {
    openBlockPickerMenuPortal(trigger, menu, wrap, { capturePinned: true });
    trigger.blur();
    document.getElementById("ub-command-tooltip-host")?.classList.add("hidden");
  }
}

export function installTextCaptureHarness(): void {
  if (typeof window === "undefined") {
    return;
  }
  const win = window as Window & {
    __unibrowserCapture?: {
      closeBlockPickerForCapture(): void;
      openBlockPickerForCapture(): void;
      blockPickerProbeRect(): { x: number; y: number; width: number; height: number };
      collectBlockPickerMetrics(): ReturnType<typeof collectBlockPickerCaptureMetrics>;
    };
  };
  win.__unibrowserCapture = {
    closeBlockPickerForCapture() {
      const host = document.getElementById("text-compact-toolbar-host");
      if (host) {
        closeBlockPickerMenuForCapture(host);
      }
    },
    openBlockPickerForCapture() {
      const host = document.getElementById("text-compact-toolbar-host");
      if (host) {
        openBlockStyleMenuForCapture(host);
      }
    },
    blockPickerProbeRect() {
      const host = document.getElementById("text-compact-toolbar-host");
      if (!host) {
        return { x: 0, y: 0, width: 0, height: 0 };
      }
      return blockPickerProbeBelowTrigger(host);
    },
    collectBlockPickerMetrics() {
      const host = document.getElementById("text-compact-toolbar-host");
      if (!host) {
        throw new Error("text-compact-toolbar-host missing");
      }
      return collectBlockPickerCaptureMetrics(host, null);
    },
  };
}

/** Assert compact toolbar icons are mapped (no generic file glyph). */
export function compactToolbarIconFallbackError(host: HTMLElement | null): string | null {
  if (!host) {
    return "compact toolbar host missing";
  }
  for (const svg of host.querySelectorAll<SVGElement>(
    "#text-compact-toolbar-host .text-compact-btn .ub-icon[data-lucide]",
  )) {
    if (svg.dataset.lucide === "file") {
      return "compact toolbar uses generic file icon fallback";
    }
  }
  for (const btn of host.querySelectorAll<HTMLElement>(".text-compact-btn[data-icon-fallback='1']")) {
    return `mod toolbar control ${btn.dataset.commandId ?? "?"} has unmapped icon`;
  }
  for (const control of host.querySelectorAll<HTMLElement>(".text-compact-btn[data-control-id]")) {
    const icon = control.querySelector<SVGElement>(".ub-icon[data-lucide='file']");
    if (icon && control.dataset.controlId !== BLOCK_STYLE_CONTROL) {
      return `control ${control.dataset.controlId} renders file icon fallback`;
    }
  }
  return null;
}
