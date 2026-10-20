import test from "node:test";
import assert from "node:assert/strict";

import { defaultFirstSaveFileName } from "../src/firstSaveDefaults.ts";

test("default first save archive name uses project title", () => {
  assert.equal(defaultFirstSaveFileName("My show"), "My show.unibrow");
});

test("default first save name falls back to untitled", () => {
  assert.match(defaultFirstSaveFileName(""), /Untitled project\.unibrow$/);
});
