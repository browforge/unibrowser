import { invoke } from "@tauri-apps/api/core";
import { documentViewer } from "./documentViewer";
import { getPrintHtmlForPdf, textEditor } from "./textEditor";
import { showToast } from "./toast";
import { t } from "./i18n";

export async function runExportPrintPdf(viewerHost: HTMLElement) {
  await textEditor.flush();
  const payload =
    getPrintHtmlForPdf() ?? documentViewer.getPrintHtmlForPdf(viewerHost);
  if (!payload) {
    showToast(t("export.pdf.noHtml"), "error");
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
    showToast(text, "error");
  }
}
