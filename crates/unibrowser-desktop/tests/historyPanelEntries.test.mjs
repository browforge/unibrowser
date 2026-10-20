import assert from "node:assert/strict";
import test from "node:test";
import {
  groupHistoryRows,
  historyEntryLabel,
  formatHistoryTimestamp,
} from "../src/historyPanelEntries.ts";

test("HIST-04 groups rapid text.replace on same block", () => {
  const rows = [
    {
      seq: 1,
      op_type: "base-text.text.replace",
      timestamp: "100Z",
      plugin_id: "base-text",
      payload: { block_id: "b1", text: "a" },
    },
    {
      seq: 2,
      op_type: "base-text.text.replace",
      timestamp: "101Z",
      plugin_id: "base-text",
      payload: { block_id: "b1", text: "b" },
    },
    {
      seq: 3,
      op_type: "base-text.block.set_style",
      timestamp: "500Z",
      plugin_id: "base-text",
      payload: { style: "heading_2" },
    },
  ];
  const grouped = groupHistoryRows(rows);
  assert.equal(grouped.length, 2);
  assert.equal(grouped[0].label, "Set style: heading_2");
  assert.match(grouped[1].label, /Typed \(2 edits\)/);
});

test("HIST-04 labels restore_at with target seq", () => {
  const label = historyEntryLabel({
    seq: 3,
    op_type: "unibrowser.history.restore_at",
    timestamp: "1Z",
    plugin_id: "base-text",
    payload: { target_seq: 1 },
  });
  assert.equal(label, "Restored to #1");
});

test("HIST-04 timestamp uses relative label for today", () => {
  const now = Date.UTC(2026, 9, 20, 2, 0, 0);
  const ts = String(Math.floor((now - 120_000) / 1000));
  const formatted = formatHistoryTimestamp(`${ts}Z`, now);
  assert.match(formatted.display, /ago|Just now/i);
  assert.ok(formatted.title.length > 8);
});
