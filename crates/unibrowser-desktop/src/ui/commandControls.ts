import { patchPictureInPictureCommand } from "../mediaPip";
import { bindCommandTooltip } from "./commandTooltip";
import { createLucideIcon, type LucideName } from "./lucideIcon";
import { getEditorFontSizePt } from "../textEditor";
import type { UiCommandDto } from "../commandManager";
import { announceCm } from "./cmA11y";
import { controlA11yState } from "./controlA11y";
export { partitionGroupsByAvailableWidth } from "./commandOverflow";

export type CapabilityDto =
  | { state: "supported" }
  | { state: "emulated"; provider_id: string; note: string }
  | { state: "unsupported"; reason: string }
  | { state: "hidden_by_user" }
  | { state: "locked"; reason: string };

export interface ResolvedControlDto {
  control_id: string;
  control_type: string;
  command_ids: string[];
  capability: CapabilityDto;
  selection: string;
  value: string | null;
  label: string;
  icon: string;
  shortcut: string | null;
  density_variant: string;
  group_id: string;
  family_id: string;
}

export interface ResolvedGroupDto {
  group_id: string;
  label: string;
  overflow_priority: string;
  controls: ResolvedControlDto[];
}

export interface ResolvedFamilyDto {
  family_id: string;
  label: string;
  overflow_priority: string;
  groups: ResolvedGroupDto[];
}

export function familiesToRenderGroups(
  families: ResolvedFamilyDto[],
): { name: string; priority: "normal" | "low"; commands: UiCommandDto[]; resolved: ResolvedGroupDto }[] {
  const out: {
    name: string;
    priority: "normal" | "low";
    commands: UiCommandDto[];
    resolved: ResolvedGroupDto;
  }[] = [];
  for (const family of families) {
    for (const group of family.groups) {
      const priority = group.overflow_priority === "low" ? "low" : "normal";
      const commands = group.controls.map(controlToLegacyCommand);
      out.push({ name: group.label, priority, commands, resolved: group });
    }
  }
  return out;
}

function controlToLegacyCommand(control: ResolvedControlDto): UiCommandDto {
  const commandId = control.command_ids[0] ?? control.control_id;
  const actionable = capabilityActionable(control.capability);
  const reason = capabilityReason(control.capability);
  return {
    command_id: commandId,
    title: control.label,
    icon: control.icon,
    enabled: actionable,
    reason,
  };
}

export function capabilityActionable(cap: CapabilityDto): boolean {
  return cap.state === "supported" || cap.state === "emulated";
}

export function capabilityReason(cap: CapabilityDto): string | null {
  switch (cap.state) {
    case "unsupported":
    case "locked":
      return cap.reason;
    case "emulated":
      return cap.note;
    default:
      return null;
  }
}

const ICON_MAP: Record<string, LucideName> = {
  undo: "undo-2",
  redo: "redo-2",
  search: "search",
  download: "download",
  "spell-check": "spell-check",
  "pen-line": "pen-line",
  type: "type",
  save: "save",
  bold: "bold",
  italic: "italic",
  underline: "underline",
  strikethrough: "strikethrough",
  subscript: "subscript",
  superscript: "superscript",
  palette: "palette",
  highlighter: "highlighter",
  stamp: "stamp",
  image: "image",
  extension: "package-open",
};

export function lucideForControlIcon(icon: string): LucideName {
  return ICON_MAP[icon] ?? "file";
}

type RunCommand = (commandId: string) => Promise<void>;

export function renderResolvedControl(
  control: ResolvedControlDto,
  onRun: RunCommand,
  chordByCommand: Map<string, string>,
  draggable: boolean,
): HTMLElement {
  const legacy = controlToLegacyCommand(control);
  const patched = patchPictureInPictureCommand(legacy);
  const a11y = controlA11yState(control);
  const actionable = a11y.actionable;
  const emulated = control.capability.state === "emulated";
  const locked = control.capability.state === "locked";
  const mixed = a11y.mixed;
  const a11yReason = a11y.reason;

  if (control.control_type === "value_stepper") {
    return renderValueStepper(control, patched, onRun, chordByCommand, actionable, emulated);
  }

  const btn = document.createElement("button");
  btn.type = "button";
  btn.dataset.controlId = control.control_id;
  const iconName = lucideForControlIcon(control.icon);
  btn.appendChild(createLucideIcon(iconName, 16, "ub-icon svg-icon"));
  const modControl =
    control.icon === "extension" || control.control_id.includes("cmd-proving");
  const label = document.createElement("span");
  label.className = "cm-cmd-label";
  label.textContent = patched.title;
  btn.appendChild(label);
  if (emulated) {
    const mark = document.createElement("span");
    mark.className = "cm-emulation-mark";
    mark.setAttribute("aria-hidden", "true");
    mark.title = capabilityReason(control.capability) ?? "";
    btn.appendChild(mark);
  }
  btn.setAttribute("role", control.control_type === "toggle" ? "switch" : "button");
  btn.setAttribute("aria-label", patched.title);
  if (a11y.ariaDisabled) {
    btn.setAttribute("aria-disabled", a11y.ariaDisabled);
  }
  if (a11yReason) {
    btn.setAttribute("aria-description", a11yReason);
  }
  if (a11y.ariaChecked) {
    btn.setAttribute("aria-checked", a11y.ariaChecked);
  }
  const chord = chordByCommand.get(patched.command_id) ?? null;
  if (chord) {
    btn.setAttribute("aria-keyshortcuts", chord);
  }
  btn.className = [
    "ui-command",
    control.control_type === "toggle" ? "toggle" : "",
    modControl ? "cm-mod-control" : "",
    !actionable ? "disabled" : "",
    emulated ? "emulated" : "",
    locked ? "locked" : "",
    mixed ? "mixed" : "",
  ]
    .filter(Boolean)
    .join(" ");
  bindCommandTooltip(btn, {
    title: patched.title,
    reason: a11yReason,
    chord,
    enabled: actionable,
  });
  if (actionable) {
    btn.onclick = () => {
      if (emulated && a11yReason) {
        announceCm(`${patched.title}, emulated, ${a11yReason}`);
      } else if (mixed) {
        announceCm(`${patched.title}, mixed selection`);
      }
      void onRun(patched.command_id);
    };
  } else if (a11yReason) {
    btn.addEventListener("click", (e) => {
      e.preventDefault();
      announceCm(a11y.announcement);
    });
  }
  if (draggable) {
    btn.draggable = true;
    btn.dataset.commandId = patched.command_id;
  }
  return btn;
}

