import test from "node:test";
import assert from "node:assert/strict";

import { pdfHighlightOverlayPointerEvents } from "../src/pdfAnnotationGeometry.ts";

test("arming Highlight lets the overlay receive a drag", () => {
  assert.equal(pdfHighlightOverlayPointerEvents(false), "none");
  assert.equal(pdfHighlightOverlayPointerEvents(true), "auto");
});
