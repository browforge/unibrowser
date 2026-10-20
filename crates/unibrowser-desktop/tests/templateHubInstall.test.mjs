import test from "node:test";
import assert from "node:assert/strict";

import { hubTemplateInstallPath } from "../src/templateHubInstall.ts";

const entry = {
  id: "unibrowser.template.hub.starter",
  title: "Hub starter board",
  version: "1.0.0",
  blurb: "fixture",
  fixture_id: "unibrowser.template.hub.starter",
  signed: true,
};

test("hub fixture installs offline even when offline mode is on", () => {
  assert.equal(hubTemplateInstallPath(entry, true), "offline");
  assert.equal(hubTemplateInstallPath(entry, false), "offline");
});

test("catalog entry without fixture needs network", () => {
  assert.equal(
    hubTemplateInstallPath({ ...entry, fixture_id: "" }, true),
    "needs-network",
  );
});
