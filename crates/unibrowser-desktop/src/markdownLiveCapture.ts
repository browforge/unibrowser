import { splitLiveBlocks, activeLiveBlockIndex } from "./markdownLiveMapping";

export interface LiveCapturePixelSample {
  x: number;
  y: number;
  red: number;
  green: number;
  blue: number;
}

export interface LiveBlockCaptureMetrics {
  mode: string;
  activeBlockIndex: number;
  markerElementCount: number;
  hiddenLineDecorationCount: number;
  renderZoneCount: number;
  markerSamples: LiveCapturePixelSample[];
  inactiveBoldMarkerVisible: boolean;
}

function samplePixelAt(x: number, y: number): LiveCapturePixelSample | null {
  const canvas = document.querySelector<HTMLCanvasElement>(
    "#text-editor-host .monaco-editor canvas",
  );
  if (!canvas) {
    return null;
  }
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    return null;
  }
  const rect = canvas.getBoundingClientRect();
  const scaleX = canvas.width / rect.width;
  const scaleY = canvas.height / rect.height;
  const px = Math.floor((x - rect.left) * scaleX);
  const py = Math.floor((y - rect.top) * scaleY);
  if (px < 0 || py < 0 || px >= canvas.width || py >= canvas.height) {
    return null;
  }
  const data = ctx.getImageData(px, py, 1, 1).data;
  return { x: px, y: py, red: data[0]!, green: data[1]!, blue: data[2]! };
}

export function collectLiveBlockCaptureMetrics(demo: string): LiveBlockCaptureMetrics | null {
  if (!demo.includes("live-caret-") && !demo.includes("doc-mode-live")) {
    return null;
  }
  const host = document.getElementById("text-editor-host");
  const modelText =
    (window as unknown as { __ubTextEditor?: { getActiveTextContent?: () => string | null } })
      .__ubTextEditor?.getActiveTextContent?.() ?? "";
  const blocks = splitLiveBlocks(modelText);
  const markerEls = host?.querySelectorAll(".view-line span.md-live-marker") ?? [];
  const samples: LiveCapturePixelSample[] = [];
  for (const el of markerEls) {
    const rect = el.getBoundingClientRect();
    const sample = samplePixelAt(rect.left + 2, rect.top + rect.height / 2);
    if (sample) {
      samples.push(sample);
    }
  }
  let inactiveBoldMarkerVisible = false;
  if (demo.includes("live-caret-outside")) {
    inactiveBoldMarkerVisible = Array.from(host?.querySelectorAll(".view-line") ?? []).some(
      (line) =>
        !line.classList.contains("md-live-line-hidden") &&
        (line.textContent?.includes("**") || line.textContent?.includes("##")),
    );
  }
  const cursorOffset = Number(host?.dataset.liveCaptureCursorOffset ?? "0");
  return {
    mode: document.body.dataset.documentEditorMode ?? "",
    activeBlockIndex: activeLiveBlockIndex(blocks, cursorOffset),
    markerElementCount: markerEls.length,
    hiddenLineDecorationCount: host?.querySelectorAll(".md-live-line-hidden").length ?? 0,
    renderZoneCount: host?.querySelectorAll(".md-live-render-zone").length ?? 0,
    markerSamples: samples,
    inactiveBoldMarkerVisible,
  };
}

export function storeLiveCaptureCursorOffset(offset: number): void {
  document.getElementById("text-editor-host")?.setAttribute(
    "data-live-capture-cursor-offset",
    String(offset),
  );
}
