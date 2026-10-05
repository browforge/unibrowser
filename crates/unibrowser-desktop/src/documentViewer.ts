import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { loadPdfJs, loadThree } from "./lazyModules";
import { appendTrustBanner } from "./trustBanner";
import { mountClickToPlay } from "./mediaPlayGate";
import { renderDownloadsManager, type DownloadHistoryEntryDto } from "./downloadsPage";
import { reportViewerZoom } from "./viewerZoom";
import { bindPreviewLinks } from "./previewLinks";
import { copyImageFile, copyText, readClipboardImagePath } from "./clipboard";
import { renderSettings } from "./settingsPage";
import { renderAbout } from "./aboutPage";
import { renderDocs } from "./docsPage";
import { renderKeys } from "./keysPage";
import { renderDiagnostics } from "./diagnosticsPage";
import { emptyStateBlock } from "./emptyState";
import { pathsFromDataTransfer } from "./filesDrag";
import { showErrorToast, showToast } from "./toast";
import { playMidiScrubBeeps } from "./midiScrubAudio";
import { patchStatusCursor, setStatusHint } from "./statusBar";
import {
  clearImagePixelSample,
  setImagePixelSample,
} from "./viewerPixelSample";
import { t } from "./i18n";
import {
  downloadPrintFriendlyHtml,
  openPrintFriendlyDocument,
  openPrintPreviewDocument,
} from "./printExport";
import {
  applySpellcheckToEditable,
  isExperimentalSpellcheckEnabled,
} from "./experimentalSpellcheck";

let viewerZoomAdjust: ((delta: number) => void) | null = null;

type FindRunner = (query: string, next: boolean) => void;
let viewerFindRunner: FindRunner | null = null;

export function setViewerFindRunner(run: FindRunner | null) {
  viewerFindRunner = run;
}

export function runViewerFind(query: string, next: boolean) {
  viewerFindRunner?.(query, next);
}

const BOARD_GRID = 24;

function snapBoardCoord(value: number, snap: boolean): number {
  const v = Math.max(0, value);
  if (!snap) {
    return v;
  }
  return Math.round(v / BOARD_GRID) * BOARD_GRID;
}

async function openUnibrowAsset(projectPath: string, asset: string) {
  const assetPath = await invoke<string>("allow_unibrow_asset", {
    projectPath,
    assetName: asset,
  });
  const data = await invoke("open_path", { path: assetPath });
  window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
}

export function adjustViewerZoom(delta: number) {
  viewerZoomAdjust?.(delta);
}

export interface ProjectNodeDto {
  id: string;
  node_type: string;
  label: string;
  asset: string | null;
  x: number;
  y: number;
  color?: string | null;
}

export interface OpenFileTabDto {
  tab_index: number;
  path: string;
  label: string;
}

export interface DocumentViewerDto {
  path: string;
  viewer: string;
  title: string;
  pdf_scale?: number;
  pdf_page?: number;
  media_note?: string;
  project_nodes?: ProjectNodeDto[];
  open_file_tabs?: OpenFileTabDto[];
  project_clips?: ProjectClipDto[];
  project_transitions?: ProjectTransitionDto[];
  project_markers?: ProjectMarkerDto[];
  midi_summary?: MidiSummaryDto;
  board_snap_to_grid?: boolean;
  midi_scrub_audio_enabled?: boolean;
  midi_note_preview?: MidiNoteOnDto[];
  pdf_engines?: PdfEngineLineDto[];
  reader_font_pt?: number;
  preview_zoom?: number;
}

interface PdfEngineLineDto {
  engine: string;
  status: string;
  detail: string;
}

export interface MidiNoteOnDto {
  track_index: number;
  tick: number;
  note: number;
  velocity: number;
}

export interface ClipMarkerDto {
  id: string;
  offset_sec: number;
  label: string;
}

export interface ProjectClipDto {
  id: string;
  node_id: string;
  node_type: string;
  start_sec: number;
  duration_sec: number;
  label: string;
  color?: string | null;
  clip_markers?: ClipMarkerDto[];
}

interface ProjectMarkerDto {
  id: string;
  time_sec: number;
  title: string;
  color?: string | null;
}

interface ProjectTransitionDto {
  id: string;
  after_clip_id: string;
  kind: string;
  duration_sec: number;
}

function clipIsVideo(nodeType: string): boolean {
  return nodeType.toLowerCase().includes("video");
}

function clipTypeAbbrev(nodeType: string): string {
  const t = nodeType.toLowerCase();
  if (t.includes("video")) {
    return "VID";
  }
  if (t.includes("midi")) {
    return "MID";
  }
  if (t.includes("audio")) {
    return "AUD";
  }
  if (t.includes("image") || t.includes("png") || t.includes("svg")) {
    return "IMG";
  }
  if (t.includes("text") || t.includes("note")) {
    return "TXT";
  }
  if (t.includes("pdf")) {
    return "PDF";
  }
  return "AST";
}

const LABEL_COLORS = ["", "blue", "green", "amber", "rose", "violet", "slate"];

function applyLabelColor(el: HTMLElement, color?: string | null) {
  if (color) {
    el.dataset.labelColor = color;
  } else {
    delete el.dataset.labelColor;
  }
}

function makeLabelColorSelect(
  value: string | null | undefined,
  onChange: (color: string | null) => void,
): HTMLSelectElement {
  const sel = document.createElement("select");
  sel.className = "unibrow-label-color";
  sel.title = "Label colour";
  LABEL_COLORS.forEach((c) => {
    const opt = document.createElement("option");
    opt.value = c;
    opt.textContent = c ? c : "none";
    sel.appendChild(opt);
  });
  sel.value = value ?? "";
  sel.onchange = () => {
    onChange(sel.value ? sel.value : null);
  };
  return sel;
}

export interface MidiSummaryDto {
  format: number;
  track_count: number;
  ticks_per_quarter: number;
}

const loadedKeys = new WeakMap<HTMLElement, string | null>();

async function readBytes(path: string): Promise<Uint8Array> {
  const data = await invoke<number[]>("read_document_bytes", { path });
  return Uint8Array.from(data);
}

async function rasterPdfFirstPageDataUrl(assetPath: string): Promise<string> {
  const pdfjsLib = await loadPdfJs();
  const bytes = await readBytes(assetPath);
  const doc = await pdfjsLib.getDocument({ data: toArrayBuffer(bytes) }).promise;
  const page = await doc.getPage(1);
  const viewport = page.getViewport({ scale: 0.75 });
  const canvas = document.createElement("canvas");
  canvas.width = viewport.width;
  canvas.height = viewport.height;
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("canvas unavailable");
  }
  await page.render({ canvasContext: ctx, viewport, canvas }).promise;
  return canvas.toDataURL("image/jpeg", 0.85);
}

type TimelineScrubPreview = {
  kind: string;
  image_path?: string;
  image_mime?: string;
  midi_notes?: MidiNoteOnDto[];
};

function findClipAtTime(clips: ProjectClipDto[], timeSec: number): ProjectClipDto | null {
  for (const clip of clips) {
    if (timeSec >= clip.start_sec && timeSec < clip.start_sec + clip.duration_sec) {
      return clip;
    }
  }
  return clips[0] ?? null;
}

function drawAudioPeaks(
  g: CanvasRenderingContext2D,
  channel: Float32Array,
  width: number,
  height: number,
) {
  g.fillStyle = "#1a1a22";
  g.fillRect(0, 0, width, height);
  const mid = height * 0.5;
  const buckets = width;
  const block = Math.max(1, Math.floor(channel.length / buckets));
  g.fillStyle = "#66ccff";
  for (let x = 0; x < buckets; x += 1) {
    let peak = 0;
    const start = x * block;
    for (let i = 0; i < block && start + i < channel.length; i += 1) {
      peak = Math.max(peak, Math.abs(channel[start + i]));
    }
    const bar = Math.max(1, peak * height * 0.9);
    g.fillRect(x, mid - bar * 0.5, 1, bar);
  }
}

function drawAudioStubBars(g: CanvasRenderingContext2D, width: number, height: number) {
  g.fillStyle = "#1a1a22";
  g.fillRect(0, 0, width, height);
  g.fillStyle = "#445566";
  const mid = height * 0.5;
  for (let x = 0; x < width; x += 3) {
    const h = 6 + ((x * 17) % 21);
    g.fillRect(x, mid - h * 0.5, 2, h);
  }
}

function toArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
}

function clearDocxSearchMarks(article: HTMLElement) {
  article.querySelectorAll("mark.docx-search-hit").forEach((mark) => {
    const parent = mark.parentNode;
    if (!parent) {
      return;
    }
    parent.replaceChild(document.createTextNode(mark.textContent ?? ""), mark);
    parent.normalize();
  });
}

function findDocxTextRanges(
  article: HTMLElement,
  query: string,
): { node: Text; start: number; end: number }[] {
  if (!query.trim()) {
    return [];
  }
  const q = query.toLowerCase();
  const out: { node: Text; start: number; end: number }[] = [];
  const walker = document.createTreeWalker(article, NodeFilter.SHOW_TEXT);
  let node = walker.nextNode() as Text | null;
  while (node) {
    const text = node.data;
    const lower = text.toLowerCase();
    let idx = 0;
    while (idx < lower.length) {
      const found = lower.indexOf(q, idx);
      if (found < 0) {
        break;
      }
      out.push({ node, start: found, end: found + q.length });
      idx = found + q.length;
    }
    node = walker.nextNode() as Text | null;
  }
  return out;
}

function applyDocxSearchHighlights(article: HTMLElement, query: string): HTMLElement[] {
  clearDocxSearchMarks(article);
  const ranges = findDocxTextRanges(article, query);
  const marks: HTMLElement[] = [];
  for (const r of [...ranges].reverse()) {
    const range = document.createRange();
    range.setStart(r.node, r.start);
    range.setEnd(r.node, r.end);
    const mark = document.createElement("mark");
    mark.className = "docx-search-hit";
    try {
      range.surroundContents(mark);
      marks.push(mark);
    } catch {
      // Range crossed element boundaries; skip this match.
    }
  }
  return marks;
}

async function renderDocx(path: string, root: HTMLElement, readerFontPt = 12) {
  const mammoth = await import("mammoth");
  const bytes = await readBytes(path);
  const result = await mammoth.convertToHtml(
    { arrayBuffer: toArrayBuffer(bytes) },
    {
      includeDefaultStyleMap: true,
      ignoreEmptyParagraphs: false,
    },
  );
  const banner = document.createElement("p");
  banner.className = "viewer-note docx-limit";
  banner.textContent =
    "View-only preview (Mammoth HTML). Word layout, fields, and tracked changes are not editable here; export or print HTML below.";

  const toolbar = document.createElement("div");
  toolbar.className = "docx-toolbar";
  const searchInput = document.createElement("input");
  searchInput.type = "search";
  searchInput.className = "docx-search-input";
  searchInput.placeholder = "Search in document";
  const searchStatus = document.createElement("span");
  searchStatus.className = "docx-search-status";
  let hits: HTMLElement[] = [];
  let hitIndex = -1;
  const focusHit = (index: number) => {
    if (!hits.length) {
      hitIndex = -1;
      searchStatus.textContent = searchInput.value.trim() ? "0 matches" : "";
      return;
    }
    hitIndex = ((index % hits.length) + hits.length) % hits.length;
    hits.forEach((h, i) => {
      h.classList.toggle("docx-search-hit--current", i === hitIndex);
    });
    hits[hitIndex].scrollIntoView({ block: "nearest", behavior: "smooth" });
    searchStatus.textContent = `${hitIndex + 1} / ${hits.length}`;
  };
  const runSearch = () => {
    hits = applyDocxSearchHighlights(article, searchInput.value.trim());
    focusHit(0);
  };
  setViewerFindRunner((query, next) => {
    searchInput.value = query;
    if (!query.trim()) {
      clearDocxSearchMarks(article);
      hits = [];
      hitIndex = -1;
      searchStatus.textContent = "";
      return;
    }
    if (!hits.length || searchInput.value.trim() !== query.trim()) {
      runSearch();
      return;
    }
    focusHit(hitIndex + (next ? 1 : -1));
  });
  searchInput.onkeydown = (ev) => {
    if (ev.key === "Enter") {
      ev.preventDefault();
      if (ev.shiftKey) {
        focusHit(hitIndex - 1);
      } else if (hits.length) {
        focusHit(hitIndex + 1);
      } else {
        runSearch();
      }
    }
  };
  const findBtn = document.createElement("button");
  findBtn.type = "button";
  findBtn.textContent = "Find";
  findBtn.onclick = () => runSearch();
  const prevBtn = document.createElement("button");
  prevBtn.type = "button";
  prevBtn.textContent = "Prev";
  prevBtn.onclick = () => focusHit(hitIndex - 1);
  const nextBtn = document.createElement("button");
  nextBtn.type = "button";
  nextBtn.textContent = "Next";
  nextBtn.onclick = () => focusHit(hitIndex + 1);
  toolbar.appendChild(searchInput);
  toolbar.appendChild(findBtn);
  toolbar.appendChild(prevBtn);
  toolbar.appendChild(nextBtn);
  toolbar.appendChild(searchStatus);
  const printBtn = document.createElement("button");
  printBtn.type = "button";
  printBtn.textContent = t("print.preview");
  printBtn.onclick = () => {
    openPrintPreviewDocument(path, result.value);
  };
  const exportBtn = document.createElement("button");
  exportBtn.type = "button";
  exportBtn.textContent = "Export print HTML";
  exportBtn.onclick = () => {
    const name = path.split(/[/\\]/).pop() ?? "document";
    downloadPrintFriendlyHtml(path, result.value, name);
  };
  toolbar.appendChild(printBtn);
  toolbar.appendChild(exportBtn);

  const article = document.createElement("article");
  article.className = "docx-view";
  article.style.fontSize = `${readerFontPt}pt`;
  article.innerHTML = result.value;
  applyDocxSpellcheckExperiment(article);
  appendTrustBanner(root, "remote-html");

  root.innerHTML = "";
  root.appendChild(banner);
  root.appendChild(toolbar);
  root.appendChild(article);
  bindPreviewLinks(article, path);
  if (result.messages.length) {
    const note = document.createElement("p");
    note.className = "viewer-note";
    note.textContent = `mammoth: ${result.messages.length} conversion note(s)`;
    root.appendChild(note);
  }
}

