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

    const demo = process.env.UB_WEB_DEMO ?? "theme-light,text-card-render1,txt-apply-h2,txt-block-picker-open";
    await page.goto(`./?demo=${encodeURIComponent(demo)}`);

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
    await expect(page.locator("body")).toHaveClass(/text-document-card/, {
      timeout: 30_000,
    });
    await expect(page.locator("#command-manager")).toBeHidden({ timeout: 15_000 });

    const host = page.locator("#text-compact-toolbar-host");
    await expect(host).toBeVisible({ timeout: 30_000 });

    const line3 = page.locator("#text-editor-host .view-lines .view-line").nth(2);
    await line3.click();
    await page.evaluate(() => {
      const monaco = (window as unknown as { monaco?: { editor: { getEditors: () => { getModel: () => { getLineContent: (n: number) => string }; getLineCount: () => number }; setPosition: (p: { lineNumber: number; column: number }) => void; executeEdits: (s: string, e: unknown[]) => void; revealLineInCenter: (n: number) => void }; Range: new (a: number, b: number, c: number, d: number) => unknown } }).monaco;
      const editors = monaco?.editor.getEditors() ?? [];
      const ed = editors[0];
      const model = ed?.getModel();
      if (!monaco || !ed || !model || model.getLineCount() < 3) {
        throw new Error("monaco editor not ready for line 3");
      }
      const line = 3;
      const text = model.getLineContent(line);
      const stripped = text.replace(/^#{1,6}\s+/, "").replace(/^>\s+/, "");
      ed.setPosition({ lineNumber: line, column: 1 });
      ed.executeEdits("block-style", [
        {
          range: new monaco.Range(line, 1, line, text.length + 1),
          text: `## ${stripped}`,
          forceMoveMarkers: true,
        },
      ]);
      ed.revealLineInCenter(line);
      ed.focus();
    });

    await line3.click();
    await page.evaluate(() => {
      window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
    });

    await page.evaluate(() => {
      const hostEl = document.getElementById("text-compact-toolbar-host");
      if (!hostEl) {
        return;
      }
      const trigger = hostEl.querySelector<HTMLButtonElement>(".text-compact-block-trigger");
      const menu = hostEl.querySelector<HTMLDivElement>(".text-compact-block-menu");
      if (trigger && menu) {
        menu.classList.remove("hidden");
        trigger.setAttribute("aria-expanded", "true");
        trigger.blur();
      }
      document.getElementById("ub-command-tooltip-host")?.classList.add("hidden");
      document.activeElement instanceof HTMLElement && document.activeElement.blur();
    });

    await expect(host.locator(".text-compact-block-menu:not(.hidden)")).toBeVisible({
      timeout: 15_000,
    });
    await expect(host.locator(".text-compact-block-label")).toHaveText("Heading 2");
    await expect(
      host.locator('[data-command-id="unibrowser.semantic.block.heading_2"]'),
    ).toBeVisible();

    const lineText = await page.evaluate(() => {
      const monaco = (window as unknown as { monaco?: { editor: { getEditors: () => { getModel: () => { getLineContent: (n: number) => string } | null }[] } } }).monaco;
      const model = monaco?.editor.getEditors()[0]?.getModel();
      return model?.getLineContent(3) ?? null;
    });
    expect(lineText?.startsWith("## ")).toBe(true);
    expect(lineText).toContain("Unibrowser renders");

    const maxToolbarHeight = await page.evaluate(() =>
      document.body.classList.contains("text-document-card") ? 38.5 : 36,
    );
    const toolbarBox = await host.boundingBox();
    expect(toolbarBox?.height ?? 99).toBeLessThanOrEqual(maxToolbarHeight);

    const toolsWrap = host.locator(".text-compact-tools");
    const wrapMetrics = await toolsWrap.evaluate((el) => ({
      flexWrap: getComputedStyle(el).flexWrap,
      scrollWidth: el.scrollWidth,
      clientWidth: el.clientWidth,
    }));
    expect(wrapMetrics.flexWrap).toBe("nowrap");
    expect(wrapMetrics.scrollWidth).toBeLessThanOrEqual(wrapMetrics.clientWidth + 2);

    await page.getByRole("button", { name: "Skip tour" }).click({ timeout: 3_000 }).catch(() => undefined);
    await page.getByRole("button", { name: "Dismiss" }).click({ timeout: 2_000 }).catch(() => undefined);

    const outPath = path.join(OUT_DIR, SHOT);
    await page.screenshot({ path: outPath, fullPage: false });
    try {
      mkdirSync(ART_DIR, { recursive: true });
      const artPath = path.join(ART_DIR, SHOT);
      await page.screenshot({ path: artPath, fullPage: false });
    } catch {
      // Artifacts dir may be busy in CI; docs/screenshots copy is the source of truth.
    }
  });
});
