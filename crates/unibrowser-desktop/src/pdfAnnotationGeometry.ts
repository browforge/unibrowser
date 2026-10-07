export interface NormalisedPdfRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface CssPdfRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

/** Map sidecar 0–1 page coordinates to CSS pixels, independent of canvas HiDPI backing size. */
export function normalisedPdfRectToCss(
  rect: NormalisedPdfRect,
  cssWidth: number,
  cssHeight: number,
): CssPdfRect {
  return {
    left: rect.x * cssWidth,
    top: rect.y * cssHeight,
    width: rect.w * cssWidth,
    height: rect.h * cssHeight,
  };
}
