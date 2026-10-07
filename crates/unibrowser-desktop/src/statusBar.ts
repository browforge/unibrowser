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
  reading?: string;
  autosave?: string;
  saved?: string;
  language?: string;
  problemsCount?: string;
  aiState?: string;
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
let readingEl: HTMLElement | null = null;
let autosaveEl: HTMLElement | null = null;
let hintEl: HTMLElement | null = null;
let savedEl: HTMLElement | null = null;
let languageEl: HTMLElement | null = null;
let problemsCountEl: HTMLElement | null = null;
let zoomOutBtn: HTMLButtonElement | null = null;
let zoomInBtn: HTMLButtonElement | null = null;
let zoomMode: "hidden" | "readout" | "viewer" | "web" = "hidden";
let onZoomDelta: ((delta: number) => void) | null = null;
let onZoomReset: (() => void) | null = null;
let onFontReset: (() => void) | null = null;
let onCopyPath: ((path: string) => void) | null = null;
let lastPath = "";
let lastFont = "";

export function bindStatusBar(root: HTMLElement) {
  pathEl = root.querySelector("#status-path");
  contextEl = root.querySelector("#status-context");
  dirtyEl = root.querySelector("#status-dirty");
  encodingEl = root.querySelector("#status-encoding");
  cursorEl = root.querySelector("#status-cursor");
  fontEl = root.querySelector("#status-font");
  zoomEl = root.querySelector("#status-zoom-label");
  encodeEl = root.querySelector("#status-encode");
  readingEl = root.querySelector("#status-reading");
  autosaveEl = root.querySelector("#status-autosave");
  hintEl = root.querySelector("#status-hint");
  savedEl = root.querySelector("#status-saved");
  languageEl = root.querySelector("#status-language");
  problemsCountEl = root.querySelector("#status-problems-count");
  encodingEl?.setAttribute("role", "status");
  cursorEl?.setAttribute("role", "status");
  encodeEl?.setAttribute("aria-live", "polite");
  readingEl?.setAttribute("role", "status");
  zoomOutBtn = root.querySelector("#status-zoom-out");
  zoomInBtn = root.querySelector("#status-zoom-in");
  zoomOutBtn?.addEventListener("click", () => onZoomDelta?.(-1));
  zoomInBtn?.addEventListener("click", () => onZoomDelta?.(1));
  zoomEl?.addEventListener("click", () => onZoomReset?.());
  fontEl?.addEventListener("click", () => {
    if (lastFont) {
      onFontReset?.();
    }
  });
  zoomEl?.classList.add("status-reset-target");
  fontEl?.classList.add("status-reset-target");
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
  mode: "hidden" | "readout" | "viewer" | "web",
  handler: ((delta: number) => void) | null,
  resetHandler?: (() => void) | null,
) {
  zoomMode = mode;
  onZoomDelta = handler;
  onZoomReset = resetHandler ?? null;
  const showControls = mode !== "hidden" && mode !== "readout" && handler !== null;
  zoomOutBtn?.classList.toggle("hidden", !showControls);
  zoomInBtn?.classList.toggle("hidden", !showControls);
  zoomEl?.classList.toggle("status-reset-target", showControls && onZoomReset !== null);
  if (zoomEl && mode === "hidden") {
    zoomEl.textContent = "";
  }
}

export function setFontResetHandler(handler: (() => void) | null) {
  onFontReset = handler;
  fontEl?.classList.toggle("status-reset-target", handler !== null);
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

export function patchStatusReading(reading: string) {
  if (readingEl) {
    readingEl.textContent = reading;
    readingEl.classList.toggle("hidden", !reading);
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
    const hint = model.pathHint ?? model.path;
    pathEl.title = hint;
    if (hint) {
      pathEl.setAttribute("aria-label", hint);
    } else {
      pathEl.removeAttribute("aria-label");
    }
  }
  setOptional(contextEl, model.context);
  setOptional(dirtyEl, model.dirty);
  setOptional(encodingEl, model.encoding);
  setOptional(cursorEl, model.cursor);
  lastFont = model.font ?? "";
  setOptional(fontEl, model.font);
  if (fontEl) {
    fontEl.classList.toggle("status-reset-target", !!lastFont && onFontReset !== null);
  }
  if (zoomEl && zoomMode !== "hidden") {
    setOptional(zoomEl, model.zoom);
  }
  setOptional(encodeEl, model.encode);
  if (encodeEl) {
    encodeEl.setAttribute("aria-busy", model.encode ? "true" : "false");
  }
  setOptional(readingEl, model.reading);
  setOptional(autosaveEl, model.autosave);
  setOptional(savedEl, model.saved);
  setOptional(languageEl, model.language);
  if (model.problemsCount && problemsCountEl) {
    problemsCountEl.textContent = model.problemsCount;
    problemsCountEl.classList.remove("hidden");
  }
  if (model.aiState) {
    const aiEl = document.getElementById("status-ai-visibility");
    if (aiEl) {
      aiEl.textContent = model.aiState;
      aiEl.classList.remove("hidden");
    }
  }
  if (hintEl) {
    hintEl.textContent = model.hint;
  }
}
