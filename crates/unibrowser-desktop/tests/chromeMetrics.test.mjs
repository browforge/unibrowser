import assert from "node:assert/strict";
import test from "node:test";

import { validateChromeHeightBudget } from "../src/ui/chromeMetrics.ts";

test("collapsed chrome is title bar only (~40px)", () => {
  const errs = validateChromeHeightBudget({
    titleBarPx: 40,
    textCompactToolbarPx: 0,
    commandManagerPx: 0,
    totalChromePx: 40,
    cmCollapsed: true,
    cmDensity: "compact",
  });
  assert.deepEqual(errs, []);
});

test("expanded compact chrome is ~80px", () => {
  const errs = validateChromeHeightBudget({
    titleBarPx: 40,
    textCompactToolbarPx: 0,
    commandManagerPx: 40,
    totalChromePx: 80,
    cmCollapsed: false,
    cmDensity: "compact",
  });
  assert.deepEqual(errs, []);
});

test("text compact chrome allows compact row plus CM strip", () => {
  const errs = validateChromeHeightBudget({
    titleBarPx: 40,
    textCompactToolbarPx: 32,
    commandManagerPx: 40,
    totalChromePx: 112,
    cmCollapsed: false,
    cmDensity: "compact",
    textCompactVisible: true,
  });
  assert.deepEqual(errs, []);
});

test("collapsed chrome fails when CM row remains visible", () => {
  const errs = validateChromeHeightBudget({
    titleBarPx: 40,
    textCompactToolbarPx: 0,
    commandManagerPx: 40,
    totalChromePx: 80,
    cmCollapsed: true,
    cmDensity: "compact",
  });
  assert.ok(errs.length >= 2);
});
