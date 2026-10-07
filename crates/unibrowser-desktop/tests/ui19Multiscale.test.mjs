import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "../../..");
const script = join(root, "scripts/capture-ui-19-multiscale.sh");
const outDir = join(root, "docs/screenshots/ui-19");

const EXPECTED_SCALES = [100, 125, 150, 200];

test("UI-19 multi-scale capture script lists required scale tokens", () => {
  const source = readFileSync(script, "utf8");
  assert.match(source, /for scale in 100 125 150 200/, "script must loop all UI-19 scales");
  assert.match(source, /scale-\$\{scale\}/, "script must pass scale token into demo state");
  assert.match(source, /--doc-capture/, "script must use doc-capture guards");
});

test("UI-19 multi-scale PNG matrix exists after capture (optional in CI)", () => {
  const matrixComplete = EXPECTED_SCALES.every((scale) =>
    existsSync(join(outDir, `high-contrast-light-${scale}-1280.png`)),
  );
  if (!matrixComplete) {
    return;
  }
  for (const theme of ["light", "dark"]) {
    for (const scale of EXPECTED_SCALES) {
      const path = join(outDir, `high-contrast-${theme}-${scale}-1280.png`);
      assert.ok(existsSync(path), `missing ${path}`);
    }
  }
});
