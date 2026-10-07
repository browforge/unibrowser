import test from "node:test";
import assert from "node:assert/strict";

import {
  SHELL_DENSITY_TOKEN_PAIRS,
  contrastRatioForColors,
  rectsOverlap,
} from "../src/ui/shellContrast.ts";

test("rectsOverlap detects intersection", () => {
  assert.equal(rectsOverlap({ left: 0, top: 0, right: 10, bottom: 10 }, { left: 5, top: 5, right: 15, bottom: 15 }), true);
  assert.equal(rectsOverlap({ left: 0, top: 0, right: 10, bottom: 10 }, { left: 20, top: 0, right: 30, bottom: 10 }), false);
});

test("shell density workshop token pairs meet WCAG AA (4.5:1) for small text", () => {
  for (const pair of SHELL_DENSITY_TOKEN_PAIRS) {
    const ratio = contrastRatioForColors(pair.foreground, pair.background);
    assert.ok(ratio !== null, `${pair.theme}/${pair.id}: could not parse colours`);
    assert.ok(
      ratio >= pair.minRatio,
      `${pair.theme}/${pair.id}: contrast ${ratio.toFixed(2)} < ${pair.minRatio}`,
    );
  }
});
