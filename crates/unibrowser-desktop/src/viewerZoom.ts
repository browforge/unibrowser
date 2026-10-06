type ZoomListener = (scale: number, label: string) => void;
type PdfPageListener = (page: number, total: number, annotationCount: number) => void;

let listener: ZoomListener | null = null;
let pdfPageListener: PdfPageListener | null = null;

export function setViewerZoomListener(fn: ZoomListener | null) {
  listener = fn;
}

export function setViewerPdfPageListener(fn: PdfPageListener | null) {
  pdfPageListener = fn;
}

export function reportViewerZoom(scale: number, kind: "pdf" | "image" | "svg") {
  const pct = Math.round(scale * 100);
  const label = kind === "pdf" ? `${pct}%` : `Image ${pct}%`;
  listener?.(scale, label);
}

export function reportViewerPdfPage(page: number, total: number, annotationCount: number) {
  pdfPageListener?.(page, total, annotationCount);
}