async function syncPdfSelection(selected: boolean) {
  await invoke("sync_pdf_viewer_state", { hasTextSelection: selected }).catch(() => {});
}

type PdfHighlightDto = {
  page: number;
  x: number;
  y: number;
  w: number;
  h: number;
  color: string;
};

let pdfHighlightArmed = false;

export function armPdfHighlightMode() {
  pdfHighlightArmed = true;
}

async function renderPdf(
  path: string,
  root: HTMLElement,
  scale: number,
  startPage: number,
  engines?: PdfEngineLineDto[],
) {
  void invoke<string | null>("ensure_pdf_raster_proxy", { path }).catch(() => {});
  const pdfjsLib = await loadPdfJs();
  const bytes = await readBytes(path);
  const doc = await pdfjsLib.getDocument({ data: bytes }).promise;
  let currentPage = Math.min(Math.max(startPage, 1), doc.numPages);
  let currentScale = scale;
  root.innerHTML = "";
  if (engines?.length) {
    const banner = document.createElement("p");
    banner.className = "viewer-note pdf-engine-banner";
    banner.textContent = engines
      .map((e) => `${e.engine} (${e.status}): ${e.detail}`)
      .join(" · ");
    root.appendChild(banner);
  }
  let highlights: PdfHighlightDto[] = [];
  try {
    highlights = await invoke<PdfHighlightDto[]>("list_pdf_highlights", { path });
  } catch {
    highlights = [];
  }

  const wrap = document.createElement("div");
  wrap.className = "pdf-page-wrap";
  root.appendChild(wrap);

  const paintHighlights = (pageNum: number, host: HTMLElement, width: number, height: number) => {
    host.querySelectorAll(".pdf-highlight-rect").forEach((el) => el.remove());
    for (const h of highlights) {
      if (h.page !== pageNum) {
        continue;
      }
      const box = document.createElement("div");
      box.className = "pdf-highlight-rect";
      box.style.left = `${h.x * width}px`;
      box.style.top = `${h.y * height}px`;
      box.style.width = `${h.w * width}px`;
      box.style.height = `${h.h * height}px`;
      box.style.background = h.color;
      host.appendChild(box);
    }
  };
  const nav = document.createElement("div");
  nav.className = "pdf-nav";
  const prev = document.createElement("button");
  prev.type = "button";
  prev.textContent = "Previous";
  const next = document.createElement("button");
  next.type = "button";
  next.textContent = "Next";
  const pageLabel = document.createElement("span");
  pageLabel.className = "pdf-page-label";
  const jumpInput = document.createElement("input");
  jumpInput.type = "number";
  jumpInput.className = "pdf-jump-input";
  jumpInput.min = "1";
  jumpInput.max = String(doc.numPages);
  jumpInput.value = String(currentPage);
  const jumpBtn = document.createElement("button");
  jumpBtn.type = "button";
  jumpBtn.textContent = "Go";
  nav.appendChild(prev);
  nav.appendChild(pageLabel);
  nav.appendChild(next);
  nav.appendChild(jumpInput);
  nav.appendChild(jumpBtn);
  const zoomOut = document.createElement("button");
  zoomOut.type = "button";
  zoomOut.textContent = "Zoom −";
  const zoomIn = document.createElement("button");
  zoomIn.type = "button";
  zoomIn.textContent = "Zoom +";
  nav.appendChild(zoomOut);
  nav.appendChild(zoomIn);
  const copyBtn = document.createElement("button");
  copyBtn.type = "button";
  copyBtn.textContent = "Copy";
  copyBtn.onclick = () => void copyViewerSelection();
  const printBtn = document.createElement("button");
  printBtn.type = "button";
  printBtn.textContent = "Print";
  printBtn.onclick = async () => {
    const page = await doc.getPage(currentPage);
    const viewport = page.getViewport({ scale: currentScale });
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    await page.render({ canvasContext: ctx, viewport, canvas }).promise;
    const img = canvas.toDataURL("image/png");
    printHtmlDocument(path, `<img src="${img}" alt="PDF page ${currentPage}" />`);
  };
  const highlightBtn = document.createElement("button");
  highlightBtn.type = "button";
  highlightBtn.textContent = "Highlight";
  highlightBtn.title = "Drag on the page to save a highlight (sidecar JSON)";
  highlightBtn.onclick = () => {
    pdfHighlightArmed = true;
    highlightBtn.classList.add("pdf-highlight-armed");
  };
  if (pdfHighlightArmed) {
    highlightBtn.classList.add("pdf-highlight-armed");
  }
  nav.appendChild(highlightBtn);
  nav.appendChild(copyBtn);
  nav.appendChild(printBtn);
  root.appendChild(nav);

  const drawPage = async (pageNum: number) => {
    wrap.innerHTML = "";
    const page = await doc.getPage(pageNum);
    const viewport = page.getViewport({ scale: currentScale });
    const overlay = document.createElement("div");
    overlay.className = "pdf-page-overlay";
    overlay.style.width = `${viewport.width}px`;
    overlay.style.height = `${viewport.height}px`;
    const canvas = document.createElement("canvas");
    canvas.className = "pdf-page";
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("canvas 2d unavailable");
    }
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    wrap.appendChild(canvas);
    const textLayer = document.createElement("div");
    textLayer.className = "pdf-text-layer";
    textLayer.style.width = `${viewport.width}px`;
    textLayer.style.height = `${viewport.height}px`;
    wrap.appendChild(textLayer);
    await page.render({ canvasContext: ctx, viewport, canvas }).promise;
    wrap.appendChild(overlay);
    overlay.style.pointerEvents = pdfHighlightArmed ? "auto" : "none";
    paintHighlights(pageNum, overlay, viewport.width, viewport.height);

    let dragStart: { x: number; y: number } | null = null;
    let draft: HTMLDivElement | null = null;
    const norm = (clientX: number, clientY: number) => {
      const rect = overlay.getBoundingClientRect();
      return {
        x: (clientX - rect.left) / rect.width,
        y: (clientY - rect.top) / rect.height,
      };
    };
    overlay.onmousedown = (ev) => {
      if (!pdfHighlightArmed || ev.button !== 0) {
        return;
      }
      ev.preventDefault();
      const p = norm(ev.clientX, ev.clientY);
      dragStart = p;
      draft = document.createElement("div");
      draft.className = "pdf-highlight-rect pdf-highlight-draft";
      overlay.appendChild(draft);
    };
    overlay.onmousemove = (ev) => {
      if (!dragStart || !draft) {
        return;
      }
      const p = norm(ev.clientX, ev.clientY);
      const x = Math.min(dragStart.x, p.x);
      const y = Math.min(dragStart.y, p.y);
      const w = Math.abs(p.x - dragStart.x);
      const h = Math.abs(p.y - dragStart.y);
      draft.style.left = `${x * viewport.width}px`;
      draft.style.top = `${y * viewport.height}px`;
      draft.style.width = `${w * viewport.width}px`;
      draft.style.height = `${h * viewport.height}px`;
    };
    overlay.onmouseup = async () => {
      if (!dragStart || !draft) {
        return;
      }
      const x = Number.parseFloat(draft.style.left) / viewport.width;
      const y = Number.parseFloat(draft.style.top) / viewport.height;
      const w = Number.parseFloat(draft.style.width) / viewport.width;
      const h = Number.parseFloat(draft.style.height) / viewport.height;
      dragStart = null;
      draft = null;
      if (w < 0.01 || h < 0.01) {
        overlay.querySelectorAll(".pdf-highlight-draft").forEach((el) => el.remove());
        return;
      }
      try {
        highlights = await invoke<PdfHighlightDto[]>("add_pdf_highlight", {
          path,
          page: pageNum,
          x,
          y,
          w,
          h,
        });
        pdfHighlightArmed = false;
        highlightBtn.classList.remove("pdf-highlight-armed");
        overlay.style.pointerEvents = "none";
        paintHighlights(pageNum, overlay, viewport.width, viewport.height);
      } catch {
        overlay.querySelectorAll(".pdf-highlight-draft").forEach((el) => el.remove());
      }
    };
    const textContent = await page.getTextContent();
    const layer = new pdfjsLib.TextLayer({
      textContentSource: textContent,
      container: textLayer,
      viewport,
    });
    await layer.render();
    textLayer.addEventListener("mouseup", () => {
      const sel = window.getSelection();
      const has = Boolean(sel && !sel.isCollapsed && textLayer.contains(sel.anchorNode));
      void syncPdfSelection(has);
    });
    pageLabel.textContent = `Page ${pageNum} / ${doc.numPages}`;
    prev.disabled = pageNum <= 1;
    next.disabled = pageNum >= doc.numPages;
    jumpInput.value = String(pageNum);
    updateThumbActive();
  };

  const thumbs = document.createElement("div");
  thumbs.className = "pdf-thumbs";
  const THUMB_EAGER = 12;
  const THUMB_SCALE = 0.22;

  const updateThumbActive = () => {
    thumbs.querySelectorAll<HTMLButtonElement>(".pdf-thumb-btn").forEach((btn) => {
      const page = Number.parseInt(btn.dataset.page ?? "", 10);
      btn.classList.toggle("pdf-thumb-btn--active", page === currentPage);
    });
  };

  const renderThumbCanvas = async (pageNum: number, host: HTMLButtonElement) => {
    if (host.dataset.rendered === "1") {
      return;
    }
    const page = await doc.getPage(pageNum);
    const viewport = page.getViewport({ scale: THUMB_SCALE });
    const canvas = document.createElement("canvas");
    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.className = "pdf-thumb-canvas";
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }
    await page.render({ canvasContext: ctx, viewport, canvas }).promise;
    host.textContent = "";
    host.appendChild(canvas);
    const cap = document.createElement("span");
    cap.className = "pdf-thumb-label";
    cap.textContent = String(pageNum);
    host.appendChild(cap);
    host.dataset.rendered = "1";
  };

  const goToPage = async (n: number) => {
    currentPage = n;
    await invoke("sync_pdf_page", { page: currentPage });
    await drawPage(currentPage);
    updateThumbActive();
  };

  const addThumb = (n: number, lazy: boolean) => {
    const tb = document.createElement("button");
    tb.type = "button";
    tb.className = "pdf-thumb-btn";
    tb.dataset.page = String(n);
    tb.title = `Page ${n}`;
    tb.textContent = String(n);
    tb.onclick = () => void goToPage(n);
    thumbs.appendChild(tb);
    if (lazy) {
      const observer = new IntersectionObserver(
        (entries) => {
          if (entries.some((e) => e.isIntersecting)) {
            void renderThumbCanvas(n, tb);
            observer.disconnect();
          }
        },
        { root: thumbs, rootMargin: "48px" },
      );
      observer.observe(tb);
    } else {
      void renderThumbCanvas(n, tb);
    }
  };

  const eager = Math.min(doc.numPages, THUMB_EAGER);
  for (let n = 1; n <= eager; n += 1) {
    addThumb(n, false);
  }
  for (let n = eager + 1; n <= doc.numPages; n += 1) {
    addThumb(n, true);
  }
  root.appendChild(thumbs);

  jumpBtn.onclick = async () => {
    const n = Number(jumpInput.value);
    if (!Number.isFinite(n) || n < 1 || n > doc.numPages) {
      return;
    }
    currentPage = Math.floor(n);
    await invoke("sync_pdf_page", { page: currentPage });
    await drawPage(currentPage);
  };

  prev.onclick = async () => {
    if (currentPage <= 1) {
      return;
    }
    currentPage -= 1;
    await invoke("sync_pdf_page", { page: currentPage });
    await drawPage(currentPage);
  };
  next.onclick = async () => {
    if (currentPage >= doc.numPages) {
      return;
    }
    currentPage += 1;
    await invoke("sync_pdf_page", { page: currentPage });
    await drawPage(currentPage);
  };

  const bumpZoom = (delta: number) => {
    const factor = delta > 0 ? 1.12 : 1 / 1.12;
    currentScale = Math.min(4, Math.max(0.5, currentScale * factor));
    void invoke("sync_pdf_scale", { scale: currentScale }).catch(() => {});
    reportViewerZoom(currentScale, "pdf");
    void drawPage(currentPage);
  };
  zoomIn.onclick = () => bumpZoom(1);
  zoomOut.onclick = () => bumpZoom(-1);
  viewerZoomAdjust = bumpZoom;
  reportViewerZoom(currentScale, "pdf");

  await drawPage(currentPage);
  updateThumbActive();
  const meta = document.createElement("p");
  meta.className = "viewer-note";
  meta.textContent =
    "Select text for font-size commands · thumbnails render for the first pages (lazy for the rest)";
  root.appendChild(meta);
}

function appendMediaNote(root: HTMLElement, note?: string) {
  if (!note) {
    return;
  }
  const meta = document.createElement("p");
  meta.className = "viewer-note";
  meta.textContent = note;
  root.appendChild(meta);
}

async function ensureAssetAllowed(path: string) {
  await invoke("allow_asset_path", { path }).catch(() => {});
}

