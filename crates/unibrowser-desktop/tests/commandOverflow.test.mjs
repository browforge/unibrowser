import assert from "node:assert/strict";
import test from "node:test";

import {
  commandGroupBudgetForViewport,
  partitionGroupsByAvailableWidth,
} from "../src/ui/commandOverflow.ts";

const groups = [
  ["normal", ["Undo", "Redo"]],
  ["normal", ["Font", "12 pt"]],
  ["normal", ["Bold", "Italic", "Underline", "Strikethrough"]],
  ["normal", ["Subscript", "Superscript"]],
  ["normal", ["Colour"]],
  ["normal", ["Find"]],
  ["normal", ["Check"]],
  ["low", ["Export"]],
].map(([priority, labels], index) => ({
  priority,
  resolved: {
    controls: labels.map((label) => ({ label })),
  },
  id: `group-${index}`,
}));

for (const viewport of [800, 1024, 1280]) {
  test(`keeps at least six complete controls visible at ${viewport}px`, () => {
    const result = partitionGroupsByAvailableWidth(
      groups,
      commandGroupBudgetForViewport(viewport),
    );
    const visibleControls = result.visible.reduce(
      (count, group) => count + group.resolved.controls.length,
      0,
    );
    assert.ok(visibleControls >= 6, `only ${visibleControls} controls visible`);
    for (const group of groups) {
      assert.ok(
        result.visible.includes(group) !== result.overflow.includes(group),
        `${group.id} must be wholly visible or wholly overflowed`,
      );
    }
  });
}
