import "./styles.css";
import "./ui/tokens.css";
import { hideShellFallback } from "./shellBootstrap";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { commandManager } from "./commandManager";
import { applyChromeI18n } from "./chromeI18n";
import { loadUiLocale } from "./i18n";
import { loadThemePreference } from "./theme";
import {
  initToolbarOnlyShell,
  mainShellShouldBeAbsent,
  mountToolbarOnlyDocument,
} from "./toolbarOnly";

const floatingToolbarHost = document.getElementById("floating-toolbar-host") as HTMLElement;
if (!floatingToolbarHost) {
  throw new Error("floating-toolbar-host missing");
}

const hiddenCmHost = document.createElement("div");
hiddenCmHost.hidden = true;
hiddenCmHost.setAttribute("aria-hidden", "true");
document.body.appendChild(hiddenCmHost);

const commandManagerGroups = document.createElement("div");
commandManagerGroups.id = "command-manager-groups-stub";
const commandManagerPin = document.createElement("button");
commandManagerPin.type = "button";
const commandManagerContext = document.createElement("button");
commandManagerContext.type = "button";
const commandManagerOverflow = document.createElement("button");
commandManagerOverflow.type = "button";
hiddenCmHost.append(
  commandManagerGroups,
  commandManagerPin,
  commandManagerContext,
  commandManagerOverflow,
);

async function runUiCommand(commandId: string) {
  await invoke("execute_command", { commandId });
  await refreshCommandManager();
}

async function refreshCommandManager() {
  await commandManager.refresh(
    commandManagerGroups,
    floatingToolbarHost,
    commandManagerPin,
    commandManagerContext,
    commandManagerOverflow,
    runUiCommand,
  );
}

mountToolbarOnlyDocument();
document.body.dataset.ubToolbarOnly = "1";

void loadThemePreference();
void loadUiLocale().then(() => applyChromeI18n());

void initToolbarOnlyShell(refreshCommandManager).then(async () => {
  hideShellFallback();
  await refreshCommandManager();
  window.addEventListener("resize", () => {
    void refreshCommandManager();
  });
  window.addEventListener("unibrowser-workspace-layout-loaded", () => {
    void refreshCommandManager();
  });
  void listen("unibrowser-demo-cm-refresh", () => {
    void refreshCommandManager();
  });

  const docCapture = await invoke<string | null>("take_doc_capture_path");
  if (!docCapture) {
    return;
  }
  const demo = await invoke<string | null>("get_demo_state");
  if (!demo?.toLowerCase().includes("cmd-proving-detached")) {
    return;
  }
  const waitForBar = async (attempts = 120) => {
    for (let i = 0; i < attempts; i += 1) {
      const host = document.getElementById("floating-toolbar-host");
      const bar = host?.querySelector(".floating-bar");
      if (bar && bar.querySelector(".ui-command")) {
        return true;
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    return false;
  };
  const painted = await waitForBar();
  if (!painted) {
    await invoke("abort_doc_capture", {
      reason: "detached toolbar bar did not paint controls",
    });
    return;
  }
  if (!mainShellShouldBeAbsent(document)) {
    await invoke("abort_doc_capture", {
      reason: "toolbar window still contains workbench shell roots",
    });
    return;
  }
  await invoke("finish_doc_capture_and_exit", { editorRegion: null });
});
