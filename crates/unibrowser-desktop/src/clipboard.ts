import { showToast } from "./toast";
import { t } from "./i18n";

export async function copyText(label: string, onCopied?: () => void): Promise<boolean> {
  const trimmed = label.trim();
  if (!trimmed) {
    showToast(t("clipboard.nothing"), "info");
    return false;
  }
  try {
    await navigator.clipboard.writeText(trimmed);
    onCopied?.();
    return true;
  } catch {
    showToast(t("clipboard.copyFailed"), "error");
    return false;
  }
}

export async function readClipboardText(): Promise<string | null> {
  try {
    const text = await navigator.clipboard.readText();
    return text.trim() || null;
  } catch {
    return null;
  }
}