async function renderVideo(path: string, root: HTMLElement, note?: string) {
  await ensureAssetAllowed(path);
  root.innerHTML = "";
  const video = document.createElement("video");
  video.className = "media-view";
  video.controls = true;
  video.src = convertFileSrc(path);
  mountClickToPlay(root, video, "Click to play video");

  const scrubWrap = document.createElement("div");
  scrubWrap.className = "video-scrub";
  const scrubLabel = document.createElement("label");
  scrubLabel.className = "video-scrub-label";
  scrubLabel.textContent = "Scrub preview";
  const scrubInput = document.createElement("input");
  scrubInput.type = "range";
  scrubInput.className = "video-scrub-range";
  scrubInput.min = "0";
  scrubInput.max = "1000";
  scrubInput.value = "0";
  scrubInput.disabled = true;
  const scrubTime = document.createElement("span");
  scrubTime.className = "video-scrub-time";
  scrubTime.textContent = "0.0s";
  const thumb = document.createElement("img");
  thumb.className = "video-scrub-thumb hidden";
  thumb.alt = "";
  const scrubNote = document.createElement("p");
  scrubNote.className = "viewer-note video-scrub-fallback hidden";
  scrubNote.textContent = "Frame preview needs ffmpeg on PATH.";
  scrubWrap.appendChild(scrubLabel);
  scrubWrap.appendChild(scrubInput);
  scrubWrap.appendChild(scrubTime);
  scrubWrap.appendChild(thumb);
  scrubWrap.appendChild(scrubNote);
  root.appendChild(scrubWrap);

  let scrubTimer: number | undefined;
  let thumbEnabled = true;

  const syncScrubMax = () => {
    const dur = video.duration;
    if (Number.isFinite(dur) && dur > 0) {
      scrubInput.disabled = false;
      scrubInput.max = String(Math.max(1, Math.floor(dur * 10)));
    }
  };
  video.addEventListener("loadedmetadata", syncScrubMax);
  syncScrubMax();

  const requestThumb = (timeSec: number) => {
    if (!thumbEnabled) {
      return;
    }
    void invoke<{ path: string; mime: string }>("video_scrub_thumbnail", {
      path,
      timeSec,
    })
      .then(async (frame) => {
        await ensureAssetAllowed(frame.path);
        thumb.src = convertFileSrc(frame.path);
        thumb.classList.remove("hidden");
        scrubNote.classList.add("hidden");
      })
      .catch(() => {
        thumb.classList.add("hidden");
        scrubNote.classList.remove("hidden");
        thumbEnabled = false;
      });
  };

  scrubInput.addEventListener("input", () => {
    const t = Number.parseFloat(scrubInput.value) / 10;
    scrubTime.textContent = `${t.toFixed(1)}s`;
    if (Number.isFinite(t)) {
      video.currentTime = t;
    }
    window.clearTimeout(scrubTimer);
    scrubTimer = window.setTimeout(() => requestThumb(t), 120);
  });

  appendMediaNote(root, note);
}

function drawAudioPeakArray(
  g: CanvasRenderingContext2D,
  peaks: number[],
  width: number,
  height: number,
) {
  g.clearRect(0, 0, width, height);
  g.fillStyle = "#3d5a80";
  const mid = height / 2;
  const step = peaks.length / width;
  for (let x = 0; x < width; x++) {
    const i = Math.min(peaks.length - 1, Math.floor(x * step));
    const h = Math.max(2, peaks[i] * (height * 0.9));
    g.fillRect(x, mid - h / 2, 1, h);
  }
}

async function renderAudio(path: string, root: HTMLElement, note?: string) {
  await ensureAssetAllowed(path);
  root.innerHTML = "";
  const banner = document.createElement("p");
  banner.className = "viewer-note";
  banner.textContent =
    "Audio proving viewer: HTML playback plus waveform sketch. Multitrack tools stay grey until a DAW engine ships.";
  root.appendChild(banner);

  const toolbar = document.createElement("div");
  toolbar.className = "audio-proving-toolbar";
  for (const label of ["Split clip", "Export stem", "Automation"]) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.disabled = true;
    btn.className = "proving-disabled";
    btn.title = "Requires DAW / multitrack engine (see Command Manager when audio tab is active)";
    btn.textContent = label;
    toolbar.appendChild(btn);
  }
  root.appendChild(toolbar);

  const audio = document.createElement("audio");
  audio.className = "media-view";
  audio.controls = true;
  audio.src = convertFileSrc(path);
  const audioWrap = document.createElement("div");
  root.appendChild(audioWrap);
  mountClickToPlay(audioWrap, audio, "Click to play audio");
  const canvas = document.createElement("canvas");
  canvas.className = "audio-waveform";
  canvas.width = 640;
  canvas.height = 48;
  root.appendChild(canvas);
  let waveformNote = "";
  try {
    const bytes = await readBytes(path);
    const ctx = new AudioContext();
    const buf = await ctx.decodeAudioData(toArrayBuffer(bytes));
    const ch = buf.getChannelData(0);
    const g = canvas.getContext("2d");
    if (g) {
      drawAudioPeaks(g, ch, canvas.width, canvas.height);
    }
    await ctx.close();
    waveformNote = "Waveform from Web Audio decode.";
  } catch {
    try {
      const peaks = await invoke<number[]>("audio_waveform_peaks", {
        path,
        maxPoints: canvas.width,
      });
      const g = canvas.getContext("2d");
      if (g && peaks.length) {
        drawAudioPeakArray(g, peaks, canvas.width, canvas.height);
        waveformNote = "Waveform from ffmpeg PCM decode.";
      }
    } catch {
      const g = canvas.getContext("2d");
      if (g) {
        drawAudioStubBars(g, canvas.width, canvas.height);
      }
      waveformNote = "Waveform stub; install ffmpeg for decode or use a browser-supported codec.";
    }
  }
  if (waveformNote) {
    appendMediaNote(root, waveformNote);
  }
  appendMediaNote(root, note);
}

type DxfVertex = { x: number; y: number; z?: number };
type DxfEntity = {
  type: string;
  layer?: string;
  vertices?: DxfVertex[];
  center?: DxfVertex;
  radius?: number;
  startAngle?: number;
  endAngle?: number;
  controlPoints?: DxfVertex[];
  majorAxisEndPoint?: DxfVertex;
  axisRatio?: number;
};

type DxfData = { entities?: DxfEntity[] };

type Vec3 = { x: number; y: number; z: number };

function pushPoint(points: Vec3[], x: number, y: number) {
  points.push({ x, y, z: 0 });
}

function arcPoints(
  center: DxfVertex,
  radius: number,
  startDeg: number,
  endDeg: number,
  segments = 32,
): Vec3[] {
  const pts: Vec3[] = [];
  let start = (startDeg * Math.PI) / 180;
  let end = (endDeg * Math.PI) / 180;
  if (end < start) {
    end += Math.PI * 2;
  }
  for (let i = 0; i <= segments; i += 1) {
    const t = start + ((end - start) * i) / segments;
    pts.push({
      x: center.x + radius * Math.cos(t),
      y: center.y + radius * Math.sin(t),
      z: 0,
    });
  }
  return pts;
}

function entitySegments(ent: DxfEntity): Vec3[] {
  const segs: Vec3[] = [];
  const t = ent.type;
  if ((t === "LINE" || t === "POLYLINE" || t === "LWPOLYLINE") && ent.vertices && ent.vertices.length >= 2) {
    const verts = ent.vertices;
    for (let i = 0; i < verts.length - 1; i += 1) {
      const a = verts[i];
      const b = verts[i + 1];
      pushPoint(segs, a.x, a.y);
      pushPoint(segs, b.x, b.y);
    }
    if (t !== "LINE" && verts.length > 2) {
      const a = verts[verts.length - 1];
      const b = verts[0];
      pushPoint(segs, a.x, a.y);
      pushPoint(segs, b.x, b.y);
    }
  } else if (t === "CIRCLE" && ent.center && ent.radius) {
    const ring = arcPoints(ent.center, ent.radius, 0, 360, 48);
    for (let i = 0; i < ring.length - 1; i += 1) {
      segs.push(ring[i], ring[i + 1]);
    }
  } else if (t === "ARC" && ent.center && ent.radius != null && ent.startAngle != null && ent.endAngle != null) {
    const arc = arcPoints(ent.center, ent.radius, ent.startAngle, ent.endAngle, 24);
    for (let i = 0; i < arc.length - 1; i += 1) {
      segs.push(arc[i], arc[i + 1]);
    }
  } else if (t === "ELLIPSE" && ent.center && ent.majorAxisEndPoint) {
    const mx = ent.majorAxisEndPoint.x - ent.center.x;
    const my = ent.majorAxisEndPoint.y - ent.center.y;
    const a = Math.hypot(mx, my);
    const ratio = ent.axisRatio ?? 0.5;
    const b = a * ratio;
    const rot = Math.atan2(my, mx);
    const pts: Vec3[] = [];
    for (let i = 0; i <= 48; i += 1) {
      const tRad = (i / 48) * Math.PI * 2;
      const lx = a * Math.cos(tRad);
      const ly = b * Math.sin(tRad);
      const x = ent.center.x + lx * Math.cos(rot) - ly * Math.sin(rot);
      const y = ent.center.y + lx * Math.sin(rot) + ly * Math.cos(rot);
      pts.push({ x, y, z: 0 });
    }
    for (let i = 0; i < pts.length - 1; i += 1) {
      segs.push(pts[i], pts[i + 1]);
    }
  } else if (t === "SPLINE" && ent.controlPoints && ent.controlPoints.length >= 2) {
    const verts = ent.controlPoints;
    for (let i = 0; i < verts.length - 1; i += 1) {
      pushPoint(segs, verts[i].x, verts[i].y);
      pushPoint(segs, verts[i + 1].x, verts[i + 1].y);
    }
  }
  return segs;
}

function bboxFromSegments(segs: Vec3[]) {
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  for (const p of segs) {
    minX = Math.min(minX, p.x);
    minY = Math.min(minY, p.y);
    maxX = Math.max(maxX, p.x);
    maxY = Math.max(maxY, p.y);
  }
  if (!Number.isFinite(minX)) {
    return null;
  }
  return { minX, minY, maxX, maxY };
}

interface CsvPreviewDto {
  headers: string[];
  rows: string[][];
  truncated: boolean;
  delimiter: string;
  total_data_rows: number;
  rows_returned: number;
  file_bytes: number;
  read_capped: boolean;
}

async function copyViewerSelection(): Promise<void> {
  const sel = window.getSelection();
  const text = sel?.toString().trim() ?? "";
  await copyText(text);
}

function printHtmlDocument(title: string, bodyHtml: string) {
  openPrintFriendlyDocument(title, bodyHtml);
}

