import assert from "node:assert/strict";
import test from "node:test";

import {
  commandGroupBudgetForViewport,
  commandGroupBudgetFromViewport,
  partitionGroupsByAvailableWidth,
} from "../src/ui/commandOverflow.ts";

const markdownGroups = [
  ["normal", ["Undo", "Redo"]],
  ["normal", ["Font", "12 pt"]],
  ["normal", ["Bold", "Italic", "Underline", "Strikethrough"]],
  ["normal", ["Subscript", "Superscript", "Proving chip"]],
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

function exportVisible(result) {
  return result.visible.some((group) =>
    group.resolved.controls.some((control) => control.label === "Export"),
  );
}

for (const viewport of [960, 1024]) {
  test(`markdown CM keeps Export in overflow at ${viewport}px`, () => {
    const budget = commandGroupBudgetFromViewport(viewport, 0);
    const result = partitionGroupsByAvailableWidth(markdownGroups, budget);
    assert.equal(
      exportVisible(result),
      false,
      `Export must not remain visible at ${viewport}px (budget ${budget})`,
    );
    assert.ok(
      result.overflow.some((group) =>
        group.resolved.controls.some((control) => control.label === "Export"),
      ),
    );
  });
}

test("runtime budget caps optimistic groups clientWidth", () => {
  const capped = commandGroupBudgetFromViewport(968, 900);
  const viewportOnly = commandGroupBudgetFromViewport(968, 0);
  assert.equal(capped, viewportOnly);
});
