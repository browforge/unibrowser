import assert from "node:assert/strict";
import test from "node:test";

import {
  backgroundExceedsRaisedSurface,
  captureRectsOverlap,
  inputBorderEdgesUniform,
  rectWithinBounds,
} from "../src/docCaptureAsserts.ts";

test("timeline ruler and endpoint labels must stay inside the viewport", () => {
  const viewport = { left: 0, right: 1000, top: 0, bottom: 180 };
  const ruler = { left: 100, right: 1000, top: 0, bottom: 36 };
  const firstTick = { left: 100, right: 114, top: 18, bottom: 34 };
  const lastTick = { left: 976, right: 1000, top: 18, bottom: 34 };

  assert.equal(rectWithinBounds(ruler, viewport), true);
  assert.equal(rectWithinBounds(firstTick, viewport), true);
  assert.equal(rectWithinBounds(lastTick, viewport), true);
  assert.equal(
    rectWithinBounds({ ...lastTick, right: 1004 }, viewport),
    false,
  );
});

test("timeline marker flags must not overlap tick labels", () => {
  const marker = { left: 104, right: 138, top: 1, bottom: 16 };
  const tick = { left: 100, right: 114, top: 18, bottom: 34 };
  assert.equal(captureRectsOverlap(marker, tick), false);
  assert.equal(
    captureRectsOverlap({ ...marker, top: 20, bottom: 35 }, tick),
    true,
  );
});

test("dark capture surface guard rejects backgrounds above its tolerance", () => {
  assert.equal(backgroundExceedsRaisedSurface(0.04, 0.03), false);
  assert.equal(backgroundExceedsRaisedSurface(0.95, 0.03), true);
});

test("form control border guard treats matching edges as uniform", () => {
  const uniform = {
    borderTopColor: "rgb(100, 100, 100)",
    borderRightColor: "rgb(100, 100, 100)",
    borderBottomColor: "rgb(100, 100, 100)",
    borderLeftColor: "rgb(100, 100, 100)",
  };
  const beveled = {
    borderTopColor: "rgb(120, 120, 120)",
    borderRightColor: "rgb(120, 120, 120)",
    borderBottomColor: "rgb(80, 80, 80)",
    borderLeftColor: "rgb(80, 80, 80)",
  };
  assert.equal(inputBorderEdgesUniform(uniform), true);
  assert.equal(inputBorderEdgesUniform(beveled), false);
});
