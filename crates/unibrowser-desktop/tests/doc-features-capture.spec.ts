import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { expect, test } from "@playwright/test";

const ART = "/opt/cursor/artifacts";
const ROOT = path.resolve(import.meta.dirname, "..");
const OUT = path.join(ROOT, "../../docs/screenshots/doc-features");
const BASE =
  "theme-light,text-card-render1,title-breadcrumb,ai,ai-mockup-composer,rail-collapsed";

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

  async function assertDocFeatureCaptureChrome(page: import("@playwright/test").Page) {
    const err = await page.evaluate(() => {
      const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
      if (
        !demo.includes("doc-mode-") &&
        !demo.includes("doc-opaque-card") &&
        !demo.includes("doc-history-panel") &&
        !demo.includes("doc-merge-banner") &&
        !demo.includes("doc-visual-docx") &&
        !demo.includes("doc-visual-md-docx")
      ) {
        return null;
      }
      if (demo.includes("doc-visual-md-docx")) {
        const viewer = document.getElementById("document-viewer-host");
        if (viewer && !viewer.classList.contains("hidden")) {
          const flyout = document.querySelector<HTMLElement>(".rail-side-panel:not(.hidden)");
          if (flyout && flyout.getBoundingClientRect().width > 8) {
            return "activity rail flyout visible during doc feature capture";
          }
          const cm = document.getElementById("command-manager");
          if (cm && !document.body.classList.contains("cm-collapsed")) {
            const rect = cm.getBoundingClientRect();
            if (rect.height > 44 && rect.width > 200) {
              return "full command manager row visible during doc feature capture";
            }
          }
          return null;
        }
      }
      if (!document.body.classList.contains("text-document-card")) {
        return "document feature capture requires text-document-card layout";
      }
      const shell = document.getElementById("document-card-shell");
      if (!shell || shell.classList.contains("hidden")) {
        return "document-card-shell hidden during doc feature capture";
      }
      const flyout = document.querySelector<HTMLElement>(".rail-side-panel:not(.hidden)");
      if (flyout && flyout.getBoundingClientRect().width > 8) {
        return "activity rail flyout visible during doc feature capture";
      }
      const cm = document.getElementById("command-manager");
      if (cm && !document.body.classList.contains("cm-collapsed")) {
        const rect = cm.getBoundingClientRect();
        if (rect.height > 44 && rect.width > 200) {
          return "full command manager row visible during doc feature capture";
        }
      }
      return null;
    });
    expect(err, err ?? "doc capture chrome").toBeNull();
  }

  async function writeWebCaptureCmJson(
    page: import("@playwright/test").Page,
    pngName: string,
  ) {
    const cm = await page.evaluate(() => {
      const flyout = document.querySelector<HTMLElement>(".rail-side-panel:not(.hidden)");
      const railFlyoutWidth = flyout ? flyout.getBoundingClientRect().width : 0;
      const controls = Array.from(
        document.querySelectorAll("#command-manager-groups .ui-command"),
      ) as HTMLElement[];
      const controlDiagnostics = controls.map((el) => {
        const rect = el.getBoundingClientRect();
        const label = el.querySelector(".cm-cmd-label") as HTMLElement | null;
        const labelRect = label?.getBoundingClientRect();
        return {
          controlId: el.dataset.controlId ?? "",
          width: rect.width,
          height: rect.height,
          labelBounds: {
            x: labelRect?.x ?? 0,
            y: labelRect?.y ?? 0,
            width: labelRect?.width ?? 0,
            height: labelRect?.height ?? 0,
          },
        };
      });
      const groups = document.getElementById("command-manager-groups");
      return {
        viewportWidth: window.innerWidth,
        viewportHeight: window.innerHeight,
        railFlyoutWidth,
        pipeline: { groupsWidth: groups?.getBoundingClientRect().width ?? 0 },
        controls: controlDiagnostics,
      };
    });
    const cmPath = path.join(OUT, pngName.replace(/\.png$/, ".cm.json"));
    mkdirSync(OUT, { recursive: true });
    writeFileSync(cmPath, `${JSON.stringify(cm, null, 2)}\n`);
    try {
      mkdirSync(ART, { recursive: true });
      writeFileSync(
        path.join(ART, pngName.replace(/\.png$/, ".cm.json")),
        `${JSON.stringify(cm, null, 2)}\n`,
      );
    } catch {
      // Artifacts dir may be busy in CI.
    }
  }

  test("web doc feature screenshots v2", async ({ browser }) => {
    mkdirSync(ART, { recursive: true });
    mkdirSync(OUT, { recursive: true });
    const shots: { name: string; demo: string; wait?: string }[] = [
      {
        name: "doc-wysiwyg-blockquote-web-v2.png",
        demo: "doc-mode-wysiwyg,txt-apply-h2",
        wait: ".rich-text-doc blockquote",
      },
      {
        name: "doc-split-apply-banner-web-v2.png",
        demo: "doc-mode-split,doc-merge-banner",
        wait: ".document-source-merge-banner:not(.hidden)",
      },
      {
        name: "doc-opaque-card-web-v2.png",
        demo: "doc-opaque-card",
        wait: ".rich-text-opaque-parse",
      },
      {
        name: "doc-history-panel-web-v2.png",
        demo: "doc-history-panel,txt-apply-h2",
        wait: "#bottom-panel-history-pane:not(.hidden)",
      },
      {
        name: "doc-visual-docx-web-v2.png",
        demo: "doc-visual-md-docx",
        wait: "#document-viewer-host:not(.hidden) .docx-reader",
      },
    ];
    for (const shot of shots) {
      const context = await browser.newContext({
        viewport: { width: 1280, height: 800 },
      });
      const page = await context.newPage();
      await openDemo(page, shot.demo);
      if (shot.wait) {
        await page.locator(shot.wait).first().waitFor({ timeout: 90_000 });
      }
      await page.getByRole("button", { name: "Skip tour" }).click({ timeout: 2000 }).catch(() => undefined);
      await page.getByRole("button", { name: "Dismiss" }).click({ timeout: 2000 }).catch(() => undefined);
      await assertDocFeatureCaptureChrome(page);
      const outPath = path.join(OUT, shot.name);
      await page.screenshot({ path: outPath, fullPage: false });
      await page.screenshot({ path: path.join(ART, shot.name), fullPage: false });
      if (!shot.name.includes("doc-visual-docx-web")) {
        await writeWebCaptureCmJson(page, shot.name);
      }
      await context.close();
    }
  });
});
