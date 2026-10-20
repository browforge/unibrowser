import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const enUs = JSON.parse(
  readFileSync(join(here, "../src/i18n/en-US.json"), "utf8"),
);

test("PKG-03: ADR storage mode labels are not in i18n yet", () => {
  const keys = Object.keys(enUs);
  assert.equal(
    keys.some((k) => enUs[k] === "View only"),
    false,
    "view-only storage mode string missing",
  );
  assert.equal(
    keys.some((k) => String(enUs[k]).includes("History will be saved in")),
    false,
    "history path status copy missing (PKG-04)",
  );
});

test("PKG-05: unpack/pack strings absent from desktop source", () => {
  const srcRoot = join(here, "../src");
  const appMenu = readFileSync(join(srcRoot, "appMenu.ts"), "utf8");
  const main = readFileSync(join(srcRoot, "main.ts"), "utf8");
  const hay = appMenu + main;
  assert.equal(hay.includes("Unpack to folder"), false);
  assert.equal(hay.includes("Pack as .unibrow"), false);
});

test("PKG-06: large-project folder hint setting keys missing", () => {
  const keys = Object.keys(enUs);
  assert.equal(
    keys.some((k) => k.includes("folder") && k.includes("threshold")),
    false,
  );
});
