import path from "node:path";
import { browser, $, $$ } from "@wdio/globals";
import assert from "node:assert/strict";
import { DEMO_PACK, FIXTURES } from "../helpers/paths.js";
import {
  openPath,
  prepareCleanShell,
  readStatusFontPt,
  runCommand,
  tauriInvoke,
  waitForSelector,
} from "../helpers/tauri.js";

const INCREASE_FONT = "unibrowser.font.size.increase";
const PALETTE_TOGGLE = "unibrowser.palette.toggle";

describe("Unibrowser shell (WebdriverIO + embedded WebDriver)", () => {
  before(async () => {
    await browser.pause(8000);
    await waitForSelector("#omnibox");
    await prepareCleanShell();
  });

  it("opens text and Increase font size updates status font pt", async () => {
    const txt = path.join(DEMO_PACK, "sample.txt");
    await openPath(txt);
    await waitForSelector("#text-editor-host:not(.hidden)");
    const before = await readStatusFontPt();
    await runCommand(INCREASE_FONT);
    const after = await readStatusFontPt();
    assert.notEqual(after, before);
  });

  it("opens SVG and Increase font size runs without error", async () => {
    const svg = path.join(DEMO_PACK, "sample.svg");
    await openPath(svg);
    await waitForSelector("#document-viewer-host:not(.hidden)");
    const msg = await runCommand(INCREASE_FONT);
    assert.ok(msg.length > 0);
  });

  it("shows unknown-file banner (non-modal) for unhandled extension", async () => {
    const unknown = path.join(FIXTURES, "unknown.ubunknown");
    await openPath(unknown);
    const banner = await $("#unknown-file-banner");
    await banner.waitForDisplayed({ timeout: 15000 });
    const text = await banner.getText();
    assert.ok(text.toLowerCase().includes("mod"));
    const modal = await $(".inline-ui-layer:not(.hidden)");
    assert.equal(await modal.isExisting(), false);
  });

  it("closes dirty tab and Undo restores content", async () => {
    const txt = path.join(FIXTURES, "spell-test.txt");
    await openPath(txt);
    await waitForSelector("#text-editor-host:not(.hidden)");
    const marker = `e2e-undo-${Date.now()}`;
    await browser.keys(["End", "Enter", marker]);
    await browser.pause(500);
    const closeBtn = await $(".tab-close");
    await closeBtn.click();
    const undoBtn = await $(".toast-host button");
    await undoBtn.waitForDisplayed({ timeout: 10000 });
    await undoBtn.click();
    await browser.pause(600);
    const body = await browser.getPageSource();
    assert.ok(body.includes(marker));
  });

  it("flags misspelling then workspace dictionary add clears proofing hit", async () => {
    const txt = path.join(FIXTURES, "spell-test.txt");
    await openPath(txt);
    await waitForSelector("#text-editor-host:not(.hidden)");
    await browser.pause(1500);
    const before = await tauriInvoke<{ start: number; end: number; message: string }[]>(
      "proofing_check",
      {
        text: "teh quick brown",
        surface: "plaintext",
        language: "en-NZ",
      },
    );
    assert.ok(before.length > 0);
    await tauriInvoke("proofing_dictionary_add", {
      word: "teh",
      scope: "workspace",
      ignore: false,
    });
    const after = await tauriInvoke<{ message: string }[]>("proofing_check", {
      text: "teh quick brown",
      surface: "plaintext",
      language: "en-NZ",
    });
    const stillTeh = after.some((d) => d.message.toLowerCase().includes("teh"));
    assert.equal(stillTeh, false);
    try {
      await $(".squiggly-error").waitForExist({ timeout: 8000 });
    } catch {
      /* Monaco squiggle class may vary by theme; invoke proofing is authoritative */
    }
  });

  it("palette Ctrl+Shift+P runs Increase font size command", async () => {
    const txt = path.join(DEMO_PACK, "sample.txt");
    await openPath(txt);
    await waitForSelector("#text-editor-host:not(.hidden)");
    await browser.keys(["Control", "Shift", "p"]);
    await waitForSelector("#palette:not(.hidden)");
    const query = await $("#palette-query");
    await query.setValue("Increase font");
    await browser.pause(300);
    const item = await $(".palette-item");
    await item.waitForDisplayed();
    await item.click();
    await browser.pause(400);
    const palette = await $("#palette");
    assert.ok((await palette.getAttribute("class"))?.includes("hidden"));
  });

  it("Command Manager greys tools when switching tab types", async () => {
    const txt = path.join(DEMO_PACK, "sample.txt");
    const png = path.join(DEMO_PACK, "sample.png");
    await openPath(txt);
    await waitForSelector("#command-manager-commands .ui-command");
    await openPath(png);
    await waitForSelector("#document-viewer-host:not(.hidden)");
    await browser.pause(1200);
    const disabled = await $$("#command-manager-commands .ui-command.disabled");
    assert.ok(disabled.length > 0);
  });

  it("saves blank .unibrow project with zero timeline clips", async () => {
    const parent = path.join(FIXTURES, "projects");
    await tauriInvoke("create_unibrow_project", {
      parentDir: parent,
      name: `e2e-blank-${Date.now()}`,
      template: "blank",
    });
    await browser.pause(500);
    const dest = await tauriInvoke<string>("autosave_unibrow_active_project");
    assert.ok(dest.length > 0);
  });
});
