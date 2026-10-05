export interface StatusBarModel {
  path: string;
  pathHint?: string;
  context?: string;
  dirty?: string;
  encoding: string;
  cursor: string;
  font?: string;
  zoom: string;
  encode?: string;
  hint: string;
}

let pathEl: HTMLElement | null = null;
let contextEl: HTMLElement | null = null;
let dirtyEl: HTMLElement | null = null;
let encodingEl: HTMLElement | null = null;
let cursorEl: HTMLElement | null = null;
let fontEl: HTMLElement | null = null;
let zoomEl: HTMLElement | null = null;
let encodeEl: HTMLElement | null = null;
let hintEl: HTMLElement | null = null;
let zoomOutBtn: HTMLButtonElement | null = null;
let zoomInBtn: HTMLButtonElement | null = null;
let zoomMode: "hidden" | "viewer" | "web" = "hidden";
let onZoomDelta: ((delta: number) => void) | null = null;
let onCopyPath: ((path: string) => void) | null = null;
let lastPath = "";

export function bindStatusBar(root: HTMLElement) {
  pathEl = root.querySelector("#status-path");
  contextEl = root.querySelector("#status-context");
  dirtyEl = root.querySelector("#status-dirty");
  encodingEl = root.querySelector("#status-encoding");
  cursorEl = root.querySelector("#status-cursor");
  fontEl = root.querySelector("#status-font");
  zoomEl = root.querySelector("#status-zoom-label");
  encodeEl = root.querySelector("#status-encode");
  hintEl = root.querySelector("#status-hint");
  zoomOutBtn = root.querySelector("#status-zoom-out");
  zoomInBtn = root.querySelector("#status-zoom-in");
  zoomOutBtn?.addEventListener("click", () => onZoomDelta?.(-1));
  zoomInBtn?.addEventListener("click", () => onZoomDelta?.(1));
  pathEl?.addEventListener("click", () => {
    if (lastPath) {
      onCopyPath?.(lastPath);
    }
  });
  pathEl?.classList.add("status-path-copy");
}

export function setPathCopyHandler(handler: ((path: string) => void) | null) {
  onCopyPath = handler;
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

export function setStatusHint(hint: string) {
  if (hintEl) {
    hintEl.textContent = hint;
  }
}

export function patchStatusCursor(cursor: string) {
  if (cursorEl) {
    cursorEl.textContent = cursor;
    cursorEl.classList.toggle("hidden", !cursor);
  }
}

function setOptional(el: HTMLElement | null, value?: string) {
  if (!el) {
    return;
  }
  const text = value ?? "";
  el.textContent = text;
  el.classList.toggle("hidden", !text);
}

export function updateStatusBar(model: StatusBarModel) {
  lastPath = model.path;
  if (pathEl) {
    pathEl.textContent = model.path;
    pathEl.title = model.pathHint ?? model.path;
  }
  setOptional(contextEl, model.context);
  setOptional(dirtyEl, model.dirty);
  setOptional(encodingEl, model.encoding);
  setOptional(cursorEl, model.cursor);
  setOptional(fontEl, model.font);
  if (zoomEl && zoomMode !== "hidden") {
    setOptional(zoomEl, model.zoom);
  }
  setOptional(encodeEl, model.encode);
  if (hintEl) {
    hintEl.textContent = model.hint;
  }
}
