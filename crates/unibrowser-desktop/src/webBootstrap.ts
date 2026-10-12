import { initWasmHost } from "./platform/wasmHost";
import { webInvoke } from "./platform/webInvoke";
import { showShellFallbackMessage } from "./shellBootstrap";
import { mountWebPrivacyNotice } from "./webPrivacyNotice";
import "./styles.css";
import "./ui/tokens.css";
import "./ui/compactChrome.css";
import "./ui/quietUtilityShell.css";

const SHELL_MOUNT_TIMEOUT_MS = 10_000;

function shellMounted(): boolean {
  return (
    document.body.dataset.ubReady === "1" &&
    Boolean(document.getElementById("command-manager")?.querySelector("#command-manager-groups"))
  );
}

function reportShellMountFailure(cause: string) {
  const detail = `${cause} Check the browser console and try a hard refresh (Ctrl+Shift+R).`;
  showShellFallbackMessage(detail);
  console.error("[web-demo] shell mount timeout:", cause);
}

let mountWatch: ReturnType<typeof setTimeout> | null = null;

function startShellMountWatch() {
  mountWatch = window.setTimeout(() => {
    if (!shellMounted()) {
      reportShellMountFailure("The shell did not finish loading in time.");
    }
  }, SHELL_MOUNT_TIMEOUT_MS);
}

function clearShellMountWatch() {
  if (mountWatch !== null) {
    window.clearTimeout(mountWatch);
    mountWatch = null;
  }
}

try {
  await initWasmHost();
  document.documentElement.dataset.ubWebWasm = "1";
  (window as unknown as { __ubWebInvoke?: typeof webInvoke }).__ubWebInvoke = webInvoke;
  mountWebPrivacyNotice();
  if ("serviceWorker" in navigator) {
    const base = import.meta.env.BASE_URL ?? "/";
    void navigator.serviceWorker.register(`${base}sw.js`).catch(() => undefined);
  }
  startShellMountWatch();
  await import("./webMain.ts");
  if (!shellMounted()) {
    await new Promise<void>((resolve) => {
      const deadline = Date.now() + SHELL_MOUNT_TIMEOUT_MS;
      const tick = () => {
        if (shellMounted() || Date.now() >= deadline) {
          resolve();
          return;
        }
        window.setTimeout(tick, 50);
      };
      tick();
    });
  }
  if (!shellMounted()) {
    reportShellMountFailure("The shell loaded but the command manager never mounted.");
  } else {
    clearShellMountWatch();
  }
} catch (err) {
  clearShellMountWatch();
  const message = err instanceof Error ? err.message : String(err);
  reportShellMountFailure(message);
}

if (new URLSearchParams(window.location.search).get("smoke") === "demo") {
  await new Promise((resolve) => window.setTimeout(resolve, 1500));
  await webInvoke("open_path", { path: "demo-pack/sample.md" });
  window.dispatchEvent(new CustomEvent("ub-browser-state-updated"));
}
