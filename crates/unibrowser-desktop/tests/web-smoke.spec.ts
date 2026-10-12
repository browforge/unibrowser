import { expect, test } from "@playwright/test";

test.describe("web demo", () => {
  test.setTimeout(180_000);

  test("loads privacy notice and WASM host", async ({ page }) => {
    await page.goto("./");
    await expect(page.locator(".ub-web-privacy-notice")).toContainText(
      "Runs entirely in your browser",
    );
    await expect(page.locator("html")).toHaveAttribute("data-ub-web-wasm", "1", {
      timeout: 120_000,
    });
  });

  test.fixme("opens demo markdown and edits text", async ({ page }) => {
    await page.goto("./?smoke=demo");
    await expect(page.locator("html")).toHaveAttribute("data-ub-web-wasm", "1", {
      timeout: 120_000,
    });
    await expect(page.locator("#text-editor-host")).toBeVisible({ timeout: 180_000 });
    const editor = page.locator(".monaco-editor textarea");
    await expect(editor).toBeVisible({ timeout: 120_000 });
    await editor.click();
    await editor.press("End");
    await editor.type("\nWeb smoke edit");
    await expect(editor).toContainText("Web smoke edit");
  });
});
