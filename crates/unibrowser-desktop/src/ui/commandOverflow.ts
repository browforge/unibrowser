export interface OverflowControl {
  label: string;
}

export interface OverflowGroup {
  priority: "critical" | "primary" | "normal" | "low" | "custom";
  resolved: { controls: OverflowControl[] };
}

const GROUP_PADDING = 8;
const DIVIDER_WIDTH = 10;
/** Slack so estimated widths never underestimate real CM buttons (never-clip rule). */
const WIDTH_SAFETY_MARGIN = 8;

export function estimateCommandWidth(label: string): number {
  return Math.max(46, 26 + label.length * 6.8);
}

export function estimateGroupWidth(group: OverflowGroup): number {
  return (
    group.resolved.controls.reduce(
      (width, control) => width + estimateCommandWidth(control.label),
      0,
    ) + GROUP_PADDING
  );
}

function subsetWidth(groups: OverflowGroup[]): number {
  return groups.reduce(
    (width, group, index) =>
      width + estimateGroupWidth(group) + (index > 0 ? DIVIDER_WIDTH : 0),
    0,
  );
}

const PRIORITY_ORDER: Record<OverflowGroup["priority"], number> = {
  critical: 0,
  primary: 1,
  normal: 2,
  low: 3,
  custom: 4,
};

export function groupsInDeclaredPriorityOrder<T extends OverflowGroup>(groups: T[]): T[] {
  return groups
    .map((group, sourceIndex) => ({ group, sourceIndex }))
    .sort(
      (a, b) =>
        PRIORITY_ORDER[a.group.priority] - PRIORITY_ORDER[b.group.priority] ||
        a.sourceIndex - b.sourceIndex,
    )
    .map(({ group }) => group);
}

export function partitionGroupsByAvailableWidth<T extends OverflowGroup>(
  groups: T[],
  availableWidth: number,
  reserveOverflow = 34,
): { visible: T[]; overflow: T[] } {
  if (!groups.length || availableWidth <= 0) {
    return { visible: groups, overflow: [] };
  }
  const ordered = groupsInDeclaredPriorityOrder(groups);
  if (subsetWidth(ordered) <= availableWidth) {
    return { visible: ordered, overflow: [] };
  }

  const budget = Math.max(0, availableWidth - reserveOverflow - WIDTH_SAFETY_MARGIN);
  const visible: T[] = [];
  for (const group of ordered) {
    const candidate = visible.concat(group);
    if (subsetWidth(candidate) > budget) {
      break;
    }
    visible.push(group);
  }

  if (visible.length === 0) {
    visible.push(ordered[0]);
  }
  return {
    visible,
    overflow: ordered.slice(visible.length),
  };
}

export function commandGroupBudgetForViewport(viewportWidth: number): number {
  return Math.max(240, viewportWidth - 300);
}

/** Width reserved for CM context label, pin, overflow, and collapse controls. */
export function commandManagerChromeReserve(viewportWidth: number): number {
  return Math.min(520, Math.max(360, viewportWidth * 0.42));
}

export function commandGroupBudgetFromViewport(
  viewportWidth: number,
  groupsClientWidth: number,
): number {
  const chromeReserve = commandManagerChromeReserve(viewportWidth);
  const fromViewport = Math.max(0, viewportWidth - chromeReserve);
  if (groupsClientWidth > 0) {
    return Math.min(groupsClientWidth, fromViewport);
  }
  return fromViewport;
}
