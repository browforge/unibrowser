import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import * as mammoth from "mammoth";
import * as pdfjsLib from "pdfjs-dist";
import DxfParser from "dxf-parser";
import * as THREE from "three";

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url,
).toString();

export interface ProjectNodeDto {
  id: string;
  node_type: string;
  label: string;
  asset: string | null;
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
}

let loadedKey: string | null = null;

async function readBytes(path: string): Promise<Uint8Array> {
  const data = await invoke<number[]>("read_document_bytes", { path });
  return Uint8Array.from(data);
}

function toArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
}

async function renderDocx(path: string, root: HTMLElement) {
  const bytes = await readBytes(path);
  const result = await mammoth.convertToHtml({ arrayBuffer: toArrayBuffer(bytes) });
  root.innerHTML = `<article class="docx-view">${result.value}</article>`;
  if (result.messages.length) {
    const note = document.createElement("p");
    note.className = "viewer-note";
    note.textContent = `mammoth: ${result.messages.length} message(s)`;
    root.appendChild(note);
  }
}

async function syncPdfSelection(selected: boolean) {
  await invoke("sync_pdf_viewer_state", { hasTextSelection: selected }).catch(() => {});
}

async function renderPdf(path: string, root: HTMLElement, scale: number, startPage: number) {
  const bytes = await readBytes(path);
  const doc = await pdfjsLib.getDocument({ data: bytes }).promise;
  let currentPage = Math.min(Math.max(startPage, 1), doc.numPages);
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
  nav.appendChild(prev);
  nav.appendChild(pageLabel);
  nav.appendChild(next);
  root.appendChild(nav);

  const drawPage = async (pageNum: number) => {
    wrap.innerHTML = "";
    const page = await doc.getPage(pageNum);
    const viewport = page.getViewport({ scale });
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

  await drawPage(currentPage);
  const meta = document.createElement("p");
  meta.className = "viewer-note";
  meta.textContent = `Select text for font-size commands · scale ${scale.toFixed(2)}`;
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

async function renderVideo(path: string, root: HTMLElement, note?: string) {
  root.innerHTML = "";
  const video = document.createElement("video");
  video.className = "media-view";
  video.controls = true;
  video.src = convertFileSrc(path);
  root.appendChild(video);
  appendMediaNote(root, note);
}

async function renderAudio(path: string, root: HTMLElement, note?: string) {
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
  vertices?: DxfVertex[];
  center?: DxfVertex;
  radius?: number;
  startAngle?: number;
  endAngle?: number;
};

type DxfData = { entities?: DxfEntity[] };

function pushPoint(points: THREE.Vector3[], x: number, y: number) {
  points.push(new THREE.Vector3(x, y, 0));
}

function arcPoints(
  center: DxfVertex,
  radius: number,
  startDeg: number,
  endDeg: number,
  segments = 32,
): THREE.Vector3[] {
  const pts: THREE.Vector3[] = [];
  let start = (startDeg * Math.PI) / 180;
  let end = (endDeg * Math.PI) / 180;
  if (end < start) {
    end += Math.PI * 2;
  }
  for (let i = 0; i <= segments; i += 1) {
    const t = start + ((end - start) * i) / segments;
    pts.push(new THREE.Vector3(center.x + radius * Math.cos(t), center.y + radius * Math.sin(t), 0));
  }
  return pts;
}

function entitySegments(ent: DxfEntity): THREE.Vector3[] {
  const segs: THREE.Vector3[] = [];
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
  }
  return segs;
}

function bboxFromSegments(segs: THREE.Vector3[]) {
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

function fitOrthographicCamera(
  camera: THREE.OrthographicCamera,
  bbox: { minX: number; minY: number; maxX: number; maxY: number },
  width: number,
  height: number,
) {
  const cx = (bbox.minX + bbox.maxX) / 2;
  const cy = (bbox.minY + bbox.maxY) / 2;
  const worldW = Math.max(bbox.maxX - bbox.minX, 1);
  const worldH = Math.max(bbox.maxY - bbox.minY, 1);
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

async function renderDxf(path: string, root: HTMLElement) {
  const text = new TextDecoder().decode(await readBytes(path));
  const parser = new DxfParser();
  const dxf = parser.parseSync(text) as DxfData | null;
  root.innerHTML = "";
  const wrap = document.createElement("div");
  wrap.className = "dxf-canvas-wrap";
  root.appendChild(wrap);
  const width = wrap.clientWidth || 800;
  const height = 400;
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x1a1a22);
  const camera = new THREE.OrthographicCamera(-width / 2, width / 2, height / 2, -height / 2, 0.1, 1000);
  camera.position.z = 10;
  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(width, height);
  wrap.appendChild(renderer.domElement);
  const allSegs: THREE.Vector3[] = [];
  for (const ent of dxf?.entities ?? []) {
    allSegs.push(...entitySegments(ent));
  }
  if (allSegs.length >= 2) {
    const geom = new THREE.BufferGeometry().setFromPoints(allSegs);
    const mat = new THREE.LineBasicMaterial({ color: 0x66ccff });
    scene.add(new THREE.LineSegments(geom, mat));
    const bbox = bboxFromSegments(allSegs);
    if (bbox) {
      fitOrthographicCamera(camera, bbox, width, height);
    }
  }
  renderer.render(scene, camera);
  const meta = document.createElement("p");
  meta.className = "viewer-note";
  meta.textContent = `DXF entities: ${dxf?.entities?.length ?? 0} (LINE, POLYLINE, CIRCLE, ARC)`;
  root.appendChild(meta);
}

async function renderUnibrow(viewer: DocumentViewerDto, root: HTMLElement) {
  root.innerHTML = "";
  const list = document.createElement("ul");
  list.className = "unibrow-nodes";
  const nodes = viewer.project_nodes ?? [];
  if (nodes.length === 0) {
    const empty = document.createElement("li");
    empty.textContent = "No assets yet.";
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
        open.onclick = async () => {
          const assetPath = `${viewer.path}/assets/${node.asset}`;
          const data = await invoke("open_path", { path: assetPath });
          window.dispatchEvent(new CustomEvent("unibrowser-refresh", { detail: data }));
        };
        li.appendChild(open);
      }
      list.appendChild(li);
    }
  }
  root.appendChild(list);
  const addRow = document.createElement("div");
  addRow.className = "unibrow-add-row";
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
  addRow.appendChild(select);
  addRow.appendChild(btn);
  addRow.appendChild(pickBtn);
  root.appendChild(addRow);
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = "Copies the file into project assets/ and updates project.json.";
  root.appendChild(note);
}

