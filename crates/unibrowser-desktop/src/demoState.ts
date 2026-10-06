import { invoke } from "@tauri-apps/api/core";
import { setAiPanelVisible } from "./aiAssistantPanel";
import { t } from "./i18n";
import { showActionToast } from "./toast";

/** Hidden preset for docs screenshots (`UNIBROWSER_DEMO_STATE`, comma-separated). */
export async function applyDemoState(raw: string) {
  const tokens = raw
    .split(/[,;]+/)
    .map((t) => t.trim().toLowerCase())
    .filter(Boolean);
  const demoRaw = raw.toLowerCase();

  for (const token of tokens) {
    switch (token) {
      case "problems":
      case "problems-open": {
        document.body.dataset.demoProblems = "1";
        const { showProblemsBottomPanel } = await import("./problemsPanel");
        await invoke("proofing_check", {
          text: "teh quick brown fox",
          surface: "plaintext",
          language: "en-NZ",
        }).catch(() => undefined);
        showProblemsBottomPanel();
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
        document.body.dataset.demoVsixBadge = "1";
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
        await invoke("open_path", { path: "unibrowser://settings", line: null });
        break;
      case "mods":
        await invoke("open_path", { path: "unibrowser://mods", line: null });
        break;
      case "theme-light":
        document.documentElement.setAttribute("data-theme", "light");
        break;
      case "theme-dark":
        document.documentElement.setAttribute("data-theme", "dark");
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
      case "rail-expanded": {
        try {
          localStorage.setItem("ub-rail-mode", "expanded");
        } catch {
          /* ignore */
        }
        const rail = document.getElementById("activity-rail");
        rail?.classList.remove("collapsed", "auto-hide", "hidden");
        rail?.classList.add("expanded");
        break;
      }
      case "rail-collapsed": {
        try {
          localStorage.setItem("ub-rail-mode", "collapsed");
        } catch {
          /* ignore */
        }
        const rail = document.getElementById("activity-rail");
        rail?.classList.remove("expanded", "auto-hide", "hidden");
        rail?.classList.add("collapsed");
        break;
      }
      case "scale-150":
        document.documentElement.style.fontSize = "150%";
        break;
      case "cm-collapsed":
        document.body.classList.add("cm-collapsed");
        break;
      case "capture-ready":
        document.getElementById("palette")?.classList.add("hidden");
        document.getElementById("bookmark-bar")?.classList.add("hidden");
        document.getElementById("onboarding-banner")?.classList.add("hidden");
        document.getElementById("floating-toolbar-host")?.classList.add("hidden");
        if (!demoRaw.includes("problems")) {
          document.getElementById("script-output-panel")?.classList.add("hidden");
        }
        break;
      default:
        break;
    }
  }
}
