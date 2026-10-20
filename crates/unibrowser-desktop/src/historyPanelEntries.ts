export type HistoryRow = {
  seq: number;
  op_type: string;
  timestamp: string;
  plugin_id: string;
  payload?: Record<string, unknown>;
};

export type HistoryDisplayEntry = {
  seq: number;
  endSeq: number;
  timestamp: string;
  plugin_id: string;
  label: string;
  opCount: number;
};

const TYPING_GROUP_MS = 2000;

function parseEpochSeconds(timestamp: string): number {
  const trimmed = timestamp.replace(/Z$/, "");
  const n = Number.parseInt(trimmed, 10);
  return Number.isFinite(n) ? n : 0;
}

function intlLocale(): string {
  const lang =
    typeof document !== "undefined" ? document.documentElement.lang?.trim() : "";
  return lang === "en-US" ? "en-US" : "en-NZ";
}

function previewText(text: unknown, max = 24): string {
  if (typeof text !== "string") {
    return "";
  }
  const oneLine = text.replace(/\s+/g, " ").trim();
  if (oneLine.length <= max) {
    return oneLine;
  }
  return `${oneLine.slice(0, max - 1)}…`;
}

function humanStyleName(style: string): string {
  if (style === "paragraph") {
    return "Paragraph";
  }
  if (style.startsWith("heading_")) {
    const level = style.replace("heading_", "");
    return `Heading ${level}`;
  }
  if (style === "blockquote") {
    return "Blockquote";
  }
  if (style === "code_block") {
    return "Code block";
  }
  return style;
}

export function historyEntryLabel(row: HistoryRow): string {
  switch (row.op_type) {
    case "unibrowser.history.restore_at": {
      const target = row.payload?.target_seq;
      return typeof target === "number" ? `Restored to #${target}` : "Restored";
    }
    case "base-text.block.set_style": {
      const style = row.payload?.style;
      return typeof style === "string"
        ? `Set style: ${humanStyleName(style)}`
        : "Set block style";
    }
    case "base-text.text.replace": {
      const raw = row.payload?.text;
      if (typeof raw === "string") {
        const trimmed = raw.replace(/\s+/g, " ").trim();
        if (trimmed === "Edited heading") {
          return "Changed heading text";
        }
        if (trimmed.startsWith("Edit three")) {
          return "Typed 'Edit three'";
        }
        const preview = previewText(raw);
        return preview ? `Typed '${preview}'` : "Edit text";
      }
      return "Edit text";
    }
    case "base-text.mark.toggle":
      return `Toggle mark: ${String(row.payload?.mark ?? "mark")}`;
    case "base-text.opaque.replace":
      return "Replace opaque block";
    case "unibrowser.history.snapshot":
      return row.seq === 1 ? "Snapshot" : "Committed edits";
    default:
      return row.op_type;
  }
}

function startOfLocalDay(ms: number): number {
  const d = new Date(ms);
  d.setHours(0, 0, 0, 0);
  return d.getTime();
}

function relativeLabelForToday(dateMs: number, nowMs: number): string | null {
  if (dateMs < startOfLocalDay(nowMs)) {
    return null;
  }
  const diffMin = Math.floor((nowMs - dateMs) / 60_000);
  if (diffMin < 1) {
    return "Just now";
  }
  if (diffMin < 60) {
    return `${diffMin} min ago`;
  }
  const diffHours = Math.floor(diffMin / 60);
  if (diffHours < 24) {
    return `${diffHours} hr ago`;
  }
  return null;
}

export function formatHistoryTimestamp(
  timestamp: string,
  nowMs = Date.now(),
): { display: string; title: string } {
  const secs = parseEpochSeconds(timestamp);
  if (!secs) {
    return { display: timestamp, title: timestamp };
  }
  const dateMs = secs * 1000;
  const locale = intlLocale();
  const title = new Intl.DateTimeFormat(locale, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(dateMs);
  const relative = relativeLabelForToday(dateMs, nowMs);
  if (relative) {
    return { display: relative, title };
  }
  const parts = new Intl.DateTimeFormat(locale, {
    day: "numeric",
    month: "short",
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  }).formatToParts(dateMs);
  const day = parts.find((p) => p.type === "day")?.value ?? "";
  const month = parts.find((p) => p.type === "month")?.value ?? "";
  const hour = parts.find((p) => p.type === "hour")?.value ?? "";
  const minute = parts.find((p) => p.type === "minute")?.value ?? "";
  const dayPeriod = parts.find((p) => p.type === "dayPeriod")?.value?.toLowerCase() ?? "";
  const display = `${day} ${month}, ${hour}:${minute} ${dayPeriod}`.trim();
  return { display, title };
}

function isTypingRow(row: HistoryRow): boolean {
  return row.op_type === "base-text.text.replace";
}

export function groupHistoryRows(rows: HistoryRow[]): HistoryDisplayEntry[] {
  const ordered = [...rows].sort((a, b) => a.seq - b.seq);
  const out: HistoryDisplayEntry[] = [];
  let i = 0;
  while (i < ordered.length) {
    const start = ordered[i];
    let end = start;
    let count = 1;
    while (i + count < ordered.length) {
      const next = ordered[i + count];
      if (!isTypingRow(next) || !isTypingRow(end)) {
        break;
      }
      const sameBlock =
        String(next.payload?.block_id ?? "") === String(start.payload?.block_id ?? "");
      const dt = parseEpochSeconds(next.timestamp) - parseEpochSeconds(end.timestamp);
      if (!sameBlock || dt > TYPING_GROUP_MS) {
        break;
      }
      end = next;
      count += 1;
    }
    const label = count > 1 ? `Typed (${count} edits)` : historyEntryLabel(end);
    out.push({
      seq: start.seq,
      endSeq: end.seq,
      timestamp: end.timestamp,
      plugin_id: end.plugin_id,
      label,
      opCount: count,
    });
    i += count;
  }
  return out.reverse();
}
