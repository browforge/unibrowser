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
import type { FloatingBarDto, ManagerTabDto } from "./commandManager";

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

function compactWidthBudget(host: HTMLElement): number {
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
  const icon = createLucideIcon("pilcrow", 16, "ub-icon svg-icon");
  btn.appendChild(icon);
  const label = document.createElement("span");
  label.className = "text-compact-block-label";
  label.textContent = control.value ?? control.label;
  btn.appendChild(label);
  const chevron = createLucideIcon("chevron-down", 14, "ub-icon svg-icon text-compact-chevron");
  btn.appendChild(chevron);
  if (emulated) {
    const mark = document.createElement("span");
    mark.className = "cm-emulation-mark";
    mark.textContent = "≈";
    mark.setAttribute("aria-hidden", "true");
    mark.title = capabilityReason(control.capability) ?? "";
    btn.appendChild(mark);
  }
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
    if (
      label.textContent === item.label ||
      (item.id.endsWith("heading_2") && label.textContent === "Heading 2")
    ) {
      row.setAttribute("aria-selected", "true");
    }
    row.onclick = (e) => {
      e.stopPropagation();
      menu.classList.add("hidden");
      btn.setAttribute("aria-expanded", "false");
      void onRun(item.id);
    };
    menu.appendChild(row);
  }
  btn.onclick = (e) => {
    e.stopPropagation();
    const open = menu.classList.toggle("hidden");
    btn.setAttribute("aria-expanded", open ? "false" : "true");
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
  container.innerHTML = "";
  const groups = familiesToRenderGroups(families);
  const budget = compactWidthBudget(container);
  let { visible, overflow } = partitionGroupsByAvailableWidth(groups, budget);
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
    return;
  }
  document.body.classList.add("text-compact-visible");
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

export function openBlockStyleMenuForCapture(host: HTMLElement): void {
  const trigger = host.querySelector<HTMLButtonElement>(".text-compact-block-trigger");
  const menu = host.querySelector<HTMLDivElement>(".text-compact-block-menu");
  if (trigger && menu) {
    menu.classList.remove("hidden");
    trigger.setAttribute("aria-expanded", "true");
    trigger.blur();
    document.getElementById("ub-command-tooltip-host")?.classList.add("hidden");
  }
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
