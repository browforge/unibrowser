import test from "node:test";
import assert from "node:assert/strict";

test("memory project URL prefix is stable for in-memory sessions", () => {
  const prefix = "unibrowser-memory://session/";
  const path = `${prefix}abc-123`;
  assert.ok(path.startsWith(prefix));
  assert.equal(path.slice(prefix.length), "abc-123");
});
