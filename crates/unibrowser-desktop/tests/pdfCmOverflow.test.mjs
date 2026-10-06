import assert from "node:assert/strict";
import test from "node:test";

import {
  commandGroupBudgetForViewport,
  partitionGroupsByAvailableWidth,
} from "../src/ui/commandOverflow.ts";

const pdfGroups = [
  ["normal", ["Select"]],
  ["normal", ["Highlight", "Note", "Area"]],
  ["normal", ["3 / 12"]],
  ["low", ["90%"]],
  ["low", ["Export PDF"]],
].map(([priority, labels], index) => ({
  priority,
  resolved: {
    controls: labels.map((label) => ({ label })),
  },
  id: `pdf-group-${index}`,
}));

for (const viewport of [800, 1024, 1280]) {
  test(`pdf command manager keeps six controls visible at ${viewport}px`, () => {
    const result = partitionGroupsByAvailableWidth(
      pdfGroups,
      commandGroupBudgetForViewport(viewport),
    );
    const visibleControls = result.visible.reduce(
      (count, group) => count + group.resolved.controls.length,
      0,
    );
    assert.ok(visibleControls >= 6, `only ${visibleControls} controls visible`);
    for (const group of pdfGroups) {
      assert.ok(
        result.visible.includes(group) !== result.overflow.includes(group),
        `${group.id} must be wholly visible or wholly overflowed`,
      );
    }
  });
}
