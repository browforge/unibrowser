import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "src");
const panel = readFileSync(join(root, "historyPanel.ts"), "utf8");
const commands = readFileSync(join(root, "historyCommands.ts"), "utf8");

test("HIST-06 trim and squash command ids are defined", () => {
  assert.match(commands, /unibrowser\.history\.trimBefore/);
  assert.match(commands, /unibrowser\.history\.squashSelection/);
  assert.match(commands, /showInlineConfirm/);
  assert.match(commands, /document_history_trim_before/);
  assert.match(commands, /document_history_squash_range/);
});

test("HIST-04 jump uses append API and restore button", () => {
  assert.match(panel, /jumpHistoryToSeq/);
  assert.match(panel, /history-restore-btn/);
});
