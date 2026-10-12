import { initWasmHost } from "./platform/wasmHost";
import { webInvoke } from "./platform/webInvoke";
import { mountWebPrivacyNotice } from "./webPrivacyNotice";

await initWasmHost();
document.documentElement.dataset.ubWebWasm = "1";
(window as unknown as { __ubWebInvoke?: typeof webInvoke }).__ubWebInvoke = webInvoke;
mountWebPrivacyNotice();
if ("serviceWorker" in navigator) {
  const base = import.meta.env.BASE_URL ?? "/";
  void navigator.serviceWorker.register(`${base}sw.js`).catch(() => undefined);
}
await import("./entry.ts");
if (new URLSearchParams(window.location.search).get("smoke") === "demo") {
  await new Promise((resolve) => window.setTimeout(resolve, 1500));
  await webInvoke("open_path", { path: "demo-pack/sample.md" });
  window.dispatchEvent(new CustomEvent("ub-browser-state-updated"));
}
