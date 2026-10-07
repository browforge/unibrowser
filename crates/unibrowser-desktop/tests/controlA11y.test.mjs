import test from "node:test";
import assert from "node:assert/strict";

import { controlA11yState } from "../src/ui/controlA11y.ts";

function control(capability, selection = "off") {
  return {
    control_id: "test.control",
    control_type: "toggle",
    command_ids: ["test.command"],
    capability,
    selection,
    value: null,
    label: "Subscript",
    icon: "subscript",
    shortcut: null,
    density_variant: "compact",
    group_id: "test.group",
    family_id: "test.family",
  };
}

test("emulated control announces its approximation", () => {
  const state = controlA11yState(
    control({ state: "emulated", provider_id: "test", note: "Uses inline HTML" }),
  );
  assert.equal(state.actionable, true);
  assert.equal(state.announcement, "Subscript, emulated, Uses inline HTML");
});

test("unsupported control remains focusable but is aria-disabled with reason", () => {
  const state = controlA11yState(
    control({ state: "unsupported", reason: "Plain text has no baseline metadata" }),
  );
  assert.equal(state.actionable, false);
  assert.equal(state.ariaDisabled, "true");
  assert.match(state.announcement, /unavailable, Plain text/);
});

test("mixed selection exposes aria mixed and announcement", () => {
  const state = controlA11yState(control({ state: "supported" }, "mixed"));
  assert.equal(state.ariaChecked, "mixed");
  assert.match(state.announcement, /mixed selection/);
});