async function renderSpreadsheet(path: string, root: HTMLElement) {
  const preview = await invoke<CsvPreviewDto>("get_csv_preview", { path, maxRows: 200 });
  root.innerHTML = "";
  const sizeKb = (preview.file_bytes / 1024).toFixed(1);
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = `Read-only CSV/TSV (${sizeKb} KiB on disk). Showing ${preview.rows_returned} of ${preview.total_data_rows} data row(s).${
    preview.truncated ? " Preview capped." : ""
  }${preview.read_capped ? " Large file: row scan may be incomplete." : ""}`;
  root.appendChild(note);
  const copyBtn = document.createElement("button");
  copyBtn.type = "button";
  copyBtn.textContent = "Copy table";
  copyBtn.onclick = () => {
    const rows = [preview.headers.join("\t"), ...preview.rows.map((r) => r.join("\t"))];
    void copyText(rows.join("\n"));
  };
  root.appendChild(copyBtn);
  const wrap = document.createElement("div");
  wrap.className = "csv-table-wrap";
  const table = document.createElement("table");
  table.className = "csv-table";
  const thead = document.createElement("thead");
  const hr = document.createElement("tr");
  for (const h of preview.headers) {
    const th = document.createElement("th");
    th.textContent = h;
    hr.appendChild(th);
  }
  thead.appendChild(hr);
  table.appendChild(thead);
  const tbody = document.createElement("tbody");
  for (const row of preview.rows) {
    const tr = document.createElement("tr");
    for (const cell of row) {
      const td = document.createElement("td");
      td.textContent = cell;
      tr.appendChild(td);
    }
    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  wrap.appendChild(table);
  root.appendChild(wrap);
}

async function renderImage(path: string, root: HTMLElement, initialZoom = 1) {
  await ensureAssetAllowed(path);
  root.innerHTML = "";
  const toolbar = document.createElement("div");
  toolbar.className = "image-toolbar";
  const zoomOut = document.createElement("button");
  zoomOut.type = "button";
  zoomOut.textContent = "−";
  const zoomIn = document.createElement("button");
  zoomIn.type = "button";
  zoomIn.textContent = "+";
  const resetBtn = document.createElement("button");
  resetBtn.type = "button";
  resetBtn.textContent = "Fit";
  const rotateBtn = document.createElement("button");
  rotateBtn.type = "button";
  rotateBtn.textContent = t("image.rotate");
  const flipHBtn = document.createElement("button");
  flipHBtn.type = "button";
  flipHBtn.textContent = t("image.flipH");
  const flipVBtn = document.createElement("button");
  flipVBtn.type = "button";
  flipVBtn.textContent = t("image.flipV");
  const saveAsBtn = document.createElement("button");
  saveAsBtn.type = "button";
  saveAsBtn.textContent = t("image.saveAs");
  const filtersBtn = document.createElement("button");
  filtersBtn.type = "button";
  filtersBtn.textContent = t("image.filters");
  filtersBtn.disabled = true;
  filtersBtn.title = t("image.filtersSoon");
  const cropBtn = document.createElement("button");
  cropBtn.type = "button";
  cropBtn.textContent = "Crop";
  const clearCropBtn = document.createElement("button");
  clearCropBtn.type = "button";
  clearCropBtn.textContent = "Clear crop";
  clearCropBtn.disabled = true;
  const saveCropBtn = document.createElement("button");
  saveCropBtn.type = "button";
  saveCropBtn.textContent = "Save cropped as…";
  saveCropBtn.disabled = true;
  const ocrBtn = document.createElement("button");
  ocrBtn.type = "button";
  ocrBtn.textContent = t("image.ocr");
  ocrBtn.disabled = true;
  ocrBtn.title = t("image.ocrDisabledReason");
  const copyImgBtn = document.createElement("button");
  copyImgBtn.type = "button";
  copyImgBtn.textContent = t("image.copy");
  copyImgBtn.onclick = () => void copyImageFile(path);
  const eyedropBtn = document.createElement("button");
  eyedropBtn.type = "button";
  eyedropBtn.textContent = t("image.eyedropper");
  eyedropBtn.title = t("image.eyedropperHint");
  let eyedropActive = false;
  eyedropBtn.onclick = () => {
    eyedropActive = !eyedropActive;
    eyedropBtn.classList.toggle("image-toolbar-active", eyedropActive);
    if (!eyedropActive) {
      clearImagePixelSample();
    }
  };
  toolbar.appendChild(zoomOut);
  toolbar.appendChild(zoomIn);
  toolbar.appendChild(resetBtn);
  toolbar.appendChild(rotateBtn);
  toolbar.appendChild(flipHBtn);
  toolbar.appendChild(flipVBtn);
  toolbar.appendChild(saveAsBtn);
  toolbar.appendChild(filtersBtn);
  toolbar.appendChild(cropBtn);
  toolbar.appendChild(clearCropBtn);
  toolbar.appendChild(saveCropBtn);
  toolbar.appendChild(ocrBtn);
  toolbar.appendChild(copyImgBtn);
  toolbar.appendChild(eyedropBtn);
  root.appendChild(toolbar);

  const viewport = document.createElement("div");
  viewport.className = "image-viewport";
  const stage = document.createElement("div");
  stage.className = "image-stage";
  const img = document.createElement("img");
  img.className = "image-view";
  img.alt = path;
  img.draggable = false;
  img.src = convertFileSrc(path);
  const cropHost = document.createElement("div");
  cropHost.className = "image-crop-host";
  const cropLayer = document.createElement("div");
  cropLayer.className = "image-crop-layer";
  cropLayer.hidden = true;
  const cropRect = document.createElement("div");
  cropRect.className = "image-crop-rect";
  cropLayer.appendChild(cropRect);
  cropHost.appendChild(img);
  cropHost.appendChild(cropLayer);
  stage.appendChild(cropHost);
  viewport.appendChild(stage);
  root.appendChild(viewport);

  const sampleCanvas = document.createElement("canvas");
  const sampleCtx = sampleCanvas.getContext("2d", { willReadFrequently: true });
  let sampleReady = false;
  const refreshSampleSurface = () => {
    if (!sampleCtx || !img.naturalWidth) {
      return;
    }
    sampleCanvas.width = img.naturalWidth;
    sampleCanvas.height = img.naturalHeight;
    sampleCtx.drawImage(img, 0, 0);
    sampleReady = true;
  };
  img.addEventListener("load", refreshSampleSurface);
  if (img.complete) {
    void img.decode().then(refreshSampleSurface).catch(() => undefined);
  }

  let scale = initialZoom;
  let rotation = 0;
  let flipH = false;
  let flipV = false;
  let panX = 0;
  let panY = 0;
  const canSaveRaster = /\.(png|jpe?g)$/i.test(path);
  if (!canSaveRaster) {
    saveAsBtn.disabled = true;
    saveAsBtn.title = t("image.saveAsPngJpegOnly");
  }
  let cropMode = false;
  let cropDragging = false;
  let cropStartX = 0;
  let cropStartY = 0;
  let cropRectNorm: { x: number; y: number; w: number; h: number } | null = null;

  const applyTransform = () => {
    const sx = flipH ? -1 : 1;
    const sy = flipV ? -1 : 1;
    stage.style.transform = `translate(${panX}px, ${panY}px) scale(${scale * sx}, ${scale * sy}) rotate(${rotation}deg)`;
  };

  async function rasterizeEditedImage(): Promise<{
    canvas: HTMLCanvasElement;
    mime: string;
  } | null> {
    await img.decode();
    const nw = img.naturalWidth;
    const nh = img.naturalHeight;
    if (!nw || !nh) {
      return null;
    }
    let crop = cropRectNorm;
    let sx = 0;
    let sy = 0;
    let sw = nw;
    let sh = nh;
    if (crop && crop.w >= 0.5 && crop.h >= 0.5) {
      sx = Math.max(0, Math.round((crop.x / 100) * nw));
      sy = Math.max(0, Math.round((crop.y / 100) * nh));
      sw = Math.min(nw - sx, Math.round((crop.w / 100) * nw));
      sh = Math.min(nh - sy, Math.round((crop.h / 100) * nh));
    }
    if (sw < 1 || sh < 1) {
      return null;
    }
    const quarter = ((rotation % 360) + 360) % 360;
    const swap = quarter === 90 || quarter === 270;
    const canvas = document.createElement("canvas");
    canvas.width = swap ? sh : sw;
    canvas.height = swap ? sw : sh;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return null;
    }
    ctx.translate(canvas.width / 2, canvas.height / 2);
    ctx.rotate((quarter * Math.PI) / 180);
    ctx.scale(flipH ? -1 : 1, flipV ? -1 : 1);
    ctx.drawImage(img, sx, sy, sw, sh, -sw / 2, -sh / 2, sw, sh);
    const mime = /\.jpe?g$/i.test(path) ? "image/jpeg" : "image/png";
    return { canvas, mime };
  }

  const bumpImageZoom = (delta: number) => {
    if (delta > 0) {
      scale = Math.min(8, scale * 1.15);
    } else {
      scale = Math.max(0.15, scale / 1.15);
    }
    applyTransform();
    reportViewerZoom(scale, "image");
  };
  zoomIn.onclick = () => bumpImageZoom(1);
  zoomOut.onclick = () => bumpImageZoom(-1);
  viewerZoomAdjust = bumpImageZoom;
  reportViewerZoom(scale, "image");
  const imageLocalPoint = (clientX: number, clientY: number) => {
    const rect = img.getBoundingClientRect();
    const x = ((clientX - rect.left) / rect.width) * 100;
    const y = ((clientY - rect.top) / rect.height) * 100;
    return { x, y };
  };

  const samplePixelAt = (clientX: number, clientY: number) => {
    if (!eyedropActive || !sampleReady || !sampleCtx || !img.naturalWidth) {
      return;
    }
    const rect = img.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) {
      return;
    }
    const px = Math.min(
      img.naturalWidth - 1,
      Math.max(0, Math.floor(((clientX - rect.left) / rect.width) * img.naturalWidth)),
    );
    const py = Math.min(
      img.naturalHeight - 1,
      Math.max(0, Math.floor(((clientY - rect.top) / rect.height) * img.naturalHeight)),
    );
    const data = sampleCtx.getImageData(px, py, 1, 1).data;
    const hex = `#${[data[0], data[1], data[2]]
      .map((v) => v.toString(16).padStart(2, "0"))
      .join("")}`;
    const line = `${hex} rgb(${data[0]}, ${data[1]}, ${data[2]})`;
    setImagePixelSample(line);
    patchStatusCursor(line);
  };

  img.addEventListener("mousemove", (ev) => samplePixelAt(ev.clientX, ev.clientY));
  img.addEventListener("mouseleave", () => {
    if (eyedropActive) {
      clearImagePixelSample();
      patchStatusCursor("");
    }
  });

  const updateCropActions = () => {
    const hasCrop = Boolean(cropRectNorm && cropRectNorm.w >= 0.5 && cropRectNorm.h >= 0.5);
    saveCropBtn.disabled = !hasCrop || rotation !== 0;
    saveCropBtn.title =
      rotation !== 0 ? "Reset rotation before exporting a crop" : "Write a new PNG; original file is unchanged";
  };

  const applyCropVisual = () => {
    if (!cropRectNorm || cropRectNorm.w < 0.5 || cropRectNorm.h < 0.5) {
      cropRect.hidden = true;
      clearCropBtn.disabled = true;
      updateCropActions();
      return;
    }
    cropRect.hidden = false;
    clearCropBtn.disabled = false;
    updateCropActions();
    const { x, y, w, h } = cropRectNorm;
    cropRect.style.left = `${x}%`;
    cropRect.style.top = `${y}%`;
    cropRect.style.width = `${w}%`;
    cropRect.style.height = `${h}%`;
  };

  cropBtn.onclick = () => {
    cropMode = !cropMode;
    cropBtn.classList.toggle("image-toolbar-active", cropMode);
    cropLayer.hidden = !cropMode;
    viewport.classList.toggle("image-viewport--crop", cropMode);
    if (!cropMode) {
      cropDragging = false;
    }
  };

  clearCropBtn.onclick = () => {
    cropRectNorm = null;
    applyCropVisual();
  };

  resetBtn.onclick = () => {
    scale = 1;
    panX = 0;
    panY = 0;
    rotation = 0;
    flipH = false;
    flipV = false;
    cropRectNorm = null;
    cropMode = false;
    cropBtn.classList.remove("image-toolbar-active");
    cropLayer.hidden = true;
    viewport.classList.remove("image-viewport--crop");
    applyCropVisual();
    applyTransform();
  };
  rotateBtn.onclick = () => {
    rotation = (rotation + 90) % 360;
    applyTransform();
    updateCropActions();
  };
  flipHBtn.onclick = () => {
    flipH = !flipH;
    applyTransform();
  };
  flipVBtn.onclick = () => {
    flipV = !flipV;
    applyTransform();
  };

  async function saveRasterCopy(suffix: string) {
    const raster = await rasterizeEditedImage();
    if (!raster) {
      return;
    }
    const quality = raster.mime === "image/jpeg" ? 0.92 : undefined;
    const blob = await new Promise<Blob | null>((resolve) => {
      raster.canvas.toBlob((b) => resolve(b), raster.mime, quality);
    });
    if (!blob) {
      return;
    }
    const bytes = Array.from(new Uint8Array(await blob.arrayBuffer()));
    const base = path.split(/[/\\]/).pop() ?? "image.png";
    const ext = raster.mime === "image/jpeg" ? ".jpg" : ".png";
    const suggested = `${base.replace(/\.[^.]+$/i, "")}${suffix}${ext}`;
    const dest = await invoke<string | null>("pick_save_image_path", { suggestedName: suggested });
    if (!dest) {
      return;
    }
    await invoke("write_document_bytes", { path: dest, bytes });
    const fileUrl = dest.startsWith("file://") ? dest : `file://${dest}`;
    window.dispatchEvent(new CustomEvent("unibrowser-navigate", { detail: fileUrl }));
  }

  saveAsBtn.onclick = () => {
    void saveRasterCopy("-edited").catch((err) => showErrorToast(err));
  };

  saveCropBtn.onclick = async () => {
    if (!cropRectNorm || cropRectNorm.w < 0.5 || cropRectNorm.h < 0.5) {
      return;
    }
    try {
      await saveRasterCopy("-crop");
    } catch (err) {
      showErrorToast(err);
    }
  };

  let dragging = false;
  let lastX = 0;
  let lastY = 0;
  viewport.onpointerdown = (e) => {
    if (cropMode) {
      cropDragging = true;
      const p = imageLocalPoint(e.clientX, e.clientY);
      cropStartX = p.x;
      cropStartY = p.y;
      cropRectNorm = { x: p.x, y: p.y, w: 0, h: 0 };
      applyCropVisual();
      viewport.setPointerCapture(e.pointerId);
      return;
    }
    dragging = true;
    lastX = e.clientX;
    lastY = e.clientY;
    viewport.setPointerCapture(e.pointerId);
  };
  viewport.onpointerup = () => {
    dragging = false;
    cropDragging = false;
  };
  viewport.onpointermove = (e) => {
    if (cropDragging && cropMode) {
      const p = imageLocalPoint(e.clientX, e.clientY);
      const x1 = Math.max(0, Math.min(100, Math.min(cropStartX, p.x)));
      const y1 = Math.max(0, Math.min(100, Math.min(cropStartY, p.y)));
      const x2 = Math.max(0, Math.min(100, Math.max(cropStartX, p.x)));
      const y2 = Math.max(0, Math.min(100, Math.max(cropStartY, p.y)));
      cropRectNorm = { x: x1, y: y1, w: x2 - x1, h: y2 - y1 };
      applyCropVisual();
      return;
    }
    if (!dragging) {
      return;
    }
    panX += e.clientX - lastX;
    panY += e.clientY - lastY;
    lastX = e.clientX;
    lastY = e.clientY;
    applyTransform();
  };
  viewport.onwheel = (e) => {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 0.9 : 1.1;
    scale = Math.min(8, Math.max(0.15, scale * factor));
    applyTransform();
  };

  applyTransform();
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = t("image.toolbarNote");
  root.appendChild(note);
}

