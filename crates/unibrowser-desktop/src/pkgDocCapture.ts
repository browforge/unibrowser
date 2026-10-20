import { invoke } from "@tauri-apps/api/core";
import { applyDemoState } from "./demoState";
import { applyThemeFromDemoState, loadThemePreference } from "./theme";
import { initAppMenu } from "./appMenu";
import { runShellChromeAudit } from "./shellChromeAudit";
import { collectDocCaptureDiagnostics } from "./docCaptureAsserts";
import { initActivityRail } from "./ui/activityRail";

const PKG_V2_TOKENS = [
  "pkg-v2-open-mode",
  "pkg-v2-history-notice",
  "pkg-v2-unpack-progress",
  "pkg-v2-file-menu",
] as const;

export function isPkgV2DocCapture(demo: string | null | undefined): boolean {
  const lower = (demo ?? "").toLowerCase();
  return PKG_V2_TOKENS.some((t) => lower.includes(t));
}

export async function runPkgV2DocCapture(
  demo: string,
  navigateOmnibox: () => void | Promise<void>,
  runKeybindingCommand: (id: string) => void | Promise<void>,
): Promise<void> {
  document.body.dataset.ubDocCapture = "1";
  document.body.dataset.ubDemoTokens = demo.toLowerCase();
  await loadThemePreference();
  applyThemeFromDemoState(demo);
  const workAreaRow = document.querySelector(".ub-work-area-row") as HTMLElement | null;
  if (workAreaRow) {
    initActivityRail({
      workAreaRow,
      onDestination: () => undefined,
    });
  }
  initAppMenu(
    (commandId) => runKeybindingCommand(commandId),
    (path) => {
      const omnibox = document.getElementById("omnibox") as HTMLInputElement | null;
      if (omnibox) {
        omnibox.value = path;
        void navigateOmnibox();
      }
    },
    { menuButtonId: "title-menu-btn" },
  );
  for (const token of PKG_V2_TOKENS) {
    if (demo.toLowerCase().includes(token)) {
      await applyDemoState(token);
    }
  }
  await new Promise((resolve) => window.setTimeout(resolve, 250));
  const auditErrors = runShellChromeAudit();
  await invoke("self_test_submit_shell_chrome_audit", { errors: auditErrors });
  if (auditErrors.length > 0) {
    await invoke("abort_doc_capture", {
      reason: `shell chrome audit: ${auditErrors.join("; ")}`,
    });
    return;
  }
  const diag = collectDocCaptureDiagnostics();
  await invoke("write_doc_capture_cm_json", {
    json: JSON.stringify({
      ...(diag.commandManager ?? {
        controlCount: 0,
        visibleCount: 0,
        contextLabel: "",
        controls: [],
      }),
      viewportWidth: window.innerWidth,
      viewportHeight: window.innerHeight,
      dom: {},
    }),
  });
  await invoke("finish_doc_capture_and_exit", { editorRegion: null });
}
