import { invoke } from "@tauri-apps/api/core";
import { applyDemoState } from "./demoState";
import { collectDocCaptureDiagnostics } from "./docCaptureAsserts";
import { runShellChromeAudit } from "./shellChromeAudit";
import { waitForShellPaintReady } from "./shellPaintReady";
import { hideShellFallback } from "./shellBootstrap";
import { commandManager } from "./commandManager";
import { applyThemeFromDemoState, loadThemePreference } from "./theme";
import { initAppMenu } from "./appMenu";
import { initActivityRail } from "./ui/activityRail";
import { initCompactTitleBar } from "./ui/compactTitleBar";
import { renderRecoveryBar } from "./recoveryBar";
import { textEditor, type TextDocumentDto } from "./textEditor";

const PKG_CAPTURE_TOKENS = [
  "pkg-v2-open-mode",
  "pkg-v2-history-notice",
  "pkg-v2-unpack-progress",
  "pkg-v2-file-menu",
  "pkg-v3-open-mode",
  "pkg-v3-history-notice",
  "pkg-v3-unpack-progress",
  "pkg-v3-file-menu",
  "pkg-v4-open-mode",
  "pkg-v4-history-notice",
  "pkg-v4-unpack-progress",
  "pkg-v4-file-menu",
  "rec-capture-banner",
  "rec-capture-restored",
] as const;

export function isPkgDocCapture(demo: string | null | undefined): boolean {
  const lower = (demo ?? "").toLowerCase();
  return PKG_CAPTURE_TOKENS.some((t) => lower.includes(t));
}

export type PkgDocCaptureDeps = {
  navigateOmnibox: () => void | Promise<void>;
  runKeybindingCommand: (commandId: string) => void | Promise<void>;
  runUiCommand: (commandId: string) => Promise<void>;
  refreshState: () => Promise<void>;
  refreshCommandManager: () => Promise<void>;
  applyState: (data: unknown) => Promise<void>;
  openTitleOverflow: (anchor: HTMLElement) => void;
};

function bindCommandManagerChrome(deps: PkgDocCaptureDeps) {
  const commandManagerGroups = document.getElementById("command-manager-groups");
  const commandManagerCollapse = document.getElementById("command-manager-collapse");
  const commandManagerPin = document.getElementById("command-manager-pin");
  const commandManagerContext = document.getElementById("command-manager-context");
  const commandManagerOverflow = document.getElementById("command-manager-overflow");
  const floatingToolbarHost = document.getElementById("floating-toolbar-host");
  if (!commandManagerGroups || !floatingToolbarHost) {
    return;
  }
  if (commandManagerCollapse) {
    commandManager.bindCollapseControl(commandManagerCollapse);
  }
  if (commandManagerPin && commandManagerContext && commandManagerOverflow) {
    commandManager.bindPinControl(
      commandManagerPin,
      commandManagerGroups,
      floatingToolbarHost,
      commandManagerContext,
      commandManagerOverflow,
      (commandId) => Promise.resolve(deps.runUiCommand(commandId)),
    );
  }
}

function mountCaptureActivityRail() {
  const workAreaRow = document.querySelector(".ub-work-area-row") as HTMLElement | null;
  if (!workAreaRow) {
    return;
  }
  initActivityRail({
    workAreaRow,
    onDestination: () => undefined,
  });
}

function initCaptureChrome(deps: PkgDocCaptureDeps) {
  hideShellFallback();
  initCompactTitleBar({
    runCommand: (commandId) => deps.runKeybindingCommand(commandId),
    openTitleOverflow: deps.openTitleOverflow,
    onBack: () => undefined,
    onForward: () => undefined,
    onReload: () => undefined,
    navigateOmnibox: () => void deps.navigateOmnibox(),
  });
  mountCaptureActivityRail();
  bindCommandManagerChrome(deps);
  initAppMenu(
    (commandId) => deps.runKeybindingCommand(commandId),
    (path) => {
      const omnibox = document.getElementById("omnibox") as HTMLInputElement | null;
      if (omnibox) {
        omnibox.value = path;
        void deps.navigateOmnibox();
      }
    },
    { menuButtonId: "title-menu-btn" },
  );
}

async function applyPkgCaptureToken(
  demo: string,
  _deps: PkgDocCaptureDeps,
  _launchPaths: string[],
): Promise<void> {
  const lower = demo.toLowerCase();
  for (const token of PKG_CAPTURE_TOKENS) {
    if (lower.includes(token)) {
      await applyDemoState(token);
      return;
    }
  }
}

export async function runPkgDocCapture(demo: string, deps: PkgDocCaptureDeps): Promise<void> {
  document.body.dataset.ubDocCapture = "1";
  document.body.dataset.ubDemoTokens = demo.toLowerCase();
  await loadThemePreference();
  applyThemeFromDemoState(demo);
  if (demo.toLowerCase().includes("theme-light") || demo.toLowerCase().includes("theme-dark")) {
    const { applyMonacoEditorTheme } = await import("./monacoEditorTheme");
    await applyMonacoEditorTheme("auto");
  }
  initCaptureChrome(deps);
  void deps.refreshCommandManager();

  const launchPaths = await invoke<string[]>("take_launch_open_paths");
  const lower = demo.toLowerCase();
  const wantsOpenMode =
    lower.includes("pkg-v3-open-mode") || lower.includes("pkg-v4-open-mode");
  const wantsRec =
    lower.includes("rec-capture-banner") || lower.includes("rec-capture-restored");
  if (wantsRec) {
    const recoveryHost = document.getElementById("recovery-bar-host");
    if (recoveryHost && lower.includes("rec-capture-banner")) {
      renderRecoveryBar(
        recoveryHost,
        [
          {
            backup_id: "0000000000000001",
            path: "e2e/fixtures/spell-test.txt",
            backend_id: "base-text",
          },
        ],
        () => {
          void deps.refreshState();
        },
      );
    }
    if (lower.includes("rec-capture-restored")) {
      const doc = await invoke<TextDocumentDto>("rec_capture_restored_text_document", {
        path: "e2e/fixtures/spell-test.txt",
      });
      const textHost = document.getElementById("text-editor-host");
      if (textHost) {
        textHost.classList.remove("hidden");
        await textEditor.apply(doc, textHost);
      }
      const omnibox = document.getElementById("omnibox") as HTMLInputElement | null;
      if (omnibox) {
        omnibox.value = "e2e/fixtures/spell-test.txt";
      }
    }
  } else if (wantsOpenMode) {
    const { applyPkgArchiveCaptureSession } = await import("./packageStorageUi");
    const { showPackageOpenModeCapture } = await import("./packageOpenModeDialog");
    await applyPkgArchiveCaptureSession();
    const path = await invoke<string>("pkg_capture_fixture_archive_path_cmd");
    const estimate = await invoke<{
      archive_path: string;
      unpacked_bytes: number;
      suggest_folder: boolean;
    }>("query_unibrow_open_cmd", { path });
    showPackageOpenModeCapture(estimate);
  } else {
    await applyPkgCaptureToken(demo, deps, launchPaths);
  }

  await new Promise((resolve) => window.setTimeout(resolve, 200));
  const painted = await waitForShellPaintReady(demo, launchPaths, 45_000);
  if (!painted) {
    await invoke("abort_doc_capture", {
      reason: "PKG doc-capture shell paint readiness timed out",
    });
    return;
  }

  const { dismissDocCaptureChrome } = await import("./docCaptureDismiss");
  dismissDocCaptureChrome();
  const { setRailMode } = await import("./ui/activityRail");
  setRailMode("collapsed");

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
