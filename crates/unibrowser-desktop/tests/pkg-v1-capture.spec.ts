import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, test } from "@playwright/test";

const ROOT = path.resolve(import.meta.dirname, "../../..");
const OUT = path.join(ROOT, "docs/screenshots/pkg-v1");
const ART = "/opt/cursor/artifacts";
const BASE = "capture-ready,rail-collapsed,theme-light,standard-contrast,scale-100,cm-collapsed";

test.describe("PKG v1 UI captures", () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  async function openDemo(page: import("@playwright/test").Page, extra: string) {
    const demo = `${BASE},${extra}`;
    await page.goto(`./?demo=${encodeURIComponent(demo)}#demo=${encodeURIComponent(demo)}`);
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
    const file = path.join(OUT, `${name}-v1-1280.png`);
    const bytes = await page.screenshot({ fullPage: false });
    writeFileSync(file, bytes);
    writeFileSync(path.join(ART, `${name}-v1-1280.png`), bytes);
    writeFileSync(
      path.join(OUT, `${name}-v1-1280.cm.json`),
      JSON.stringify({ viewportWidth: 1280, viewportHeight: 800, dom: {} }),
    );
    const box = await page.viewportSize();
    expect(box).toEqual({ width: 1280, height: 800 });
  }

  test("open mode dialog", async ({ page }) => {
    await openDemo(page, "pkg-v1-open-mode");
    await expect(page.locator(".pkg-open-mode-overlay")).toBeVisible();
    await shot(page, "open-mode-dialog");
  });

  test("history location notice", async ({ page }) => {
    await openDemo(page, "pkg-v1-history-notice");
    await expect(page.locator("[data-pkg-history-notice]")).toBeVisible();
    await shot(page, "history-location-notice");
  });

  test("unpack progress", async ({ page }) => {
    await openDemo(page, "pkg-v1-unpack-progress");
    await expect(page.locator("[data-pkg-unpack-progress]")).toBeVisible();
    await shot(page, "unpack-progress");
  });

  test("file menu pack unpack", async ({ page }) => {
    await openDemo(page, "pkg-v1-file-menu");
    await expect(page.locator(".app-menu-dropdown")).toBeVisible();
    await shot(page, "file-menu-pack-unpack");
  });
});
