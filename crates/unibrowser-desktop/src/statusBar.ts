export interface StatusBarModel {
  path: string;
  encoding: string;
  cursor: string;
  zoom: string;
  hint: string;
}

let pathEl: HTMLElement | null = null;
let encodingEl: HTMLElement | null = null;
let cursorEl: HTMLElement | null = null;
let zoomEl: HTMLElement | null = null;
let hintEl: HTMLElement | null = null;
let zoomOutBtn: HTMLButtonElement | null = null;
let zoomInBtn: HTMLButtonElement | null = null;
let zoomMode: "hidden" | "viewer" | "web" = "hidden";
let onZoomDelta: ((delta: number) => void) | null = null;

export function bindStatusBar(root: HTMLElement) {
  pathEl = root.querySelector("#status-path");
  encodingEl = root.querySelector("#status-encoding");
  cursorEl = root.querySelector("#status-cursor");
  zoomEl = root.querySelector("#status-zoom-label");
  hintEl = root.querySelector("#status-hint");
  zoomOutBtn = root.querySelector("#status-zoom-out");
  zoomInBtn = root.querySelector("#status-zoom-in");
  zoomOutBtn?.addEventListener("click", () => onZoomDelta?.(-1));
  zoomInBtn?.addEventListener("click", () => onZoomDelta?.(1));
}

export function setZoomHandler(
  mode: "hidden" | "viewer" | "web",
  handler: ((delta: number) => void) | null,
) {
  zoomMode = mode;
  onZoomDelta = handler;
  const show = mode !== "hidden" && handler !== null;
  zoomOutBtn?.classList.toggle("hidden", !show);
  zoomInBtn?.classList.toggle("hidden", !show);
  if (zoomEl && !show) {
    zoomEl.textContent = "";
  }
}

export function updateStatusBar(model: StatusBarModel) {
  if (pathEl) {
    pathEl.textContent = model.path;
    pathEl.title = model.path;
  }
  if (encodingEl) {
    encodingEl.textContent = model.encoding;
    encodingEl.classList.toggle("hidden", !model.encoding);
  }
  if (cursorEl) {
    cursorEl.textContent = model.cursor;
    cursorEl.classList.toggle("hidden", !model.cursor);
  }
  if (zoomEl && zoomMode !== "hidden") {
    zoomEl.textContent = model.zoom;
    zoomEl.classList.toggle("hidden", !model.zoom);
  }
  if (hintEl) {
    hintEl.textContent = model.hint;
  }
}
