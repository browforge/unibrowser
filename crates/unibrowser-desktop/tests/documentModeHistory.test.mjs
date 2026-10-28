import assert from "node:assert/strict";
import test from "node:test";
import {
  isMarkdownFilePath,
  markdownKeepsMonacoEditor,
  markdownUsesMonacoLiveVisual,
  modeSwitchPreservesMonacoUndoStack,
} from "../src/documentModeHistory.ts";

test("markdown Visual uses live Monaco, not a fifth mode", () => {
  assert.equal(markdownUsesMonacoLiveVisual("wysiwyg", "/x/note.md"), true);
  assert.equal(markdownUsesMonacoLiveVisual("source", "/x/note.md"), false);
  assert.equal(markdownUsesMonacoLiveVisual("wysiwyg", "/x/doc.docx"), false);
});

test("source ↔ Visual on markdown keeps one Monaco undo stack", () => {
  const path = "/docs/sample.md";
  assert.equal(modeSwitchPreservesMonacoUndoStack("source", "wysiwyg", path), true);
  assert.equal(modeSwitchPreservesMonacoUndoStack("wysiwyg", "split", path), true);
  assert.equal(markdownKeepsMonacoEditor("wysiwyg", path), true);
  assert.equal(markdownKeepsMonacoEditor("source", path), true);
});

test("docx Visual still uses ProseMirror host", () => {
  assert.equal(isMarkdownFilePath("a.docx"), false);
  assert.equal(markdownKeepsMonacoEditor("wysiwyg", "a.docx"), false);
});
