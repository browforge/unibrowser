import { mkdirSync } from "node:fs";
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

  test("web document mode screenshots v1", async ({ page }) => {
    mkdirSync(ART, { recursive: true });
    const shots: { name: string; demo: string }[] = [
      { name: "doc-modes-wysiwyg-web-v1.png", demo: "doc-mode-wysiwyg" },
      { name: "doc-modes-source-web-v1.png", demo: "doc-mode-source" },
      { name: "doc-modes-split-web-v1.png", demo: "doc-mode-split" },
      {
        name: "doc-modes-picker-web-v1.png",
        demo: "doc-mode-wysiwyg,txt-block-picker-open",
      },
    ];
    for (const shot of shots) {
      await openDemo(page, shot.demo);
      await page.getByRole("button", { name: "Skip tour" }).click({ timeout: 2000 }).catch(() => undefined);
      await page.screenshot({
        path: path.join(ART, shot.name),
        fullPage: false,
      });
    }
  });
});
