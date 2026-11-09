import { expect, test } from "@playwright/test";

const BASE_DEMO =
  "theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-expanded,txt-apply-h2";

test.describe("Everyday document editing", () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  async function openMarkdownVisual(page: import("@playwright/test").Page) {
    const demo = `${BASE_DEMO},doc-mode-wysiwyg`;
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
    await page
      .getByRole("button", { name: "Skip tour" })
      .click({ timeout: 2000 })
      .catch(() => undefined);
    await page
      .getByRole("button", { name: "Dismiss" })
      .click({ timeout: 2000 })
      .catch(() => undefined);
    await expect(page.locator(".rich-text-doc .ProseMirror")).toBeVisible({ timeout: 30_000 });
  }

  test("visual typing, source round-trip, undo, status word count", async ({ page }) => {
    await openMarkdownVisual(page);

    await expect(page.locator('[data-status-item="document-integrated"]')).toContainText(/words/i, {
      timeout: 15_000,
    });

    const marker = `everyday-edit-${Date.now()}`;
    await page.locator('[data-doc-mode="source"]').click();
    await expect(page.locator("body")).toHaveAttribute("data-document-editor-mode", "source", {
      timeout: 15_000,
    });
    const monaco = page.locator("#text-editor-host .monaco-editor");
    await expect(monaco).toBeVisible({ timeout: 30_000 });
    await monaco.click();
    await page.keyboard.press("Control+End");
    await page.keyboard.type(`\n\n${marker}\n`);
    await expect(page.locator("#text-editor-host .view-lines")).toContainText(marker, {
      timeout: 10_000,
    });

    await page.locator('[data-doc-mode="wysiwyg"]').click();
    await expect(page.locator("body")).toHaveAttribute("data-document-editor-mode", "wysiwyg", {
      timeout: 15_000,
    });
    const editor = page.locator(".rich-text-doc .ProseMirror");
    await expect(editor).toContainText(marker, { timeout: 15_000 });

    await monaco.click({ timeout: 5000 }).catch(() => undefined);
    await page.locator('[data-doc-mode="source"]').click();
    await expect(page.locator("#text-editor-host .view-lines")).toContainText(marker, {
      timeout: 10_000,
    });

    await page.locator('[data-doc-mode="wysiwyg"]').click();
    await editor.click();
    await page.keyboard.press(process.platform === "darwin" ? "Meta+Z" : "Control+Z");
    await expect(editor).not.toContainText(marker, { timeout: 15_000 });
  });
});
