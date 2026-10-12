import test from "node:test";
import assert from "node:assert/strict";

import {
  formatCapabilityChip,
  pickActiveOpenWith,
  sortOpenWithOptions,
} from "../src/capabilityProviders.ts";

test("formatCapabilityChip shortens capability ids", () => {
  assert.equal(
    formatCapabilityChip("unibrowser.capability.open_text"),
    "open text",
  );
});

test("pickActiveOpenWith prefers the active provider", () => {
  const active = pickActiveOpenWith([
    { mod_name: "base-text", mod_title: "Plain text", active: false },
    { mod_name: "toy-markdown", mod_title: "Toy Markdown", active: true },
  ]);
  assert.equal(active?.mod_name, "toy-markdown");
});

test("sortOpenWithOptions orders by title", () => {
  const sorted = sortOpenWithOptions([
    { mod_name: "z", mod_title: "Zebra", active: false },
    { mod_name: "a", mod_title: "Alpha", active: true },
  ]);
  assert.equal(sorted[0].mod_title, "Alpha");
});
