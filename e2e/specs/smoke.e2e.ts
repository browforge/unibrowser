import { browser } from "@wdio/globals";
import assert from "node:assert/strict";
import { waitForWindowHandle } from "../helpers/tauri.js";

describe("Unibrowser smoke (tauri-driver)", () => {
  it("opens a WebDriver session with at least one window handle", async () => {
    await waitForWindowHandle(120000);
    const handles = await browser.getWindowHandles();
    assert.ok(handles.length > 0);
  });
});
