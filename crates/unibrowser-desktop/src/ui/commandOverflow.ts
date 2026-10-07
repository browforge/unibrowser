export interface OverflowControl {
  label: string;
}

export interface OverflowGroup {
  priority: "normal" | "low";
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

function subsetScore(groups: OverflowGroup[], source: OverflowGroup[]): number {
  const controls = groups.reduce(
    (count, group) => count + group.resolved.controls.length,
    0,
  );
  const normalGroups = groups.filter((group) => group.priority === "normal").length;
  const stableOrder = groups.reduce(
    (score, group) => score + (source.length - source.indexOf(group)),
    0,
  );
  return controls * 10_000 + normalGroups * 100 + stableOrder;
}

export function partitionGroupsByAvailableWidth<T extends OverflowGroup>(
  groups: T[],
  availableWidth: number,
  reserveOverflow = 34,
): { visible: T[]; overflow: T[] } {
  if (!groups.length || availableWidth <= 0) {
    return { visible: groups, overflow: [] };
  }
  if (subsetWidth(groups) <= availableWidth) {
    return { visible: groups, overflow: [] };
  }

  const budget = Math.max(0, availableWidth - reserveOverflow - WIDTH_SAFETY_MARGIN);
  let best: T[] = [];
  let bestScore = -1;
  const combinations = 1 << groups.length;
  for (let mask = 1; mask < combinations; mask += 1) {
    const candidate = groups.filter((_, index) => (mask & (1 << index)) !== 0);
    if (subsetWidth(candidate) > budget) {
      continue;
    }
    const score = subsetScore(candidate, groups);
    if (score > bestScore) {
      best = candidate;
      bestScore = score;
    }
  }

  const visible = best.length > 0 ? best : [groups[0]];
  return {
    visible,
    overflow: groups.filter((group) => !visible.includes(group)),
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
