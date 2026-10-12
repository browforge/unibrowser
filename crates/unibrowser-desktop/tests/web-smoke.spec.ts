import { expect, test } from "@playwright/test";

const DEMO_HEADING = "Demo markdown";
const ARTIFACT_SCREENSHOT = "/opt/cursor/artifacts/web-demo.png";

async function expectVisibleMonacoDemoContent(page: import("@playwright/test").Page) {
  await page.waitForFunction(
    (heading) => {
      const host = document.querySelector("#text-editor-host");
      if (!host || host.classList.contains("hidden")) {
        return false;
      }
      const monaco = (window as unknown as {
        monaco?: {
          editor: {
            getEditors: () => { getDomNode: () => HTMLElement | null; getModel: () => { getValue: () => string } | null }[];
          };
        };
      }).monaco;
      const editors = monaco?.editor?.getEditors()?.filter((ed) => host.contains(ed.getDomNode())) ?? [];
      const editor = editors.find((ed) => ed.getModel()?.getValue().includes(heading)) ?? editors[0];
      const value = editor?.getModel()?.getValue() ?? "";
      if (!value.includes(heading)) {
        return false;
      }
      const normalize = (text: string) => text.replace(/\s+/g, " ").trim();
      const lines = Array.from(host.querySelectorAll(".view-line"));
      return lines.some((line) => normalize(line.textContent ?? "").includes(heading));
    },
    DEMO_HEADING,
    { timeout: 30_000 },
  );
}

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
    await page.goto("./");
    await expect(page.locator("html")).toHaveAttribute("data-ub-web-wasm", "1", {
      timeout: 120_000,
    });
    await expect(page.locator("body")).toHaveAttribute("data-ub-ready", "1", {
      timeout: 15_000,
    });
    const commandManager = page.locator("#command-manager");
    await expect(commandManager).toBeVisible({ timeout: 15_000 });

    await expect(page.locator(".tab[aria-selected='true']")).toContainText("sample.md", {
      timeout: 30_000,
    });
    await expect(page.locator("#text-editor-host")).toBeVisible({ timeout: 30_000 });
    await expectVisibleMonacoDemoContent(page);

    const cmCommands = page.locator("#command-manager-groups button:not([disabled])");
    await expect
      .poll(async () => await cmCommands.count(), { timeout: 15_000 })
      .toBeGreaterThanOrEqual(3);
    await expect(page.locator("body")).toHaveClass(/text-compact-visible/, {
      timeout: 15_000,
    });
    const compactBar = page.locator("#text-compact-toolbar-host");
    await expect(compactBar).toBeVisible({ timeout: 15_000 });
    await expect(compactBar.locator("[aria-label='Bold']")).toBeVisible({
      timeout: 15_000,
    });
    await expect(compactBar.locator("[aria-label='Italic']")).toBeVisible({
      timeout: 15_000,
    });

    await page.getByRole("button", { name: "Skip tour" }).click({ timeout: 5_000 }).catch(() => undefined);
    await page.getByRole("button", { name: "Dismiss" }).click({ timeout: 2_000 }).catch(() => undefined);

    await page.screenshot({ path: ARTIFACT_SCREENSHOT });

    const editor = page.locator("#text-editor-host .native-edit-context").first();
    await expect(editor).toBeVisible({ timeout: 30_000 });
    await page.locator("#text-editor-host .view-lines").first().click();
    await page.keyboard.press("Control+End");
    await page.keyboard.insertText("\nWeb smoke edit");
    await expectVisibleMonacoDemoContent(page);
    await page.waitForFunction(() => {
      const host = document.querySelector("#text-editor-host");
      if (!host) {
        return false;
      }
      const monaco = (window as unknown as {
        monaco?: {
          editor: {
            getEditors: () => { getDomNode: () => HTMLElement | null; getModel: () => { getValue: () => string } | null }[];
          };
        };
      }).monaco;
      const editor = monaco?.editor
        ?.getEditors()
        ?.find((ed) => host.contains(ed.getDomNode()));
      return Boolean(editor?.getModel()?.getValue().includes("Web smoke edit"));
    }, undefined, { timeout: 15_000 });
  });
});
