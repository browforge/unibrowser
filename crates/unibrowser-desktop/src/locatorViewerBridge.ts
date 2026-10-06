/** Handlers registered by the active document viewer (Problems → Locator). */

export type ViewerLocatorBridge = {
  docxParagraph?: (paragraphIndex: number) => void;
  svgTextOffset?: (startByte: number, endByte: number) => void;
  dxfEntity?: (handle: string, layer: string) => void;
  timelineRange?: (track: string, t0Ms: number, t1Ms: number) => void;
  pdfRegion?: (page: number, x: number, y: number, w: number, h: number) => void;
  imageRegion?: (x: number, y: number, w: number, h: number) => void;
};

let bridge: ViewerLocatorBridge = {};

export function setViewerLocatorBridge(partial: ViewerLocatorBridge) {
  bridge = { ...bridge, ...partial };
}

export function clearViewerLocatorBridge() {
  bridge = {};
}

export function getViewerLocatorBridge(): ViewerLocatorBridge {
  return bridge;
}
