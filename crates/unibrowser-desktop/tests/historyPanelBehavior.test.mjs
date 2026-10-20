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

test("HIST-04 panel wires jump, labels, toolbar, and restore action", () => {
  assert.match(source, /groupHistoryRows/);
  assert.match(source, /formatHistoryTimestamp/);
  assert.match(source, /jumpHistoryToSeq/);
  assert.match(source, /history-panel-toolbar/);
  assert.match(source, /history-restore-btn/);
  assert.match(source, /history-action-trim/);
});

test("bottom panel empty-state keeps History tab open", () => {
  const panelSource = readFileSync(join(root, "scriptOutputPanel.ts"), "utf8");
  assert.match(panelSource, /bottom-panel-history-pane/);
  assert.match(panelSource, /historyOpen/);
});
