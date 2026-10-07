type ZoomListener = (scale: number, label: string) => void;
type PdfPageListener = (page: number, total: number, annotationCount: number) => void;

let listener: ZoomListener | null = null;
let pdfPageListener: PdfPageListener | null = null;

let pdfCmPage: { current: number; total: number } | null = null;
let pdfCmZoomPct: number | null = null;

export function setViewerZoomListener(fn: ZoomListener | null) {
  listener = fn;
}

export function setViewerPdfPageListener(fn: PdfPageListener | null) {
  pdfPageListener = fn;
}

export function getPdfCommandManagerLabels(): {
  page: { current: number; total: number } | null;
  zoomPct: number | null;
} {
  return { page: pdfCmPage, zoomPct: pdfCmZoomPct };
}

export function reportViewerZoom(scale: number, kind: "pdf" | "image" | "svg") {
  const pct = Math.round(scale * 100);
  if (kind === "pdf") {
    pdfCmZoomPct = pct;
  }
  const label = kind === "pdf" ? `${pct}%` : `Image ${pct}%`;
  listener?.(scale, label);
}

export function reportViewerPdfPage(page: number, total: number, annotationCount: number) {
  pdfCmPage = total > 0 ? { current: page, total } : null;
  pdfPageListener?.(page, total, annotationCount);
}
