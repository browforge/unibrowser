import { execFileSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, test } from "@playwright/test";

const OUT_DIR = path.resolve(
  import.meta.dirname,
  "../../../docs/screenshots/txt-toolbar",
);
const ART_DIR = "/opt/cursor/artifacts";
const SHOT = "txt-web-markdown-picker-h2-v3.png";
const ROOT = path.resolve(import.meta.dirname, "../../..");
const BLOCK_PICKER_LABELS = [
  "Paragraph",
  "Heading 1",
  "Heading 2",
  "Heading 3",
  "Heading 4",
  "Heading 5",
  "Heading 6",
  "Block quote",
  "Code block",
];

test.describe("TXT toolbar web evidence", () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test("markdown block picker on H2 at 1280x800 light", async ({ page }) => {
    mkdirSync(OUT_DIR, { recursive: true });
    mkdirSync(ART_DIR, { recursive: true });

    const demo =
      process.env.UB_WEB_DEMO ??
      "theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded,txt-apply-h2,txt-block-picker-open";
    await page.goto(`./?demo=${encodeURIComponent(demo)}`);

    await expect(page.locator("html")).toHaveAttribute("data-ub-web-wasm", "1", {
      timeout: 120_000,
    });
    await expect(page.locator("body")).toHaveAttribute("data-ub-ready", "1", {
      timeout: 30_000,
    });
    await expect(page.locator(".tab[aria-selected='true']")).toContainText("sample.md", {
      timeout: 30_000,
    });
    await expect(page.locator("body")).toHaveClass(/text-compact-visible/, {
      timeout: 30_000,
    });
    await expect(page.locator("body")).toHaveClass(/text-document-card/, {
      timeout: 30_000,
    });
    await expect(page.locator("#command-manager")).toBeHidden({ timeout: 15_000 });

    await expect(page.locator("#shell-subsystem-notice")).toBeHidden({ timeout: 5_000 });
    await expect(page.locator("#activity-rail")).toHaveClass(/expanded/, { timeout: 15_000 });
    await expect(page.locator("#activity-rail .rail-destination")).toHaveCount(6, {
      timeout: 15_000,
    });
    await expect(page.locator(".ai-assistant-panel")).toBeVisible({ timeout: 15_000 });

    const host = page.locator("#text-compact-toolbar-host");
    await expect(host).toBeVisible({ timeout: 30_000 });

    const line3 = page.locator("#text-editor-host .view-lines .view-line").nth(2);
    await line3.click();
    await page.evaluate(() => {
      const monaco = (window as unknown as {
        monaco?: {
          editor: {
            getEditors: () => {
              getModel: () => { getLineContent: (n: number) => string; getLineCount: () => number };
              setPosition: (p: { lineNumber: number; column: number }) => void;
              executeEdits: (s: string, e: unknown[]) => void;
              revealLineInCenter: (n: number) => void;
            }[];
          };
          Range: new (a: number, b: number, c: number, d: number) => unknown;
        };
      }).monaco;
      const editors = monaco?.editor.getEditors() ?? [];
      const ed = editors[0];
      const model = ed?.getModel();
      if (!monaco || !ed || !model || model.getLineCount() < 3) {
        throw new Error("monaco editor not ready for line 3");
      }
      const line = 3;
      const text = model.getLineContent(line);
      const stripped = text.replace(/^#{1,6}\s+/, "").replace(/^>\s+/, "");
      ed.setPosition({ lineNumber: line, column: 1 });
      ed.executeEdits("block-style", [
        {
          range: new monaco.Range(line, 1, line, text.length + 1),
          text: `## ${stripped}`,
          forceMoveMarkers: true,
        },
      ]);
      ed.revealLineInCenter(line);
      ed.focus();
    });

    await line3.click();

    await page.evaluate(() => {
      window.__unibrowserCapture?.closeBlockPickerForCapture();
      document.getElementById("ub-command-tooltip-host")?.classList.add("hidden");
      document.activeElement instanceof HTMLElement && document.activeElement.blur();
    });

    const probe = await page.evaluate(() => {
      const rect = window.__unibrowserCapture?.blockPickerProbeRect();
      if (!rect || rect.width <= 0) {
        throw new Error("block picker probe rect missing");
      }
      return rect;
    });
    const baselinePath = path.join(OUT_DIR, `.${SHOT}.probe-closed.png`);
    await page.screenshot({
      path: baselinePath,
      clip: {
        x: probe.x,
        y: probe.y,
        width: probe.width,
        height: probe.height,
      },
    });

    await page.evaluate(() => {
      window.__unibrowserCapture?.openBlockPickerForCapture();
    });

    const openMenu = page.locator(
      "#ub-block-picker-popover-host .text-compact-block-menu:not(.hidden), #text-compact-toolbar-host .text-compact-block-menu:not(.hidden)",
    );
    await expect(openMenu.first()).toBeVisible({ timeout: 15_000 });
    await expect(openMenu.first()).toHaveAttribute("role", "listbox");
    for (const label of BLOCK_PICKER_LABELS) {
      await expect(openMenu.first().getByRole("option", { name: label })).toBeVisible();
    }
    await expect(host.locator(".text-compact-block-label")).toHaveText("Heading 2");
    await expect(
      openMenu
        .first()
        .locator('[data-command-id="unibrowser.semantic.block.heading_2"][aria-selected]'),
    ).toBeVisible();

    const lineText = await page.evaluate(() => {
      const monaco = (window as unknown as {
        monaco?: { editor: { getEditors: () => { getModel: () => { getLineContent: (n: number) => string } | null }[] } };
      }).monaco;
      const model = monaco?.editor.getEditors()[0]?.getModel();
      return model?.getLineContent(3) ?? null;
    });
    expect(lineText?.startsWith("## ")).toBe(true);
    expect(lineText).toContain("Unibrowser renders");

    const maxToolbarHeight = await page.evaluate(() =>
      document.body.classList.contains("text-document-card") ? 38.5 : 36,
    );
    const toolbarBox = await host.boundingBox();
    expect(toolbarBox?.height ?? 99).toBeLessThanOrEqual(maxToolbarHeight);

    const toolsWrap = host.locator(".text-compact-tools");
    const wrapMetrics = await toolsWrap.evaluate((el) => ({
      flexWrap: getComputedStyle(el).flexWrap,
      scrollWidth: el.scrollWidth,
      clientWidth: el.clientWidth,
    }));
    expect(wrapMetrics.flexWrap).toBe("nowrap");
    expect(wrapMetrics.scrollWidth).toBeLessThanOrEqual(wrapMetrics.clientWidth + 2);

    await page.getByRole("button", { name: "Skip tour" }).click({ timeout: 3_000 }).catch(() => undefined);
    await page.getByRole("button", { name: "Dismiss" }).click({ timeout: 2_000 }).catch(() => undefined);

    const outPath = path.join(OUT_DIR, SHOT);
    await page.screenshot({ path: outPath, fullPage: false });
    try {
      mkdirSync(ART_DIR, { recursive: true });
      const artPath = path.join(ART_DIR, SHOT);
      await page.screenshot({ path: artPath, fullPage: false });
    } catch {
      // Artifacts dir may be busy in CI; docs/screenshots copy is the source of truth.
    }

    const metrics = await page.evaluate(() => window.__unibrowserCapture!.collectBlockPickerMetrics());
    const cmPath = path.join(OUT_DIR, `${SHOT}.cm.json`);
    writeFileSync(
      cmPath,
      `${JSON.stringify(
        {
          viewportWidth: 1280,
          viewportHeight: 800,
          blockPicker: metrics,
        },
        null,
        2,
      )}\n`,
    );
    execFileSync(
      "python3",
      [
        path.join(ROOT, "scripts/check-block-picker-open-pixels.py"),
        outPath,
        cmPath,
        "--baseline-png",
        baselinePath,
      ],
      { stdio: "inherit" },
    );
  });
});
