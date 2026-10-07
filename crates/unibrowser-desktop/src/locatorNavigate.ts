import { invoke } from "@tauri-apps/api/core";

export type NavigationTarget =
  | { kind: "text_line_col"; line: number; col: number; end_line: number; end_col: number }
  | { kind: "text_byte_range"; start_byte: number; end_byte: number }
  | { kind: "docx_paragraph"; paragraph_index: number }
  | { kind: "svg_text_offset"; start_byte: number; end_byte: number }
  | { kind: "dxf_entity"; handle: string; layer: string }
  | { kind: "timeline_range"; track: string; t0_ms: number; t1_ms: number }
  | { kind: "pdf_region"; page: number; x: number; y: number; w: number; h: number }
  | { kind: "image_region"; x: number; y: number; w: number; h: number }
  | { kind: "dom_node"; selector: string };

export async function navigateToTarget(target: NavigationTarget): Promise<void> {
  switch (target.kind) {
    case "text_line_col":
      window.dispatchEvent(
        new CustomEvent("unibrowser-navigate-text", {
          detail: target,
        }),
      );
      break;
    case "text_byte_range":
      window.dispatchEvent(
        new CustomEvent("unibrowser-navigate-text-bytes", {
          detail: target,
        }),
      );
      break;
    case "docx_paragraph":
      window.dispatchEvent(
        new CustomEvent("unibrowser-navigate-docx", {
          detail: target,
        }),
      );
      break;
    case "svg_text_offset":
      window.dispatchEvent(
        new CustomEvent("unibrowser-navigate-svg", {
          detail: target,
        }),
      );
      break;
    case "dxf_entity":
      window.dispatchEvent(
        new CustomEvent("unibrowser-navigate-dxf", {
          detail: target,
        }),
      );
      break;
    case "timeline_range":
      window.dispatchEvent(
        new CustomEvent("unibrowser-navigate-timeline", {
          detail: target,
        }),
      );
      break;
    case "pdf_region":
      await invoke("sync_pdf_page", { page: target.page, pageCount: null }).catch(() => undefined);
      window.dispatchEvent(
        new CustomEvent("unibrowser-navigate-pdf", {
          detail: target,
        }),
      );
      break;
    case "image_region":
      window.dispatchEvent(
        new CustomEvent("unibrowser-navigate-image", {
          detail: target,
        }),
      );
      break;
    case "dom_node":
      window.dispatchEvent(
        new CustomEvent("unibrowser-navigate-dom", {
          detail: target,
        }),
      );
      break;
    default:
      break;
  }
}
