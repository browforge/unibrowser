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
    jumpInput.value = String(pageNum);
  };

  const thumbs = document.createElement("div");
  thumbs.className = "pdf-thumbs";
  const thumbMax = Math.min(doc.numPages, 8);
  for (let n = 1; n <= thumbMax; n += 1) {
    const tb = document.createElement("button");
    tb.type = "button";
    tb.className = "pdf-thumb-btn";
    tb.textContent = String(n);
    tb.onclick = async () => {
      currentPage = n;
      await invoke("sync_pdf_page", { page: currentPage });
      await drawPage(currentPage);
    };
    thumbs.appendChild(tb);
  }
  if (doc.numPages > thumbMax) {
    const more = document.createElement("span");
    more.className = "viewer-note";
    more.textContent = `… ${doc.numPages} pages`;
    thumbs.appendChild(more);
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
  } else if (t === "ELLIPSE" && ent.center && ent.majorAxisEndPoint) {
    const mx = ent.majorAxisEndPoint.x - ent.center.x;
    const my = ent.majorAxisEndPoint.y - ent.center.y;
    const a = Math.hypot(mx, my);
    const ratio = ent.axisRatio ?? 0.5;
    const b = a * ratio;
    const rot = Math.atan2(my, mx);
    const pts: THREE.Vector3[] = [];
    for (let i = 0; i <= 48; i += 1) {
      const tRad = (i / 48) * Math.PI * 2;
      const lx = a * Math.cos(tRad);
      const ly = b * Math.sin(tRad);
      const x = ent.center.x + lx * Math.cos(rot) - ly * Math.sin(rot);
      const y = ent.center.y + lx * Math.sin(rot) + ly * Math.cos(rot);
      pts.push(new THREE.Vector3(x, y, 0));
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

async function renderDxf(path: string, root: HTMLElement) {
  const text = new TextDecoder().decode(await readBytes(path));
  const parser = new DxfParser();
  const dxf = parser.parseSync(text) as DxfData | null;
  const entities = dxf?.entities ?? [];
  const layers = Array.from(
    new Set(entities.map((e) => e.layer).filter((l): l is string => Boolean(l))),
  ).sort();
  root.innerHTML = "";
  const toolbar = document.createElement("div");
  toolbar.className = "dxf-toolbar";
  const layerSelect = document.createElement("select");
  const allOpt = document.createElement("option");
  allOpt.value = "";
  allOpt.textContent = "All layers";
  layerSelect.appendChild(allOpt);
  for (const layer of layers) {
    const opt = document.createElement("option");
    opt.value = layer;
    opt.textContent = layer;
    layerSelect.appendChild(opt);
  }
  toolbar.appendChild(layerSelect);
  root.appendChild(toolbar);
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
  let panX = 0;
  let panY = 0;
  let zoom = 1;
  let baseBbox: { minX: number; minY: number; maxX: number; maxY: number } | null = null;

  const rebuild = () => {
    while (scene.children.length > 0) {
      scene.remove(scene.children[0]);
    }
    const layerFilter = layerSelect.value;
    const allSegs: THREE.Vector3[] = [];
    for (const ent of entities) {
      if (layerFilter && ent.layer !== layerFilter) {
        continue;
      }
      allSegs.push(...entitySegments(ent));
    }
    if (allSegs.length >= 2) {
      const geom = new THREE.BufferGeometry().setFromPoints(allSegs);
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

  layerSelect.onchange = () => {
    panX = 0;
    panY = 0;
    zoom = 1;
    rebuild();
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
  meta.textContent = `DXF: ${entities.length} entities · drag pan · wheel zoom · SPLINE/ELLIPSE sketch`;
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

  nav.appendChild(prev);
  nav.appendChild(label);
  nav.appendChild(next);
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