async function renderDxf(path: string, root: HTMLElement) {
  root.innerHTML = "";
  const proving = document.createElement("p");
  proving.className = "viewer-note dxf-proving-banner";
  proving.textContent =
    "CAD proving mod (base-dxf): wireframe preview only. Measure and STEP export stay greyed out until a full engine ships. See docs/proving-mod-dxf.md.";
  root.appendChild(proving);

  const DxfParser = (await import("dxf-parser")).default;
  const text = new TextDecoder().decode(await readBytes(path));
  const parser = new DxfParser();
  const dxf = parser.parseSync(text) as DxfData | null;
  const entities = dxf?.entities ?? [];

  const geomPanel = document.createElement("aside");
  geomPanel.className = "dxf-geom-panel";
  const geomTitle = document.createElement("h3");
  geomTitle.textContent = "Geometry";
  geomPanel.appendChild(geomTitle);
  const geomList = document.createElement("ul");
  geomList.className = "dxf-geom-list";
  const counts = new Map<string, number>();
  for (const ent of entities) {
    const kind = ent.type ?? "UNKNOWN";
    counts.set(kind, (counts.get(kind) ?? 0) + 1);
  }
  const summary = Array.from(counts.entries()).sort((a, b) => a[0].localeCompare(b[0]));
  if (summary.length === 0) {
    const li = document.createElement("li");
    li.textContent = "No entities parsed";
    geomList.appendChild(li);
  } else {
    for (const [kind, count] of summary.slice(0, 40)) {
      const li = document.createElement("li");
      li.textContent = `${kind}: ${count}`;
      geomList.appendChild(li);
    }
  }
  geomPanel.appendChild(geomList);

  let THREE: Awaited<ReturnType<typeof loadThree>> | null = null;
  try {
    THREE = await loadThree();
  } catch {
    THREE = null;
  }
  const layers = Array.from(
    new Set(entities.map((e) => e.layer).filter((l): l is string => Boolean(l))),
  ).sort();
  const layerVisible = new Map<string, boolean>();
  for (const layer of layers) {
    layerVisible.set(layer, true);
  }
  const layout = document.createElement("div");
  layout.className = "dxf-layout";
  layout.appendChild(geomPanel);
  const layerPanel = document.createElement("aside");
  layerPanel.className = "dxf-layer-panel";
  const panelTitle = document.createElement("h3");
  panelTitle.textContent = "Layers";
  layerPanel.appendChild(panelTitle);
  const layerList = document.createElement("ul");
  layerList.className = "dxf-layer-list";
  for (const layer of layers) {
    const li = document.createElement("li");
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = true;
    cb.id = `dxf-layer-${layer}`;
    const label = document.createElement("label");
    label.htmlFor = cb.id;
    label.textContent = layer;
    cb.onchange = () => {
      layerVisible.set(layer, cb.checked);
      rebuild();
    };
    li.appendChild(cb);
    li.appendChild(label);
    layerList.appendChild(li);
  }
  if (layers.length === 0) {
    const empty = document.createElement("p");
    empty.className = "viewer-note";
    empty.textContent = "No layer names in file";
    layerPanel.appendChild(empty);
  } else {
    layerPanel.appendChild(layerList);
  }
  layout.appendChild(layerPanel);
  const canvasCol = document.createElement("div");
  canvasCol.className = "dxf-canvas-col";
  layout.appendChild(canvasCol);
  root.appendChild(layout);
  const wrap = document.createElement("div");
  wrap.className = "dxf-canvas-wrap";
  canvasCol.appendChild(wrap);
  const width = wrap.clientWidth || 800;
  const height = 400;

  const collectSegments = () => {
    const allSegs: Vec3[] = [];
    for (const ent of entities) {
      const layer = ent.layer ?? "";
      if (layer && layerVisible.has(layer) && !layerVisible.get(layer)) {
        continue;
      }
      allSegs.push(...entitySegments(ent));
    }
    return allSegs;
  };

  if (!THREE) {
    const note = document.createElement("p");
    note.className = "viewer-note";
    note.textContent = "WebGL unavailable — SVG wireframe proxy.";
    wrap.appendChild(note);
    const segs = collectSegments();
    const bbox = segs.length >= 2 ? bboxFromSegments(segs) : null;
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("width", String(width));
    svg.setAttribute("height", String(height));
    svg.classList.add("dxf-svg-proxy");
    if (bbox) {
      const pad = 12;
      const w = Math.max(bbox.maxX - bbox.minX, 1);
      const h = Math.max(bbox.maxY - bbox.minY, 1);
      svg.setAttribute(
        "viewBox",
        `${bbox.minX - pad} ${-bbox.maxY - pad} ${w + pad * 2} ${h + pad * 2}`,
      );
      for (let i = 0; i + 1 < segs.length; i += 2) {
        const a = segs[i];
        const b = segs[i + 1];
        const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
        line.setAttribute("x1", String(a.x));
        line.setAttribute("y1", String(-a.y));
        line.setAttribute("x2", String(b.x));
        line.setAttribute("y2", String(-b.y));
        line.setAttribute("stroke", "#66ccff");
        line.setAttribute("stroke-width", "1");
        svg.appendChild(line);
      }
    }
    wrap.appendChild(svg);
    return;
  }

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x1a1a22);
  const camera = new THREE.OrthographicCamera(-width / 2, width / 2, height / 2, -height / 2, 0.1, 1000);
  camera.position.z = 10;
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(width, height);
  wrap.appendChild(renderer.domElement);
  let panX = 0;
  let panY = 0;
  let zoom = 1;
  let baseBbox: { minX: number; minY: number; maxX: number; maxY: number } | null = null;

  const rebuild = () => {
    while (scene.children.length > 0) {
      scene.remove(scene.children[0]);
    }
    const allSegs = collectSegments();
    if (allSegs.length >= 2) {
      const points = allSegs.map((p) => new THREE.Vector3(p.x, p.y, p.z));
      const geom = new THREE.BufferGeometry().setFromPoints(points);
      const mat = new THREE.LineBasicMaterial({ color: 0x66ccff });
      scene.add(new THREE.LineSegments(geom, mat));
      baseBbox = bboxFromSegments(allSegs);
    } else {
      baseBbox = null;
    }
    if (baseBbox) {
      const cx = (baseBbox.minX + baseBbox.maxX) / 2 + panX;
      const cy = (baseBbox.minY + baseBbox.maxY) / 2 + panY;
      const worldW = Math.max(baseBbox.maxX - baseBbox.minX, 1) / zoom;
      const worldH = Math.max(baseBbox.maxY - baseBbox.minY, 1) / zoom;
      const pad = 1.12;
      const aspect = width / height;
      let halfW = (worldW / 2) * pad;
      let halfH = (worldH / 2) * pad;
      if (halfW / halfH > aspect) {
        halfH = halfW / aspect;
      } else {
        halfW = halfH * aspect;
      }
      camera.left = cx - halfW;
      camera.right = cx + halfW;
      camera.top = cy + halfH;
      camera.bottom = cy - halfH;
      camera.updateProjectionMatrix();
    }
    renderer.render(scene, camera);
  };

  let dragging = false;
  let lastX = 0;
  let lastY = 0;
  renderer.domElement.onpointerdown = (e) => {
    dragging = true;
    lastX = e.clientX;
    lastY = e.clientY;
    renderer.domElement.setPointerCapture(e.pointerId);
  };
  renderer.domElement.onpointerup = () => {
    dragging = false;
  };
  renderer.domElement.onpointermove = (e) => {
    if (!dragging || !baseBbox) {
      return;
    }
    const dx = (e.clientX - lastX) * (camera.right - camera.left) / width;
    const dy = (e.clientY - lastY) * (camera.top - camera.bottom) / height;
    panX -= dx;
    panY += dy;
    lastX = e.clientX;
    lastY = e.clientY;
    rebuild();
  };
  renderer.domElement.onwheel = (e) => {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 1.1 : 0.9;
    zoom = Math.min(20, Math.max(0.2, zoom * factor));
    rebuild();
  };

  rebuild();
  const meta = document.createElement("p");
  meta.className = "viewer-note";
  meta.textContent = `DXF: ${entities.length} entities · toggle layers · drag pan · wheel zoom`;
  canvasCol.appendChild(meta);
}

function renderMidi(viewer: DocumentViewerDto, root: HTMLElement) {
  const summary = viewer.midi_summary;
  if (!summary) {
    root.textContent = viewer.media_note ?? "MIDI file";
    return;
  }
  const dl = document.createElement("dl");
  dl.className = "midi-summary";
  const appendMidiRow = (label: string, value: string) => {
    const dt = document.createElement("dt");
    dt.textContent = label;
    const dd = document.createElement("dd");
    dd.textContent = value;
    dl.appendChild(dt);
    dl.appendChild(dd);
  };
  appendMidiRow("SMF format", String(summary.format));
  appendMidiRow("Track count", String(summary.track_count));
  appendMidiRow("Ticks per quarter note", String(summary.ticks_per_quarter));
  root.appendChild(dl);
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = "Native SMF header only — synthesis and encode are not wired yet.";
  root.appendChild(note);
  const preview = viewer.midi_note_preview ?? [];
  if (preview.length > 0) {
    const h = document.createElement("h3");
    h.textContent = "Note-on preview (scrub stub)";
    root.appendChild(h);
    const table = document.createElement("table");
    table.className = "midi-note-table";
    const head = document.createElement("tr");
    ["Track", "Tick", "Note", "Velocity"].forEach((c) => {
      const th = document.createElement("th");
      th.textContent = c;
      head.appendChild(th);
    });
    table.appendChild(head);
    preview.forEach((ev) => {
      const row = document.createElement("tr");
      [ev.track_index, ev.tick, ev.note, ev.velocity].forEach((v) => {
        const td = document.createElement("td");
        td.textContent = String(v);
        row.appendChild(td);
      });
      table.appendChild(row);
    });
    root.appendChild(table);
  }
}

