import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const enUs = JSON.parse(
  readFileSync(join(here, "../src/i18n/en-US.json"), "utf8"),
);

test("PKG-03: ADR storage mode labels are in i18n", () => {
  assert.equal(enUs["pkg.storage.viewOnly"], "View only");
  assert.equal(enUs["pkg.storage.archive"], "Archive");
  assert.equal(enUs["pkg.storage.folder"], "Folder");
});

test("PKG-04: history path status copy is in i18n", () => {
  assert.match(
    String(enUs["pkg.storage.historyWillBeSaved"]),
    /History will be saved in/,
  );
});

test("PKG-05: unpack/pack strings in desktop source", () => {
  assert.equal(enUs["pkg.menu.unpack"], "Unpack to folder");
  assert.equal(enUs["pkg.menu.pack"], "Pack as .unibrow");
  const appMenu = readFileSync(join(here, "../src/appMenu.ts"), "utf8");
  assert.match(appMenu, /pkg\.menu\.unpack/);
  assert.match(appMenu, /pkg\.menu\.pack/);
  const families = readFileSync(
    join(here, "../../../crates/unibrowser-engine/src/command_system/families.rs"),
    "utf8",
  );
  assert.match(families, /Unpack to folder/);
  assert.match(families, /Pack as \.unibrow/);
});

test("PKG-06: large-project folder hint setting key present", () => {
  const keys = Object.keys(enUs);
  assert.equal(
    keys.some((k) => k.includes("folder") && k.includes("threshold")) ||
      keys.includes("settings.projectFolderHint"),
    true,
  );
});

test("package open mode dialog source defines capture helper", () => {
  const src = readFileSync(join(here, "../src/packageOpenModeDialog.ts"), "utf8");
  assert.match(src, /showPackageOpenModeCapture/);
  assert.match(src, /pkg-open-mode-overlay/);
});

test("history notice markup uses dismissible quiet button", () => {
  const src = readFileSync(join(here, "../src/packageStorageUi.ts"), "utf8");
  assert.match(src, /pkg-history-notice/);
  assert.match(src, /quiet-button/);
  assert.match(src, /pkg\.storage\.historyWillBeSaved/);
});
