import fs from "node:fs";
import path from "node:path";
import { browser, $ } from "@wdio/globals";
import { DEMO_PACK, FIXTURES, REPO_ROOT } from "../helpers/paths.js";
import { openPath, prepareCleanShell, tauriInvoke, waitForSelector } from "../helpers/tauri.js";

const OUT = path.join(REPO_ROOT, "docs/screenshots");

async function shot(name: string) {
  fs.mkdirSync(OUT, { recursive: true });
  const file = path.join(OUT, `${name}.png`);
  await browser.saveScreenshot(file);
  const size = fs.statSync(file).size;
  if (size > 400 * 1024) {
    console.warn(`screenshot ${name} is ${Math.round(size / 1024)}KB (>400KB)`);
  }
}

describe("Screenshot capture (1600x1000)", () => {
  before(async () => {
    await waitForSelector("#omnibox");
    await prepareCleanShell();
  });

  it("captures key product surfaces", async () => {
    const txt = path.join(DEMO_PACK, "sample.txt");
    const svg = path.join(DEMO_PACK, "sample.svg");
    const docx = path.join(FIXTURES, "minimal.docx");
    const unibrow = path.join(DEMO_PACK, "sample.unibrow");
    const unknown = path.join(FIXTURES, "unknown.ubunknown");

    await openPath(txt);
    await waitForSelector("#text-editor-host:not(.hidden)");
    await tauriInvoke("execute_command", { commandId: "unibrowser.font.size.increase" });
    await browser.pause(400);
    await shot("font-increase-text");

    await openPath(svg);
    await waitForSelector("#document-viewer-host:not(.hidden)");
    await tauriInvoke("execute_command", { commandId: "unibrowser.font.size.increase" });
    await browser.pause(400);
    await shot("font-increase-svg");

    await openPath(docx);
    await waitForSelector(".docx-view");
    await tauriInvoke("execute_command", { commandId: "unibrowser.font.size.increase" });
    await browser.pause(600);
    await shot("font-increase-docx");

    await openPath(unknown);
    await $("#unknown-file-banner").waitForDisplayed({ timeout: 15000 });
    await shot("unknown-file-banner");

    await openPath(txt);
    await waitForSelector("#text-editor-host:not(.hidden)");
    await tauriInvoke("proofing_check", {
      text: "teh colour centre",
      surface: "plaintext",
      language: "en-NZ",
    });
    await tauriInvoke("proofing_check_unibrow_project", {
      projectJson: JSON.stringify({
        format_version: 2,
        name: "shot",
        timeline: { clips: [{ id: "c1", start_sec: 0, duration_sec: 2, node_id: "n1" }] },
        board: { nodes: [{ id: "n1", label: "Title", node_type: "title" }] },
      }),
    });
    await browser.execute(() => {
      document.getElementById("bottom-panel-tab-problems")?.click();
      const pane = document.getElementById("bottom-panel-problems-pane");
      pane?.classList.remove("hidden");
      pane?.setAttribute("aria-hidden", "false");
    });
    await browser.pause(500);
    await shot("problems-panel-mixed");

    await openPath(unibrow);
    await waitForSelector(".workshop-timeline-canvas");
    await shot("unibrow-timeline");

    await tauriInvoke("open_path", { path: "unibrowser://settings" });
    await browser.pause(800);
    await shot("settings-overview");

    await tauriInvoke("open_path", { path: "unibrowser://mods" });
    await browser.pause(1000);
    await shot("mod-portal");

    await browser.execute(() => {
      const panel = document.getElementById("ai-assistant-panel");
      panel?.classList.remove("hidden");
    });
    await browser.pause(400);
    await shot("ai-assistant-panel");

    await openPath(txt);
    await waitForSelector("#command-manager-groups .ui-command.disabled");
    await shot("command-manager-greyed-tools");
  });
});