async function renderUnibrow(viewer: DocumentViewerDto, root: HTMLElement) {
  root.innerHTML = "";
  const snapGrid = viewer.board_snap_to_grid ?? false;
  const hint = document.createElement("p");
  hint.className = "viewer-note unibrow-board-hint";
  hint.textContent =
    "Alt+drag pan · Ctrl+wheel zoom · Double-click card to open · Drag to move";
  root.appendChild(hint);

  const viewport = document.createElement("div");
  viewport.className = "unibrow-board-viewport";
  viewport.setAttribute("role", "img");
  viewport.setAttribute("aria-label", "Project board");
  const canvas = document.createElement("div");
  canvas.className = "unibrow-board-canvas";
  let panX = 0;
  let panY = 0;
  let scale = 1;
  const applyTransform = () => {
    canvas.style.transform = `translate(${panX}px, ${panY}px) scale(${scale})`;
  };
  applyTransform();

  let panning = false;
  let panStartX = 0;
  let panStartY = 0;
  let panOriginX = 0;
  let panOriginY = 0;
  viewport.addEventListener("pointerdown", (e) => {
    if (!e.altKey || e.target !== viewport) {
      return;
    }
    panning = true;
    panStartX = e.clientX;
    panStartY = e.clientY;
    panOriginX = panX;
    panOriginY = panY;
    viewport.setPointerCapture(e.pointerId);
  });
  viewport.addEventListener("pointermove", (e) => {
    if (!panning) {
      return;
    }
    panX = panOriginX + (e.clientX - panStartX);
    panY = panOriginY + (e.clientY - panStartY);
    applyTransform();
  });
  viewport.addEventListener("pointerup", (e) => {
    if (!panning) {
      return;
    }
    panning = false;
    viewport.releasePointerCapture(e.pointerId);
  });
  viewport.addEventListener(
    "wheel",
    (e) => {
      if (!e.ctrlKey) {
        return;
      }
      e.preventDefault();
      const delta = e.deltaY < 0 ? 0.1 : -0.1;
      scale = Math.min(2.5, Math.max(0.35, scale + delta));
      applyTransform();
    },
    { passive: false },
  );

  const nodesOnBoard = viewer.project_nodes ?? [];
  for (const node of nodesOnBoard) {
    const card = document.createElement("button");
    card.type = "button";
    card.className = "unibrow-board-card";
    card.style.left = `${node.x}px`;
    card.style.top = `${node.y}px`;
    applyLabelColor(card, node.color);
    const cardLabel = document.createElement("span");
    cardLabel.className = "unibrow-board-card-label";
    cardLabel.textContent = node.label;
    card.appendChild(cardLabel);
    const cardColor = makeLabelColorSelect(node.color, (color) => {
      void invoke("set_unibrow_node_color", {
        projectPath: viewer.path,
        nodeId: node.id,
        color,
      }).then((data) => {
        window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
      });
    });
    cardColor.addEventListener("pointerdown", (e) => e.stopPropagation());
    card.appendChild(cardColor);
    card.title = node.asset ? `${node.label} · ${node.asset}` : node.label;
    let startNodeX = node.x;
    let startNodeY = node.y;
    let startMX = 0;
    let startMY = 0;
    let moved = false;
    card.addEventListener("dblclick", (e) => {
      e.preventDefault();
      if (node.asset) {
        void openUnibrowAsset(viewer.path, node.asset);
      }
    });
    card.addEventListener("pointerdown", (e) => {
      if (e.button !== 0 || e.altKey) {
        return;
      }
      e.preventDefault();
      e.stopPropagation();
      card.setPointerCapture(e.pointerId);
      moved = false;
      startNodeX = node.x;
      startNodeY = node.y;
      startMX = e.clientX;
      startMY = e.clientY;
    });
    card.addEventListener("pointermove", (e) => {
      if (!card.hasPointerCapture(e.pointerId)) {
        return;
      }
      const dx = (e.clientX - startMX) / scale;
      const dy = (e.clientY - startMY) / scale;
      if (Math.abs(dx) > 2 || Math.abs(dy) > 2) {
        moved = true;
      }
      const x = Math.max(0, startNodeX + dx);
      const y = Math.max(0, startNodeY + dy);
      card.style.left = `${x}px`;
      card.style.top = `${y}px`;
    });
    card.addEventListener("pointerup", async (e) => {
      if (!card.hasPointerCapture(e.pointerId)) {
        return;
      }
      card.releasePointerCapture(e.pointerId);
      if (!moved) {
        return;
      }
      const dx = (e.clientX - startMX) / scale;
      const dy = (e.clientY - startMY) / scale;
      const x = snapBoardCoord(startNodeX + dx, snapGrid);
      const y = snapBoardCoord(startNodeY + dy, snapGrid);
      card.style.left = `${x}px`;
      card.style.top = `${y}px`;
      const data = await invoke("set_unibrow_node_position", {
        projectPath: viewer.path,
        nodeId: node.id,
        x,
        y,
      });
      window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
    });
    canvas.appendChild(card);
  }
  viewport.addEventListener("dragover", (e) => {
    if (pathsFromDataTransfer(e.dataTransfer).length) {
      e.preventDefault();
      viewport.classList.add("unibrow-drop-target");
    }
  });
  viewport.addEventListener("dragleave", () => viewport.classList.remove("unibrow-drop-target"));
  viewport.addEventListener("drop", (e) => {
    viewport.classList.remove("unibrow-drop-target");
    const paths = pathsFromDataTransfer(e.dataTransfer);
    if (!paths.length) {
      return;
    }
    e.preventDefault();
    void (async () => {
      try {
        let data: unknown = null;
        for (const path of paths) {
          data = await invoke("add_unibrow_asset", {
            projectPath: viewer.path,
            sourcePath: path,
          });
        }
        if (data) {
          window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
        }
      } catch (err) {
        showToast(String(err), "error");
      }
    })();
  });

  viewport.appendChild(canvas);
  root.appendChild(viewport);

  const clips = viewer.project_clips ?? [];
  const timelineTotalSec = (list: typeof clips) => {
    const end = list.reduce(
      (max, c) => Math.max(max, c.start_sec + c.duration_sec),
      0,
    );
    return Math.max(1, end);
  };
  const totalSec = timelineTotalSec(clips);
  const boardNodes = viewer.project_nodes ?? [];
  const markers = viewer.project_markers ?? [];

  const timeline = document.createElement("section");
  timeline.className = "unibrow-timeline";
  const tlTitle = document.createElement("h3");
  tlTitle.id = "unibrow-timeline-heading";
  tlTitle.textContent = "Timeline";
  timeline.appendChild(tlTitle);
  timeline.setAttribute("aria-labelledby", "unibrow-timeline-heading");

  const ruler = document.createElement("div");
  ruler.className = "unibrow-timeline-ruler";
  ruler.setAttribute("aria-hidden", "true");
  const tickStep = totalSec > 30 ? 5 : totalSec > 12 ? 2 : 1;
  for (let t = 0; t <= totalSec; t += tickStep) {
    const tick = document.createElement("span");
    tick.className = "unibrow-timeline-ruler-tick";
    tick.style.left = `${(t / totalSec) * 100}%`;
    tick.textContent = `${t}s`;
    ruler.appendChild(tick);
  }
  markers.forEach((marker) => {
    const pin = document.createElement("span");
    pin.className = "unibrow-timeline-marker-pin";
    pin.style.left = `${(marker.time_sec / totalSec) * 100}%`;
    pin.title = `${marker.title} @ ${marker.time_sec.toFixed(1)}s`;
    applyLabelColor(pin, marker.color);
    pin.textContent = "|";
    ruler.appendChild(pin);
  });
  timeline.appendChild(ruler);

  const track = document.createElement("div");
  track.className = "unibrow-timeline-track";
  const transitions = viewer.project_transitions ?? [];
  clips.forEach((clip) => {
    const block = document.createElement("div");
    block.className = "unibrow-timeline-block";
    block.title = clip.label;
    block.style.left = `${(clip.start_sec / totalSec) * 100}%`;
    block.style.width = `${Math.max((clip.duration_sec / totalSec) * 100, 2)}%`;
    block.textContent = clipTypeAbbrev(clip.node_type);
    const nodeColor = boardNodes.find((n) => n.id === clip.node_id)?.color;
    applyLabelColor(block, clip.color ?? nodeColor);
    track.appendChild(block);
  });
  transitions.forEach((tr) => {
    const clip = clips.find((c) => c.id === tr.after_clip_id);
    if (!clip || tr.kind !== "fade") {
      return;
    }
    const fade = document.createElement("span");
    fade.className = "unibrow-timeline-fade";
    const end = clip.start_sec + clip.duration_sec;
    fade.style.left = `${(end / totalSec) * 100}%`;
    fade.title = `Fade ${tr.duration_sec.toFixed(1)}s (export stub)`;
    fade.textContent = "fade";
    track.appendChild(fade);
  });
  timeline.appendChild(track);

  const scrubPanel = document.createElement("div");
  scrubPanel.className = "unibrow-timeline-scrub";
  const scrubLabel = document.createElement("label");
  scrubLabel.className = "video-scrub-label";
  scrubLabel.textContent = "Playhead scrub";
  const playhead = document.createElement("input");
  playhead.type = "range";
  playhead.className = "video-scrub-range";
  playhead.setAttribute("aria-label", "Timeline playhead");
  playhead.min = "0";
  playhead.max = String(Math.max(1, Math.floor(totalSec * 10)));
  playhead.value = "0";
  const playheadTime = document.createElement("span");
  playheadTime.className = "video-scrub-time";
  playheadTime.textContent = "0.0s";
  const scrubImg = document.createElement("img");
  scrubImg.className = "video-scrub-thumb hidden";
  scrubImg.alt = "";
  const scrubMidi = document.createElement("div");
  scrubMidi.className = "unibrow-scrub-midi hidden";
  const scrubNote = document.createElement("p");
  scrubNote.className = "viewer-note hidden";
  scrubPanel.appendChild(scrubLabel);
  scrubPanel.appendChild(playhead);
  scrubPanel.appendChild(playheadTime);
  scrubPanel.appendChild(scrubImg);
  scrubPanel.appendChild(scrubMidi);
  scrubPanel.appendChild(scrubNote);
  timeline.appendChild(scrubPanel);

  let scrubDebounce: number | undefined;
  let playheadSec = 0;
  const nodeById = new Map(boardNodes.map((n) => [n.id, n]));

  const showMidiNotes = (notes: MidiNoteOnDto[]) => {
    if (viewer.midi_scrub_audio_enabled) {
      playMidiScrubBeeps(notes);
    }
    scrubMidi.innerHTML = "";
    if (!notes.length) {
      scrubMidi.textContent = "No note-ons near playhead.";
      scrubMidi.classList.remove("hidden");
      return;
    }
    const table = document.createElement("table");
    table.className = "midi-note-table";
    const head = document.createElement("tr");
    ["Track", "Tick", "Note", "Vel"].forEach((c) => {
      const th = document.createElement("th");
      th.textContent = c;
      head.appendChild(th);
    });
    table.appendChild(head);
    notes.forEach((ev) => {
      const row = document.createElement("tr");
      [ev.track_index, ev.tick, ev.note, ev.velocity].forEach((v) => {
        const td = document.createElement("td");
        td.textContent = String(v);
        row.appendChild(td);
      });
      table.appendChild(row);
    });
    scrubMidi.appendChild(table);
    scrubMidi.classList.remove("hidden");
    const noteSummary = notes
      .slice(0, 5)
      .map((n) => `n${n.note}@${n.tick}`)
      .join(", ");
    setStatusHint(
      `MIDI scrub: ${noteSummary}${notes.length > 5 ? "…" : ""} — MIDI stays MIDI until encode`,
    );
  };

  const updatePlayheadPreview = async (timeSec: number) => {
    scrubImg.classList.add("hidden");
    scrubMidi.classList.add("hidden");
    scrubNote.classList.add("hidden");
    const clip = findClipAtTime(clips, timeSec);
    if (!clip) {
      scrubNote.textContent = "No clip at playhead.";
      scrubNote.classList.remove("hidden");
      return;
    }
    const offset = timeSec - clip.start_sec;
    try {
      const preview = await invoke<TimelineScrubPreview>("timeline_clip_scrub_preview", {
        projectPath: viewer.path,
        clipId: clip.id,
        offsetSec: offset,
      });
      if (preview.image_path) {
        await ensureAssetAllowed(preview.image_path);
        scrubImg.src = convertFileSrc(preview.image_path);
        scrubImg.classList.remove("hidden");
      } else if (preview.midi_notes) {
        showMidiNotes(preview.midi_notes);
      } else if (preview.kind === "audio") {
        scrubNote.textContent = "Audio clip — open asset for waveform.";
        scrubNote.classList.remove("hidden");
      }
    } catch {
      const node = nodeById.get(clip.node_id);
      const asset = node?.asset;
      if (clip.node_type === "pdf" && asset) {
        try {
          const assetPath = await invoke<string>("allow_unibrow_asset", {
            projectPath: viewer.path,
            assetName: asset,
          });
          scrubImg.src = await rasterPdfFirstPageDataUrl(assetPath);
          scrubImg.classList.remove("hidden");
          return;
        } catch {
          /* fall through */
        }
      }
      scrubNote.textContent = "Scrub preview unavailable for this clip.";
      scrubNote.classList.remove("hidden");
    }
  };

  playhead.addEventListener("input", () => {
    const t = Number.parseFloat(playhead.value) / 10;
    playheadSec = t;
    playheadTime.textContent = `${t.toFixed(1)}s`;
    window.clearTimeout(scrubDebounce);
    scrubDebounce = window.setTimeout(() => {
      void updatePlayheadPreview(t);
    }, 100);
  });
  if (clips.length) {
    void updatePlayheadPreview(0);
  }

  const markerRow = document.createElement("div");
  markerRow.className = "unibrow-timeline-markers";
  const markerTitle = document.createElement("h4");
  markerTitle.textContent = "Markers / chapters";
  markerRow.appendChild(markerTitle);
  const addMarkerBtn = document.createElement("button");
  addMarkerBtn.type = "button";
  addMarkerBtn.textContent = "Add marker at playhead";
  addMarkerBtn.setAttribute("aria-label", "Add chapter marker at playhead");
  addMarkerBtn.onclick = async () => {
    const title = window.prompt("Marker title", `Chapter ${markers.length + 1}`);
    if (!title) {
      return;
    }
    const data = await invoke("add_unibrow_timeline_marker", {
      projectPath: viewer.path,
      timeSec: playheadSec,
      title,
      color: "amber",
    });
    window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
  };
  markerRow.appendChild(addMarkerBtn);
  const markerList = document.createElement("ul");
  markerList.className = "unibrow-marker-list";
  markers.forEach((marker) => {
    const li = document.createElement("li");
    li.textContent = `${marker.time_sec.toFixed(1)}s — ${marker.title}`;
    applyLabelColor(li, marker.color);
    const remove = document.createElement("button");
    remove.type = "button";
    remove.textContent = "Remove";
    remove.onclick = async () => {
      const data = await invoke("remove_unibrow_timeline_marker", {
        projectPath: viewer.path,
        markerId: marker.id,
      });
      window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
    };
    li.appendChild(remove);
    markerList.appendChild(li);
  });
  if (!markers.length) {
    const empty = document.createElement("li");
    empty.className = "viewer-note";
    empty.textContent = "No markers yet.";
    markerList.appendChild(empty);
  }
  markerRow.appendChild(markerList);
  timeline.appendChild(markerRow);

  const timelineAddRow = document.createElement("div");
  timelineAddRow.className = "unibrow-timeline-add";
  const nodeSelect = document.createElement("select");
  const ph = document.createElement("option");
  ph.value = "";
  ph.textContent = boardNodes.length ? "Add clip from asset…" : "Add board assets first";
  nodeSelect.appendChild(ph);
  boardNodes.forEach((n) => {
    const opt = document.createElement("option");
    opt.value = n.id;
    opt.textContent = n.label;
    nodeSelect.appendChild(opt);
  });
  const addBtn = document.createElement("button");
  addBtn.type = "button";
  addBtn.textContent = "Add clip";
  addBtn.setAttribute("aria-label", "Add timeline clip from board asset");
  addBtn.disabled = boardNodes.length === 0;
  addBtn.onclick = async () => {
    if (!nodeSelect.value) {
      return;
    }
    const data = await invoke("add_unibrow_timeline_clip", {
      projectPath: viewer.path,
      nodeId: nodeSelect.value,
    });
    window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
  };
  const pasteStillBtn = document.createElement("button");
  pasteStillBtn.type = "button";
  pasteStillBtn.textContent = t("unibrow.pasteStill");
  pasteStillBtn.title = t("unibrow.pasteStillHint");
  pasteStillBtn.onclick = async () => {
    const src = await readClipboardImagePath();
    if (!src) {
      showToast(t("unibrow.pasteStillEmpty"), "info");
      return;
    }
    try {
      const data = await invoke("paste_unibrow_still_clip", {
        projectPath: viewer.path,
        sourcePath: src,
      });
      window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
      showToast(t("unibrow.pasteStillDone"), "info");
    } catch (err) {
      showToast(String(err), "error");
    }
  };
  timelineAddRow.appendChild(nodeSelect);
  timelineAddRow.appendChild(addBtn);
  timelineAddRow.appendChild(pasteStillBtn);
  timeline.appendChild(timelineAddRow);
  const ol = document.createElement("ol");
  ol.className = "unibrow-timeline-clips";
  let dragFromIndex: number | null = null;

  const reorderTo = async (fromIndex: number, toIndex: number) => {
    if (fromIndex === toIndex) {
      return;
    }
    const data = await invoke("reorder_unibrow_timeline_clips", {
      projectPath: viewer.path,
      fromIndex,
      toIndex,
    });
    window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
  };

  clips.forEach((clip, index) => {
    const li = document.createElement("li");
    li.className = "unibrow-timeline-clip";
    li.draggable = true;
    li.dataset.clipIndex = String(index);
    li.addEventListener("dragstart", (ev) => {
      dragFromIndex = index;
      li.classList.add("unibrow-timeline-clip--dragging");
      if (ev.dataTransfer) {
        ev.dataTransfer.effectAllowed = "move";
        ev.dataTransfer.setData("text/plain", String(index));
      }
    });
    li.addEventListener("dragend", () => {
      dragFromIndex = null;
      li.classList.remove("unibrow-timeline-clip--dragging");
      ol.querySelectorAll(".unibrow-timeline-clip--over").forEach((el) => {
        el.classList.remove("unibrow-timeline-clip--over");
      });
    });
    li.addEventListener("dragover", (ev) => {
      ev.preventDefault();
      if (ev.dataTransfer) {
        ev.dataTransfer.dropEffect = "move";
      }
      li.classList.add("unibrow-timeline-clip--over");
    });
    li.addEventListener("dragleave", () => {
      li.classList.remove("unibrow-timeline-clip--over");
    });
    li.addEventListener("drop", (ev) => {
      ev.preventDefault();
      li.classList.remove("unibrow-timeline-clip--over");
      const from =
        dragFromIndex ??
        Number.parseInt(ev.dataTransfer?.getData("text/plain") ?? "", 10);
      if (Number.isNaN(from) || from === index) {
        return;
      }
      void reorderTo(from, index);
    });

    const typeIcon = document.createElement("span");
    typeIcon.className = "unibrow-timeline-clip-type";
    typeIcon.title = clip.node_type;
    typeIcon.textContent = clipTypeAbbrev(clip.node_type);
    li.appendChild(typeIcon);

    const label = document.createElement("span");
    label.className = "unibrow-timeline-clip-label";
    label.textContent = clip.label;
    li.appendChild(label);

    const timing = document.createElement("div");
    timing.className = "unibrow-timeline-timing";
    const startLabel = document.createElement("label");
    startLabel.textContent = "Start (s)";
    const startInput = document.createElement("input");
    startInput.type = "number";
    startInput.min = "0";
    startInput.step = "0.1";
    startInput.value = String(clip.start_sec);
    startLabel.appendChild(startInput);
    const durLabel = document.createElement("label");
    durLabel.textContent = "Duration (s)";
    const durInput = document.createElement("input");
    durInput.type = "number";
    durInput.min = "0.1";
    durInput.step = "0.1";
    durInput.value = String(clip.duration_sec);
    durLabel.appendChild(durInput);
    const applyTiming = async () => {
      const start = Number.parseFloat(startInput.value);
      const duration = Number.parseFloat(durInput.value);
      if (!Number.isFinite(start) || !Number.isFinite(duration)) {
        return;
      }
      const data = await invoke("update_unibrow_timeline_clip", {
        projectPath: viewer.path,
        clipId: clip.id,
        startSec: start,
        durationSec: duration,
      });
      window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
    };
    startInput.addEventListener("change", () => void applyTiming());
    durInput.addEventListener("change", () => void applyTiming());
    timing.appendChild(startLabel);
    timing.appendChild(durLabel);
    li.appendChild(timing);

    const clipColor = makeLabelColorSelect(clip.color, (color) => {
      void invoke("set_unibrow_timeline_clip_color", {
        projectPath: viewer.path,
        clipId: clip.id,
        color,
      }).then((data) => {
        window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
      });
    });
    li.appendChild(clipColor);

    if (clipIsVideo(clip.node_type)) {
      const markerSec = document.createElement("div");
      markerSec.className = "unibrow-clip-markers";
      const markerHeading = document.createElement("span");
      markerHeading.className = "unibrow-clip-markers-title";
      markerHeading.textContent = "Clip markers";
      markerSec.appendChild(markerHeading);
      const addClipMarkerBtn = document.createElement("button");
      addClipMarkerBtn.type = "button";
      addClipMarkerBtn.textContent = "Add marker at playhead";
      addClipMarkerBtn.onclick = async () => {
        const clipLocal = playheadSec - clip.start_sec;
        if (clipLocal < 0 || clipLocal > clip.duration_sec) {
          showToast("Move playhead onto this clip first.", "error");
          return;
        }
        const name = window.prompt("Marker label", `Cue ${(clip.clip_markers?.length ?? 0) + 1}`);
        if (!name) {
          return;
        }
        const data = await invoke("add_unibrow_clip_marker", {
          projectPath: viewer.path,
          clipId: clip.id,
          offsetSec: clipLocal,
          label: name,
        });
        window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
      };
      markerSec.appendChild(addClipMarkerBtn);
      const clipMarkerList = document.createElement("ul");
      clipMarkerList.className = "unibrow-clip-marker-list";
      const markersOnClip = clip.clip_markers ?? [];
      markersOnClip.forEach((m) => {
        const mli = document.createElement("li");
        const abs = clip.start_sec + m.offset_sec;
        mli.textContent = `+${m.offset_sec.toFixed(1)}s (${abs.toFixed(1)}s) — ${m.label}`;
        const jump = document.createElement("button");
        jump.type = "button";
        jump.textContent = "Jump";
        jump.onclick = () => {
          playheadSec = abs;
          playhead.value = String(Math.round(abs * 10));
          playheadTime.textContent = `${abs.toFixed(1)}s`;
          void updatePlayheadPreview(abs);
        };
        const removeM = document.createElement("button");
        removeM.type = "button";
        removeM.textContent = "Remove";
        removeM.onclick = async () => {
          const data = await invoke("remove_unibrow_clip_marker", {
            projectPath: viewer.path,
            clipId: clip.id,
            markerId: m.id,
          });
          window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
        };
        mli.appendChild(jump);
        mli.appendChild(removeM);
        clipMarkerList.appendChild(mli);
      });
      if (!markersOnClip.length) {
        const emptyM = document.createElement("li");
        emptyM.className = "viewer-note";
        emptyM.textContent = "No clip markers yet.";
        clipMarkerList.appendChild(emptyM);
      }
      markerSec.appendChild(clipMarkerList);
      li.appendChild(markerSec);
    }

    if (index < clips.length - 1) {
      const tr = transitions.find((t) => t.after_clip_id === clip.id);
      const fadeRow = document.createElement("div");
      fadeRow.className = "unibrow-timeline-fade-row";
      const fadeCb = document.createElement("input");
      fadeCb.type = "checkbox";
      fadeCb.checked = Boolean(tr && tr.kind === "fade");
      const fadeLbl = document.createElement("label");
      fadeLbl.textContent = "Fade to next (stub)";
      fadeLbl.prepend(fadeCb);
      const fadeDur = document.createElement("input");
      fadeDur.type = "number";
      fadeDur.min = "0.1";
      fadeDur.step = "0.1";
      fadeDur.value = String(tr?.duration_sec ?? 0.5);
      fadeDur.disabled = !fadeCb.checked;
      fadeCb.onchange = () => {
        fadeDur.disabled = !fadeCb.checked;
        void invoke("set_unibrow_timeline_transition", {
          projectPath: viewer.path,
          afterClipId: clip.id,
          enabled: fadeCb.checked,
          durationSec: Number.parseFloat(fadeDur.value) || 0.5,
        }).then((data) => {
          window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
        });
      };
      fadeDur.onchange = () => {
        if (!fadeCb.checked) {
          return;
        }
        void invoke("set_unibrow_timeline_transition", {
          projectPath: viewer.path,
          afterClipId: clip.id,
          enabled: true,
          durationSec: Number.parseFloat(fadeDur.value) || 0.5,
        }).then((data) => {
          window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
        });
      };
      fadeRow.appendChild(fadeLbl);
      fadeRow.appendChild(fadeDur);
      li.appendChild(fadeRow);
    }

    const up = document.createElement("button");
    up.type = "button";
    up.textContent = "↑";
    up.disabled = index === 0;
    up.onclick = async () => {
      const data = await invoke("reorder_unibrow_timeline_clips", {
        projectPath: viewer.path,
        fromIndex: index,
        toIndex: index - 1,
      });
      window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
    };
    const down = document.createElement("button");
    down.type = "button";
    down.textContent = "↓";
    down.disabled = index === clips.length - 1;
    down.onclick = async () => {
      const data = await invoke("reorder_unibrow_timeline_clips", {
        projectPath: viewer.path,
        fromIndex: index,
        toIndex: index + 1,
      });
      window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
    };
    const remove = document.createElement("button");
    remove.type = "button";
    remove.textContent = "Remove";
    remove.onclick = async () => {
      const data = await invoke("remove_unibrow_timeline_clip", {
        projectPath: viewer.path,
        clipId: clip.id,
      });
      window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
    };
    li.appendChild(up);
    li.appendChild(down);
    li.appendChild(remove);
    ol.appendChild(li);
  });
  if (!clips.length) {
    const empty = document.createElement("li");
    empty.className = "unibrow-empty-wrap";
    empty.appendChild(emptyStateBlock("No clips yet", ["Add clips from the timeline tools."]));
    ol.appendChild(empty);
  }
  timeline.appendChild(ol);
  root.appendChild(timeline);
  const list = document.createElement("ul");
  list.className = "unibrow-nodes";
  const nodes = viewer.project_nodes ?? [];
  if (nodes.length === 0) {
    const empty = document.createElement("li");
    empty.className = "unibrow-empty-wrap";
    empty.appendChild(
      emptyStateBlock("No assets yet", ["Drop files onto the project or use Add asset."]),
    );
    list.appendChild(empty);
  } else {
    for (const node of nodes) {
      const li = document.createElement("li");
      li.className = "unibrow-node";
      const text = document.createElement("span");
      text.textContent = `${node.label} (${node.node_type})${node.asset ? ` · ${node.asset}` : ""}`;
      li.appendChild(text);
      if (node.asset) {
        const open = document.createElement("button");
        open.type = "button";
        open.textContent = "Open";
        open.onclick = () => {
          if (node.asset) {
            void openUnibrowAsset(viewer.path, node.asset);
          }
        };
        li.appendChild(open);
      }
      const rename = document.createElement("button");
      rename.type = "button";
      rename.textContent = "Rename";
      rename.onclick = async () => {
        const next = window.prompt("Node label", node.label);
        if (!next) {
          return;
        }
        const data = await invoke("rename_unibrow_node", {
          projectPath: viewer.path,
          nodeId: node.id,
          label: next,
        });
        window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
      };
      const remove = document.createElement("button");
      remove.type = "button";
      remove.textContent = "Remove";
      remove.onclick = async () => {
        if (!window.confirm(`Remove “${node.label}” from the project?`)) {
          return;
        }
        const data = await invoke("remove_unibrow_node", {
          projectPath: viewer.path,
          nodeId: node.id,
        });
        window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
      };
      li.appendChild(rename);
      li.appendChild(remove);
      list.appendChild(li);
    }
  }
  root.appendChild(list);

  const drop = document.createElement("div");
  drop.className = "unibrow-drop";
  drop.textContent = "Drop files here to add to project";
  drop.ondragover = (e) => {
    e.preventDefault();
    drop.classList.add("drag-over");
  };
  drop.ondragleave = () => drop.classList.remove("drag-over");
  drop.ondrop = async (e) => {
    e.preventDefault();
    drop.classList.remove("drag-over");
    const files = e.dataTransfer?.files;
    if (!files?.length) {
      return;
    }
    for (const file of files) {
      const withPath = file as File & { path?: string };
      if (!withPath.path) {
        continue;
      }
      const data = await invoke("add_unibrow_asset", {
        projectPath: viewer.path,
        sourcePath: withPath.path,
      });
      window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
    }
  };
  root.appendChild(drop);
  const assetAddRow = document.createElement("div");
  assetAddRow.className = "unibrow-add-row";
  const select = document.createElement("select");
  const tabs = viewer.open_file_tabs ?? [];
  const placeholder = document.createElement("option");
  placeholder.value = "";
  placeholder.textContent = tabs.length ? "Choose open file tab…" : "Open a file in another tab first";
  select.appendChild(placeholder);
  for (const tab of tabs) {
    const opt = document.createElement("option");
    opt.value = tab.path;
    opt.textContent = tab.label;
    select.appendChild(opt);
  }
  const btn = document.createElement("button");
  btn.type = "button";
  btn.textContent = "Add to project";
  btn.disabled = tabs.length === 0;
  btn.onclick = async () => {
    const sourcePath = select.value;
    if (!sourcePath) {
      return;
    }
    const data = await invoke("add_unibrow_asset", {
      projectPath: viewer.path,
      sourcePath,
    });
    window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
  };
  const pickBtn = document.createElement("button");
  pickBtn.type = "button";
  pickBtn.textContent = "Add from disk…";
  pickBtn.onclick = async () => {
    const picked = await invoke<string | null>("pick_unibrow_asset_file");
    if (!picked) {
      return;
    }
    const data = await invoke("add_unibrow_asset", {
      projectPath: viewer.path,
      sourcePath: picked,
    });
    window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
  };
  assetAddRow.appendChild(select);
  assetAddRow.appendChild(btn);
  assetAddRow.appendChild(pickBtn);
  root.appendChild(assetAddRow);
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = "Copies the file into project assets/ and updates project.json.";
  root.appendChild(note);

  const exportBtn = document.createElement("button");
  exportBtn.type = "button";
  exportBtn.textContent = "Export .unibrow zip…";
  exportBtn.onclick = async () => {
    try {
      const dest = await invoke<string>("export_unibrow_project", {
        projectPath: viewer.path,
      });
      const status = document.createElement("p");
      status.className = "viewer-note";
      status.textContent = `Exported to ${dest}`;
      root.appendChild(status);
    } catch (err) {
      window.alert(String(err));
    }
  };
  root.appendChild(exportBtn);

  const shareExportBtn = document.createElement("button");
  shareExportBtn.type = "button";
  shareExportBtn.textContent = "Export share package…";
  shareExportBtn.setAttribute(
    "aria-label",
    "Export collaboration share zip without derived folder",
  );
  shareExportBtn.onclick = async () => {
    try {
      const dest = await invoke<string>("export_unibrow_share_package", {
        projectPath: viewer.path,
      });
      showToast(`Share package: ${dest}`, "info");
    } catch (err) {
      showErrorToast(err);
    }
  };
  root.appendChild(shareExportBtn);

  const shareImportBtn = document.createElement("button");
  shareImportBtn.type = "button";
  shareImportBtn.textContent = "Import share package…";
  shareImportBtn.setAttribute("aria-label", "Import collaboration share zip as new project");
  shareImportBtn.onclick = async () => {
    const name = window.prompt("Folder name for imported project", "SharedProject");
    if (!name) {
      return;
    }
    const home = await invoke<string>("get_home_dir").catch(() => "/tmp");
    try {
      const data = await invoke("import_unibrow_share_package", {
        parentDir: home,
        name,
      });
      window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
    } catch (err) {
      showErrorToast(err);
    }
  };
  root.appendChild(shareImportBtn);

  const exportMp4Btn = document.createElement("button");
  exportMp4Btn.type = "button";
  exportMp4Btn.textContent = "Export MP4 (preview)";
  exportMp4Btn.onclick = async () => {
    try {
      const result = await invoke<{ path: string; mode: string; message: string }>(
        "export_unibrow_mp4_preview",
        { projectPath: viewer.path },
      );
      showToast(result.message, result.mode === "encoded" ? "info" : "info");
      const status = document.createElement("p");
      status.className = "viewer-note";
      status.textContent = `${result.message} (${result.path})`;
      root.appendChild(status);
    } catch (err) {
      showErrorToast(err, t("toast.encodeFailed"));
    }
  };
  root.appendChild(exportMp4Btn);

  const h264Row = document.createElement("div");
  h264Row.className = "docx-toolbar";
  const presetSelect = document.createElement("select");
  presetSelect.innerHTML =
    '<option value="fast">H.264 fast</option><option value="balanced">H.264 balanced</option>';
  const exportH264Btn = document.createElement("button");
  exportH264Btn.type = "button";
  exportH264Btn.textContent = "Export MP4 (H.264)";
  exportH264Btn.onclick = async () => {
    exportH264Btn.disabled = true;
    try {
      const result = await invoke<{ path: string; mode: string; message: string }>(
        "export_unibrow_mp4_h264",
        { projectPath: viewer.path, preset: presetSelect.value },
      );
      showToast(result.message, "info");
      const status = document.createElement("p");
      status.className = "viewer-note";
      status.textContent = `${result.message} (${result.path})`;
      root.appendChild(status);
    } catch (err) {
      showErrorToast(err, t("toast.encodeFailed"));
    } finally {
      exportH264Btn.disabled = false;
    }
  };
  h264Row.appendChild(presetSelect);
  h264Row.appendChild(exportH264Btn);
  root.appendChild(h264Row);
}

