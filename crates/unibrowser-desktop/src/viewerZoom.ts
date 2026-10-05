type ZoomListener = (scale: number, label: string) => void;

let listener: ZoomListener | null = null;

export function setViewerZoomListener(fn: ZoomListener | null) {
  listener = fn;
}

export function reportViewerZoom(scale: number, kind: "pdf" | "image" | "svg") {
  const pct = Math.round(scale * 100);
  const label = kind === "pdf" ? `PDF ${pct}%` : `Image ${pct}%`;
  listener?.(scale, label);
}
