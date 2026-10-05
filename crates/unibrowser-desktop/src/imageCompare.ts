import { convertFileSrc } from "@tauri-apps/api/core";
import { t } from "./i18n";

export interface ImageCompareViewDto {
  left_path: string;
  right_path: string;
}

export async function applyImageCompare(
  view: ImageCompareViewDto | null,
  host: HTMLElement,
) {
  if (!view) {
    host.classList.add("hidden");
    host.innerHTML = "";
    return;
  }
  host.classList.remove("hidden");
  host.innerHTML = "";

  const head = document.createElement("div");
  head.className = "image-compare-head";
  const title = document.createElement("span");
  title.textContent = t("imageCompare.title");
  const closeBtn = document.createElement("button");
  closeBtn.type = "button";
  closeBtn.dataset.action = "close-image-compare";
  closeBtn.textContent = t("imageCompare.close");
  head.appendChild(title);
  head.appendChild(closeBtn);
  host.appendChild(head);

  const sliderRow = document.createElement("label");
  sliderRow.className = "image-compare-slider-row";
  sliderRow.textContent = t("imageCompare.slider");
  const slider = document.createElement("input");
  slider.type = "range";
  slider.min = "0";
  slider.max = "100";
  slider.value = "50";
  slider.className = "image-compare-slider";
  sliderRow.appendChild(slider);
  host.appendChild(sliderRow);

  const stage = document.createElement("div");
  stage.className = "image-compare-stage";
  const leftWrap = document.createElement("div");
  leftWrap.className = "image-compare-pane";
  const rightWrap = document.createElement("div");
  rightWrap.className = "image-compare-pane";
  const leftImg = document.createElement("img");
  const rightImg = document.createElement("img");
  leftImg.alt = view.left_path;
  rightImg.alt = view.right_path;
  leftImg.src = convertFileSrc(view.left_path);
  rightImg.src = convertFileSrc(view.right_path);
  leftWrap.appendChild(leftImg);
  rightWrap.appendChild(rightImg);
  stage.appendChild(leftWrap);
  stage.appendChild(rightWrap);
  host.appendChild(stage);

  let zoom = 1;
  const applyZoom = () => {
    leftImg.style.transform = `scale(${zoom})`;
    rightImg.style.transform = `scale(${zoom})`;
  };
  const syncZoom = (delta: number) => {
    zoom = Math.min(4, Math.max(0.25, zoom + delta));
    applyZoom();
  };

  const zoomRow = document.createElement("div");
  zoomRow.className = "image-compare-zoom";
  const zoomOut = document.createElement("button");
  zoomOut.type = "button";
  zoomOut.textContent = "−";
  const zoomIn = document.createElement("button");
  zoomIn.type = "button";
  zoomIn.textContent = "+";
  zoomOut.onclick = () => syncZoom(-0.15);
  zoomIn.onclick = () => syncZoom(0.15);
  zoomRow.appendChild(zoomOut);
  zoomRow.appendChild(zoomIn);
  host.appendChild(zoomRow);

  const updateSplit = () => {
    const pct = Number(slider.value);
    leftWrap.style.flex = `0 0 ${pct}%`;
    rightWrap.style.flex = `1 1 auto`;
  };
  slider.oninput = updateSplit;
  updateSplit();
  applyZoom();
}
