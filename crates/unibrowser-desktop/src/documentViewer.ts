import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { loadPdfJs, loadThree } from "./lazyModules";
import { appendTrustBanner } from "./trustBanner";
import { bindMediaViewerKeys } from "./mediaKeyboard";
import { renderDownloadsManager, type DownloadHistoryEntryDto } from "./downloadsPage";
import { reportViewerPdfPage, reportViewerZoom } from "./viewerZoom";
import { bindPreviewLinks } from "./previewLinks";
import { copyText } from "./clipboard";
import { renderSettings } from "./settingsPage";
import { renderAbout } from "./aboutPage";
import { renderDocs } from "./docsPage";
import { mountSvgInspector } from "./svgInspector";
import { renderKeys } from "./keysPage";
import { renderDiagnostics } from "./diagnosticsPage";
import { invokeWithToast, showErrorToast, showToast } from "./toast";
import { t } from "./i18n";
import { resetWorkshopListeners } from "./workshop";
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
import {
  normalisedPdfRectToCss,
  pdfHighlightOverlayPointerEvents,
} from "./pdfAnnotationGeometry";

let viewerZoomAdjust: ((delta: number) => void) | null = null;
let imageViewCommand: ((commandId: string) => boolean) | null = null;
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
  if (commandId.startsWith("unibrowser.image.")) {
    return imageViewCommand?.(commandId) ?? false;
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
  const buckets = Math.ceil(width / 3);
  const block = Math.max(1, Math.floor(channel.length / buckets));
  g.fillStyle = "#66ccff";
  for (let bucket = 0; bucket < buckets; bucket += 1) {
    let peak = 0;
    const start = bucket * block;
    for (let i = 0; i < block && start + i < channel.length; i += 1) {
      peak = Math.max(peak, Math.abs(channel[start + i]));
    }
    const bar = Math.max(1, peak * height * 0.9);
    g.fillRect(bucket * 3, mid - bar * 0.5, 2, bar);
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
  selectedIndex: number,
  onSelect: (index: number) => void,
  onJump: (page: number) => void,
) {
  annotDock.innerHTML = "";
  const head = document.createElement("header");
  head.className = "pdf-annotations-head";
  const annotTitle = document.createElement("h3");
  annotTitle.className = "pdf-annotations-title";
  annotTitle.textContent = `Annotations (${highlights.length})`;
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
    const safeIndex =
      selectedIndex >= 0 && selectedIndex < highlights.length ? selectedIndex : 0;
    const selected = highlights[safeIndex];
    const inspector = document.createElement("section");
    inspector.className = "pdf-annotation-inspector pdf-annotation-card pdf-annotation-card--selected";
    inspector.setAttribute("aria-label", "Selected annotation");
    const toolRow = document.createElement("div");
    toolRow.className = "pdf-annotation-card-head";
    const identity = document.createElement("span");
    identity.className = "pdf-annotation-identity";
    const icon = document.createElement("span");
    icon.className = `pdf-annotation-icon pdf-annotation-icon--${selected.kind ?? "highlight"}`;
    icon.setAttribute("aria-hidden", "true");
    icon.textContent = selected.kind === "note" ? "N" : selected.kind === "area" ? "A" : "H";
    const kind = document.createElement("span");
    kind.className = "pdf-annotation-kind";
    kind.textContent = pdfAnnotationKindLabel(selected.kind);
    identity.append(icon, kind);
    const pageLink = document.createElement("button");
    pageLink.type = "button";
    pageLink.className = "pdf-annotation-page";
    pageLink.textContent = `Page ${selected.page}`;
    pageLink.onclick = () => onJump(selected.page);
    toolRow.append(identity, pageLink);
    const title = document.createElement("label");
    title.className = "pdf-annotation-field";
    const titleCaption = document.createElement("span");
    titleCaption.textContent = "Label";
    const titleInput = document.createElement("input");
    titleInput.type = "text";
    titleInput.className = "ub-form-control";
    titleInput.value = selected.label ?? `Annotation ${safeIndex + 1}`;
    titleInput.setAttribute("aria-label", "Annotation label");
    title.append(titleCaption, titleInput);
    const meta = document.createElement("p");
    meta.className = "pdf-annotation-meta";
    const who = selected.author ?? "Unknown";
    const when = formatPdfAnnotationTime(selected.created_at_ms);
    meta.textContent = when ? `${who} · ${when}` : who;
    const colourField = document.createElement("label");
    colourField.className = "pdf-annotation-field";
    const colourCaption = document.createElement("span");
    colourCaption.textContent = "Colour";
    const colour = document.createElement("input");
    colour.type = "color";
    colour.className = "ub-form-control";
    colour.value = selected.kind === "area" ? "#c0392b" : "#ffe600";
    colour.setAttribute("aria-label", "Annotation colour");
    colourField.append(colourCaption, colour);
    const reply = document.createElement("textarea");
    reply.className = "pdf-annotation-reply ub-form-control";
    reply.rows = 3;
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
    actions.append(resolveBtn, replyBtn);
    inspector.append(toolRow, title, meta, colourField, reply, actions);
    annotBody.appendChild(inspector);

    if (highlights.length > 1) {
      const indexTitle = document.createElement("h4");
      indexTitle.className = "pdf-annotations-index-title";
      indexTitle.textContent = "Other annotations";
      annotBody.appendChild(indexTitle);
      const list = document.createElement("ul");
      list.className = "pdf-annotations-index";
      highlights.forEach((h, index) => {
        if (index === safeIndex) {
          return;
        }
        const li = document.createElement("li");
        const pick = document.createElement("button");
        pick.type = "button";
        pick.className = "pdf-annotation-index-row";
        pick.textContent = `${pdfAnnotationKindLabel(h.kind)} · ${h.label ?? `Annotation ${index + 1}`}`;
        pick.onclick = () => onSelect(index);
        li.appendChild(pick);
        list.appendChild(li);
      });
      annotBody.appendChild(list);
    }
  }
  annotDock.appendChild(annotBody);
}

