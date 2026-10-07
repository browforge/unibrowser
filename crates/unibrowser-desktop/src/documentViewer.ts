import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { loadPdfJs, loadThree } from "./lazyModules";
import { appendTrustBanner } from "./trustBanner";
import { mountClickToPlay } from "./mediaPlayGate";
import { bindMediaViewerKeys } from "./mediaKeyboard";
import { renderDownloadsManager, type DownloadHistoryEntryDto } from "./downloadsPage";
import { reportViewerPdfPage, reportViewerZoom } from "./viewerZoom";
import { bindPreviewLinks } from "./previewLinks";
import { copyImageFile, copyText } from "./clipboard";
import { renderSettings } from "./settingsPage";
import { renderAbout } from "./aboutPage";
import { renderDocs } from "./docsPage";
import { mountSvgInspector } from "./svgInspector";
import { renderKeys } from "./keysPage";
import { renderDiagnostics } from "./diagnosticsPage";
import { invokeWithToast, showErrorToast, showToast } from "./toast";
import { patchStatusCursor } from "./statusBar";
import {
  clearImagePixelSample,
  setImagePixelSample,
} from "./viewerPixelSample";
import { t } from "./i18n";
import { renderWorkshop, resetWorkshopListeners } from "./workshop";
import { clearViewerLocatorBridge, setViewerLocatorBridge } from "./locatorViewerBridge";
import {
  downloadPrintFriendlyHtml,
  openPrintFriendlyDocument,
  openPrintPreviewDocument,
} from "./printExport";
import {
  applySpellcheckToEditable,
  isExperimentalSpellcheckEnabled,
} from "./experimentalSpellcheck";
import { normalisedPdfRectToCss } from "./pdfAnnotationGeometry";

let viewerZoomAdjust: ((delta: number) => void) | null = null;
let dxfViewCommand: ((commandId: string) => boolean) | null = null;
let pdfViewCommand: ((commandId: string) => boolean) | null = null;

type FindRunner = (query: string, next: boolean) => void;
let viewerFindRunner: FindRunner | null = null;

export function setViewerFindRunner(run: FindRunner | null) {
  viewerFindRunner = run;
}

export function runViewerFind(query: string, next: boolean) {
  viewerFindRunner?.(query, next);
}

export function adjustViewerZoom(delta: number) {
  viewerZoomAdjust?.(delta);
}

