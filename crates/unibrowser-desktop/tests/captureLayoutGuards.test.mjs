import assert from "node:assert/strict";
import test from "node:test";

import {
  backgroundExceedsRaisedSurface,
  captureRectsOverlap,
  colourContrastRatio,
  focusRingMeetsRequirements,
  inputBorderEdgesUniform,
  knobVerticalOffsetCentered,
  rectWithinBounds,
  switchTrackSizeWithinTolerance,
  timelineEndpointClearsScrollbar,
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

test("timeline endpoint label clears the scrollbar safe inset at every width", () => {
  for (const viewportRight of [1024, 1280, 1920]) {
    assert.equal(timelineEndpointClearsScrollbar(viewportRight - 14, viewportRight), true);
    assert.equal(timelineEndpointClearsScrollbar(viewportRight - 8, viewportRight), false);
  }
});

test("dark capture surface guard rejects backgrounds above its tolerance", () => {
  assert.equal(backgroundExceedsRaisedSurface(0.04, 0.03), false);
  assert.equal(backgroundExceedsRaisedSurface(0.95, 0.03), true);
});

test("ub-switch track size tolerance is 32x18", () => {
  assert.equal(switchTrackSizeWithinTolerance(32, 18), true);
  assert.equal(switchTrackSizeWithinTolerance(33, 18), true);
  assert.equal(switchTrackSizeWithinTolerance(32, 28), false);
});

test("ub-switch knob vertical centre helper", () => {
  assert.equal(knobVerticalOffsetCentered(18, 2, 14), true);
  assert.equal(knobVerticalOffsetCentered(28, 2, 12), false);
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

test("focus ring guard requires two pixels and three-to-one contrast", () => {
  assert.ok(colourContrastRatio("rgb(23, 107, 148)", "rgb(255, 255, 255)") >= 3);
  assert.equal(
    focusRingMeetsRequirements(2, "rgb(23, 107, 148)", "rgb(255, 255, 255)"),
    true,
  );
  assert.equal(
    focusRingMeetsRequirements(1, "rgb(23, 107, 148)", "rgb(255, 255, 255)"),
    false,
  );
  assert.equal(
    focusRingMeetsRequirements(2, "rgb(190, 190, 190)", "rgb(255, 255, 255)"),
    false,
  );
});

test("focus tokens meet three-to-one contrast in every captured theme", () => {
  const pairs = [
    ["dark", "#83c7e6", "#232526"],
    ["light", "#176b94", "#ffffff"],
    ["dark high contrast", "#ffe566", "#000000"],
    ["light high contrast", "#7a5200", "#ffffff"],
    ["shared chrome", "#83c7e6", "#172532"],
  ];
  for (const [theme, ring, surrounding] of pairs) {
    assert.equal(
      focusRingMeetsRequirements(2, ring, surrounding),
      true,
      `${theme} focus ring must meet 2px / 3:1`,
    );
  }
});
