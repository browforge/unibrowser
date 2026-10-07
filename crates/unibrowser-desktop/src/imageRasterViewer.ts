import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { copyImageFile } from "./clipboard";
import { showErrorToast, showToast } from "./toast";
import { t } from "./i18n";
import { reportViewerZoom } from "./viewerZoom";
import { patchStatusCursor } from "./statusBar";
import {
  clearImagePixelSample,
  setImagePixelSample,
} from "./viewerPixelSample";
import { setStatusHint } from "./statusBar";
import { setViewerLocatorBridge } from "./locatorViewerBridge";

type FitMode = "fit" | "actual" | "free";

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

async function ensureAssetAllowed(path: string) {
  await invoke("allow_asset_path", { path }).catch(() => undefined);
}

export async function mountRasterImageViewer(
  path: string,
  root: HTMLElement,
): Promise<(commandId: string) => boolean> {
  await ensureAssetAllowed(path);
  root.innerHTML = "";
  setStatusHint(t("image.toolbarNote"));

  const layout = document.createElement("div");
  layout.className = "image-viewer-layout";
  const mainCol = document.createElement("div");
  mainCol.className = "image-viewer-main";

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
  const histNote = document.createElement("p");
  histNote.className = "viewer-note image-histogram-note";
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

  let scale = 1;
  let rotation = 0;
  let flipH = false;
  let flipV = false;
  let panX = 0;
  let panY = 0;
  let fitMode: FitMode = "fit";
  let cropMode = false;
  let cropDragging = false;
  let cropStartX = 0;
  let cropStartY = 0;
  let cropRectNorm: { x: number; y: number; w: number; h: number } | null = null;
  let eyedropActive = false;
  let histogramOpen = false;

  const orientedSize = () => {
    const quarter = ((rotation % 360) + 360) % 360;
    const swap = quarter === 90 || quarter === 270;
    const nw = img.naturalWidth;
    const nh = img.naturalHeight;
    if (!nw || !nh) {
      return { w: 0, h: 0 };
    }
    return { w: swap ? nh : nw, h: swap ? nw : nh };
  };

  const reportZoom = () => {
    reportViewerZoom(scale, "image");
  };

  const applyTransform = () => {
    const sx = flipH ? -1 : 1;
    const sy = flipV ? -1 : 1;
    stage.style.transform = `translate(${panX}px, ${panY}px) scale(${scale * sx}, ${scale * sy}) rotate(${rotation}deg)`;
    reportZoom();
  };

  const applyFit = () => {
    const { w, h } = orientedSize();
    if (!w || !h) {
      return;
    }
    const pad = 24;
    const availableWidth = Math.max(1, viewport.clientWidth - pad);
    const availableHeight = Math.max(1, viewport.clientHeight - pad);
    scale = Math.min(8, Math.max(0.02, Math.min(availableWidth / w, availableHeight / h)));
    panX = 0;
    panY = 0;
    fitMode = "fit";
    applyTransform();
  };

  const applyActualSize = () => {
    scale = 1;
    panX = 0;
    panY = 0;
    fitMode = "actual";
    applyTransform();
  };

  const bumpImageZoom = (delta: number) => {
    fitMode = "free";
    if (delta > 0) {
      scale = Math.min(8, scale * 1.15);
    } else {
      scale = Math.max(0.02, scale / 1.15);
    }
    applyTransform();
  };

  const toggleHistogram = () => {
    histogramOpen = !histogramOpen;
    histAside.classList.toggle("hidden", !histogramOpen);
    if (histogramOpen) {
      refreshHistogram();
    }
  };

  const toggleEyedropper = () => {
    eyedropActive = !eyedropActive;
    viewport.classList.toggle("image-viewport--eyedrop", eyedropActive);
    if (!eyedropActive) {
      clearImagePixelSample();
      patchStatusCursor("");
    }
  };

  const toggleCropMode = () => {
    if (cropMode) {
      cropMode = false;
      cropLayer.hidden = true;
      viewport.classList.remove("image-viewport--crop");
      cropDragging = false;
      return;
    }
    cropMode = true;
    cropLayer.hidden = false;
    viewport.classList.add("image-viewport--crop");
  };

  const updateCropActions = () => {
    const hasCrop = Boolean(cropRectNorm && cropRectNorm.w >= 0.5 && cropRectNorm.h >= 0.5);
    if (!hasCrop) {
      cropRect.hidden = true;
      return;
    }
    cropRect.hidden = false;
    const { x, y, w, h } = cropRectNorm!;
    cropRect.style.left = `${x}%`;
    cropRect.style.top = `${y}%`;
    cropRect.style.width = `${w}%`;
    cropRect.style.height = `${h}%`;
  };

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

  viewport.addEventListener("mousemove", (ev) => samplePixelAt(ev.clientX, ev.clientY));
  viewport.addEventListener("mouseleave", () => {
    if (eyedropActive) {
      clearImagePixelSample();
      patchStatusCursor("");
    }
  });

  const onImageReady = () => {
    refreshSampleSurface();
    applyFit();
  };
  img.addEventListener("load", onImageReady);
  if (img.complete) {
    void img.decode().then(onImageReady).catch(() => undefined);
  }

  const resizeObserver = new ResizeObserver(() => {
    if (fitMode === "fit") {
      applyFit();
    }
  });
  resizeObserver.observe(viewport);
  root.dataset.imageViewerMounted = "1";

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
      updateCropActions();
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
      updateCropActions();
      return;
    }
    if (!dragging) {
      return;
    }
    fitMode = "free";
    panX += e.clientX - lastX;
    panY += e.clientY - lastY;
    lastX = e.clientX;
    lastY = e.clientY;
    applyTransform();
  };
  viewport.onwheel = (e) => {
    e.preventDefault();
    fitMode = "free";
    const factor = e.deltaY > 0 ? 0.9 : 1.1;
    scale = Math.min(8, Math.max(0.02, scale * factor));
    applyTransform();
  };

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

  const runOcr = () => {
    void invoke<boolean>("ocr_tesseract_available_cmd").then((ok) => {
      if (!ok) {
        showToast(t("image.ocrDisabledReason"), "info");
        return;
      }
      void invoke<string>("execute_command", {
        commandId: "unibrowser.image.ocr.copy",
      })
        .then((msg) => {
          if (msg.startsWith("ocr-copy:")) {
            const text = msg.slice("ocr-copy:".length);
            void navigator.clipboard.writeText(text).then(() => {
              showToast(t("image.ocrCopied"), "info");
            });
          }
        })
        .catch(showErrorToast);
    });
  };

  return (commandId: string) => {
    switch (commandId) {
      case "unibrowser.image.fit":
        applyFit();
        return true;
      case "unibrowser.image.actualSize":
        applyActualSize();
        return true;
      case "unibrowser.image.zoomOut":
        bumpImageZoom(-1);
        return true;
      case "unibrowser.image.zoomIn":
        bumpImageZoom(1);
        return true;
      case "unibrowser.image.crop":
        toggleCropMode();
        return true;
      case "unibrowser.image.rotate":
        rotation = (rotation + 90) % 360;
        if (fitMode === "fit") {
          applyFit();
        } else {
          applyTransform();
        }
        return true;
      case "unibrowser.image.flipHorizontal":
        flipH = !flipH;
        applyTransform();
        return true;
      case "unibrowser.image.flipVertical":
        flipV = !flipV;
        applyTransform();
        return true;
      case "unibrowser.image.ocr.copy":
        runOcr();
        return true;
      case "unibrowser.image.copy":
        void copyImageFile(path);
        return true;
      case "unibrowser.image.histogram":
        toggleHistogram();
        return true;
      case "unibrowser.image.eyedropper":
        toggleEyedropper();
        return true;
      default:
        return false;
    }
  };
}
