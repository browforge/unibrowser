import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const srcDir = join(here, "../src");

const VIEWER_SOURCES = ["documentViewer.ts", "imageRasterViewer.ts"];

test("raster image viewer does not ship a duplicate command toolbar row", () => {
  for (const name of VIEWER_SOURCES) {
    const source = readFileSync(join(srcDir, name), "utf8");
    assert.equal(
      source.includes("image-toolbar"),
      false,
      `${name} must not reference .image-toolbar`,
    );
    assert.equal(
      /className\s*=\s*["'`]image-toolbar/.test(source),
      false,
      `${name} must not create an image-toolbar element`,
    );
  }
  const raster = readFileSync(join(srcDir, "imageRasterViewer.ts"), "utf8");
  assert.match(
    raster,
    /mountRasterImageViewer/,
    "raster path should mount imageRasterViewer",
  );
  assert.equal(
    raster.includes("Flip horizontal"),
    false,
    "flip labels belong in CM i18n, not a local toolbar",
  );
});
