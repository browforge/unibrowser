import { invoke } from "@tauri-apps/api/core";
import { documentViewer } from "./documentViewer";
import { getPrintHtmlForPdf, getPrintSelectionHtml, textEditor } from "./textEditor";
import { openPrintFriendlyDocument, openPrintPreviewDocument } from "./printExport";
import { showErrorToast, showToast } from "./toast";
import { t } from "./i18n";

export async function runPrintSelection(viewerHost: HTMLElement) {
  await textEditor.flush();
  const payload =
    getPrintSelectionHtml() ?? documentViewer.getPrintSelectionHtml(viewerHost);
  if (!payload) {
    showToast(t("print.selection.none"), "error");
    return;
  }
  openPrintFriendlyDocument(payload.title, payload.html);
}

export async function runExportPrintPdf(viewerHost: HTMLElement) {
  await textEditor.flush();
  const payload =
    getPrintHtmlForPdf() ?? documentViewer.getPrintHtmlForPdf(viewerHost);
  if (!payload) {
    showToast(t("export.pdf.noHtml"), "error");
    return;
  }
  if (!openPrintPreviewDocument(payload.title, payload.html)) {
    showToast(t("export.pdf.previewBlocked"), "error");
    return;
  }
  try {
    const msg = await invoke<string>("export_print_pdf", {
      htmlBody: payload.html,
      suggestedTitle: payload.title,
    });
    showToast(msg, "info");
  } catch (err) {
    const text = String(err);
    if (text.includes("cancelled")) {
      return;
    }
    showErrorToast(err);
  }
}
