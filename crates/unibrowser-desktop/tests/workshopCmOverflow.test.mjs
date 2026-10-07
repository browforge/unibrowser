import assert from "node:assert/strict";
import test from "node:test";

import {
  commandGroupBudgetForViewport,
  partitionGroupsByAvailableWidth,
} from "../src/ui/commandOverflow.ts";

const workshopGroups = [
  ["primary", ["Add asset", "Add title"]],
  ["primary", ["Split", "Delete clip"]],
  ["primary", ["Export"]],
  ["normal", ["Snap", "Marker", "Timeline zoom"]],
  ["normal", ["Preview", "LIVE", "Record"]],
].map(([priority, labels], index) => ({
  priority,
  resolved: {
    controls: labels.map((label) => ({ label })),
  },
  id: `workshop-${index}`,
}));

test("workshop Export stays visible at 1280", () => {
  const result = partitionGroupsByAvailableWidth(
    workshopGroups,
    commandGroupBudgetForViewport(1280),
  );
  const visible = result.visible.flatMap((group) =>
    group.resolved.controls.map((control) => control.label),
  );
  assert.ok(visible.includes("Export"), `Export missing from ${visible.join(", ")}`);
  assert.ok(visible.includes("Add title"));
});
