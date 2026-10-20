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

export function historyEntryLabel(row: HistoryRow): string {
  switch (row.op_type) {
    case "base-text.block.set_style": {
      const style = row.payload?.style;
      return typeof style === "string" ? `Set style: ${style}` : "Set block style";
    }
    case "base-text.text.replace": {
      const text = previewText(row.payload?.text);
      return text ? `Typed: ${text}` : "Edit text";
    }
    case "base-text.mark.toggle":
      return `Toggle mark: ${String(row.payload?.mark ?? "mark")}`;
    case "base-text.opaque.replace":
      return "Replace opaque block";
    case "unibrowser.history.snapshot":
      return "Snapshot";
    default:
      return row.op_type;
  }
}

export function formatHistoryTimestamp(timestamp: string): string {
  const secs = parseEpochSeconds(timestamp);
  if (!secs) {
    return timestamp;
  }
  return new Date(secs * 1000).toLocaleString();
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
    const label =
      count > 1
        ? `Typed (${count} edits)`
        : historyEntryLabel(end);
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
