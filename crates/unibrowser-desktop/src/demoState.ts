import { invoke } from "@tauri-apps/api/core";
import { setAiDemoComposerSeed, setAiPanelVisible } from "./aiAssistantPanel";
import { applyThemeFromDemoState, applyThemeOverride } from "./theme";
import { t } from "./i18n";
import { showActionToast } from "./toast";
import { isToolbarOnlyWindow } from "./toolbarOnly";

/** Hidden preset for docs screenshots (`UNIBROWSER_DEMO_STATE`, comma-separated). */
export async function applyDemoState(raw: string) {
  const tokens = raw
    .split(/[,;]+/)
    .map((t) => t.trim().toLowerCase())
    .filter(Boolean);
  const demoRaw = raw.toLowerCase();
  const waitForElement = async <T extends Element>(selector: string): Promise<T | null> => {
    for (let attempt = 0; attempt < 80; attempt += 1) {
      const element = document.querySelector<T>(selector);
      if (element) {
        return element;
      }
      await new Promise((resolve) => window.setTimeout(resolve, 50));
    }
    return null;
  };

  for (const token of tokens) {
    switch (token) {
      case "document-mockup":
      case "document-mockup-problems": {
        const { showProblemsBottomPanel } = await import("./problemsPanel");
        await invoke("seed_document_mockup_problems_cmd").catch(() => undefined);
        await showProblemsBottomPanel();
        break;
      }
      case "cm-text-pin": {
        await invoke("select_command_manager_tab", { tabId: "text" });
        await invoke("pin_command_manager_context");
        window.dispatchEvent(new Event("unibrowser-demo-cm-refresh"));
        break;
      }
      case "cmd-proving-detached": {
        if (!isToolbarOnlyWindow()) {
          await invoke("detach_floating_toolbar_window", {
            barId: "cmd-proving.detached-markdown",
          });
        }
        break;
      }
      case "problems":
      case "problems-open": {
        const { showProblemsBottomPanel } = await import("./problemsPanel");
        await invoke("proofing_check", {
          text: "teh quick brown fox",
          surface: "plaintext",
          language: "en-NZ",
        }).catch(() => undefined);
        await showProblemsBottomPanel();
        break;
      }
      case "ai":
      case "ai-sidebar":
      case "ai-open":
        await invoke("set_ai_assistant_prefs", {
          prefs: {
            base_url: "mock://unibrowser-self-test",
            model: "docs",
            api_key: null,
            grants: [
              { permission: "read", scope_kind: "global", scope_value: null },
              { permission: "ask", scope_kind: "workspace", scope_value: "/workspace" },
            ],
            panel_visible: true,
            panel_x: 24,
            panel_y: 96,
            panel_docked: true,
          },
        });
        await setAiPanelVisible(true);
        break;
      case "live":
      case "live-preview":
        try {
          await invoke("set_unibrow_live_mode", { enabled: true });
        } catch {
          /* unibrow tab may not be active yet */
        }
        break;
      case "vsix":
      case "vsix-badge":
        await invoke("open_path", { path: "unibrowser://mods", line: null });
        break;
      case "palette":
        document.getElementById("palette")?.classList.remove("hidden");
        const pq = document.getElementById("palette-query") as HTMLInputElement | null;
        if (pq) {
          pq.value = "font";
          pq.dispatchEvent(new Event("input", { bubbles: true }));
        }
        break;
      case "settings":
        window.dispatchEvent(
          new CustomEvent("unibrowser-navigate", { detail: "unibrowser://settings" }),
        );
        await waitForElement(".settings-page");
        break;
      case "mods":
        await invoke("open_path", { path: "unibrowser://mods", line: null });
        break;
      case "theme-light":
        applyThemeOverride("light");
        {
          const { applyMonacoEditorTheme } = await import("./monacoEditorTheme");
          await applyMonacoEditorTheme("auto");
        }
        break;
      case "theme-dark":
        applyThemeOverride("dark");
        {
          const { applyMonacoEditorTheme } = await import("./monacoEditorTheme");
          await applyMonacoEditorTheme("auto");
        }
        break;
      case "title-breadcrumb":
        document.body.dataset.titleBreadcrumb = "1";
        break;
      case "second-tab-sample-pdf":
        await invoke("open_path", {
          path: "/workspace/examples/demo-pack/sample.pdf",
          line: null,
        }).catch(() => undefined);
        break;
      case "ai-mockup-composer":
        setAiDemoComposerSeed("Why does line 8 trigger a warning?");
        break;
      case "feedback-demo": {
        showActionToast(t("tabs.closedUndo"), t("tabs.undo"), () => undefined, "info");
        const host = document.getElementById("recovery-bar-host");
        if (host) {
          host.classList.remove("hidden");
          const row = document.createElement("div");
          row.className = "recovery-bar-row";
          const label = document.createElement("span");
          label.textContent = "Unsaved changes were recovered for spell-test.txt";
          const view = document.createElement("button");
          view.type = "button";
          view.textContent = "View original";
          const discard = document.createElement("button");
          discard.type = "button";
          discard.textContent = "Discard";
          row.append(label, view, discard);
          host.replaceChildren(row);
        }
        break;
      }
      case "ntp":
        await invoke("new_tab", {});
        break;
      case "files":
        await invoke("open_path", { path: "~", line: null }).catch(() => undefined);
        break;
      case "ui26-file-preview":
      case "ui13-files-preview": {
        const links = await waitForElement<HTMLButtonElement>(".files-table .file-link");
        if (!links) {
          break;
        }
        const preferred = [...document.querySelectorAll<HTMLButtonElement>(".files-table .file-link")]
          .find((link) => link.textContent?.toLowerCase().endsWith(".svg"));
        const row = (preferred ?? links).closest("tr");
        row?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
        await waitForElement(".files-side-panel:not(.hidden) .files-props-dl");
        break;
      }
      case "ui13-settings-developer-collapsed":
      case "ui13-settings-developer-expanded": {
        const details = await waitForElement<HTMLDetailsElement>(".settings-developer");
        if (!details) {
          break;
        }
        details.open = token.endsWith("expanded");
        const category = document.querySelector<HTMLButtonElement>(
          '[data-settings-category-target="developer"]',
        );
        category?.click();
        details.scrollIntoView({ block: "start" });
        break;
      }
      case "cm-large": {
        const { applyCommandManagerDensity } = await import("./accessibility");
        applyCommandManagerDensity(false);
        break;
      }
      case "cm-compact": {
        const { applyCommandManagerDensity } = await import("./accessibility");
        applyCommandManagerDensity(true);
        break;
      }
      case "cmd11-plain-proof":
      case "cmd11-markdown-proof": {
        const controlId =
          token === "cmd11-plain-proof"
            ? "unibrowser.control.text.strikethrough"
            : "unibrowser.control.text.subscript";
        await new Promise((resolve) => window.setTimeout(resolve, 50));
        const control = document.querySelector<HTMLElement>(
          `[data-control-id="${controlId}"]`,
        );
        control?.focus();
        break;
      }
      case "ui14-dxf-measure-reason":
        break;
      case "ui14-open-command-overflow": {
        const overflow = await waitForElement<HTMLButtonElement>(
          "#command-manager-overflow.visible",
        );
        overflow?.click();
        await waitForElement(".cm-overflow-menu");
        break;
      }
      case "ui14-pdf-100":
        await invoke("sync_pdf_scale", { scale: 1 }).catch(() => undefined);
        break;
      case "ui15-unibrow-title": {
        const clip = await waitForElement<HTMLElement>(
          '.workshop-clip[data-clip-id="clip-1"]',
        );
        clip?.focus();
        break;
      }
      case "ui16-live-preview":
        window.dispatchEvent(
          new CustomEvent("unibrow-workshop-visual-mode", {
            detail: "preview",
          }),
        );
        break;
      case "ui16-live-on": {
        window.dispatchEvent(
          new CustomEvent("unibrow-workshop-visual-mode", {
            detail: "live",
          }),
        );
        break;
      }
      case "ui16-live-recording": {
        window.dispatchEvent(
          new CustomEvent("unibrow-workshop-visual-mode", {
            detail: "recording",
          }),
        );
        break;
      }
      case "ui17-mod-portal": {
        const data = await invoke("open_mod_portal", { ext: "step" }).catch(
          () => null,
        );
        if (data) {
          window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
        }
        break;
      }
      case "ui18-layout-edit": {
        const { setLayoutEditMode } = await import("./layoutEditMode");
        setLayoutEditMode(true);
        break;
      }
      case "ui19-focus-visible": {
        const command = await waitForElement<HTMLButtonElement>(
          '[data-control-id="unibrowser.control.find"], #command-manager-groups .ui-command',
        );
        command?.focus();
        break;
      }
      case "rail-expanded": {
        const { setRailMode } = await import("./ui/activityRail");
        setRailMode("expanded");
        break;
      }
      case "rail-collapsed": {
        const { setRailMode } = await import("./ui/activityRail");
        setRailMode("collapsed");
        break;
      }
      case "scale-100":
      case "scale-150":
        {
          const { applyUiScaleToDocument } = await import("./accessibility");
          const percent = token === "scale-150" ? 150 : 100;
          applyUiScaleToDocument(percent);
          await invoke("set_ui_scale_percent", { percent }).catch(() => undefined);
        }
        break;
      case "standard-contrast": {
        const { applyHighContrastToDocument } = await import("./accessibility");
        applyHighContrastToDocument(false);
        await invoke("set_accessibility_prefs", {
          high_contrast: false,
          reduce_motion: null,
        }).catch(() => undefined);
        break;
      }
      case "high-contrast": {
        const { applyHighContrastToDocument } = await import("./accessibility");
        applyHighContrastToDocument(true);
        await invoke("set_accessibility_prefs", {
          high_contrast: true,
          reduce_motion: null,
        }).catch(() => undefined);
        break;
      }
      case "cm-collapsed":
        document.body.classList.add("cm-collapsed");
        break;
      case "capture-ready":
        await invoke("seed_demo_ntp_fixture_cmd").catch(() => undefined);
        await invoke("clear_command_manager_context_pin").catch(() => undefined);
        for (const ctx of ["ntp", "pdf", "image"] as const) {
          try {
            localStorage.setItem(`ub-dock-ai-pref:${ctx}`, "0");
          } catch {
            /* ignore */
          }
        }
        await setAiPanelVisible(false);
        applyThemeFromDemoState(demoRaw);
        {
          const { applyMonacoEditorTheme } = await import("./monacoEditorTheme");
          await applyMonacoEditorTheme("auto");
        }
        break;
      default:
        break;
    }
  }
}
