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

test("normal groups overflow after wider low-priority groups", () => {
  const important = {
    priority: "normal",
    resolved: { controls: [{ label: "Zoom" }] },
  };
  const low = {
    priority: "low",
    resolved: {
      controls: [
        { label: "Highlight" },
        { label: "Note" },
        { label: "Area" },
      ],
    },
  };
  const result = partitionGroupsByAvailableWidth([low, important], 110, 0);
  assert.deepEqual(result.visible, [important]);
  assert.deepEqual(result.overflow, [low]);
});

const browseGroups = [
  ["normal", ["New folder", "New text file"]],
  ["normal", ["Open", "Copy paths"]],
  ["normal", ["Copy to", "Move to"]],
  ["low", ["Rename", "Compress", "Properties", "Delete"]],
  ["low", ["Sort", "Direction", "Hidden files"]],
].map(([priority, labels], index) => ({
  priority,
  resolved: {
    controls: labels.map((label) => ({ label })),
  },
  id: `browse-group-${index}`,
}));

for (const viewport of [800, 1024, 1600]) {
  test(`Browse moves complete groups to overflow at ${viewport}px`, () => {
    const result = partitionGroupsByAvailableWidth(
      browseGroups,
      commandGroupBudgetForViewport(viewport),
    );
    for (const group of browseGroups) {
      assert.ok(
        result.visible.includes(group) !== result.overflow.includes(group),
        `${group.id} must be wholly visible or wholly overflowed`,
      );
    }
    assert.ok(
      result.visible.every((group) =>
        group.resolved.controls.every((control) => control.label.length > 1),
      ),
      "visible Browse controls must retain complete labels",
    );
  });
}
