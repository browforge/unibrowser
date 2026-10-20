import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "src");
const source = readFileSync(join(root, "historyPanel.ts"), "utf8");

test("HIST-04 panel lists ops from document_history_list", () => {
  assert.match(source, /listDocumentHistory/);
  assert.match(source, /history-panel-table/);
});

test("HIST-04 rollback and squash entry points are not wired in panel yet", () => {
  assert.doesNotMatch(source, /rollback|truncate_head|squash|clean before sharing/i);
  assert.doesNotMatch(source, /addEventListener\([^)]*click[^)]*seq/i);
});
