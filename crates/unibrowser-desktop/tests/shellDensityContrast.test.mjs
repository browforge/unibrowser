import test from "node:test";
import assert from "node:assert/strict";

import {
  SHELL_DENSITY_TOKEN_PAIRS,
  contrastRatioForColors,
} from "../src/ui/shellContrast.ts";

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
