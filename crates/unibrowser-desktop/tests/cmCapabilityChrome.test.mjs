import assert from "node:assert/strict";
import test from "node:test";

import {
  capabilityActionableForState,
  uiCommandChromeClassList,
} from "../src/ui/cmControlClasses.ts";

function cadClasses(controlId, state) {
  const actionable = capabilityActionableForState(state);
  return uiCommandChromeClassList({
    controlType: "button",
    capabilityState: state,
    actionable,
    emulated: false,
    locked: false,
    mixed: false,
    modControl: controlId.includes("cmd-proving"),
  });
}

test("CAD supported view controls are not muted (no unsupported class)", () => {
  for (const id of [
    "unibrowser.control.cad.fitView",
    "unibrowser.control.cad.zoomIn",
    "unibrowser.control.cad.zoomOut",
  ]) {
    const classes = cadClasses(id, "supported");
    assert.equal(classes.includes("unsupported"), false, id);
    assert.equal(classes.includes("disabled"), false, id);
    assert.equal(classes.includes("cm-cap-supported"), true, id);
  }
});

test("CAD unsupported controls use unsupported class only (not disabled)", () => {
  const classes = cadClasses("unibrowser.control.cad.measure", "unsupported");
  assert.equal(classes.includes("unsupported"), true);
  assert.equal(classes.includes("disabled"), false);
});

test("image fit control is supported chrome (no unsupported class)", () => {
  const classes = uiCommandChromeClassList({
    controlType: "button",
    capabilityState: "supported",
    actionable: true,
    emulated: false,
    locked: false,
    mixed: false,
    modControl: false,
  });
  assert.equal(classes.includes("unsupported"), false);
  assert.equal(classes.includes("cm-cap-supported"), true);
});

test("locked capability uses disabled without unsupported", () => {
  const classes = uiCommandChromeClassList({
    controlType: "button",
    capabilityState: "locked",
    actionable: false,
    emulated: false,
    locked: true,
    mixed: false,
    modControl: false,
  });
  assert.equal(classes.includes("unsupported"), false);
  assert.equal(classes.includes("disabled"), true);
});
