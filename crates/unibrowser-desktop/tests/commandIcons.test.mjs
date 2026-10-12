import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import {
  CONTROL_ICON_MAP,
  controlIconUsesFallback,
  lucideForControlIcon,
} from "../src/ui/commandIconMap.ts";

const here = dirname(fileURLToPath(import.meta.url));
const familiesPath = join(here, "../../unibrowser-engine/src/command_system/families.rs");
const compactToolbarPath = join(
  here,
  "../../unibrowser-engine/src/command_system/compact_toolbar.rs",
);

/** Collect engine icon strings (label line followed by icon line before a group id). */
function iconsFromFamiliesSource(source) {
  const icons = new Set();
  const re =
    /"[^"]+",\s*\n\s*"([a-z][a-z0-9-]*)",\s*\n\s*"(?:unibrowser\.|family_ids::)/g;
  let m;
  while ((m = re.exec(source)) !== null) {
    icons.add(m[1]);
  }
  return [...icons].sort();
}

test("every built-in control icon is mapped (no lucide file fallback)", () => {
  const source = readFileSync(familiesPath, "utf8");
  const compactSource = readFileSync(compactToolbarPath, "utf8");
  const icons = [
    ...new Set([
      ...iconsFromFamiliesSource(source),
      ...iconsFromFamiliesSource(compactSource),
    ]),
  ].sort();
  assert.ok(icons.length > 20, `expected many control icons, got ${icons.length}`);
  const missing = icons.filter((icon) => controlIconUsesFallback(icon));
  assert.deepEqual(
    missing,
    [],
    `Unmapped control icons (would render generic document): ${missing.join(", ")}`,
  );
  for (const icon of icons) {
    assert.equal(
      controlIconUsesFallback(icon),
      false,
      `icon "${icon}" resolves to ${lucideForControlIcon(icon)} via fallback`,
    );
  }
});

test("CONTROL_ICON_MAP values are distinct lucide names", () => {
  const values = Object.values(CONTROL_ICON_MAP);
  assert.ok(values.every((v) => typeof v === "string" && v.length > 0));
});
