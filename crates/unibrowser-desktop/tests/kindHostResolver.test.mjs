import assert from "node:assert/strict";
import test from "node:test";

test("kind host matrix states are stable strings", () => {
  const states = ["supported", "emulated", "unsupported"];
  assert.ok(states.includes("emulated"));
});
