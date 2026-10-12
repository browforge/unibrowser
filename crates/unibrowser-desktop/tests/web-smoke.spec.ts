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

  test("mounts shell, opens demo markdown, and edits text", async ({ page }) => {
    await page.goto("./?smoke=demo");
    await expect(page.locator("html")).toHaveAttribute("data-ub-web-wasm", "1", {
      timeout: 120_000,
    });
    await expect(page.locator("body")).toHaveAttribute("data-ub-ready", "1", {
      timeout: 15_000,
    });
    const commandManager = page.locator("#command-manager");
    await expect(commandManager).toBeVisible({ timeout: 15_000 });
    const cmCommands = page.locator("#command-manager-groups button:not([disabled])");
    await expect
      .poll(async () => await cmCommands.count(), { timeout: 15_000 })
      .toBeGreaterThanOrEqual(3);
    await expect(cmCommands.first()).toBeVisible();

    await expect(page.locator("#text-editor-host")).toBeVisible({ timeout: 30_000 });
    await expect(page.locator("#text-editor-host")).toContainText("Demo markdown", {
      timeout: 30_000,
    });
    await page.getByRole("button", { name: "Skip tour" }).click({ timeout: 5_000 }).catch(() => undefined);
    const editor = page.locator("#text-editor-host .native-edit-context").first();
    await expect(editor).toBeVisible({ timeout: 30_000 });
    await page.locator("#text-editor-host .view-lines").first().click();
    await page.keyboard.press("Control+End");
    await page.keyboard.insertText("\nWeb smoke edit");
    await expect(page.locator("#text-editor-host")).toContainText("Web smoke edit", {
      timeout: 15_000,
    });
  });
});
