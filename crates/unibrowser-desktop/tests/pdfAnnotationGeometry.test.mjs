import test from "node:test";
import assert from "node:assert/strict";

import { normalisedPdfRectToCss } from "../src/pdfAnnotationGeometry.ts";

test("normalised PDF annotation rect scales at 100% and 133%", () => {
  const rect = { x: 0.12, y: 0.18, w: 0.76, h: 0.04 };
  assert.deepEqual(normalisedPdfRectToCss(rect, 600, 800), {
    left: 72,
    top: 144,
    width: 456,
    height: 32,
  });
  const zoomed = normalisedPdfRectToCss(rect, 798, 1064);
  assert.ok(Math.abs(zoomed.left - 95.76) < 1e-9);
  assert.ok(Math.abs(zoomed.top - 191.52) < 1e-9);
  assert.ok(Math.abs(zoomed.width - 606.48) < 1e-9);
  assert.ok(Math.abs(zoomed.height - 42.56) < 1e-9);
});
