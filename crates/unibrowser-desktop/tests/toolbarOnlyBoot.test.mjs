import assert from "node:assert/strict";
import test from "node:test";

import {
  isToolbarWindowLabel,
  mainShellShouldBeAbsent,
  mountToolbarOnlyDocument,
} from "../src/toolbarOnlyShell.ts";

test("toolbar label selects toolbar-only boot", () => {
  assert.equal(
    isToolbarWindowLabel("toolbar-cmd-proving__dot__detached-markdown"),
    true,
  );
  assert.equal(isToolbarWindowLabel("main"), false);
});

test("mountToolbarOnlyDocument removes workbench shell roots", () => {
  const previousDocument = globalThis.document;
  const previousWindow = globalThis.window;
  const nodes = new Map();
  const makeEl = (id) => {
    const el = {
      id,
      classList: {
        _c: new Set(),
        add(...xs) {
          xs.forEach((x) => this._c.add(x));
        },
        toggle() {},
      },
      dataset: {},
      remove() {
        nodes.delete(id);
      },
      prepend() {},
    };
    nodes.set(id, el);
    return el;
  };
  globalThis.document = {
    body: { classList: { add() {} }, dataset: {} },
    getElementById(id) {
      return nodes.get(id) ?? null;
    },
  };
  globalThis.window = { location: { search: "" } };
  makeEl("app-shell");
  makeEl("work-area");
  makeEl("command-manager");
  makeEl("chrome");
  mountToolbarOnlyDocument();
  assert.equal(mainShellShouldBeAbsent(globalThis.document), true);
  globalThis.document = previousDocument;
  globalThis.window = previousWindow;
});