async function renderScript(viewer: DocumentViewerDto, root: HTMLElement) {
  root.innerHTML = "";
  const ta = document.createElement("textarea");
  ta.className = "script-source";
  ta.spellcheck = false;
  const bytes = await readBytes(viewer.path);
  ta.value = new TextDecoder().decode(bytes);
  const run = document.createElement("button");
  run.type = "button";
  run.textContent = "Run (Rhai)";
  const out = document.createElement("pre");
  out.className = "script-output";
  run.onclick = async () => {
    const result = await invoke<string>("run_rhai_script", { source: ta.value });
    out.textContent = result;
  };
  root.appendChild(ta);
  root.appendChild(run);
  root.appendChild(out);
  const demo = await invoke<string>("run_rhai_script", {
    source: "list_commands().to_string()",
  });
  out.textContent = `list_commands(): ${demo}`;
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
      loadedKey = null;
      return;
    }
    const key = viewerCacheKey(viewer);
    if (key === loadedKey && el.childElementCount > 0) {
      el.classList.remove("hidden");
      return;
    }
    loadedKey = key;
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
        case "dxf":
          await renderDxf(viewer.path, body);
          break;
        case "script":
          await renderScript(viewer, body);
          break;
        case "unibrow":
          await renderUnibrow(viewer, body);
          break;
        default:
          body.textContent = `Unknown viewer: ${viewer.viewer}`;
      }
    } catch (err) {
      el.innerHTML = `<p class='viewer-error'>${String(err)}</p>`;
    }
  },
  hide(el: HTMLElement) {
    el.classList.add("hidden");
  },
};
