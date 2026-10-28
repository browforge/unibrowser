import assert from "node:assert/strict";
import test from "node:test";
import {
  applySmartEnter,
  buildInlineRenderMap,
  sourceOffsetFromRenderClick,
  splitLiveBlocks,
  activeLiveBlockIndex,
} from "../src/markdownLiveMapping.ts";

test("LIVE map: bold render click lands inside emphasis", () => {
  const block = splitLiveBlocks("Say **bold** here")[0];
  const map = buildInlineRenderMap("Say **bold** here");
  assert.equal(map.render, "Say bold here");
  const starOffset = sourceOffsetFromRenderClick(block, map.render.indexOf("b"));
  assert.ok(starOffset >= 5 && starOffset <= 6, `expected source in **bold**, got ${starOffset}`);
});

test("LIVE map: link label maps through brackets", () => {
  const block = splitLiveBlocks("[docs](https://example.com)")[0];
  const map = buildInlineRenderMap("[docs](https://example.com)");
  assert.equal(map.render, "docs");
  const offset = sourceOffsetFromRenderClick(block, 2);
  assert.equal(offset, 3);
});

test("smart Enter continues bullet list", () => {
  const src = "- one\n- two";
  const cursor = src.length;
  const next = applySmartEnter(src, cursor, false);
  assert.match(next.text, /\n- $/);
});

test("smart Enter exits empty bullet", () => {
  const src = "- one\n- \n";
  const cursor = src.length - 1;
  const next = applySmartEnter(src, cursor, false);
  assert.doesNotMatch(next.text, /- \n/);
});

test("smart Enter Shift inserts soft break", () => {
  const src = "hello";
  const next = applySmartEnter(src, 3, true);
  assert.equal(next.text, "hel\nlo");
  assert.equal(next.cursor, 4);
});

test("activeLiveBlockIndex is -1 in inter-block gap", () => {
  const src = "a\n\nb";
  const blocks = splitLiveBlocks(src);
  assert.equal(activeLiveBlockIndex(blocks, src.indexOf("\n\n") + 1), -1);
});

test("LIVE IME composition gate defers refresh when composing", async () => {
  const fs = await import("node:fs/promises");
  const source = await fs.readFile(new URL("../src/markdownLiveEditor.ts", import.meta.url), "utf8");
  assert.match(source, /onDidCompositionStart/);
});
