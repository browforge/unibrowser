import type { UiCommandDto } from "../commandManager";

export interface CommandGroupModel {
  name: string;
  priority: "normal" | "low";
  commands: UiCommandDto[];
}

type GroupRule = {
  name: string;
  priority: "normal" | "low";
  match: (commandId: string) => boolean;
};

const SHARED_RULES: GroupRule[] = [
  {
    name: "History",
    priority: "normal",
    match: (id) => id.includes("undo") || id.includes("redo"),
  },
  {
    name: "Type",
    priority: "normal",
    match: (id) => id.includes("font") || id.includes("format"),
  },
  {
    name: "Find",
    priority: "normal",
    match: (id) => id.includes("find") && !id.includes("files"),
  },
  {
    name: "Proofing",
    priority: "low",
    match: (id) => id.includes("proof") || id.includes("spell"),
  },
  {
    name: "Export",
    priority: "low",
    match: (id) =>
      id.includes("export") || id.includes("print") || id.includes("download"),
  },
  {
    name: "Open",
    priority: "normal",
    match: (id) => id.includes("open") || id.includes("pick"),
  },
  {
    name: "Navigate",
    priority: "normal",
    match: (id) =>
      id.includes("back") || id.includes("forward") || id.includes("reload"),
  },
  {
    name: "View",
    priority: "normal",
    match: (id) => id.includes("zoom") || id.includes("fullscreen"),
  },
  {
    name: "File",
    priority: "normal",
    match: (id) => id.includes("save") || id.includes("text.save"),
  },
];

const TAB_EXTRA: Record<string, GroupRule[]> = {
  browse: [
    {
      name: "Browse",
      priority: "normal",
      match: (id) => id.includes("browse") || id.includes("folder"),
    },
  ],
  web: [
    {
      name: "Web",
      priority: "normal",
      match: (id) => id.includes("page") || id.includes("mute"),
    },
  ],
  image: [
    {
      name: "Adjust",
      priority: "normal",
      match: (id) => id.includes("image") || id.includes("crop"),
    },
  ],
  code: [
    {
      name: "Edit",
      priority: "normal",
      match: (id) => id.includes("comment") || id.includes("indent"),
    },
  ],
};

function rulesForTab(tabId: string): GroupRule[] {
  const extra = TAB_EXTRA[tabId] ?? [];
  return [...SHARED_RULES, ...extra];
}

export function buildCommandGroups(
  tabId: string,
  commands: UiCommandDto[],
): CommandGroupModel[] {
  const rules = rulesForTab(tabId);
  const buckets = new Map<string, CommandGroupModel>();
  const order: string[] = [];

  const ensure = (rule: GroupRule) => {
    if (!buckets.has(rule.name)) {
      buckets.set(rule.name, {
        name: rule.name,
        priority: rule.priority,
        commands: [],
      });
      order.push(rule.name);
    }
    return buckets.get(rule.name)!;
  };

  for (const cmd of commands) {
    const rule = rules.find((r) => r.match(cmd.command_id));
    if (rule) {
      ensure(rule).commands.push(cmd);
    } else {
      const fallback = ensure({
        name: "Tools",
        priority: "normal",
        match: () => true,
      });
      fallback.commands.push(cmd);
    }
  }

  return order
    .map((name) => buckets.get(name)!)
    .filter((g) => g.commands.length > 0);
}

export function partitionGroupsForOverflow(
  groups: CommandGroupModel[],
  maxVisible: number,
): { visible: CommandGroupModel[]; overflow: CommandGroupModel[] } {
  if (groups.length <= maxVisible) {
    return { visible: groups, overflow: [] };
  }
  const low = groups.filter((g) => g.priority === "low");
  const normal = groups.filter((g) => g.priority !== "low");
  if (low.length > 0) {
    const visible = normal.concat(low.slice(0, Math.max(0, maxVisible - normal.length)));
    const overflow = groups.filter((g) => !visible.includes(g));
    return { visible, overflow };
  }
  return {
    visible: groups.slice(0, maxVisible),
    overflow: groups.slice(maxVisible),
  };
}
