import { invoke } from "@tauri-apps/api/core";
import { getViewerLocatorBridge } from "./locatorViewerBridge";
import { navigateToTextByteRange, navigateToTextLineCol } from "./textEditor";

export function initLocatorNavigationListeners() {
  window.addEventListener("unibrowser-navigate-text", (ev) => {
    const d = (ev as CustomEvent).detail as {
      line: number;
      col: number;
      end_line: number;
      end_col: number;
    };
    navigateToTextLineCol(d.line, d.col, d.end_line, d.end_col);
  });
  window.addEventListener("unibrowser-navigate-text-bytes", (ev) => {
    const d = (ev as CustomEvent).detail as { start_byte: number; end_byte: number };
    navigateToTextByteRange(d.start_byte, d.end_byte);
  });
  window.addEventListener("unibrowser-navigate-docx", (ev) => {
    const d = (ev as CustomEvent).detail as { paragraph_index: number };
    getViewerLocatorBridge().docxParagraph?.(d.paragraph_index);
  });
  window.addEventListener("unibrowser-navigate-svg", (ev) => {
    const d = (ev as CustomEvent).detail as { start_byte: number; end_byte: number };
    getViewerLocatorBridge().svgTextOffset?.(d.start_byte, d.end_byte);
  });
  window.addEventListener("unibrowser-navigate-dxf", (ev) => {
    const d = (ev as CustomEvent).detail as { handle: string; layer: string };
    getViewerLocatorBridge().dxfEntity?.(d.handle, d.layer);
  });
  window.addEventListener("unibrowser-navigate-timeline", (ev) => {
    const d = (ev as CustomEvent).detail as {
      track: string;
      t0_ms: number;
      t1_ms: number;
    };
    getViewerLocatorBridge().timelineRange?.(d.track, d.t0_ms, d.t1_ms);
  });
  window.addEventListener("unibrowser-navigate-pdf", (ev) => {
    const d = (ev as CustomEvent).detail as {
      page: number;
      x: number;
      y: number;
      w: number;
      h: number;
    };
    getViewerLocatorBridge().pdfRegion?.(d.page, d.x, d.y, d.w, d.h);
  });
  window.addEventListener("unibrowser-navigate-image", (ev) => {
    const d = (ev as CustomEvent).detail as { x: number; y: number; w: number; h: number };
    getViewerLocatorBridge().imageRegion?.(d.x, d.y, d.w, d.h);
  });
  window.addEventListener("unibrowser-navigate-dom", (ev) => {
    const d = (ev as CustomEvent).detail as { selector: string };
    void invoke("focus_content_dom_selector", { selector: d.selector }).catch(() => undefined);
  });
}