interface RhaiScriptInfoDto {
  allowlist: string[] | null;
  effective_commands: string[];
  document_commands_blocked: boolean;
}

async function refreshRhaiMeta(
  source: string,
  metaEl: HTMLElement,
  allowDoc: HTMLInputElement,
) {
  const info = await invoke<RhaiScriptInfoDto>("describe_rhai_script", { source });
  const allowPart = info.allowlist?.length
    ? `allow-commands: ${info.allowlist.join(", ")}`
    : "allow-commands: (all registry ids)";
  metaEl.textContent = `${allowPart} · ${info.effective_commands.length} callable · palette browse-only without doc`;
  allowDoc.disabled = !info.document_commands_blocked;
  if (!info.document_commands_blocked) {
    allowDoc.checked = false;
  }
}

interface EpubChapterDto {
  index: number;
  total: number;
  title: string;
  html: string;
}

interface EpubSearchHitDto {
  chapter_index: number;
  title: string;
  snippet: string;
  match_count: number;
}

interface EpubChapterListItem {
  index: number;
  title: string;
}

function applyDocxSpellcheckExperiment(article: HTMLElement) {
  if (!isExperimentalSpellcheckEnabled()) {
    article.contentEditable = "false";
    applySpellcheckToEditable(article, false);
    return;
  }
  article.contentEditable = "true";
  article.setAttribute("role", "textbox");
  article.setAttribute("aria-label", "DOCX preview (experimental edit)");
  applySpellcheckToEditable(article, true);
}

