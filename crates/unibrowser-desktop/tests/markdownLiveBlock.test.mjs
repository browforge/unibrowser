import assert from "node:assert/strict";
import test from "node:test";
import { EditorState, TextSelection } from "prosemirror-state";
import { history, undo } from "prosemirror-history";
import {
  applyLiveInputRules,
  buildLiveBlockDecorations,
  createMinimalLiveDoc,
  liveBlockTargetAtSelection,
  markdownLiveBlockPlugins,
} from "../src/markdownLiveBlock.ts";
import { richTextSchema } from "../src/richTextProseMirror.ts";

function markerCount(decoSet) {
  return decoSet?.find()?.length ?? 0;
}

test("live markers only on the active non-empty block", () => {
  const doc = createMinimalLiveDoc();
  const emptyPos = doc.content.size - 1;
  const emptyDecos = buildLiveBlockDecorations(doc, { from: emptyPos, to: emptyPos }, true);
  assert.equal(markerCount(emptyDecos), 0);

  const headingPos = 3;
  const headingDecos = buildLiveBlockDecorations(doc, { from: headingPos, to: headingPos }, true);
  assert.ok(markerCount(headingDecos) >= 1);
  assert.equal(liveBlockTargetAtSelection(doc, headingPos)?.kind, "heading");

  const boldPos = doc.content.size - 5;
  const boldDecos = buildLiveBlockDecorations(doc, { from: boldPos, to: boldPos }, true);
  assert.ok(markerCount(boldDecos) >= 2);
});

test("disabled live plugin emits no decorations", () => {
  const doc = createMinimalLiveDoc();
  const decos = buildLiveBlockDecorations(doc, { from: 3, to: 3 }, false);
  assert.equal(markerCount(decos), 0);
});

test("heading marker input rule and undo share one history stack", () => {
  const plugins = [...markdownLiveBlockPlugins(() => true), history()];
  let state = EditorState.create({
    doc: richTextSchema.node("doc", null, [
      richTextSchema.node("paragraph", { blockId: "p" }, [richTextSchema.text("## Title!")]),
    ]),
    plugins,
  });
  state = applyLiveInputRules(state);
  assert.equal(state.doc.firstChild?.type.name, "heading");
  assert.equal(state.doc.firstChild?.textContent, "Title!");

  const undone = undo(state, (tr) => {
    state = state.apply(tr);
  });
  assert.ok(undone);
  assert.equal(state.doc.firstChild?.type.name, "paragraph");
  assert.match(state.doc.firstChild?.textContent ?? "", /^## Title/);
});

test("link marks do not emit live marker widgets", () => {
  const link = richTextSchema.marks.link.create({ href: "https://example.com" });
  const bold = richTextSchema.marks.bold.create();
  const doc = richTextSchema.node("doc", null, [
    richTextSchema.node("paragraph", { blockId: "p" }, [
      richTextSchema.text("See "),
      richTextSchema.text("docs", [link]),
      richTextSchema.text(" after "),
      richTextSchema.text("bold", [bold]),
    ]),
  ]);
  const boldPos = doc.content.size - 3;
  const decos = buildLiveBlockDecorations(doc, { from: boldPos, to: boldPos }, true);
  assert.equal(markerCount(decos), 2);
});

test("bold marker input rule converts trailing ** pair", () => {
  let state = EditorState.create({
    doc: richTextSchema.node("doc", null, [
      richTextSchema.node("paragraph", { blockId: "p" }, [richTextSchema.text("**x**")]),
    ]),
    plugins: markdownLiveBlockPlugins(() => true),
  });
  state = applyLiveInputRules(state);
  const para = state.doc.firstChild;
  assert.equal(para?.textContent, "x");
  assert.equal(para?.firstChild?.marks[0]?.type.name, "bold");
});
