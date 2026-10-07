const SELECTABLE =
  "path,text,rect,circle,ellipse,line,polyline,polygon";

export function mountSvgInspector(
  previewHost: HTMLElement,
  sourceTextarea: HTMLTextAreaElement,
  onSourceChange: () => void,
  mountTo?: HTMLElement,
): void {
  const panel = document.createElement("aside");
  panel.className = "svg-inspector";
  panel.innerHTML = "<h3>Selection</h3><p class='muted'>Click a shape or text in the preview.</p>";

  const fillLabel = document.createElement("label");
  fillLabel.textContent = "Fill";
  const fillInput = document.createElement("input");
  fillInput.type = "text";
  fillInput.placeholder = "none or #rrggbb";
  fillLabel.appendChild(fillInput);

  const strokeLabel = document.createElement("label");
  strokeLabel.textContent = "Stroke";
  const strokeInput = document.createElement("input");
  strokeInput.type = "text";
  strokeInput.placeholder = "none or #rrggbb";
  strokeLabel.appendChild(strokeInput);

  const strokeWidthLabel = document.createElement("label");
  strokeWidthLabel.textContent = "Stroke width";
  const strokeWidthInput = document.createElement("input");
  strokeWidthInput.type = "text";
  strokeWidthInput.placeholder = "1";
  strokeWidthLabel.appendChild(strokeWidthInput);

  const applyBtn = document.createElement("button");
  applyBtn.type = "button";
  applyBtn.textContent = "Apply to selection";
  applyBtn.disabled = true;

  const status = document.createElement("p");
  status.className = "viewer-note svg-inspector-status";

  panel.append(fillLabel, strokeLabel, strokeWidthLabel, applyBtn, status);

  let selected: SVGElement | null = null;
  let svgRoot: SVGSVGElement | null = null;

  const syncPreview = () => {
    previewHost.innerHTML = "";
    const parser = new DOMParser();
    const doc = parser.parseFromString(sourceTextarea.value, "image/svg+xml");
    const parseErr = doc.querySelector("parsererror");
    if (parseErr) {
      status.textContent = "SVG parse error — fix source text.";
      svgRoot = null;
      return;
    }
    svgRoot = doc.documentElement as unknown as SVGSVGElement;
    const clone = svgRoot.cloneNode(true) as SVGSVGElement;
    clone.classList.add("svg-inline-root");
    clone.querySelectorAll(SELECTABLE).forEach((el) => {
      el.classList.add("svg-selectable");
      el.addEventListener("click", (ev) => {
        ev.stopPropagation();
        previewHost.querySelectorAll(".svg-selected").forEach((n) => {
          n.classList.remove("svg-selected");
        });
        const target = ev.currentTarget as SVGElement;
        target.classList.add("svg-selected");
        selected = target;
        fillInput.value = target.getAttribute("fill") ?? "";
        strokeInput.value = target.getAttribute("stroke") ?? "";
        strokeWidthInput.value = target.getAttribute("stroke-width") ?? "";
        applyBtn.disabled = false;
        status.textContent = `${target.tagName.toLowerCase()} selected`;
      });
    });
    previewHost.appendChild(clone);
    status.textContent = "Click a path or text node to edit fill/stroke.";
  };

  sourceTextarea.addEventListener("input", () => {
    syncPreview();
    onSourceChange();
  });

  applyBtn.onclick = () => {
    if (!selected) {
      return;
    }
    const tag = selected.tagName.toLowerCase();
    const setAttrs = (el: Element) => {
      if (fillInput.value.trim()) {
        el.setAttribute("fill", fillInput.value.trim());
      }
      if (strokeInput.value.trim()) {
        el.setAttribute("stroke", strokeInput.value.trim());
      }
      if (strokeWidthInput.value.trim()) {
        el.setAttribute("stroke-width", strokeWidthInput.value.trim());
      }
    };
    setAttrs(selected);
    const liveSvg = previewHost.querySelector("svg");
    if (!liveSvg) {
      return;
    }
    const next = new XMLSerializer().serializeToString(liveSvg);
    sourceTextarea.value = next;
    onSourceChange();
    syncPreview();
    status.textContent = `Updated ${tag}`;
  };

  syncPreview();
  const host = mountTo ?? previewHost.parentElement;
  host?.appendChild(panel);
}
