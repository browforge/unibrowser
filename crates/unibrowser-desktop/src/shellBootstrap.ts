import { invoke } from "@tauri-apps/api/core";

const FALLBACK_ID = "shell-fallback";
const NOTICE_ID = "shell-subsystem-notice";

function fallbackEl(): HTMLElement | null {
  return document.getElementById(FALLBACK_ID);
}

function noticeEl(): HTMLElement | null {
  return document.getElementById(NOTICE_ID);
}

export function hideShellFallback() {
  document.body.dataset.ubReady = "1";
  const fb = fallbackEl();
  if (fb) {
    fb.classList.add("hidden");
  }
}

export function showShellFallbackMessage(message: string) {
  const fb = fallbackEl();
  if (!fb) {
    return;
  }
  fb.classList.remove("hidden");
  const msg = fb.querySelector(".shell-fallback-detail");
  if (msg) {
    msg.textContent = message;
  }
}

export function reportSubsystemError(subsystem: string, err: unknown) {
  const text = err instanceof Error ? err.message : String(err);
  const line = `${subsystem}: ${text}`;
  console.error("[shell]", line);
  void invoke("log_shell_ui_event", {
    level: "error",
    message: subsystem,
    detail: text,
  }).catch(() => undefined);
  const notice = noticeEl();
  if (notice) {
    notice.classList.remove("hidden");
    const span = notice.querySelector(".shell-subsystem-notice-text");
    if (span) {
      span.textContent = `Some panels failed to load (${subsystem}). The shell is still usable.`;
    }
  }
}

export function safeSubsystemInit(name: string, fn: () => void) {
  try {
    fn();
  } catch (err) {
    reportSubsystemError(name, err);
  }
}

function logUi(level: string, message: string, detail?: string) {
  void invoke("log_shell_ui_event", {
    level,
    message,
    detail: detail ?? null,
  }).catch(() => undefined);
}

window.addEventListener("error", (ev) => {
  const detail = ev.filename ? `${ev.filename}:${ev.lineno ?? 0}` : undefined;
  logUi("error", ev.message || "window error", detail);
  showShellFallbackMessage(ev.message || "A script error occurred. Check diagnostics.");
});

window.addEventListener("unhandledrejection", (ev) => {
  const reason = ev.reason instanceof Error ? ev.reason.message : String(ev.reason);
  logUi("error", "unhandled rejection", reason);
  showShellFallbackMessage(reason);
});