function renderValueStepper(
  control: ResolvedControlDto,
  patched: UiCommandDto,
  onRun: RunCommand,
  chordByCommand: Map<string, string>,
  actionable: boolean,
  emulated: boolean,
): HTMLElement {
  const split = document.createElement("button");
  split.type = "button";
  split.className = `ui-command split-button value-stepper${emulated ? " emulated" : ""}`;
  split.dataset.controlId = control.control_id;
  split.appendChild(createLucideIcon(lucideForControlIcon(control.icon), 16, "ub-icon svg-icon"));
  const label = document.createElement("span");
  label.className = "cm-cmd-label";
  const pt = control.value ?? `${getEditorFontSizePt()} pt`;
  label.textContent = pt;
  split.appendChild(label);
  if (emulated) {
    const mark = document.createElement("span");
    mark.className = "cm-emulation-mark";
    mark.setAttribute("aria-hidden", "true");
    split.appendChild(mark);
  }
  const caret = document.createElement("span");
  caret.className = "split-caret";
  caret.textContent = "v";
  caret.setAttribute("aria-hidden", "true");
  split.appendChild(caret);
  const inc = control.command_ids.find((id) => id.includes("increase"));
  const dec = control.command_ids.find((id) => id.includes("decrease"));
  bindCommandTooltip(split, {
    title: patched.title,
    reason: patched.reason,
    chord: chordByCommand.get(patched.command_id) ?? null,
    enabled: actionable,
  });
  if (!actionable) {
    split.classList.add("disabled");
    split.setAttribute("aria-disabled", "true");
    const reason = capabilityReason(control.capability);
    if (reason) {
      split.setAttribute("aria-description", reason);
      split.addEventListener("click", (e) => {
        e.preventDefault();
        announceCm(`${control.label}, unavailable, ${reason}`);
      });
    }
  } else {
    split.addEventListener("click", (e) => {
      if ((e.target as HTMLElement).classList.contains("split-caret")) {
        e.stopPropagation();
        if (inc) {
          void onRun(inc);
        } else if (dec) {
          void onRun(dec);
        }
        return;
      }
      if (inc) {
        void onRun(inc);
      }
    });
  }
  return split;
}

export function renderResolvedGroups(
  container: HTMLElement,
  groups: ReturnType<typeof familiesToRenderGroups>,
  onRun: RunCommand,
  chordByCommand: Map<string, string>,
) {
  container.innerHTML = "";
  groups.forEach((group, index) => {
    if (index > 0) {
      const div = document.createElement("span");
      div.className = "cm-divider";
      div.setAttribute("aria-hidden", "true");
      container.appendChild(div);
    }
    const wrap = document.createElement("div");
    wrap.className = `cm-group priority-${group.priority}`;
    wrap.setAttribute("role", "group");
    wrap.setAttribute("aria-label", group.name);
    const tools = document.createElement("div");
    tools.className = "cm-group-tools";
    for (const control of group.resolved.controls) {
      tools.appendChild(renderResolvedControl(control, onRun, chordByCommand, true));
    }
    const name = document.createElement("span");
    name.className = "cm-group-name";
    name.textContent = group.name;
    wrap.appendChild(tools);
    wrap.appendChild(name);
    container.appendChild(wrap);
  });
}

export function partitionFamiliesForOverflow(
  groups: ReturnType<typeof familiesToRenderGroups>,
  maxVisible: number,
): { visible: ReturnType<typeof familiesToRenderGroups>; overflow: ReturnType<typeof familiesToRenderGroups> } {
  if (groups.length <= maxVisible) {
    return { visible: groups, overflow: [] };
  }
  const low = groups.filter((g) => g.priority === "low");
  const normal = groups.filter((g) => g.priority !== "low");
  const visible = normal.concat(low.slice(0, Math.max(0, maxVisible - normal.length)));
  const overflow = groups.filter((g) => !visible.includes(g));
  return { visible, overflow };
}
