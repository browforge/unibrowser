/** ADR 0013: non-blocking errors in chrome, never over the document surface. */

let timer: ReturnType<typeof setTimeout> | null = null;

export function showToast(message: string, kind: "error" | "info" = "error") {
  const host = document.getElementById("toast-host");
  if (!host) {
    return;
  }
  host.classList.remove("hidden", "toast-error", "toast-info");
  host.classList.add(kind === "error" ? "toast-error" : "toast-info");
  host.textContent = message;
  if (timer) {
    clearTimeout(timer);
  }
  timer = setTimeout(() => {
    host.classList.add("hidden");
    host.textContent = "";
  }, 6000);
}

export async function invokeWithToast<T>(
  fn: () => Promise<T>,
  fallback?: string,
): Promise<T | undefined> {
  const { t } = await import("./i18n");
  const message = fallback ?? t("toast.errorDefault");
  try {
    return await fn();
  } catch (err) {
    showToast(String(err) || message, "error");
    return undefined;
  }
}
