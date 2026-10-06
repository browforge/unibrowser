/** ADR 0013: non-blocking errors in chrome, never over the document surface. */

import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { pushNotification } from "./notifications";

let timer: ReturnType<typeof setTimeout> | null = null;

function formatErrorDetails(err: unknown): string {
  if (err instanceof Error) {
    const stack = err.stack ? `\n${err.stack}` : "";
    return `${err.name}: ${err.message}${stack}`;
  }
  return String(err);
}

/** Map raw invoke errors to short user-facing copy; keep full text in details. */
export function friendlyErrorMessage(err: unknown, fallback?: string): string {
  const raw = String(err).trim();
  const lower = raw.toLowerCase();
  if (!raw || lower === "undefined") {
    return fallback ?? t("toast.errorDefault");
  }
  if (lower.includes("cancelled") || lower.includes("canceled")) {
    return t("toast.cancelled");
  }
  if (lower.includes("epub")) {
    return t("toast.openEpubFailed");
  }
  if (lower.includes("encode") || lower.includes("ffmpeg") || lower.includes("h.264")) {
    return t("toast.encodeFailed");
  }
  if (lower.includes("permission") || lower.includes("denied")) {
    return t("toast.permissionDenied");
  }
  if (raw.startsWith("panic") || lower.includes("thread panicked")) {
    return fallback ?? t("toast.errorDefault");
  }
  if (raw.length > 160) {
    return fallback ?? t("toast.errorDefault");
  }
  return fallback ?? t("toast.errorDefault");
}

export function showActionToast(
  message: string,
  actionLabel: string,
  onAction: () => void,
  kind: "error" | "info" = "info",
) {
  pushNotification(message, kind);
  const host = document.getElementById("toast-host");
  if (!host) {
    return;
  }
  host.classList.remove("hidden", "toast-error", "toast-info");
  host.classList.add(kind === "error" ? "toast-error" : "toast-info");
  host.innerHTML = "";
  const msg = document.createElement("span");
  msg.className = "toast-message";
  msg.textContent = message;
  host.appendChild(msg);
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "toast-action";
  btn.textContent = actionLabel;
  btn.onclick = () => {
    onAction();
    host.classList.add("hidden");
    host.innerHTML = "";
  };
  host.appendChild(btn);
  if (timer) {
    clearTimeout(timer);
  }
  timer = setTimeout(() => {
    host.classList.add("hidden");
    host.innerHTML = "";
  }, 12_000);
}

export function showToast(
  message: string,
  kind: "error" | "info" = "error",
  details?: string,
) {
  pushNotification(message, kind, details);
  const host = document.getElementById("toast-host");
  if (!host) {
    return;
  }
  host.classList.remove("hidden", "toast-error", "toast-info");
  host.classList.add(kind === "error" ? "toast-error" : "toast-info");
  host.innerHTML = "";
  const msg = document.createElement("span");
  msg.className = "toast-message";
  msg.textContent = message;
  host.appendChild(msg);
  if (details && details.trim() && kind === "error") {
    const copyBtn = document.createElement("button");
    copyBtn.type = "button";
    copyBtn.className = "toast-copy-details";
    copyBtn.textContent = t("toast.copyDetails");
    copyBtn.onclick = async () => {
      try {
        await navigator.clipboard.writeText(details);
        copyBtn.textContent = t("toast.copied");
      } catch {
        copyBtn.textContent = t("toast.copyFailed");
      }
    };
    host.appendChild(copyBtn);
  }
  if (timer) {
    clearTimeout(timer);
  }
  timer = setTimeout(() => {
    host.classList.add("hidden");
    host.innerHTML = "";
  }, 8000);
}

export function showErrorToast(err: unknown, fallback?: string) {
  const details = formatErrorDetails(err);
  const message = friendlyErrorMessage(err, fallback);
  void invoke("record_diagnostic_error", {
    message,
    details,
  }).catch(() => {});
  showToast(message, "error", details);
}

export async function invokeWithToast<T>(
  fn: () => Promise<T>,
  fallback?: string,
): Promise<T | undefined> {
  const message = fallback ?? t("toast.errorDefault");
  try {
    return await fn();
  } catch (err) {
    showErrorToast(err, message);
    return undefined;
  }
}