let pdfHighlightArmed = false;

export function applyPdfHighlightArmedToOverlays(
  root: ParentNode = document,
  armed = true,
): void {
  const events = pdfHighlightOverlayPointerEvents(armed);
  root.querySelectorAll<HTMLElement>(".pdf-page-overlay").forEach((el) => {
    el.style.pointerEvents = events;
  });
}

export function armPdfHighlightMode() {
  pdfHighlightArmed = true;
  document.querySelectorAll(".pdf-workspace").forEach((el) => {
    el.classList.add("pdf-highlight-armed");
  });
  applyPdfHighlightArmedToOverlays(document, true);
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
  let selectedHighlightIndex = 0;

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
    rebuildPdfAnnotationsDock(
      annotDock,
      highlights,
      selectedHighlightIndex,
      (index) => {
        selectedHighlightIndex = index;
        void drawPage(currentPage);
        syncAnnotationDock();
      },
      (p) => {
        void goToPage(p);
      },
    );
    reportViewerPdfPage(currentPage, doc.numPages, highlights.length);
  };

  const paintHighlights = (pageNum: number, host: HTMLElement, width: number, height: number) => {
    host.querySelectorAll(".pdf-highlight-rect").forEach((el) => el.remove());
    highlights.forEach((h, index) => {
      if (h.page !== pageNum) {
        return;
      }
      const box = document.createElement("div");
      const isArea = h.kind === "area";
      box.className = isArea ? "pdf-highlight-rect pdf-area-rect" : "pdf-highlight-rect";
      if (index === selectedHighlightIndex) {
        box.classList.add("pdf-highlight-rect--selected");
        box.setAttribute("aria-current", "true");
      }
      box.tabIndex = 0;
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
      box.onclick = () => {
        selectedHighlightIndex = index;
        syncAnnotationDock();
        paintHighlights(pageNum, host, width, height);
      };
      host.appendChild(box);
    });
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
    overlay.style.pointerEvents = pdfHighlightOverlayPointerEvents(pdfHighlightArmed);
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
        overlay.style.pointerEvents = pdfHighlightOverlayPointerEvents(false);
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
  if (scale === 1.25) {
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

async function ensureAssetAllowed(path: string) {
  await invoke("allow_asset_path", { path }).catch(() => {});
}

function formatMediaTime(value: number): string {
  if (!Number.isFinite(value) || value < 0) {
    return "0:00";
  }
  if (value > 0 && value < 1) {
    return `${value.toFixed(1)}s`;
  }
  const minutes = Math.floor(value / 60);
  const seconds = Math.floor(value % 60);
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

function mountMediaControls(host: HTMLElement, media: HTMLMediaElement): void {
  const controls = document.createElement("div");
  controls.className = "media-controls";
  const play = document.createElement("button");
  play.type = "button";
  play.className = "media-play-button";
  play.textContent = "Play";
  play.setAttribute("aria-label", "Play media");
  const seek = document.createElement("input");
  seek.type = "range";
  seek.className = "media-seek";
  seek.min = "0";
  seek.max = "1000";
  seek.value = "0";
  seek.disabled = true;
  seek.setAttribute("aria-label", "Media position");
  const time = document.createElement("span");
  time.className = "media-time";
  time.textContent = "0:00 / 0:00";

  const syncTime = () => {
    const duration = Number.isFinite(media.duration) ? media.duration : 0;
    const progress = duration > 0 ? media.currentTime / duration : 0;
    seek.value = String(Math.round(progress * 1000));
    seek.disabled = duration <= 0;
    const durationLabel = duration > 0 ? formatMediaTime(duration) : "--:--";
    time.textContent = `${formatMediaTime(media.currentTime)} / ${durationLabel}`;
  };
  const syncPlay = () => {
    const playing = !media.paused && !media.ended;
    play.textContent = playing ? "Pause" : "Play";
    play.setAttribute("aria-label", playing ? "Pause media" : "Play media");
  };

  play.addEventListener("click", () => {
    if (media.paused || media.ended) {
      void media.play().catch(() => {});
      return;
    }
    media.pause();
  });
  seek.addEventListener("input", () => {
    if (!Number.isFinite(media.duration) || media.duration <= 0) {
      return;
    }
    media.currentTime = (Number.parseInt(seek.value, 10) / 1000) * media.duration;
    syncTime();
  });
  media.addEventListener("loadedmetadata", syncTime);
  media.addEventListener("durationchange", syncTime);
  media.addEventListener("timeupdate", syncTime);
  media.addEventListener("play", syncPlay);
  media.addEventListener("pause", syncPlay);
  media.addEventListener("ended", syncPlay);
  controls.append(play, seek, time);
  host.appendChild(controls);
  syncTime();
  syncPlay();
}

async function renderVideo(
  path: string,
  root: HTMLElement,
  _note?: string,
  tabMuted = false,
) {
  await ensureAssetAllowed(path);
  root.innerHTML = "";
  root.classList.add("media-viewer", "media-viewer--video");
  const stage = document.createElement("div");
  stage.className = "media-stage";
  const video = document.createElement("video");
  video.className = "media-view";
  video.controls = false;
  video.preload = "metadata";
  video.muted = tabMuted;
  video.src = convertFileSrc(path);
  stage.appendChild(video);
  const prompt = document.createElement("p");
  prompt.className = "media-video-prompt";
  prompt.textContent = "Press Play to watch";
  stage.appendChild(prompt);
  root.appendChild(stage);
  mountMediaControls(root, video);
  bindMediaViewerKeys(video);

  let posterReady = false;
  try {
    const frame = await invoke<{ path: string; mime: string }>("video_scrub_thumbnail", {
      path,
      timeSec: 0,
    });
    await ensureAssetAllowed(frame.path);
    const poster = document.createElement("img");
    poster.className = "media-poster";
    poster.alt = "";
    poster.src = convertFileSrc(frame.path);
    stage.appendChild(poster);
    posterReady = await new Promise<boolean>((resolve) => {
      if (poster.complete && poster.naturalWidth > 0) {
        resolve(true);
        return;
      }
      poster.addEventListener("load", () => resolve(true), { once: true });
      poster.addEventListener("error", () => resolve(false), { once: true });
      window.setTimeout(() => resolve(false), 3000);
    });
    if (posterReady) {
      prompt.remove();
    }
    video.addEventListener("play", () => {
      poster.classList.add("hidden");
      prompt.classList.add("hidden");
    });
    video.addEventListener("ended", () => poster.classList.remove("hidden"));
  } catch {
    posterReady = false;
  }
  if (!posterReady) {
    await new Promise<void>((resolve) => {
      if (video.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA) {
        resolve();
        return;
      }
      video.addEventListener("loadeddata", () => resolve(), { once: true });
      window.setTimeout(resolve, 3000);
    });
  }
  video.addEventListener("play", () => prompt.classList.add("hidden"));
  root.dataset.mediaReady = "1";
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
  const buckets = Math.ceil(width / 3);
  const step = peaks.length / buckets;
  for (let bucket = 0; bucket < buckets; bucket += 1) {
    const i = Math.min(peaks.length - 1, Math.floor(bucket * step));
    const h = Math.max(2, peaks[i] * (height * 0.9));
    g.fillRect(bucket * 3, mid - h / 2, 2, h);
  }
}

async function renderAudio(
  path: string,
  root: HTMLElement,
  _note?: string,
  tabMuted = false,
) {
  await ensureAssetAllowed(path);
  root.innerHTML = "";
  root.classList.add("media-viewer", "media-viewer--audio");

  const audio = document.createElement("audio");
  audio.className = "media-view";
  audio.controls = false;
  audio.preload = "metadata";
  audio.muted = tabMuted;
  audio.src = convertFileSrc(path);
  root.appendChild(audio);
  bindMediaViewerKeys(audio);
  const canvas = document.createElement("canvas");
  canvas.className = "audio-waveform";
  canvas.width = 640;
  canvas.height = 160;
  root.appendChild(canvas);
  mountMediaControls(root, audio);
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
  } catch {
    try {
      const peaks = await invoke<number[]>("audio_waveform_peaks", {
        path,
        maxPoints: canvas.width,
      });
      const g = canvas.getContext("2d");
      if (g && peaks.length) {
        drawAudioPeakArray(g, peaks, canvas.width, canvas.height);
      }
    } catch {
      const g = canvas.getContext("2d");
      if (g) {
        drawAudioStubBars(g, canvas.width, canvas.height);
      }
    }
  }
  root.dataset.mediaReady = "1";
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

async function renderImage(path: string, root: HTMLElement, _initialZoom = 1) {
  const { mountRasterImageViewer } = await import("./imageRasterViewer");
  imageViewCommand = await mountRasterImageViewer(path, root);
  viewerZoomAdjust = (delta) => {
    void imageViewCommand?.(
      delta > 0 ? "unibrowser.image.zoomIn" : "unibrowser.image.zoomOut",
    );
  };
}

async function renderDxf(path: string, root: HTMLElement) {
  root.innerHTML = "";
  dxfViewCommand = null;
  const { setStatusHint } = await import("./statusBar");
  const setDxfStatusHint = (entityCount: number) => {
    setStatusHint(
      `CAD proving mod: wireframe preview. ${entityCount} entities — toggle layers, drag pan, wheel zoom.`,
    );
  };
  setDxfStatusHint(0);

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
  const geomTable = document.createElement("table");
  geomTable.className = "dxf-geom-table inspector-kv-table";
  const geomHead = document.createElement("thead");
  const geomHeadRow = document.createElement("tr");
  for (const label of ["Entity", "Count"]) {
    const th = document.createElement("th");
    th.scope = "col";
    th.textContent = label;
    geomHeadRow.appendChild(th);
  }
  geomHead.appendChild(geomHeadRow);
  geomTable.appendChild(geomHead);
  const geomBody = document.createElement("tbody");
  const counts = new Map<string, number>();
  for (const ent of entities) {
    const kind = ent.type ?? "UNKNOWN";
    counts.set(kind, (counts.get(kind) ?? 0) + 1);
  }
  const summary = Array.from(counts.entries()).sort((a, b) => a[0].localeCompare(b[0]));
  if (summary.length === 0) {
    const row = document.createElement("tr");
    const cell = document.createElement("td");
    cell.colSpan = 2;
    cell.textContent = "No entities parsed";
    row.appendChild(cell);
    geomBody.appendChild(row);
  } else {
    for (const [kind, count] of summary.slice(0, 40)) {
      const row = document.createElement("tr");
      const entityCell = document.createElement("td");
      entityCell.textContent = kind;
      const countCell = document.createElement("td");
      countCell.textContent = String(count);
      row.append(entityCell, countCell);
      geomBody.appendChild(row);
    }
  }
  geomTable.appendChild(geomBody);
  geomPanel.appendChild(geomTable);

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
  let rebuildScene: (() => void) | null = null;
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
    li.className = "dxf-layer-row";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.className = "ub-shell-checkbox";
    cb.checked = true;
    cb.id = `dxf-layer-${layer}`;
    const label = document.createElement("label");
    label.className = "dxf-layer-label";
    label.htmlFor = cb.id;
    label.textContent = layer;
    cb.onchange = () => {
      layerVisible.set(layer, cb.checked);
      rebuildScene?.();
    };
    li.append(cb, label);
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
  let viewW = Math.max(1, wrap.clientWidth || 800);
  let viewH = Math.max(1, wrap.clientHeight || 400);

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
    svg.setAttribute("width", "100%");
    svg.setAttribute("height", "100%");
    svg.setAttribute("preserveAspectRatio", "xMidYMid meet");
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
    rebuildScene = () => {
      /* SVG proxy has no live rebuild */
    };
    registerDxfLocator();
    return;
  }

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x1a1a22);
  const camera = new THREE.OrthographicCamera(-viewW / 2, viewW / 2, viewH / 2, -viewH / 2, 0.1, 1000);
  camera.position.z = 10;
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(viewW, viewH);
  wrap.appendChild(renderer.domElement);
  let panX = 0;
  let panY = 0;
  let zoom = 1;
  let baseBbox: { minX: number; minY: number; maxX: number; maxY: number } | null = null;

  const reportDxfZoom = () => {
    if (!baseBbox) {
      reportViewerZoom(zoom, "cad");
      return;
    }
    const worldW = Math.max(baseBbox.maxX - baseBbox.minX, 1);
    const worldH = Math.max(baseBbox.maxY - baseBbox.minY, 1);
    const pad = 1.12;
    const displayScale = Math.min(viewW / (worldW * pad), viewH / (worldH * pad)) * zoom;
    const clamped = Math.min(8, Math.max(0.02, displayScale));
    document.documentElement.dataset.dxfFitZoomReady = "1";
    document.documentElement.dataset.dxfFitZoomPct = String(Math.round(clamped * 100));
    reportViewerZoom(clamped, "cad");
  };

  const applyFitView = () => {
    panX = 0;
    panY = 0;
    zoom = 1;
    rebuild();
  };

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
      const aspect = viewW / viewH;
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
    reportDxfZoom();
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
    const dx = (e.clientX - lastX) * (camera.right - camera.left) / viewW;
    const dy = (e.clientY - lastY) * (camera.top - camera.bottom) / viewH;
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

  const resizeObserver = new ResizeObserver(() => {
    viewW = Math.max(1, wrap.clientWidth);
    viewH = Math.max(1, wrap.clientHeight);
    renderer.setSize(viewW, viewH);
    if (panX === 0 && panY === 0 && zoom === 1) {
      rebuild();
    } else {
      rebuild();
    }
  });
  resizeObserver.observe(wrap);
  rebuildScene = rebuild;

  setDxfStatusHint(entities.length);
  requestAnimationFrame(() => applyFitView());
  dxfViewCommand = (commandId) => {
    if (commandId === "unibrowser.cad.fitView") {
      applyFitView();
      return true;
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
      const hideTitleForProject = viewer.viewer === "unibrow";
      if (viewer.viewer !== "pdf" && !hideTitleForProject) {
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
        case "unibrow": {
          resetWorkshopListeners();
          const { renderProjectSurface, classifyProjectSurface } = await import("./projectSurface");
          const surface = classifyProjectSurface(viewer);
          if (surface === "document") {
            body.replaceChildren();
            body.className = "viewer-body project-document-host";
            break;
          }
          await renderProjectSurface(viewer, body);
          break;
        }
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
    imageViewCommand = null;
    void import("./statusBar").then(({ setStatusHint }) => setStatusHint(""));
    dxfViewCommand = null;
    pdfViewCommand = null;
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
