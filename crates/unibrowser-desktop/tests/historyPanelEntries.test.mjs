import assert from "node:assert/strict";
import test from "node:test";
import { groupHistoryRows, historyEntryLabel } from "../src/historyPanelEntries.ts";

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

test("HIST-04 labels include plugin op summary", () => {
  const label = historyEntryLabel({
    seq: 1,
    op_type: "base-text.text.replace",
    timestamp: "1Z",
    plugin_id: "base-text",
    payload: { text: "Hello" },
  });
  assert.equal(label, "Typed: Hello");
});
