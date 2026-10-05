import { showToast } from "./toast";
import { t } from "./i18n";

export type FilesClipboardMode = "copy" | "cut";

export interface FilesClipboardPayload {
  mode: FilesClipboardMode;
  paths: string[];
}

let internal: FilesClipboardPayload | null = null;

export function filesClipboardSet(mode: FilesClipboardMode, paths: string[]) {
  if (!paths.length) {
    internal = null;
    return;
  }
  internal = { mode, paths: [...paths] };
  showToast(
    mode === "copy" ? t("files.clipboardCopied") : t("files.clipboardCut"),
    "info",
  );
}

export function filesClipboardPeek(): FilesClipboardPayload | null {
  return internal;
}

export function filesClipboardClear() {
  internal = null;
}
