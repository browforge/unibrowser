import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import { showToast } from "./toast";
import { t } from "./i18n";

let lastCopiedImagePath: string | null = null;

export function getLastCopiedImagePath(): string | null {
  return lastCopiedImagePath;
}

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

export async function copyImageFile(path: string): Promise<boolean> {
  try {
    const res = await fetch(convertFileSrc(path));
    const blob = await res.blob();
    const type = blob.type || "image/png";
    await navigator.clipboard.write([new ClipboardItem({ [type]: blob })]);
    lastCopiedImagePath = path;
    showToast(t("clipboard.imageCopied"), "info");
    return true;
  } catch {
    showToast(t("clipboard.copyFailed"), "error");
    return false;
  }
}

export async function readClipboardImagePath(): Promise<string | null> {
  if (!navigator.clipboard.read) {
    return lastCopiedImagePath;
  }
  try {
    const items = await navigator.clipboard.read();
    for (const item of items) {
      const type = item.types.find((t) => t.startsWith("image/"));
      if (!type) {
        continue;
      }
      const blob = await item.getType(type);
      const bytes = Array.from(new Uint8Array(await blob.arrayBuffer()));
      const path = await invoke<string>("write_temp_png", { bytes });
      return path;
    }
  } catch {
    // Fall back to last copied file path from the viewer.
  }
  return lastCopiedImagePath;
}

export async function readClipboardText(): Promise<string | null> {
  try {
    const text = await navigator.clipboard.readText();
    return text.trim() || null;
  } catch {
    return null;
  }
}