function refreshViewerSpellcheckTargets() {
  document.querySelectorAll<HTMLElement>(".docx-view").forEach((article) => {
    applyDocxSpellcheckExperiment(article);
  });
  document.querySelectorAll<HTMLElement>(".epub-reading-notes").forEach((notes) => {
    applySpellcheckToEditable(notes, isExperimentalSpellcheckEnabled());
  });
}

window.addEventListener("unibrowser-spellcheck-refresh", () => {
  refreshViewerSpellcheckTargets();
});

async function renderEpub(viewer: DocumentViewerDto, root: HTMLElement) {
  root.innerHTML = "";
  let chapterIndex = 0;
  let chapters: EpubChapterListItem[] = [];

  const notes = document.createElement("textarea");
  notes.className = "epub-reading-notes";
  notes.rows = 3;
  notes.placeholder = t("epub.readingNotesPlaceholder");
  applySpellcheckToEditable(notes, isExperimentalSpellcheckEnabled());

  const searchRow = document.createElement("div");
  searchRow.className = "epub-search-row";
  const searchInput = document.createElement("input");
  searchInput.type = "search";
  searchInput.className = "epub-search-input";
  searchInput.placeholder = t("epub.searchPlaceholder");
  const searchStatus = document.createElement("span");
  searchStatus.className = "epub-search-status";
  searchRow.appendChild(searchInput);
  searchRow.appendChild(searchStatus);

  const results = document.createElement("ul");
  results.className = "epub-search-results";

  const nav = document.createElement("div");
  nav.className = "epub-nav";
  const prev = document.createElement("button");
  prev.type = "button";
  prev.textContent = t("epub.previous");
  const next = document.createElement("button");
  next.type = "button";
  next.textContent = t("epub.next");
  const label = document.createElement("span");
  label.className = "epub-chapter-label";
  const chapterSelect = document.createElement("select");
  chapterSelect.className = "epub-chapter-select";
  chapterSelect.setAttribute("aria-label", t("epub.chapterJump"));

  const frame = document.createElement("iframe");
  frame.className = "epub-chapter-frame";
  frame.title = "EPUB chapter";
  frame.sandbox.add("allow-same-origin");
  appendTrustBanner(root, "remote-html", "EPUB spine HTML preview.");

  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent =
    "Spine HTML preview (Rust zip/OPF). Images and fonts may be missing until asset URLs are resolved.";

  async function loadChapter(index: number) {
    const ch = await invoke<EpubChapterDto>("get_epub_chapter", {
      path: viewer.path,
      index,
    });
    chapterIndex = ch.index;
    label.textContent = `${ch.title} (${ch.index + 1} / ${ch.total})`;
    if (chapterSelect.value !== String(ch.index)) {
      chapterSelect.value = String(ch.index);
    }
    prev.disabled = ch.index <= 0;
    next.disabled = ch.index + 1 >= ch.total;
    const pt = viewer.reader_font_pt ?? 12;
    frame.srcdoc = `<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{font-size:${pt}pt;line-height:1.45;font-family:system-ui,sans-serif;margin:0.75rem;}</style></head><body>${ch.html}</body></html>`;
    void invoke("remember_epub_chapter_position", {
      path: viewer.path,
      index: ch.index,
    }).catch(() => undefined);
  }

  prev.onclick = () => loadChapter(chapterIndex - 1).catch(showEpubError);
  next.onclick = () => loadChapter(chapterIndex + 1).catch(showEpubError);
  chapterSelect.onchange = () => {
    const idx = Number(chapterSelect.value);
    if (!Number.isNaN(idx)) {
      loadChapter(idx).catch(showEpubError);
    }
  };

  setViewerFindRunner((query, forward) => {
    const win = frame.contentWindow;
    if (!win || !query.trim()) {
      return;
    }
    const finder = win as unknown as Window & {
      find: (
        query: string,
        caseSensitive: boolean,
        backwards: boolean,
        wrapAround: boolean,
        wholeWord: boolean,
        searchInFrames: boolean,
        showDialog: boolean,
      ) => boolean;
    };
    finder.find(query, false, !forward, false, false, false, false);
  });

  function showEpubError(err: unknown) {
    showErrorToast(err, t("toast.openEpubFailed"));
  }

  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  async function runEpubSearch() {
    const q = searchInput.value.trim();
    results.innerHTML = "";
    if (!q) {
      searchStatus.textContent = "";
      return;
    }
    const hits = await invoke<EpubSearchHitDto[]>("search_epub_in_book", {
      path: viewer.path,
      query: q,
      limit: 48,
    });
    if (!hits.length) {
      searchStatus.textContent = t("epub.searchEmpty");
      return;
    }
    const totalMatches = hits.reduce((n, h) => n + h.match_count, 0);
    searchStatus.textContent = t("epub.searchStatus", { count: totalMatches });
    for (const hit of hits) {
      const li = document.createElement("li");
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "epub-search-hit";
      btn.textContent = `${hit.title}: ${hit.snippet}`;
      btn.onclick = () => loadChapter(hit.chapter_index).catch(showEpubError);
      li.appendChild(btn);
      results.appendChild(li);
    }
  }

  searchInput.oninput = () => {
    if (searchTimer) {
      clearTimeout(searchTimer);
    }
    searchTimer = setTimeout(() => {
      runEpubSearch().catch((err) => {
        showErrorToast(err, t("epub.searchEmpty"));
      });
    }, 280);
  };

  const printBtn = document.createElement("button");
  printBtn.type = "button";
  printBtn.textContent = t("epub.printChapter");
  printBtn.onclick = () => {
    const html = frame.srcdoc || "";
    printHtmlDocument(viewer.path, html);
  };
  nav.appendChild(prev);
  nav.appendChild(chapterSelect);
  nav.appendChild(label);
  nav.appendChild(next);
  nav.appendChild(printBtn);
  root.appendChild(searchRow);
  root.appendChild(notes);
  root.appendChild(results);
  root.appendChild(nav);
  root.appendChild(frame);
  root.appendChild(note);
  try {
    chapters = await invoke<EpubChapterListItem[]>("list_epub_chapters", {
      path: viewer.path,
    });
    chapterSelect.innerHTML = "";
    for (const item of chapters) {
      const opt = document.createElement("option");
      opt.value = String(item.index);
      opt.textContent = item.title;
      chapterSelect.appendChild(opt);
    }
    const saved = await invoke<number>("get_epub_read_position", { path: viewer.path });
    const start =
      saved >= 0 && saved < chapters.length ? saved : 0;
    await loadChapter(start);
  } catch (err) {
    showErrorToast(err, t("toast.openEpubFailed"));
  }
}

async function renderSvg(viewer: DocumentViewerDto, root: HTMLElement) {
  root.innerHTML = "";
  await ensureAssetAllowed(viewer.path);
  const text = new TextDecoder().decode(await readBytes(viewer.path));
  const layout = document.createElement("div");
  layout.className = "svg-editor-layout";
  const preview = document.createElement("div");
  preview.className = "svg-preview";
  const frame = document.createElement("iframe");
  frame.className = "svg-preview-frame";
  frame.title = "SVG preview";
  frame.src = convertFileSrc(viewer.path);
  const fontPt = viewer.reader_font_pt ?? 14;
  frame.style.transformOrigin = "top left";
  frame.style.transform = `scale(${fontPt / 14})`;
  preview.appendChild(frame);
  const ta = document.createElement("textarea");
  ta.className = "svg-source";
  applySpellcheckToEditable(ta, true);
  ta.value = text;
  let timer: ReturnType<typeof setTimeout> | null = null;
  ta.oninput = () => {
    if (timer) {
      clearTimeout(timer);
    }
    timer = setTimeout(() => {
      invoke("sync_svg_document", { content: ta.value }).catch(() => {});
    }, 300);
  };
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = "Edit source; Save with Ctrl+S or the command palette.";
  layout.appendChild(preview);
  layout.appendChild(ta);
  root.appendChild(layout);
  root.appendChild(note);
}

async function renderScript(viewer: DocumentViewerDto, root: HTMLElement) {
  root.innerHTML = "";
  appendTrustBanner(root, "user-script", viewer.path);
  const ta = document.createElement("textarea");
  ta.className = "script-source";
  ta.spellcheck = false;
  const bytes = await readBytes(viewer.path);
  ta.value = new TextDecoder().decode(bytes);
  const meta = document.createElement("p");
  meta.className = "script-meta viewer-note";
  const allowDoc = document.createElement("input");
  allowDoc.type = "checkbox";
  allowDoc.id = "rhai-allow-doc-cmds";
  const allowLabel = document.createElement("label");
  allowLabel.htmlFor = allowDoc.id;
  allowLabel.textContent = "Allow execute_command on open document";
  const run = document.createElement("button");
  run.type = "button";
  run.textContent = "Run (Rhai)";
  const out = document.createElement("pre");
  out.className = "script-output";
  const syncMeta = () => refreshRhaiMeta(ta.value, meta, allowDoc).catch(() => {});
  ta.addEventListener("input", () => {
    void syncMeta();
  });
  run.onclick = async () => {
    try {
      const result = await invoke<string>("run_rhai_script", {
        source: ta.value,
        allowDocumentCommands: allowDoc.checked,
      });
      out.textContent = result;
    } catch (err) {
      out.textContent = String(err);
    }
  };
  root.appendChild(ta);
  root.appendChild(meta);
  const allowRow = document.createElement("div");
  allowRow.className = "script-allow-row";
  allowRow.appendChild(allowDoc);
  allowRow.appendChild(allowLabel);
  root.appendChild(allowRow);
  root.appendChild(run);
  root.appendChild(out);
  await syncMeta();
  try {
    const demo = await invoke<string>("run_rhai_script", {
      source: "list_commands().to_string()",
      allowDocumentCommands: false,
    });
    out.textContent = `list_commands(): ${demo}`;
  } catch (err) {
    out.textContent = String(err);
  }
}

function viewerCacheKey(viewer: DocumentViewerDto): string {
  const nodeCount = viewer.project_nodes?.length ?? 0;
  const scale = viewer.pdf_scale ?? 0;
  const page = viewer.pdf_page ?? 0;
  const font = viewer.reader_font_pt ?? 0;
  return `${viewer.viewer}:${viewer.path}:${scale}:${page}:${font}:${nodeCount}`;
}

export interface DocumentViewerApplyExtras {
  downloadHistory?: DownloadHistoryEntryDto[];
  onRefreshState?: () => void | Promise<void>;
}

export const documentViewer = {
  async apply(
    viewer: DocumentViewerDto | null,
    el: HTMLElement,
    extras?: DocumentViewerApplyExtras,
  ) {
    if (!viewer) {
      el.classList.add("hidden");
      el.innerHTML = "";
      loadedKeys.delete(el);
      return;
    }
    const key = viewerCacheKey(viewer);
    const prevKey = loadedKeys.get(el) ?? null;
    if (key === prevKey && el.childElementCount > 0) {
      el.classList.remove("hidden");
      return;
    }
    loadedKeys.set(el, key);
    el.classList.remove("hidden");
    el.innerHTML = "<p class='viewer-loading'>Loading…</p>";
    try {
      el.innerHTML = "";
      const title = document.createElement("h2");
      title.className = "viewer-title";
      title.textContent = viewer.title;
      el.appendChild(title);
      const body = document.createElement("div");
      body.className = "viewer-body";
      el.appendChild(body);
      switch (viewer.viewer) {
        case "docx":
          await renderDocx(viewer.path, body, viewer.reader_font_pt ?? 12);
          break;
        case "pdf":
          await renderPdf(
            viewer.path,
            body,
            viewer.pdf_scale ?? 1.25,
            viewer.pdf_page ?? 1,
            viewer.pdf_engines,
          );
          break;
        case "video":
          await renderVideo(viewer.path, body, viewer.media_note);
          break;
        case "audio":
          await renderAudio(viewer.path, body, viewer.media_note);
          break;
        case "image":
          await renderImage(viewer.path, body, viewer.preview_zoom ?? 1);
          break;
        case "spreadsheet":
          await renderSpreadsheet(viewer.path, body);
          break;
        case "dxf":
          await renderDxf(viewer.path, body);
          break;
        case "svg":
          await renderSvg(viewer, body);
          break;
        case "epub":
          await renderEpub(viewer, body);
          break;
        case "script":
          await renderScript(viewer, body);
          break;
        case "unibrow":
          await renderUnibrow(viewer, body);
          break;
        case "midi":
          renderMidi(viewer, body);
          break;
        case "settings":
          await renderSettings(body);
          break;
        case "about":
          await renderAbout(body);
          break;
        case "keys":
          await renderKeys(body);
          break;
        case "diagnostics":
          await renderDiagnostics(body);
          break;
        case "downloads":
          await renderDownloadsManager(
            body,
            extras?.downloadHistory ?? [],
            extras?.onRefreshState ?? (() => {}),
          );
          break;
        case "docs": {
          const slug = viewer.media_note ?? "getting-started";
          await renderDocs(body, slug);
          break;
        }
        default:
          body.textContent = `Unknown viewer: ${viewer.viewer}`;
      }
    } catch (err) {
      el.innerHTML = `<p class='viewer-error'>${String(err)}</p>`;
    }
  },
  hide(el: HTMLElement) {
    viewerZoomAdjust = null;
    setViewerFindRunner(null);
    el.classList.add("hidden");
  },
  getPrintHtmlForPdf(el: HTMLElement): { title: string; html: string } | null {
    const article = el.querySelector(".docx-view");
    if (!article) {
      return null;
    }
    const titleEl = el.querySelector(".viewer-title");
    const title = titleEl?.textContent?.trim() || "document";
    return { title, html: article.innerHTML };
  },
};
