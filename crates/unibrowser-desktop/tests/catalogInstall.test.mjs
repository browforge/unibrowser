import test from "node:test";
import assert from "node:assert/strict";

import { catalogInstallPath } from "../src/catalogInstall.ts";

test("Open VSX install with the network off stays on the card", () => {
  assert.equal(
    catalogInstallPath({ open_vsx_id: "dbaeumer.vscode-eslint" }, false),
    "network-off",
  );
});

test("a local ubmod fixture installs without the network", () => {
  assert.equal(
    catalogInstallPath({ ubmod_id: "unibrowser.base-cad" }, false),
    "local",
  );
});
