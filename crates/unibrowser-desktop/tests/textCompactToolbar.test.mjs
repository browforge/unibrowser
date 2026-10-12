import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";

test("compact text toolbar host exists in shell HTML", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.match(html, /id="text-compact-toolbar-host"/);
  assert.match(html, /id="text-mod-toolbar-host"/);
});

test("textCompactToolbar strips cm labels from icon buttons", () => {
  const source = readFileSync(
    new URL("../src/textCompactToolbar.ts", import.meta.url),
    "utf8",
  );
  assert.match(source, /label\.remove\(\)/);
  assert.match(source, /partitionGroupsByAvailableWidth/);
});
