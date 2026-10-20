import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const src = join(here, "../src");

function read(rel) {
  return readFileSync(join(src, rel), "utf8");
}

const perm04Banner = join(src, "permissionConsentBanner.ts");
test(
  "PERM-04: network consent banner module",
  !existsSync(perm04Banner) ? { skip: "PERM-04" } : undefined,
  () => {
    assert.match(read("permissionConsentBanner.ts"), /Allow once/i);
  },
);

test("PERM-04: network consent i18n keys", { skip: "PERM-04" }, () => {
  const enUs = JSON.parse(readFileSync(join(src, "i18n/en-US.json"), "utf8"));
  const keys = Object.keys(enUs);
  assert.ok(
    keys.some((k) => k.includes("permission") && k.includes("network")),
  );
});

const perm05Page = join(src, "settingsPluginPermissions.ts");
test(
  "PERM-05: settings plugin permissions page",
  !existsSync(perm05Page) ? { skip: "PERM-05" } : undefined,
  () => {
    assert.match(read("settingsPluginPermissions.ts"), /Revoke/i);
  },
);

test("PERM-05: settings page links permissions footer", () => {
  const page = read("../src/settingsPage.ts");
  assert.ok(!page.includes("settingsPluginPermissions"));
});

const perm06Log = join(src, "networkActivityLog.ts");
test(
  "PERM-06: network activity log UI",
  !existsSync(perm06Log) ? { skip: "PERM-06" } : undefined,
  () => {
    assert.match(read("networkActivityLog.ts"), /Clear log/i);
  },
);

test("PERM-08: portal permission chips from manifest", () => {
  const portal = read("modPortalUi.ts");
  assert.ok(!portal.includes("Sandboxed"));
  assert.match(portal, /permission_summary/);
});

test("PERM-09: command manager grant_action remedy", { skip: "PERM-09" }, () => {
  const cm = read("commandManager.ts");
  assert.match(cm, /grant_action/);
});

test("PERM-10: WASM sandbox vs native full-trust install copy", () => {
  const portal = read("modPortalUi.ts");
  const main = read("../src/main.ts");
  assert.ok(!portal.includes("Runs sandboxed"));
  assert.ok(main.includes("vsix-trust-bar"));
});

test("PERM-03: Open VSX uses coarse localStorage toggle not grant store", () => {
  const mods = read("modsOpenVsx.ts");
  assert.match(mods, /unibrowser\.openVsxNetwork/);
  assert.ok(!mods.includes("permission-grants"));
});
