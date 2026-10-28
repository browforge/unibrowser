import { createHash } from "node:crypto";
import { mkdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { expect, test } from "@playwright/test";

const ART = "/opt/cursor/artifacts";
const BASE_DEMO =
  "theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded,txt-apply-h2";

test.describe("Document mode captures", () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  async function openDemo(page: import("@playwright/test").Page, extra: string) {
    const demo = `${BASE_DEMO},${extra}`;
    await page.goto(`./?demo=${encodeURIComponent(demo)}`);
    await expect(page.locator("html")).toHaveAttribute("data-ub-web-wasm", "1", {
      timeout: 120_000,
    });
    await expect(page.locator("body")).toHaveAttribute("data-ub-ready", "1", {
      timeout: 60_000,
    });
    await expect(page.locator("body")).toHaveClass(/text-document-card/, {
      timeout: 30_000,
    });
  }

  async function expectModeFooter(page: import("@playwright/test").Page) {
    await expect(page.locator("#document-mode-inline-host [data-doc-mode]")).toHaveCount(4, {
      timeout: 30_000,
    });
    await expect(page.locator('[data-doc-mode="wysiwyg"]')).toBeVisible();
    await expect(page.locator('[data-doc-mode="source"]')).toBeVisible();
    await expect(page.locator('[data-doc-mode="split"]')).toBeVisible();
    await expect(page.locator('[data-doc-mode="reading"]')).toBeVisible();
  }

  async function expectSplitPanes(page: import("@playwright/test").Page) {
    const host = page.locator("#text-editor-host");
    const monacoSlot = page.locator("#text-editor-host .document-monaco-slot");
    const preview = page.locator("#document-split-preview-host");
    await expect(host).toHaveClass(/document-editor-split-layout/);
    await expect(preview).not.toHaveClass(/hidden/);
    await expect(preview.locator(".rich-text-doc")).toBeVisible();
    const hostBox = await host.boundingBox();
    const slotBox = await monacoSlot.boundingBox();
    const previewBox = await preview.boundingBox();
    expect(hostBox).not.toBeNull();
    expect(slotBox).not.toBeNull();
    expect(previewBox).not.toBeNull();
    expect(slotBox!.width).toBeLessThan(hostBox!.width * 0.58);
    expect(previewBox!.width).toBeGreaterThan(hostBox!.width * 0.32);
    expect(previewBox!.x).toBeGreaterThan(slotBox!.x + slotBox!.width - 4);
  }

  test("web document mode screenshots v3", async ({ page }) => {
    mkdirSync(ART, { recursive: true });
    const shots: { name: string; demo: string }[] = [
      { name: "doc-modes-wysiwyg-web-v3.png", demo: "doc-mode-wysiwyg" },
      { name: "doc-modes-source-web-v3.png", demo: "doc-mode-source" },
      { name: "doc-modes-split-web-v3.png", demo: "doc-mode-split" },
      {
        name: "doc-modes-picker-web-v3.png",
        demo: "doc-mode-wysiwyg,txt-block-picker-open",
      },
    ];
    const digests: string[] = [];
    for (const shot of shots) {
      await openDemo(page, shot.demo);
      await page
        .getByRole("button", { name: "Skip tour" })
        .click({ timeout: 2000 })
        .catch(() => undefined);
      await page
        .getByRole("button", { name: "Dismiss" })
        .click({ timeout: 2000 })
        .catch(() => undefined);
      await expect(page.locator("#document-mode-strip-host")).toBeHidden();
      await expectModeFooter(page);
      if (shot.demo.includes("doc-mode-split")) {
        await expect(page.locator("body")).toHaveAttribute("data-document-editor-mode", "split", {
          timeout: 15_000,
        });
        await expectSplitPanes(page);
      }
      if (shot.demo.includes("doc-mode-source")) {
        await expect(page.locator("body")).toHaveAttribute("data-document-editor-mode", "source", {
          timeout: 15_000,
        });
        await expect(page.locator("#document-split-preview-host")).toHaveClass(/hidden/);
      }
      if (shot.demo.includes("doc-mode-wysiwyg")) {
        await expect(page.locator("body")).toHaveAttribute("data-document-editor-mode", "wysiwyg", {
          timeout: 15_000,
        });
        await expect(page.locator("#rich-text-editor-host")).not.toHaveClass(/hidden/);
      }
      const outPath = path.join(ART, shot.name);
      await page.screenshot({ path: outPath, fullPage: false });
      const digest = createHash("md5").update(readFileSync(outPath)).digest("hex");
      expect(digests.includes(digest)).toBe(false);
      digests.push(digest);
    }
  });
});
