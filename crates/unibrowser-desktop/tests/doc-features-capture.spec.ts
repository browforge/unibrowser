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
        if (!document.body.classList.contains("text-document-card")) {
          return "doc-visual-md-docx capture requires text-document-card layout";
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

  async function assertDocFeatureCaptureDom(page: import("@playwright/test").Page) {
    const err = await page.evaluate(() => {
      const visible = (el: HTMLElement) => {
        const rect = el.getBoundingClientRect();
        const style = getComputedStyle(el);
        return (
          rect.width > 0 &&
          rect.height > 0 &&
          style.display !== "none" &&
          style.visibility !== "hidden"
        );
      };
      const cards = Array.from(
        document.querySelectorAll<HTMLElement>(".document-card, .document-card-shell"),
      ).filter(visible);
      if (cards.length !== 1) {
        return `expected exactly one visible document card, found ${cards.length}`;
      }
      const cm = document.getElementById("command-manager");
      if (
        cm &&
        visible(cm) &&
        !document.body.classList.contains("cm-collapsed") &&
        cm.getBoundingClientRect().height > 44 &&
        cm.getBoundingClientRect().width > 200
      ) {
        return "command manager row visible during doc feature capture";
      }
      const parseRgb = (rgb: string) => {
        const hex = rgb.trim().match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i);
        if (hex) {
          const value =
            hex[1].length === 3
              ? hex[1]
                  .split("")
                  .map((c) => `${c}${c}`)
                  .join("")
              : hex[1];
          return {
            r: Number.parseInt(value.slice(0, 2), 16),
            g: Number.parseInt(value.slice(2, 4), 16),
            b: Number.parseInt(value.slice(4, 6), 16),
          };
        }
        const m = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
        if (!m) {
          return null;
        }
        return { r: Number(m[1]), g: Number(m[2]), b: Number(m[3]) };
      };
      const colorsMatch = (a: string, b: string) => {
        const pa = parseRgb(a);
        const pb = parseRgb(b);
        if (pa && pb) {
          return pa.r === pb.r && pa.g === pb.g && pa.b === pb.b;
        }
        return a.trim().toLowerCase() === b.trim().toLowerCase();
      };
      const footerMain = document.querySelector<HTMLElement>(".document-card-status-main");
      const bodyStyle = getComputedStyle(document.body);
      const mutedToken = bodyStyle.getPropertyValue("--ub-doc-card-muted").trim();
      const inkToken = bodyStyle.getPropertyValue("--ub-doc-card-ink").trim();
      const paperToken = bodyStyle.getPropertyValue("--ub-doc-card-paper").trim();
      const statusHost = footerMain?.closest("#document-card-status");
      const footerStyle = footerMain ? getComputedStyle(footerMain) : null;
      const statusStyle =
        statusHost instanceof HTMLElement ? getComputedStyle(statusHost) : null;
      const footerOk = Boolean(
        footerStyle &&
          statusStyle &&
          mutedToken &&
          paperToken &&
          (colorsMatch(footerStyle.color, mutedToken) || colorsMatch(footerStyle.color, inkToken)) &&
          statusStyle.fontSize === "12px" &&
          colorsMatch(statusStyle.backgroundColor, paperToken),
      );
      if (!footerOk) {
        return "document card footer is not styled by card tokens";
      }
      const mode = document.body.dataset.documentEditorMode ?? "";
      const monacoHost = document.getElementById("text-editor-host");
      const richHost = document.getElementById("rich-text-editor-host");
      const splitHost = document.getElementById("document-split-preview-host");
      const monacoVisible = Boolean(
        monacoHost &&
          !monacoHost.classList.contains("hidden") &&
          !monacoHost.classList.contains("document-surface-source-hidden"),
      );
      const richVisible = Boolean(
        richHost &&
          !richHost.classList.contains("hidden") &&
          richHost.querySelector(".rich-text-doc"),
      );
      const splitVisible = Boolean(
        splitHost &&
          !splitHost.classList.contains("hidden") &&
          splitHost.querySelector(".rich-text-doc"),
      );
      const host =
        mode === "source"
          ? monacoVisible && !richVisible
            ? "monaco"
            : "none"
          : mode === "split"
            ? splitVisible && monacoVisible
              ? "split"
              : "none"
            : richVisible && !monacoVisible
              ? "rich"
              : "none";
      const modeOk =
        (mode === "source" && host === "monaco") ||
        (mode === "split" && host === "split") ||
        ((mode === "wysiwyg" || mode === "reading") && host === "rich");
      if (!modeOk) {
        return `active mode ${mode} does not match visible editor host (${host})`;
      }
      const bodyMount = document.getElementById("document-card-body-mount");
      const bodyRect = bodyMount?.getBoundingClientRect();
      const straySelector =
        ".monaco-editor, .view-lines, .view-line, .md-preview-body, .md-preview-shell, #document-split-preview-host";
      if (bodyMount && (mode === "wysiwyg" || mode === "reading") && richHost && !richHost.classList.contains("hidden")) {
        const rect = bodyMount.getBoundingClientRect();
        const xs = [rect.left + rect.width * 0.25, rect.left + rect.width / 2, rect.left + rect.width * 0.75];
        const ys = [rect.top + 16, rect.top + rect.height * 0.35, rect.top + rect.height * 0.65];
        for (const y of ys) {
          for (const x of xs) {
            let reachedRich = false;
            for (const el of document.elementsFromPoint(x, y)) {
              if (!(el instanceof HTMLElement)) {
                continue;
              }
              if (!bodyMount.contains(el)) {
                break;
              }
              if (richHost.contains(el)) {
                reachedRich = true;
                break;
              }
              if (!visible(el)) {
                continue;
              }
              if (
                el.id === "text-editor-host" ||
                el.id === "document-card-body-mount" ||
                el.classList.contains("document-monaco-slot") ||
                el.classList.contains("rich-text-editor-shell")
              ) {
                continue;
              }
              if (el.closest(straySelector)) {
                return "card body hit-test must stay inside rich editor";
              }
            }
            if (!reachedRich) {
              return "card body hit-test must stay inside rich editor";
            }
          }
        }
      }
      if (bodyRect && bodyRect.width > 40 && bodyRect.height > 40) {
        const shell = document.getElementById("document-card-shell");
        const sampleYs = [
          bodyRect.top + 8,
          bodyRect.top + bodyRect.height * 0.35,
          bodyRect.top + bodyRect.height * 0.65,
        ];
        for (const y of sampleYs) {
          const x = bodyRect.left + bodyRect.width / 2;
          for (const el of document.elementsFromPoint(x, y)) {
            if (!(el instanceof HTMLElement)) {
              continue;
            }
            if (shell?.contains(el) || el === shell || el === bodyMount) {
              break;
            }
            if (el.id === "command-manager" || el.closest("#command-manager")) {
              return "outside chrome overlaps the document card body";
            }
            if (el.id === "document-viewer-host" && !el.classList.contains("hidden")) {
              return "outside chrome overlaps the document card body";
            }
          }
        }
      }
      const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
      if (demo.includes("doc-visual-md-docx")) {
        const footerText = footerMain?.textContent ?? "";
        if (!footerText.includes("Word")) {
          return "doc-visual-md-docx capture footer must show Word kind";
        }
        const approx = document.querySelector<HTMLElement>(".block-emulation-badge, .cm-emulation-mark");
        if (!approx?.textContent?.includes("≈")) {
          return "doc-visual-md-docx capture must show emulated block-style (≈) badge";
        }
      }
      return null;
    });
    expect(err, err ?? "doc capture dom").toBeNull();
  }

  async function writeWebCaptureCmJson(
    page: import("@playwright/test").Page,
    pngName: string,
  ) {
    const cm = await page.evaluate(() => {
      const visible = (el: HTMLElement) => {
        const rect = el.getBoundingClientRect();
        const style = getComputedStyle(el);
        return (
          rect.width > 0 &&
          rect.height > 0 &&
          style.display !== "none" &&
          style.visibility !== "hidden"
        );
      };
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
      const cards = Array.from(
        document.querySelectorAll<HTMLElement>(".document-card, .document-card-shell"),
      ).filter(visible);
      const cmEl = document.getElementById("command-manager");
      const commandManagerVisible = Boolean(
        cmEl &&
          visible(cmEl) &&
          !document.body.classList.contains("cm-collapsed") &&
          cmEl.getBoundingClientRect().height > 44 &&
          cmEl.getBoundingClientRect().width > 200,
      );
      const parseRgb = (rgb: string) => {
        const hex = rgb.trim().match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i);
        if (hex) {
          const value =
            hex[1].length === 3
              ? hex[1]
                  .split("")
                  .map((c) => `${c}${c}`)
                  .join("")
              : hex[1];
          return {
            r: Number.parseInt(value.slice(0, 2), 16),
            g: Number.parseInt(value.slice(2, 4), 16),
            b: Number.parseInt(value.slice(4, 6), 16),
          };
        }
        const m = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
        if (!m) {
          return null;
        }
        return { r: Number(m[1]), g: Number(m[2]), b: Number(m[3]) };
      };
      const colorsMatch = (a: string, b: string) => {
        const pa = parseRgb(a);
        const pb = parseRgb(b);
        if (pa && pb) {
          return pa.r === pb.r && pa.g === pb.g && pa.b === pb.b;
        }
        return a.trim().toLowerCase() === b.trim().toLowerCase();
      };
      const footerMain = document.querySelector<HTMLElement>(".document-card-status-main");
      const bodyStyle = getComputedStyle(document.body);
      const mutedToken = bodyStyle.getPropertyValue("--ub-doc-card-muted").trim();
      const inkToken = bodyStyle.getPropertyValue("--ub-doc-card-ink").trim();
      const paperToken = bodyStyle.getPropertyValue("--ub-doc-card-paper").trim();
      const statusHost = footerMain?.closest("#document-card-status");
      const footerStyle = footerMain ? getComputedStyle(footerMain) : null;
      const statusStyle =
        statusHost instanceof HTMLElement ? getComputedStyle(statusHost) : null;
      const mode = document.body.dataset.documentEditorMode ?? "";
      const monacoHost = document.getElementById("text-editor-host");
      const richHost = document.getElementById("rich-text-editor-host");
      const splitHost = document.getElementById("document-split-preview-host");
      const monacoVisible = Boolean(
        monacoHost &&
          !monacoHost.classList.contains("hidden") &&
          !monacoHost.classList.contains("document-surface-source-hidden"),
      );
      const richVisible = Boolean(
        richHost &&
          !richHost.classList.contains("hidden") &&
          richHost.querySelector(".rich-text-doc"),
      );
      const splitVisible = Boolean(
        splitHost &&
          !splitHost.classList.contains("hidden") &&
          splitHost.querySelector(".rich-text-doc"),
      );
      const visibleEditorHost =
        mode === "source"
          ? monacoVisible && !richVisible
            ? "monaco"
            : "none"
          : mode === "split"
            ? splitVisible && monacoVisible
              ? "split"
              : "none"
            : richVisible && !monacoVisible
              ? "rich"
              : "none";
      const modeMatchesVisibleHost =
        (mode === "source" && visibleEditorHost === "monaco") ||
        (mode === "split" && visibleEditorHost === "split") ||
        ((mode === "wysiwyg" || mode === "reading") && visibleEditorHost === "rich");
      const bodyMount = document.getElementById("document-card-body-mount");
      const bodyRect = bodyMount?.getBoundingClientRect();
      const visible = (el: HTMLElement) => {
        const rect = el.getBoundingClientRect();
        const style = getComputedStyle(el);
        return (
          rect.width > 0 &&
          rect.height > 0 &&
          style.display !== "none" &&
          style.visibility !== "hidden"
        );
      };
      const straySelector =
        ".monaco-editor, .view-lines, .view-line, .md-preview-body, .md-preview-shell, #document-split-preview-host";
      let richEditorSubtreeViolations = 0;
      if (
        bodyMount &&
        (mode === "wysiwyg" || mode === "reading") &&
        richHost &&
        !richHost.classList.contains("hidden")
      ) {
        const rect = bodyMount.getBoundingClientRect();
        const xs = [rect.left + rect.width * 0.25, rect.left + rect.width / 2, rect.left + rect.width * 0.75];
        const ys = [rect.top + 16, rect.top + rect.height * 0.35, rect.top + rect.height * 0.65];
        for (const y of ys) {
          for (const x of xs) {
            let reachedRich = false;
            for (const el of document.elementsFromPoint(x, y)) {
              if (!(el instanceof HTMLElement)) {
                continue;
              }
              if (!bodyMount.contains(el)) {
                break;
              }
              if (richHost.contains(el)) {
                reachedRich = true;
                break;
              }
              if (!visible(el)) {
                continue;
              }
              if (
                el.id === "text-editor-host" ||
                el.id === "document-card-body-mount" ||
                el.classList.contains("document-monaco-slot") ||
                el.classList.contains("rich-text-editor-shell")
              ) {
                continue;
              }
              if (el.closest(straySelector)) {
                richEditorSubtreeViolations += 1;
                break;
              }
            }
            if (!reachedRich) {
              richEditorSubtreeViolations += 1;
            }
          }
        }
      }
      let outsideOverlapCount = 0;
      if (bodyRect && bodyRect.width > 40 && bodyRect.height > 40) {
        const shell = document.getElementById("document-card-shell");
        const sampleYs = [
          bodyRect.top + 8,
          bodyRect.top + bodyRect.height * 0.35,
          bodyRect.top + bodyRect.height * 0.65,
        ];
        for (const y of sampleYs) {
          const x = bodyRect.left + bodyRect.width / 2;
          for (const el of document.elementsFromPoint(x, y)) {
            if (!(el instanceof HTMLElement)) {
              continue;
            }
            if (shell?.contains(el) || el === shell || el === bodyMount) {
              break;
            }
            if (el.id === "command-manager" || el.closest("#command-manager")) {
              outsideOverlapCount += 1;
              break;
            }
            if (el.id === "document-viewer-host" && !el.classList.contains("hidden")) {
              outsideOverlapCount += 1;
              break;
            }
          }
        }
      }
      const approxBadge = document.querySelector<HTMLElement>(
        ".block-emulation-badge, .cm-emulation-mark",
      );
      return {
        viewportWidth: window.innerWidth,
        viewportHeight: window.innerHeight,
        railFlyoutWidth,
        pipeline: { groupsWidth: groups?.getBoundingClientRect().width ?? 0 },
        controls: controlDiagnostics,
        dom: {
          visibleDocumentCardCount: cards.length,
          commandManagerVisible,
          footerUsesCardTokens: Boolean(
            footerStyle &&
              statusStyle &&
              mutedToken &&
              paperToken &&
              (colorsMatch(footerStyle.color, mutedToken) ||
                colorsMatch(footerStyle.color, inkToken)) &&
              statusStyle.fontSize === "12px" &&
              colorsMatch(statusStyle.backgroundColor, paperToken),
          ),
          activeMode: mode,
          visibleEditorHost,
          modeMatchesVisibleHost,
          outsideOverlapCount,
          richEditorSubtreeViolations,
          documentCardFooterText: footerMain?.textContent?.trim() ?? "",
          docxEmulationBadgeVisible: Boolean(approxBadge?.textContent?.includes("≈")),
        },
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

  test("web doc feature screenshots v4", async ({ browser }) => {
    mkdirSync(ART, { recursive: true });
    mkdirSync(OUT, { recursive: true });
    const shots: { name: string; demo: string; wait?: string }[] = [
      {
        name: "doc-wysiwyg-blockquote-web-v4.png",
        demo: "doc-mode-wysiwyg,txt-apply-h2",
        wait: ".rich-text-doc blockquote",
      },
      {
        name: "doc-split-apply-banner-web-v4.png",
        demo: "doc-mode-split,doc-merge-banner",
        wait: ".document-source-merge-banner:not(.hidden)",
      },
      {
        name: "doc-opaque-card-web-v4.png",
        demo: "doc-opaque-card",
        wait: ".rich-text-opaque-parse",
      },
      {
        name: "doc-history-panel-web-v4.png",
        demo: "doc-history-panel,txt-apply-h2",
        wait: "#bottom-panel-history-pane:not(.hidden)",
      },
      {
        name: "doc-visual-docx-web-v4.png",
        demo: "doc-visual-md-docx",
        wait: "#document-card-tab .document-card-tab-label:text-is('sample.docx')",
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
      await assertDocFeatureCaptureDom(page);
      const outPath = path.join(OUT, shot.name);
      await page.screenshot({ path: outPath, fullPage: false });
      await page.screenshot({ path: path.join(ART, shot.name), fullPage: false });
      await writeWebCaptureCmJson(page, shot.name);
      await context.close();
    }
  });
});
