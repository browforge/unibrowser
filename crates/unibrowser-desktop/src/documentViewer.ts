import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import * as mammoth from "mammoth";
import { loadPdfJs, loadThree } from "./lazyModules";
import { reportViewerZoom } from "./viewerZoom";
import { bindPreviewLinks } from "./previewLinks";
import { copyText } from "./clipboard";
import { renderSettings } from "./settingsPage";
import { renderAbout } from "./aboutPage";
import { renderKeys } from "./keysPage";
import { emptyStateBlock } from "./emptyState";
import { pathsFromDataTransfer } from "./filesDrag";
import { showToast } from "./toast";

let viewerZoomAdjust: ((delta: number) => void) | null = null;

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
  midi_summary?: MidiSummaryDto;
  board_snap_to_grid?: boolean;
  midi_note_preview?: MidiNoteOnDto[];
}

export interface MidiNoteOnDto {
  track_index: number;
  tick: number;
  note: number;
  velocity: number;
}

export interface ProjectClipDto {
  id: string;
  node_id: string;
  node_type: string;
  start_sec: number;
  duration_sec: number;
  label: string;
}

function clipTypeAbbrev(nodeType: string): string {
  const t = nodeType.toLowerCase();
  if (t.includes("video")) {
    return "VID";
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

async function renderDocx(path: string, root: HTMLElement) {
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
  printBtn.textContent = "Print";
  printBtn.onclick = () => {
    const w = window.open("", "_blank", "noopener");
    if (!w) {
      return;
    }
    w.document.write(
      `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${path}</title></head><body>${result.value}</body></html>`,
    );
    w.document.close();
    w.focus();
    w.print();
  };
  const exportBtn = document.createElement("button");
  exportBtn.type = "button";
  exportBtn.textContent = "Export HTML";
  exportBtn.onclick = () => {
    const blob = new Blob(
      [
        `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${path}</title></head><body>${result.value}</body></html>`,
      ],
      { type: "text/html;charset=utf-8" },
    );
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${path.split(/[/\\]/).pop()?.replace(/\.docx$/i, "") ?? "document"}.html`;
    a.click();
    URL.revokeObjectURL(url);
  };
  toolbar.appendChild(printBtn);
  toolbar.appendChild(exportBtn);

  const article = document.createElement("article");
  article.className = "docx-view";
  article.innerHTML = result.value;

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

async function renderPdf(path: string, root: HTMLElement, scale: number, startPage: number) {
  const pdfjsLib = await loadPdfJs();
  const bytes = await readBytes(path);
  const doc = await pdfjsLib.getDocument({ data: bytes }).promise;
  let currentPage = Math.min(Math.max(startPage, 1), doc.numPages);
  let currentScale = scale;
  root.innerHTML = "";
  const wrap = document.createElement("div");
  wrap.className = "pdf-page-wrap";
  root.appendChild(wrap);
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
  nav.appendChild(copyBtn);
  nav.appendChild(printBtn);
  root.appendChild(nav);

  const drawPage = async (pageNum: number) => {
    wrap.innerHTML = "";
    const page = await doc.getPage(pageNum);
    const viewport = page.getViewport({ scale: currentScale });
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
  root.appendChild(video);
  appendMediaNote(root, note);
}

async function renderAudio(path: string, root: HTMLElement, note?: string) {
  await ensureAssetAllowed(path);
  root.innerHTML = "";
  const audio = document.createElement("audio");
  audio.className = "media-view";
  audio.controls = true;
  audio.src = convertFileSrc(path);
  root.appendChild(audio);
  const canvas = document.createElement("canvas");
  canvas.className = "audio-waveform";
  canvas.width = 640;
  canvas.height = 48;
  root.appendChild(canvas);
  try {
    const bytes = await readBytes(path);
    const ctx = new AudioContext();
    const buf = await ctx.decodeAudioData(toArrayBuffer(bytes));
    const ch = buf.getChannelData(0);
    const g = canvas.getContext("2d");
    if (g) {
      g.fillStyle = "#1a1a22";
      g.fillRect(0, 0, canvas.width, canvas.height);
      g.strokeStyle = "#66ccff";
      g.beginPath();
      const step = Math.max(1, Math.floor(ch.length / canvas.width));
      for (let x = 0; x < canvas.width; x += 1) {
        const i = x * step;
        const y = (1 - ch[i]) * canvas.height * 0.5;
        if (x === 0) {
          g.moveTo(x, y);
        } else {
          g.lineTo(x, y);
        }
      }
      g.stroke();
    }
    await ctx.close();
  } catch {
    appendMediaNote(root, "Waveform preview unavailable; playback still works.");
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
  const w = window.open("", "_blank", "noopener");
  if (!w) {
    return;
  }
  w.document.write(
    `<!DOCTYPE html><html><head><meta charset="utf-8"><title>${title}</title></head><body>${bodyHtml}</body></html>`,
  );
  w.document.close();
  w.focus();
  w.print();
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

async function renderImage(path: string, root: HTMLElement) {
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
  rotateBtn.textContent = "Rotate 90°";
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
  toolbar.appendChild(zoomOut);
  toolbar.appendChild(zoomIn);
  toolbar.appendChild(resetBtn);
  toolbar.appendChild(rotateBtn);
  toolbar.appendChild(cropBtn);
  toolbar.appendChild(clearCropBtn);
  toolbar.appendChild(saveCropBtn);
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

  let scale = 1;
  let rotation = 0;
  let panX = 0;
  let panY = 0;
  let cropMode = false;
  let cropDragging = false;
  let cropStartX = 0;
  let cropStartY = 0;
  let cropRectNorm: { x: number; y: number; w: number; h: number } | null = null;

  const applyTransform = () => {
    stage.style.transform = `translate(${panX}px, ${panY}px) scale(${scale}) rotate(${rotation}deg)`;
  };

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

  saveCropBtn.onclick = async () => {
    if (!cropRectNorm || cropRectNorm.w < 0.5 || cropRectNorm.h < 0.5) {
      return;
    }
    await img.decode();
    const nw = img.naturalWidth;
    const nh = img.naturalHeight;
    const sx = Math.max(0, Math.round((cropRectNorm.x / 100) * nw));
    const sy = Math.max(0, Math.round((cropRectNorm.y / 100) * nh));
    const sw = Math.min(nw - sx, Math.round((cropRectNorm.w / 100) * nw));
    const sh = Math.min(nh - sy, Math.round((cropRectNorm.h / 100) * nh));
    if (sw < 1 || sh < 1) {
      return;
    }
    const canvas = document.createElement("canvas");
    canvas.width = sw;
    canvas.height = sh;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      return;
    }
    ctx.drawImage(img, sx, sy, sw, sh, 0, 0, sw, sh);
    const blob = await new Promise<Blob | null>((resolve) => {
      canvas.toBlob((b) => resolve(b), "image/png");
    });
    if (!blob) {
      return;
    }
    const bytes = Array.from(new Uint8Array(await blob.arrayBuffer()));
    const base = path.split(/[/\\]/).pop() ?? "image.png";
    const suggested = `${base.replace(/\.[^.]+$/i, "")}-crop.png`;
    const dest = await invoke<string | null>("pick_save_image_path", { suggestedName: suggested });
    if (!dest) {
      return;
    }
    await invoke("write_document_bytes", { path: dest, bytes });
    const fileUrl = dest.startsWith("file://") ? dest : `file://${dest}`;
    window.dispatchEvent(new CustomEvent("unibrowser-navigate", { detail: fileUrl }));
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
  note.textContent =
    "Pan: drag (when Crop is off) · Crop draws a non-destructive overlay only · file on disk is unchanged.";
  root.appendChild(note);
}

async function renderDxf(path: string, root: HTMLElement) {
  const THREE = await loadThree();
  const DxfParser = (await import("dxf-parser")).default;
  const text = new TextDecoder().decode(await readBytes(path));
  const parser = new DxfParser();
  const dxf = parser.parseSync(text) as DxfData | null;
  const entities = dxf?.entities ?? [];
  const layers = Array.from(
    new Set(entities.map((e) => e.layer).filter((l): l is string => Boolean(l))),
  ).sort();
  const layerVisible = new Map<string, boolean>();
  for (const layer of layers) {
    layerVisible.set(layer, true);
  }
  root.innerHTML = "";
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
  root.appendChild(layout);
  const wrap = document.createElement("div");
  wrap.className = "dxf-canvas-wrap";
  canvasCol.appendChild(wrap);
  const width = wrap.clientWidth || 800;
  const height = 400;
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
    const allSegs: Vec3[] = [];
    for (const ent of entities) {
      const layer = ent.layer ?? "";
      if (layer && layerVisible.has(layer) && !layerVisible.get(layer)) {
        continue;
      }
      allSegs.push(...entitySegments(ent));
    }
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
    card.textContent = node.label;
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
  const timeline = document.createElement("section");
  timeline.className = "unibrow-timeline";
  const tlTitle = document.createElement("h3");
  tlTitle.textContent = "Timeline";
  timeline.appendChild(tlTitle);
  const timelineAddRow = document.createElement("div");
  timelineAddRow.className = "unibrow-timeline-add";
  const nodeSelect = document.createElement("select");
  const boardNodes = viewer.project_nodes ?? [];
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
  timelineAddRow.appendChild(nodeSelect);
  timelineAddRow.appendChild(addBtn);
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
    label.textContent = `${clip.start_sec.toFixed(1)}s · ${clip.duration_sec.toFixed(1)}s — ${clip.label}`;
    li.appendChild(label);
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

async function renderEpub(viewer: DocumentViewerDto, root: HTMLElement) {
  root.innerHTML = "";
  let chapterIndex = 0;

  const nav = document.createElement("div");
  nav.className = "epub-nav";
  const prev = document.createElement("button");
  prev.type = "button";
  prev.textContent = "Previous";
  const next = document.createElement("button");
  next.type = "button";
  next.textContent = "Next";
  const label = document.createElement("span");
  label.className = "epub-chapter-label";

  const frame = document.createElement("iframe");
  frame.className = "epub-chapter-frame";
  frame.title = "EPUB chapter";
  frame.sandbox.add("allow-same-origin");

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
    prev.disabled = ch.index <= 0;
    next.disabled = ch.index + 1 >= ch.total;
    frame.srcdoc = ch.html;
  }

  prev.onclick = () => loadChapter(chapterIndex - 1).catch(showEpubError);
  next.onclick = () => loadChapter(chapterIndex + 1).catch(showEpubError);

  function showEpubError(err: unknown) {
    label.textContent = String(err);
  }

  const printBtn = document.createElement("button");
  printBtn.type = "button";
  printBtn.textContent = "Print chapter";
  printBtn.onclick = () => {
    const html = frame.srcdoc || "";
    printHtmlDocument(viewer.path, html);
  };
  nav.appendChild(prev);
  nav.appendChild(label);
  nav.appendChild(next);
  nav.appendChild(printBtn);
  root.appendChild(nav);
  root.appendChild(frame);
  root.appendChild(note);
  await loadChapter(0);
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
  preview.appendChild(frame);
  const ta = document.createElement("textarea");
  ta.className = "svg-source";
  ta.spellcheck = false;
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
  return `${viewer.viewer}:${viewer.path}:${scale}:${page}:${nodeCount}`;
}

export const documentViewer = {
  async apply(viewer: DocumentViewerDto | null, el: HTMLElement) {
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
          await renderDocx(viewer.path, body);
          break;
        case "pdf":
          await renderPdf(viewer.path, body, viewer.pdf_scale ?? 1.25, viewer.pdf_page ?? 1);
          break;
        case "video":
          await renderVideo(viewer.path, body, viewer.media_note);
          break;
        case "audio":
          await renderAudio(viewer.path, body, viewer.media_note);
          break;
        case "image":
          await renderImage(viewer.path, body);
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
        default:
          body.textContent = `Unknown viewer: ${viewer.viewer}`;
      }
    } catch (err) {
      el.innerHTML = `<p class='viewer-error'>${String(err)}</p>`;
    }
  },
  hide(el: HTMLElement) {
    viewerZoomAdjust = null;
    el.classList.add("hidden");
  },
};
