import { expect, test } from "@playwright/test";
import { mkdirSync } from "node:fs";
import path from "node:path";

const OUT_DIR = path.resolve(
  import.meta.dirname,
  "../../../docs/screenshots/txt-toolbar",
);
const ART_DIR = "/opt/cursor/artifacts";
const SHOT = "txt-web-markdown-picker-h2.png";

test.describe("TXT toolbar web evidence", () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test("markdown block picker on H2 at 1280x800 light", async ({ page }) => {
    mkdirSync(OUT_DIR, { recursive: true });
    mkdirSync(ART_DIR, { recursive: true });

    await page.goto(`./?demo=${encodeURIComponent("theme-light")}`);

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

    const host = page.locator("#text-compact-toolbar-host");
    await expect(host).toBeVisible({ timeout: 30_000 });
    await page.locator("#text-editor-host .view-lines").first().click();
    await page.keyboard.press("Home");
    const blockTrigger = host.locator(".text-compact-block-trigger");
    await blockTrigger.click();
    await host.locator('[data-command-id="unibrowser.semantic.block.heading_2"]').click();
    await blockTrigger.click();
    await expect(host.locator(".text-compact-block-menu:not(.hidden)")).toBeVisible({
      timeout: 15_000,
    });
    await expect(
      host.locator('[data-command-id="unibrowser.semantic.block.heading_2"][aria-selected="true"]'),
    ).toBeVisible();

    const toolbarBox = await host.boundingBox();
    expect(toolbarBox?.height ?? 99).toBeLessThanOrEqual(33.5);

    const toolsWrap = host.locator(".text-compact-tools");
    const wrapMetrics = await toolsWrap.evaluate((el) => ({
      flexWrap: getComputedStyle(el).flexWrap,
      scrollWidth: el.scrollWidth,
      clientWidth: el.clientWidth,
    }));
    expect(wrapMetrics.flexWrap).toBe("nowrap");
    expect(wrapMetrics.scrollWidth).toBeLessThanOrEqual(wrapMetrics.clientWidth + 2);

    const pickerLabel = await host.locator(".text-compact-block-label").textContent();
    expect(pickerLabel?.trim().length ?? 0).toBeGreaterThan(0);

    await page.getByRole("button", { name: "Skip tour" }).click({ timeout: 3_000 }).catch(() => undefined);
    await page.getByRole("button", { name: "Dismiss" }).click({ timeout: 2_000 }).catch(() => undefined);

    const outPath = path.join(OUT_DIR, SHOT);
    const artPath = path.join(ART_DIR, SHOT);
    await page.screenshot({ path: outPath, fullPage: false });
    await page.screenshot({ path: artPath, fullPage: false });
  });
});
