import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { showErrorToast, showToast } from "./toast";

interface CaptureTabResultDto {
  saved_path: string | null;
  need_shell: boolean;
}

function canvasFromImage(img: HTMLImageElement): HTMLCanvasElement | null {
  if (!img.naturalWidth || !img.naturalHeight) {
    return null;
  }
  const canvas = document.createElement("canvas");
  canvas.width = img.naturalWidth;
  canvas.height = img.naturalHeight;
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    return null;
  }
  ctx.drawImage(img, 0, 0);
  return canvas;
}

async function captureShellSurfacePng(): Promise<string | null> {
  const img = document.querySelector(
    "#document-viewer-host img.image-view",
  ) as HTMLImageElement | null;
  if (img) {
    const canvas = canvasFromImage(img);
    if (canvas) {
      return canvas.toDataURL("image/png");
    }
  }
  const pdfCanvas = document.querySelector(
    "#document-viewer-host canvas",
  ) as HTMLCanvasElement | null;
  if (pdfCanvas) {
    return pdfCanvas.toDataURL("image/png");
  }
  return null;
}

export async function runCaptureVisibleTab() {
  try {
    const first = await invoke<CaptureTabResultDto>("capture_visible_tab");
    if (first.saved_path) {
      showToast(t("capture.saved", { path: first.saved_path }), "info");
      window.dispatchEvent(new CustomEvent("unibrowser-refresh"));
      return;
    }
    if (!first.need_shell) {
      showToast(t("capture.failed"), "error");
      return;
    }
    const dataUrl = await captureShellSurfacePng();
    if (!dataUrl) {
      showToast(t("capture.unsupported"), "error");
      return;
    }
    await invoke("save_tab_capture_png", { pngBase64: dataUrl });
    showToast(t("capture.savedGeneric"), "info");
    window.dispatchEvent(new CustomEvent("unibrowser-refresh"));
  } catch (err) {
    showErrorToast(err, t("capture.failed"));
  }
}
