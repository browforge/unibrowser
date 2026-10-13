import { mkdirSync } from "node:fs";
import path from "node:path";
import { expect, test } from "@playwright/test";

const ART = "/opt/cursor/artifacts";
const BASE =
  "theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded";

test.describe("Document feature captures", () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  async function openDemo(page: import("@playwright/test").Page, extra: string) {
    const demo = `${BASE},${extra}`;
    await page.goto(`./?demo=${encodeURIComponent(demo)}`);
    await expect(page.locator("html")).toHaveAttribute("data-ub-web-wasm", "1", {
      timeout: 120_000,
    });
    await expect(page.locator("body")).toHaveAttribute("data-ub-ready", "1", {
      timeout: 60_000,
    });
  }

  test("web doc feature screenshots v1", async ({ page }) => {
    mkdirSync(ART, { recursive: true });
    const shots: { name: string; demo: string; wait?: string }[] = [
      {
        name: "doc-wysiwyg-blockquote-web-v1.png",
        demo: "doc-mode-wysiwyg,txt-apply-h2",
        wait: ".rich-text-doc blockquote",
      },
      {
        name: "doc-split-apply-banner-web-v1.png",
        demo: "doc-mode-split,doc-merge-banner",
        wait: ".document-source-merge-banner:not(.hidden)",
      },
      {
        name: "doc-opaque-card-web-v1.png",
        demo: "doc-opaque-card",
        wait: ".rich-text-opaque-parse",
      },
      {
        name: "doc-history-panel-web-v1.png",
        demo: "doc-history-panel,txt-apply-h2",
        wait: "#bottom-panel-history-pane:not(.hidden)",
      },
    ];
    for (const shot of shots) {
      await openDemo(page, shot.demo);
      if (shot.wait) {
        await page.locator(shot.wait).first().waitFor({ timeout: 60_000 });
      }
      await page.getByRole("button", { name: "Skip tour" }).click({ timeout: 2000 }).catch(() => undefined);
      await page.screenshot({ path: path.join(ART, shot.name), fullPage: false });
    }
  });
});
