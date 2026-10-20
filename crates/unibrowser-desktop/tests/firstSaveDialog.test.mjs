import test from "node:test";
import assert from "node:assert/strict";

import {
  defaultFirstSaveFileName,
  formatFirstSavePathPreview,
} from "../src/firstSaveDefaults.ts";

test("default first save archive name uses project title", () => {
  assert.equal(defaultFirstSaveFileName("My show"), "My show.unibrow");
});

test("default first save name falls back to untitled", () => {
  assert.match(defaultFirstSaveFileName(""), /Untitled project\.unibrow$/);
});

test("path preview joins location and archive file", () => {
  assert.equal(
    formatFirstSavePathPreview("/home/me/Documents/Unibrowser", "archive", "Demo.unibrow"),
    "/home/me/Documents/Unibrowser/Demo.unibrow",
  );
});

test("path preview folder mode ends with slash", () => {
  assert.equal(
    formatFirstSavePathPreview("/home/me/Documents/Unibrowser", "folder", "Demo"),
    "/home/me/Documents/Unibrowser/Demo/",
  );
});
