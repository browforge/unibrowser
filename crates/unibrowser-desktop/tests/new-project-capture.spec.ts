import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, test } from "@playwright/test";

const ART = "/opt/cursor/artifacts";
const ROOT = path.resolve(import.meta.dirname, "..");
const OUT = path.join(ROOT, "../../docs/screenshots/new-project");
const BASE = "capture-ready,rail-collapsed,theme-light,standard-contrast,scale-100";

test.describe("New project flow (web WASM)", () => {
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
    await page.waitForTimeout(400);
  }

  async function shot(page: import("@playwright/test").Page, name: string) {
    mkdirSync(OUT, { recursive: true });
    mkdirSync(ART, { recursive: true });
    const file = path.join(OUT, `${name}-v2-web-1280.png`);
    const bytes = await page.screenshot({ fullPage: false });
    writeFileSync(file, bytes);
    writeFileSync(path.join(ART, `${name}-v2-web-1280.png`), bytes);
    const box = await page.viewportSize();
    expect(box).toEqual({ width: 1280, height: 800 });
  }

  test("NTP paper card", async ({ page }) => {
    await openDemo(page, "new-project-capture-ntp");
    await shot(page, "new-tab-paper-card");
  });

  test("Template gallery pick", async ({ page }) => {
    await openDemo(page, "new-project-capture-gallery");
    await expect(page.locator(".template-gallery-overlay")).toBeVisible();
    await expect(page.locator("#template-gallery-title")).toHaveText("Create a new project");
    await shot(page, "template-gallery-pick");
  });
});