export function runViewerContextCommand(commandId: string): boolean {
  if (commandId.startsWith("unibrowser.project.")) {
    window.dispatchEvent(new CustomEvent("unibrow-project-command", { detail: commandId }));
    return true;
  }
  if (commandId === "unibrowser.image.zoomOut") {
    adjustViewerZoom(-1);
    return true;
  }
  if (commandId === "unibrowser.image.zoomIn") {
    adjustViewerZoom(1);
    return true;
  }
  if (commandId.startsWith("unibrowser.pdf.")) {
    return pdfViewCommand?.(commandId) ?? false;
  }
  return dxfViewCommand?.(commandId) ?? false;
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
  orphan_assets?: string[];
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
  tab_muted?: boolean;
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
  duck_under_video?: boolean;
  fade_in_sec?: number;
  fade_out_sec?: number;
  asset?: string | null;
  pdf_page?: number | null;
  title_text?: string | null;
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

function docxEditHtmlSiblingPath(docxPath: string): string {
  return `${docxPath}.ub-edit.html`;
}

async function saveDocxEditsToDocx(docxPath: string, article: HTMLElement) {
  try {
    const report = await invoke<{ warnings: string[]; paragraphs_written: number }>(
      "save_docx_from_preview_html",
      { path: docxPath, html: article.innerHTML },
    );
    const warn = report.warnings.length ? ` ${report.warnings[0]}` : "";
    showToast(t("docx.savedEditsDocx", { path: docxPath }) + warn, "info");
  } catch (err) {
    showErrorToast(err);
  }
}

async function saveDocxEditsAsHtml(
  docxPath: string,
  article: HTMLElement,
  readerFontPt: number,
) {
  const dest = docxEditHtmlSiblingPath(docxPath);
  const body = article.innerHTML;
  const html = `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${docxPath
    .split(/[/\\]/)
    .pop() ?? "document"}</title><style>body{font-size:${readerFontPt}pt;line-height:1.45;font-family:system-ui,sans-serif;margin:1rem;}</style></head><body>${body}</body></html>`;
  const bytes = Array.from(new TextEncoder().encode(html));
  try {
    await invoke("write_document_bytes", { path: dest, bytes });
    showToast(t("docx.savedEditsHtml", { path: dest }), "info");
  } catch (err) {
    showErrorToast(err);
  }
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
    "Mammoth HTML preview (not a full Word round-trip). Edits stay in this session until you save a sibling .ub-edit.html file.";

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
  const saveHtmlBtn = document.createElement("button");
  saveHtmlBtn.type = "button";
  saveHtmlBtn.textContent = t("docx.saveEditsHtml");
  const saveDocxBtn = document.createElement("button");
  saveDocxBtn.type = "button";
  saveDocxBtn.textContent = t("docx.saveEditsDocx");
  toolbar.appendChild(printBtn);
  toolbar.appendChild(exportBtn);
  toolbar.appendChild(saveHtmlBtn);
  toolbar.appendChild(saveDocxBtn);

  const article = document.createElement("article");
  article.className = "docx-view docx-view--paper";
  article.style.fontSize = `${readerFontPt}pt`;
  article.innerHTML = result.value;
  saveHtmlBtn.onclick = () => {
    void saveDocxEditsAsHtml(path, article, readerFontPt);
  };
  saveDocxBtn.onclick = () => {
    void saveDocxEditsToDocx(path, article);
  };
  applyDocxSpellcheckExperiment(article);

  root.innerHTML = "";
  const reader = document.createElement("div");
  reader.className = "docx-reader";
  const paper = document.createElement("div");
  paper.className = "docx-paper";
  paper.appendChild(article);
  reader.appendChild(paper);
  root.appendChild(reader);
  setViewerLocatorBridge({
    docxParagraph: (paragraphIndex) => {
      const nodes = article.querySelectorAll("p, h1, h2, h3, h4, li");
      const target = nodes[paragraphIndex] ?? nodes[0];
      if (!target) {
        return;
      }
      target.classList.add("docx-locator-flash");
      target.scrollIntoView({ block: "center", behavior: "smooth" });
      window.setTimeout(() => target.classList.remove("docx-locator-flash"), 2400);
    },
  });
  bindPreviewLinks(article, path);
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
  kind?: string;
  author?: string | null;
  created_at_ms?: number | null;
  label?: string | null;
};

function formatPdfAnnotationTime(ms: number | null | undefined): string {
  if (!ms) {
    return "";
  }
  const d = new Date(ms);
  return d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
}

function pdfAnnotationKindLabel(kind: string | undefined): string {
  switch (kind) {
    case "area":
      return "Area";
    case "note":
      return "Note";
    default:
      return "Highlight";
  }
}

function rebuildPdfAnnotationsDock(
  annotDock: HTMLElement,
  highlights: PdfHighlightDto[],
  currentPage: number,
  onJump: (page: number) => void,
) {
  annotDock.innerHTML = "";
  const head = document.createElement("header");
  head.className = "pdf-annotations-head";
  const annotTitle = document.createElement("h3");
  annotTitle.className = "pdf-annotations-title";
  annotTitle.textContent = `Annotations ${highlights.length}`;
  head.appendChild(annotTitle);
  annotDock.appendChild(head);
  const annotBody = document.createElement("div");
  annotBody.className = "pdf-annotations-body";
  if (!highlights.length) {
    const empty = document.createElement("p");
    empty.className = "pdf-annotations-empty";
    empty.textContent = "No annotations on this document.";
    annotBody.appendChild(empty);
  } else {
    const list = document.createElement("ul");
    list.className = "pdf-annotations-list";
    highlights.forEach((h, index) => {
      const li = document.createElement("li");
      li.className = "pdf-annotation-card";
      if (h.page === currentPage) {
        li.classList.add("pdf-annotation-card--current");
      }
      const row = document.createElement("div");
      row.className = "pdf-annotation-card-head";
      const identity = document.createElement("span");
      identity.className = "pdf-annotation-identity";
      const icon = document.createElement("span");
      icon.className = `pdf-annotation-icon pdf-annotation-icon--${h.kind ?? "highlight"}`;
      icon.setAttribute("aria-hidden", "true");
      icon.textContent = h.kind === "note" ? "N" : h.kind === "area" ? "A" : "H";
      const swatch = document.createElement("span");
      swatch.className = "pdf-annotation-colour";
      swatch.style.backgroundColor = h.color;
      swatch.setAttribute("aria-label", `Annotation colour ${h.color}`);
      const kind = document.createElement("span");
      kind.className = "pdf-annotation-kind";
      kind.textContent = pdfAnnotationKindLabel(h.kind);
      identity.append(icon, swatch, kind);
      const pageLink = document.createElement("button");
      pageLink.type = "button";
      pageLink.className = "pdf-annotation-page";
      pageLink.textContent = `Page ${h.page}`;
      pageLink.onclick = () => onJump(h.page);
      row.appendChild(identity);
      row.appendChild(pageLink);
      const title = document.createElement("p");
      title.className = "pdf-annotation-label";
      title.textContent = h.label ?? `Annotation ${index + 1}`;
      const meta = document.createElement("p");
      meta.className = "pdf-annotation-meta";
      const who = h.author ?? "Unknown";
      const when = formatPdfAnnotationTime(h.created_at_ms);
      meta.textContent = when ? `${who} · ${when}` : who;
      const appearance = document.createElement("details");
      appearance.className = "pdf-annotation-appearance";
      const appearanceTrigger = document.createElement("summary");
      appearanceTrigger.setAttribute("aria-label", "Annotation style");
      const appearanceSwatch = document.createElement("span");
      appearanceSwatch.className = "pdf-annotation-colour";
      appearanceSwatch.style.backgroundColor = h.color;
      appearanceTrigger.append(appearanceSwatch, document.createTextNode("Style"));
      const popover = document.createElement("div");
      popover.className = "pdf-annotation-swatch-popover";
      const colour = document.createElement("input");
      colour.type = "color";
      colour.value = h.kind === "area" ? "#c0392b" : "#ffe600";
      colour.setAttribute("aria-label", "Annotation colour");
      const thickness = document.createElement("input");
      thickness.type = "range";
      thickness.min = "1";
      thickness.max = "8";
      thickness.value = h.kind === "area" ? "2" : "1";
      thickness.setAttribute("aria-label", "Annotation thickness");
      popover.append(colour, thickness);
      appearance.append(appearanceTrigger, popover);
      const reply = document.createElement("textarea");
      reply.className = "pdf-annotation-reply";
      reply.rows = 2;
      reply.placeholder = "Reply to annotation";
      const actions = document.createElement("div");
      actions.className = "pdf-annotation-actions";
      const resolveBtn = document.createElement("button");
      resolveBtn.type = "button";
      resolveBtn.className = "pdf-annotation-resolve";
      resolveBtn.textContent = "Resolve";
      const replyBtn = document.createElement("button");
      replyBtn.type = "button";
      replyBtn.className = "pdf-annotation-reply-btn";
      replyBtn.textContent = "Post reply";
      actions.append(appearance, replyBtn, resolveBtn);
      li.appendChild(row);
      li.appendChild(title);
      li.appendChild(meta);
      li.appendChild(reply);
      li.appendChild(actions);
      list.appendChild(li);
    });
    annotBody.appendChild(list);
  }
  annotDock.appendChild(annotBody);
}

let pdfHighlightArmed = false;

export function armPdfHighlightMode() {
  pdfHighlightArmed = true;
  document.querySelectorAll(".pdf-workspace").forEach((el) => {
    el.classList.add("pdf-highlight-armed");
  });
}

async function renderPdf(
  path: string,
  root: HTMLElement,
  scale: number,
  startPage: number,
  _engines?: PdfEngineLineDto[],
) {
  void _engines;
  void invoke<string | null>("ensure_pdf_raster_proxy", { path }).catch(() => {});
  const pdfjsLib = await loadPdfJs();
  const bytes = await readBytes(path);
  const doc = await pdfjsLib.getDocument({ data: bytes }).promise;
  let currentPage = Math.min(Math.max(startPage, 1), doc.numPages);
  let currentScale = scale;
  root.innerHTML = "";
  let highlights: PdfHighlightDto[] = [];
  try {
    highlights = await invoke<PdfHighlightDto[]>("list_pdf_highlights", { path });
  } catch {
    highlights = [];
  }

  const workspace = document.createElement("div");
  workspace.className = "pdf-workspace";
  if (pdfHighlightArmed) {
    workspace.classList.add("pdf-highlight-armed");
  }
  const center = document.createElement("div");
  center.className = "pdf-workspace-center";
  const scrollSurface = document.createElement("div");
  scrollSurface.className = "pdf-scroll-surface";
  const pageStage = document.createElement("div");
  pageStage.className = "pdf-page-stage";
  const wrap = document.createElement("div");
  wrap.className = "pdf-page-wrap";
  pageStage.appendChild(wrap);
  scrollSurface.appendChild(pageStage);
  center.appendChild(scrollSurface);

  root.appendChild(workspace);
  workspace.appendChild(center);

  const annotDock = document.createElement("aside");
  annotDock.className = "pdf-annotations-dock";
  annotDock.setAttribute("aria-label", "PDF annotations");
  workspace.appendChild(annotDock);

  const syncAnnotationDock = () => {
    rebuildPdfAnnotationsDock(annotDock, highlights, currentPage, (p) => {
      void goToPage(p);
    });
    reportViewerPdfPage(currentPage, doc.numPages, highlights.length);
  };

  const paintHighlights = (pageNum: number, host: HTMLElement, width: number, height: number) => {
    host.querySelectorAll(".pdf-highlight-rect").forEach((el) => el.remove());
    for (const h of highlights) {
      if (h.page !== pageNum) {
        continue;
      }
      const box = document.createElement("div");
      const isArea = h.kind === "area";
      box.className = isArea ? "pdf-highlight-rect pdf-area-rect" : "pdf-highlight-rect";
      const cssRect = normalisedPdfRectToCss(h, width, height);
      box.style.left = `${cssRect.left}px`;
      box.style.top = `${cssRect.top}px`;
      box.style.width = `${cssRect.width}px`;
      box.style.height = `${cssRect.height}px`;
      if (isArea) {
        box.style.borderColor = h.color.includes("rgb") ? "#c0392b" : h.color;
      } else {
        box.style.background = h.color;
      }
      host.appendChild(box);
    }
  };

  const thumbsPanel = document.createElement("aside");
  thumbsPanel.className = "pdf-thumbs-panel";
  const thumbsHead = document.createElement("div");
  thumbsHead.className = "pdf-thumbs-head";
  thumbsHead.textContent = `Pages ${doc.numPages}`;
  const thumbs = document.createElement("div");
  thumbs.className = "pdf-thumbs pdf-thumbs--strip";
  thumbs.setAttribute("role", "navigation");
  thumbs.setAttribute("aria-label", "PDF page thumbnails");
  const THUMB_EAGER = 8;
  const THUMB_SCALE = 0.22;

  const updateThumbActive = () => {
    thumbs.querySelectorAll<HTMLButtonElement>(".pdf-thumb-btn").forEach((btn) => {
      const page = Number.parseInt(btn.dataset.page ?? "", 10);
      const active = page === currentPage;
      btn.classList.toggle("pdf-thumb-btn--active", active);
      btn.setAttribute("aria-current", active ? "page" : "false");
      if (active) {
        btn.scrollIntoView({ block: "nearest", inline: "nearest" });
      }
    });
  };

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
    const outputScale = Math.max(1, window.devicePixelRatio || 1);
    canvas.width = Math.floor(viewport.width * outputScale);
    canvas.height = Math.floor(viewport.height * outputScale);
    canvas.style.width = `${viewport.width}px`;
    canvas.style.height = `${viewport.height}px`;
    wrap.appendChild(canvas);
    const textLayer = document.createElement("div");
    textLayer.className = "pdf-text-layer";
    textLayer.style.width = `${viewport.width}px`;
    textLayer.style.height = `${viewport.height}px`;
    wrap.appendChild(textLayer);
    await page.render({
      canvasContext: ctx,
      viewport,
      canvas,
      transform: outputScale === 1 ? undefined : [outputScale, 0, 0, outputScale, 0, 0],
    }).promise;
    wrap.appendChild(overlay);
    overlay.style.pointerEvents = pdfHighlightArmed ? "auto" : "none";
    paintHighlights(pageNum, overlay, viewport.width, viewport.height);

    let dragStart: { x: number; y: number } | null = null;
    let draft: HTMLDivElement | null = null;
    const norm = (clientX: number, clientY: number) => {
      const rect = overlay.getBoundingClientRect();
      return {
        x: Math.min(1, Math.max(0, (clientX - rect.left) / rect.width)),
        y: Math.min(1, Math.max(0, (clientY - rect.top) / rect.height)),
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
        workspace.classList.remove("pdf-highlight-armed");
        overlay.style.pointerEvents = "none";
        paintHighlights(pageNum, overlay, viewport.width, viewport.height);
        syncAnnotationDock();
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
    updateThumbActive();
    syncAnnotationDock();
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
    const clamped = Math.min(Math.max(Math.floor(n), 1), doc.numPages);
    currentPage = clamped;
    await invoke("sync_pdf_page", { page: currentPage, pageCount: doc.numPages });
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

  thumbsPanel.appendChild(thumbsHead);
  thumbsPanel.appendChild(thumbs);
  if (doc.numPages >= 1) {
    const eager = Math.min(doc.numPages, THUMB_EAGER);
    for (let n = 1; n <= eager; n += 1) {
      addThumb(n, false);
    }
    for (let n = eager + 1; n <= doc.numPages; n += 1) {
      addThumb(n, true);
    }
    workspace.insertBefore(thumbsPanel, center);
  }

  const bumpZoom = (delta: number) => {
    const factor = delta > 0 ? 1.12 : 1 / 1.12;
    currentScale = Math.min(4, Math.max(0.5, currentScale * factor));
    void invoke("sync_pdf_scale", { scale: currentScale }).catch(() => {});
    reportViewerZoom(currentScale, "pdf");
    void drawPage(currentPage);
  };
  viewerZoomAdjust = bumpZoom;
  pdfViewCommand = (commandId: string) => {
    if (commandId === "unibrowser.pdf.page.nav") {
      void goToPage(currentPage >= doc.numPages ? 1 : currentPage + 1);
      return true;
    }
    if (commandId === "unibrowser.pdf.view.zoom") {
      bumpZoom(1);
      return true;
    }
    if (commandId === "unibrowser.pdf.annotate.highlight") {
      armPdfHighlightMode();
      return true;
    }
    return false;
  };

  const fitWidthScale = async () => {
    const page = await doc.getPage(currentPage);
    const base = page.getViewport({ scale: 1 });
    const maxWidth = Math.max(320, scrollSurface.clientWidth - 48);
    return Math.min(4, Math.max(0.5, maxWidth / base.width));
  };
  if (Math.abs(currentScale - scale) < 0.001 || scale === 1.25) {
    currentScale = await fitWidthScale();
    void invoke("sync_pdf_scale", { scale: currentScale }).catch(() => {});
  }
  reportViewerZoom(currentScale, "pdf");

  syncAnnotationDock();
  await drawPage(currentPage);
  updateThumbActive();
  await invoke("sync_pdf_page", { page: currentPage, pageCount: doc.numPages }).catch(() => undefined);
  setViewerLocatorBridge({
    pdfRegion: (page, x, y, w, h) => {
      void goToPage(page).then(() => {
        const overlay = wrap.querySelector<HTMLElement>(".pdf-page-overlay");
        if (!overlay) {
          return;
        }
        const flash = document.createElement("div");
        flash.className = "pdf-highlight-rect pdf-locator-flash";
        const width = overlay.clientWidth;
        const height = overlay.clientHeight;
        flash.style.left = `${x * width}px`;
        flash.style.top = `${y * height}px`;
        flash.style.width = `${w * width}px`;
        flash.style.height = `${h * height}px`;
        overlay.appendChild(flash);
        window.setTimeout(() => flash.remove(), 2400);
      });
    },
  });
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

async function renderVideo(
  path: string,
  root: HTMLElement,
  note?: string,
  tabMuted = false,
) {
  await ensureAssetAllowed(path);
  root.innerHTML = "";
  const video = document.createElement("video");
  video.className = "media-view";
  video.controls = true;
  video.muted = tabMuted;
  video.src = convertFileSrc(path);
  mountClickToPlay(root, video, "Click to play video");
  bindMediaViewerKeys(video);

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

async function renderAudio(
  path: string,
  root: HTMLElement,
  note?: string,
  tabMuted = false,
) {
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
  audio.muted = tabMuted;
  audio.src = convertFileSrc(path);
  const audioWrap = document.createElement("div");
  root.appendChild(audioWrap);
  mountClickToPlay(audioWrap, audio, "Click to play audio");
  bindMediaViewerKeys(audio);
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
  handle?: string;
  id?: string | number;
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

function computeRgbHistogram(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
): { r: Uint32Array; g: Uint32Array; b: Uint32Array } {
  const r = new Uint32Array(256);
  const g = new Uint32Array(256);
  const b = new Uint32Array(256);
  const step = Math.max(1, Math.floor(Math.sqrt((width * height) / 65536)));
  const data = ctx.getImageData(0, 0, width, height).data;
  for (let y = 0; y < height; y += step) {
    for (let x = 0; x < width; x += step) {
      const i = (y * width + x) * 4;
      r[data[i]] += 1;
      g[data[i + 1]] += 1;
      b[data[i + 2]] += 1;
    }
  }
  return { r, g, b };
}

function paintHistogramChannel(
  canvas: HTMLCanvasElement,
  bins: Uint32Array,
  stroke: string,
) {
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    return;
  }
  const w = canvas.width;
  const h = canvas.height;
  ctx.clearRect(0, 0, w, h);
  let max = 1;
  for (let i = 0; i < 256; i++) {
    if (bins[i] > max) {
      max = bins[i];
    }
  }
  ctx.strokeStyle = stroke;
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let i = 0; i < 256; i++) {
    const x = (i / 255) * (w - 1);
    const y = h - 1 - (bins[i] / max) * (h - 4);
    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  }
  ctx.stroke();
}

async function renderImage(path: string, root: HTMLElement, initialZoom = 1) {
  await ensureAssetAllowed(path);
  root.innerHTML = "";
  const layout = document.createElement("div");
  layout.className = "image-viewer-layout";
  const mainCol = document.createElement("div");
  mainCol.className = "image-viewer-main";
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
  void invoke<boolean>("ocr_tesseract_available_cmd").then((ok) => {
    ocrBtn.disabled = !ok;
    ocrBtn.title = ok ? t("image.ocr") : t("image.ocrDisabledReason");
  });
  ocrBtn.onclick = () => {
    void invoke<string>("execute_command", {
      commandId: "unibrowser.image.ocr.copy",
    }).then((msg) => {
      if (msg.startsWith("ocr-copy:")) {
        const text = msg.slice("ocr-copy:".length);
        void navigator.clipboard.writeText(text).then(() => {
          showToast(t("image.ocrCopied"), "info");
        });
      }
    }).catch(showErrorToast);
  };
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
  const histBtn = document.createElement("button");
  histBtn.type = "button";
  histBtn.textContent = t("image.histogram");
  histBtn.title = t("image.histogramHint");
  toolbar.appendChild(histBtn);
  toolbar.appendChild(eyedropBtn);
  mainCol.appendChild(toolbar);

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
  mainCol.appendChild(viewport);

  const histAside = document.createElement("aside");
  histAside.className = "image-histogram-panel hidden";
  const histTitle = document.createElement("h3");
  histTitle.textContent = t("image.histogram");
  histAside.appendChild(histTitle);
  const curvesBtn = document.createElement("button");
  curvesBtn.type = "button";
  curvesBtn.textContent = t("image.curves");
  curvesBtn.disabled = true;
  curvesBtn.title = t("image.curvesSoon");
  histAside.appendChild(curvesBtn);
  const histNote = document.createElement("p");
  histNote.className = "viewer-note";
  histNote.textContent = t("image.histogramNote");
  histAside.appendChild(histNote);
  const histCanvases: HTMLCanvasElement[] = [];
  for (const [label] of [["R"], ["G"], ["B"]] as const) {
    const row = document.createElement("div");
    row.className = "image-histogram-row";
    const lbl = document.createElement("span");
    lbl.textContent = label;
    const c = document.createElement("canvas");
    c.width = 256;
    c.height = 48;
    c.className = "image-histogram-canvas";
    histCanvases.push(c);
    row.appendChild(lbl);
    row.appendChild(c);
    histAside.appendChild(row);
  }
  layout.appendChild(mainCol);
  layout.appendChild(histAside);
  root.appendChild(layout);

  const sampleCanvas = document.createElement("canvas");
  const sampleCtx = sampleCanvas.getContext("2d", { willReadFrequently: true });
  let sampleReady = false;
  const refreshHistogram = () => {
    if (!sampleCtx || !sampleReady || !img.naturalWidth) {
      return;
    }
    const { r, g, b } = computeRgbHistogram(
      sampleCtx,
      sampleCanvas.width,
      sampleCanvas.height,
    );
    paintHistogramChannel(histCanvases[0], r, "#f66");
    paintHistogramChannel(histCanvases[1], g, "#6c6");
    paintHistogramChannel(histCanvases[2], b, "#68f");
  };
  const refreshSampleSurface = () => {
    if (!sampleCtx || !img.naturalWidth) {
      return;
    }
    sampleCanvas.width = img.naturalWidth;
    sampleCanvas.height = img.naturalHeight;
    sampleCtx.drawImage(img, 0, 0);
    sampleReady = true;
    refreshHistogram();
  };
  histBtn.onclick = () => {
    histAside.classList.toggle("hidden");
    if (!histAside.classList.contains("hidden")) {
      refreshHistogram();
    }
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
  setViewerLocatorBridge({
    imageRegion: (x, y, w, h) => {
      const box = document.createElement("div");
      box.className = "image-locator-flash";
      box.style.left = `${x * 100}%`;
      box.style.top = `${y * 100}%`;
      box.style.width = `${w * 100}%`;
      box.style.height = `${h * 100}%`;
      cropHost.appendChild(box);
      window.setTimeout(() => box.remove(), 2400);
    },
  });
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = t("image.toolbarNote");
  root.appendChild(note);
}

async function renderDxf(path: string, root: HTMLElement) {
  root.innerHTML = "";
  dxfViewCommand = null;
  const proving = document.createElement("p");
  proving.className = "viewer-note dxf-proving-banner";
  proving.textContent =
    "CAD proving mod (base-dxf): wireframe preview only. Measure and STEP export stay greyed out until a full engine ships. See docs/proving-mod-dxf.md.";
  root.appendChild(proving);

  const DxfParser = (await import("dxf-parser")).default;
  const text = new TextDecoder().decode(await readBytes(path));
  const parser = new DxfParser();
  let dxf: DxfData | null = null;
  let parseError: string | null = null;
  try {
    dxf = parser.parseSync(text) as DxfData | null;
  } catch (err) {
    parseError = err instanceof Error ? err.message : String(err);
  }
  const entities = dxf?.entities ?? [];
  const showDxfWireframeError = (host: HTMLElement, detail: string) => {
    const err = document.createElement("p");
    err.className = "viewer-note dxf-wireframe-error";
    err.textContent = detail;
    host.appendChild(err);
  };

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
  layout.appendChild(geomPanel);
  root.appendChild(layout);
  const wrap = document.createElement("div");
  wrap.className = "dxf-canvas-wrap";
  canvasCol.appendChild(wrap);
  const width = wrap.clientWidth || 800;
  const height = Math.max(400, wrap.clientHeight || 600);

  if (parseError) {
    showDxfWireframeError(
      wrap,
      `Could not read this DXF: ${parseError}. Try exporting ASCII DXF from your CAD tool.`,
    );
    return;
  }
  if (entities.length === 0) {
    showDxfWireframeError(
      wrap,
      "No geometry found in this DXF. The file may be empty, binary-only, or use entities the proving parser does not support yet.",
    );
    return;
  }

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

  const registerDxfLocator = (onZoom?: () => void) => {
    setViewerLocatorBridge({
      dxfEntity: (handle, layer) => {
        const layerItem = layerList.querySelector(`label[for="dxf-layer-${layer}"]`);
        layerItem?.classList.add("dxf-locator-flash");
        window.setTimeout(() => layerItem?.classList.remove("dxf-locator-flash"), 2400);
        const ent = entities.find(
          (e) => String(e.handle ?? "") === handle || String(e.id ?? "") === handle,
        );
        const layerName = ent?.layer ?? layer;
        if (layerName && layerVisible.get(layerName) === false) {
          const cb = layerList.querySelector<HTMLInputElement>(`#dxf-layer-${layerName}`);
          if (cb) {
            cb.checked = true;
            layerVisible.set(layerName, true);
            onZoom?.();
          }
        }
        onZoom?.();
        geomTitle.textContent = handle ? `Geometry · ${handle}` : "Geometry";
      },
    });
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
    } else {
      showDxfWireframeError(
        wrap,
        "No drawable wireframe segments in this DXF (unsupported entity types or zero-length geometry).",
      );
    }
    wrap.appendChild(svg);
    let fallbackZoom = 1;
    dxfViewCommand = (commandId) => {
      if (commandId === "unibrowser.cad.fitView") {
        fallbackZoom = 1;
      } else if (commandId === "unibrowser.cad.zoomOut") {
        fallbackZoom = Math.max(0.2, fallbackZoom / 1.25);
      } else if (commandId === "unibrowser.cad.zoomIn") {
        fallbackZoom = Math.min(20, fallbackZoom * 1.25);
      } else {
        return false;
      }
      svg.style.transform = `scale(${fallbackZoom})`;
      svg.style.transformOrigin = "center";
      return true;
    };
    registerDxfLocator();
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
    if (!baseBbox && entities.length > 0) {
      if (!wrap.querySelector(".dxf-wireframe-error")) {
        showDxfWireframeError(
          wrap,
          "No drawable wireframe segments in this DXF (unsupported entity types or zero-length geometry).",
        );
      }
    }
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
  dxfViewCommand = (commandId) => {
    if (commandId === "unibrowser.cad.fitView") {
      panX = 0;
      panY = 0;
      zoom = 1;
    } else if (commandId === "unibrowser.cad.zoomOut") {
      zoom = Math.max(0.2, zoom / 1.25);
    } else if (commandId === "unibrowser.cad.zoomIn") {
      zoom = Math.min(20, zoom * 1.25);
    } else {
      return false;
    }
    rebuild();
    return true;
  };
  const meta = document.createElement("p");
  meta.className = "viewer-note";
  meta.textContent = `DXF: ${entities.length} entities · toggle layers · drag pan · wheel zoom`;
  canvasCol.appendChild(meta);
  registerDxfLocator(() => {
    zoom = Math.min(20, zoom * 1.35);
    rebuild();
  });
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
  note.textContent = "Native SMF header only — full synthesis is not wired yet.";
  root.appendChild(note);
  const renderRow = document.createElement("div");
  renderRow.className = "viewer-actions";
  const renderBtn = document.createElement("button");
  renderBtn.type = "button";
  renderBtn.textContent = "Render to audio (stub)";
  renderBtn.disabled = true;
  renderBtn.title = "Checking tools…";
  void invoke<{ available: boolean; hint: string }>("probe_midi_audio_render").then((probe) => {
    renderBtn.disabled = !probe.available;
    renderBtn.title = probe.hint;
    if (!probe.available) {
      renderBtn.classList.add("cmd-disabled-stub");
    }
  });
  renderBtn.onclick = () => {
    void invokeWithToast(
      () => invoke<string>("execute_command", { commandId: "unibrowser.midi.renderAudio" }),
      "MIDI render unavailable",
    ).then((msg) => {
      if (msg) {
        showToast(String(msg), "info");
      }
    });
  };
  renderRow.appendChild(renderBtn);
  root.appendChild(renderRow);
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

interface RhaiScriptInfoDto {
  allowlist?: string[];
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
  try {
    notes.value = await invoke<string>("get_epub_reading_notes", { path: viewer.path });
  } catch {
    notes.value = "";
  }
  let notesSaveTimer: ReturnType<typeof setTimeout> | undefined;
  notes.addEventListener("input", () => {
    if (notesSaveTimer) {
      clearTimeout(notesSaveTimer);
    }
    notesSaveTimer = setTimeout(() => {
      void invoke("set_epub_reading_notes", {
        path: viewer.path,
        notes: notes.value,
      });
    }, 400);
  });
  const exportNotesBtn = document.createElement("button");
  exportNotesBtn.type = "button";
  exportNotesBtn.textContent = t("epub.exportReadingNotes");
  exportNotesBtn.onclick = () => {
    void invoke<string>("export_epub_reading_notes", { path: viewer.path })
      .then((out) => showToast(t("epub.exportedReadingNotes", { path: out }), "info"))
      .catch((err) => showErrorToast(err));
  };

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
  root.appendChild(exportNotesBtn);
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
  layout.className = "svg-workspace";
  const main = document.createElement("div");
  main.className = "svg-workspace-main";
  const preview = document.createElement("div");
  preview.className = "svg-preview svg-preview--stage";
  const inspector = document.createElement("aside");
  inspector.className = "svg-inspector-rail";
  inspector.setAttribute("aria-label", "Object properties");
  const ta = document.createElement("textarea");
  ta.className = "svg-source hidden";
  ta.setAttribute("aria-hidden", "true");
  applySpellcheckToEditable(ta, true);
  ta.value = text;
  let timer: ReturnType<typeof setTimeout> | null = null;
  const syncRust = () => {
    if (timer) {
      clearTimeout(timer);
    }
    timer = setTimeout(() => {
      invoke("sync_svg_document", { content: ta.value }).catch(() => {});
    }, 300);
  };
  ta.oninput = () => syncRust();
  const modeBar = document.createElement("div");
  modeBar.className = "svg-mode-bar";
  const previewBtn = document.createElement("button");
  previewBtn.type = "button";
  previewBtn.className = "svg-mode-btn is-active";
  previewBtn.textContent = "Preview";
  const sourceBtn = document.createElement("button");
  sourceBtn.type = "button";
  sourceBtn.className = "svg-mode-btn";
  sourceBtn.textContent = "Source";
  const setMode = (mode: "preview" | "source") => {
    const previewMode = mode === "preview";
    preview.classList.toggle("hidden", !previewMode);
    inspector.classList.toggle("hidden", !previewMode);
    ta.classList.toggle("hidden", previewMode);
    previewBtn.classList.toggle("is-active", previewMode);
    sourceBtn.classList.toggle("is-active", !previewMode);
  };
  previewBtn.onclick = () => setMode("preview");
  sourceBtn.onclick = () => {
    setMode("source");
    ta.focus();
  };
  modeBar.append(previewBtn, sourceBtn);
  main.append(modeBar, preview, ta);
  layout.append(main, inspector);
  root.appendChild(layout);
  mountSvgInspector(preview, ta, syncRust, inspector);
  let previewZoom = 1;
  viewerZoomAdjust = (delta) => {
    previewZoom = Math.min(8, Math.max(0.2, previewZoom * (delta > 0 ? 1.15 : 1 / 1.15)));
    const svg = preview.querySelector<SVGSVGElement>("svg");
    if (svg) {
      svg.style.transform = `scale(${previewZoom})`;
      svg.style.transformOrigin = "center";
    }
    reportViewerZoom(previewZoom, "image");
  };
  reportViewerZoom(previewZoom, "image");
  setViewerLocatorBridge({
    svgTextOffset: (startByte, endByte) => {
      setMode("source");
      ta.focus();
      const len = ta.value.length;
      const start = Math.min(Math.max(0, startByte), len);
      const end = Math.min(Math.max(start, endByte), len);
      ta.setSelectionRange(start, end);
      const lineHeight = 18;
      ta.scrollTop = Math.max(0, Math.floor(start / 80) * lineHeight - 40);
    },
  });
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
  const muted = viewer.tab_muted ? 1 : 0;
  return `${viewer.viewer}:${viewer.path}:${scale}:${page}:${font}:${nodeCount}:${muted}`;
}

function applyTabMutedToViewer(el: HTMLElement, muted: boolean) {
  el.querySelectorAll<HTMLMediaElement>("video, audio").forEach((media) => {
    media.muted = muted;
  });
}

export interface DocumentViewerApplyExtras {
  downloadHistory?: DownloadHistoryEntryDto[];
  onRefreshState?: () => void | Promise<void>;
}

export interface ProjectionExtrasPayload {
  web_dom_text: string | null;
  ocr_text: string | null;
  epub_chapter_text: string | null;
  pdf_page_text: string | null;
  docx_preview_text: string | null;
}

export function gatherProjectionExtras(host: HTMLElement): ProjectionExtrasPayload {
  const docx_preview_text = documentViewer.getReadableText(host);
  const pdfLayer = host.querySelector(".pdf-text-layer");
  const pdf_page_text = pdfLayer?.textContent?.trim().slice(0, 12_000) || null;
  const epubFrame = host.querySelector<HTMLIFrameElement>(".epub-chapter-frame");
  const epubBody = epubFrame?.contentDocument?.body;
  const epub_chapter_text = epubBody?.innerText?.trim().slice(0, 12_000) || null;
  const hasSpecial =
    Boolean(docx_preview_text) || Boolean(pdf_page_text) || Boolean(epub_chapter_text);
  const web_dom_text = hasSpecial
    ? null
    : host.querySelector<HTMLElement>(".viewer-body")?.innerText?.trim().slice(0, 12_000) || null;
  return {
    web_dom_text,
    ocr_text: null,
    epub_chapter_text,
    pdf_page_text,
    docx_preview_text,
  };
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
      applyTabMutedToViewer(el, Boolean(viewer.tab_muted));
      return;
    }
    loadedKeys.set(el, key);
    el.classList.remove("hidden");
    el.innerHTML = "<p class='viewer-loading'>Loading…</p>";
    try {
      el.innerHTML = "";
      if (viewer.viewer !== "pdf") {
        const title = document.createElement("h2");
        title.className = "viewer-title";
        title.textContent = viewer.title;
        el.appendChild(title);
      }
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
          await renderVideo(
            viewer.path,
            body,
            viewer.media_note,
            Boolean(viewer.tab_muted),
          );
          break;
        case "audio":
          await renderAudio(
            viewer.path,
            body,
            viewer.media_note,
            Boolean(viewer.tab_muted),
          );
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
          resetWorkshopListeners();
          await renderWorkshop(viewer, body);
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
      applyTabMutedToViewer(el, Boolean(viewer.tab_muted));
    } catch (err) {
      el.innerHTML = `<p class='viewer-error'>${String(err)}</p>`;
    }
  },
  hide(el: HTMLElement) {
    viewerZoomAdjust = null;
    dxfViewCommand = null;
    setViewerFindRunner(null);
    clearViewerLocatorBridge();
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
  getReadableText(el: HTMLElement): string | null {
    const article = el.querySelector<HTMLElement>(".docx-view");
    if (!article) {
      return null;
    }
    const text = article.innerText.trim();
    return text || null;
  },
  getPrintSelectionHtml(el: HTMLElement): { title: string; html: string } | null {
    const pdfLayer = el.querySelector(".pdf-text-layer");
    if (pdfLayer) {
      const sel = window.getSelection();
      const text = sel?.toString().trim() ?? "";
      if (!text || !sel?.anchorNode || !pdfLayer.contains(sel.anchorNode)) {
        return null;
      }
      const titleEl = el.querySelector(".viewer-title");
      const title = titleEl?.textContent?.trim() || "PDF";
      const escaped = text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
      return { title: `${title} (selection)`, html: `<pre class="pdf-selection">${escaped}</pre>` };
    }
    const article = el.querySelector(".docx-view");
    if (!article) {
      return null;
    }
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || !sel.toString().trim()) {
      return null;
    }
    const anchor = sel.anchorNode;
    if (!anchor || !article.contains(anchor)) {
      return null;
    }
    const range = sel.getRangeAt(0);
    const holder = document.createElement("div");
    holder.appendChild(range.cloneContents());
    const titleEl = el.querySelector(".viewer-title");
    const title = titleEl?.textContent?.trim() || "document";
    return { title: `${title} (selection)`, html: holder.innerHTML };
  },
};
