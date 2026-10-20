import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

test("HIST-03 banner module wires replay status command", () => {
  const src = readFileSync(join(root, "src/historyPluginBanner.ts"), "utf8");
  assert.match(src, /document_history_replay_status/);
  assert.match(src, /history-plugin-banner/);
  assert.match(src, /historyPluginBanner\.install/);
});

test("HIST-03 i18n names plugin and version in banner lead", () => {
  const en = JSON.parse(readFileSync(join(root, "src/i18n/en-US.json"), "utf8"));
  assert.match(en["historyPluginBanner.lead"], /\{plugin\}/);
  assert.match(en["historyPluginBanner.lead"], /\{version\}/);
});

test("HIST-03 demo token seeds missing-plugin capture", () => {
  const demo = readFileSync(join(root, "src/demoState.ts"), "utf8");
  assert.match(demo, /doc-history-missing-plugin/);
  assert.match(demo, /document_history_seed_missing_plugin_capture/);
});
