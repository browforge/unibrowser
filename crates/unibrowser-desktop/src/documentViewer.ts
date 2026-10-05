import { invoke } from "@tauri-apps/api/core";
import * as mammoth from "mammoth";
import * as pdfjsLib from "pdfjs-dist";
import DxfParser from "dxf-parser";
import * as THREE from "three";

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url,
).toString();

export interface DocumentViewerDto {
  path: string;
  viewer: string;
  title: string;
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

async function renderPdf(path: string, root: HTMLElement) {
  const bytes = await readBytes(path);
  const doc = await pdfjsLib.getDocument({ data: bytes }).promise;
  const page = await doc.getPage(1);
  const scale = 1.25;
  const viewport = page.getViewport({ scale });
  const canvas = document.createElement("canvas");
  canvas.className = "pdf-page";
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("canvas 2d unavailable");
  }
  canvas.width = viewport.width;
  canvas.height = viewport.height;
  root.innerHTML = "";
  root.appendChild(canvas);
  await page.render({ canvasContext: ctx, viewport, canvas }).promise;
  const meta = document.createElement("p");
  meta.className = "viewer-note";
  meta.textContent = `Page 1 / ${doc.numPages} (pdf.js raster view; vector engine in project graph per ADR 0011)`;
  root.appendChild(meta);
}

type DxfData = {
  entities?: { type: string; vertices?: { x: number; y: number }[] }[];
};

function lineEntities(dxf: DxfData | null) {
  const lines: THREE.Vector3[] = [];
  if (!dxf?.entities) {
    return lines;
  }
  for (const ent of dxf.entities) {
    const verts = ent.vertices;
    if (ent.type === "LINE" && verts && verts.length >= 2) {
      const a = verts[0];
      const b = verts[1];
      lines.push(new THREE.Vector3(a.x, a.y, 0), new THREE.Vector3(b.x, b.y, 0));
    }
  }
  return lines;
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
  const pts = lineEntities(dxf);
  if (pts.length >= 2) {
    const geom = new THREE.BufferGeometry().setFromPoints(pts);
    const mat = new THREE.LineBasicMaterial({ color: 0x66ccff });
    scene.add(new THREE.LineSegments(geom, mat));
  }
  renderer.render(scene, camera);
  const meta = document.createElement("p");
  meta.className = "viewer-note";
  meta.textContent = `DXF entities: ${dxf?.entities?.length ?? 0} (dxf-parser + three.js)`;
  root.appendChild(meta);
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
    source: "42",
  });
  out.textContent = `demo: ${demo}`;
}

export const documentViewer = {
  async apply(viewer: DocumentViewerDto | null, el: HTMLElement) {
    if (!viewer) {
      el.classList.add("hidden");
      el.innerHTML = "";
      loadedKey = null;
      return;
    }
    const key = `${viewer.viewer}:${viewer.path}`;
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
          await renderPdf(viewer.path, body);
          break;
        case "dxf":
          await renderDxf(viewer.path, body);
          break;
        case "script":
          await renderScript(viewer, body);
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
