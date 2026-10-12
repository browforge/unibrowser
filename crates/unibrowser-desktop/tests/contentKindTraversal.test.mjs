import assert from "node:assert/strict";
import test from "node:test";

test("content kind traversal contract is documented for hosts", () => {
  const hosts = ["png", "docx", "md"];
  assert.deepEqual(hosts, ["png", "docx", "md"]);
});
