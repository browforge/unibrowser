/** Shared docked bottom panel behaviour (UI-09). */

const STORAGE_KEY = "unibrowser.bottomPanelHeight";

export function initDockPanel() {
  const panel = document.getElementById("script-output-panel");
  if (!panel) {
    return;
  }
  panel.classList.add("ub-dock-panel");
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    panel.style.setProperty("--ub-bottom-panel-height", saved);
    panel.style.height = saved;
  }

  const grip = document.getElementById("bottom-panel-resize-grip");
  if (grip) {
    let dragging = false;
    let startY = 0;
    let startHeight = 0;
    grip.addEventListener("mousedown", (ev) => {
      dragging = true;
      startY = ev.clientY;
      startHeight = panel.getBoundingClientRect().height;
      ev.preventDefault();
    });
    window.addEventListener("mousemove", (ev) => {
      if (!dragging) {
        return;
      }
      const delta = startY - ev.clientY;
      const next = Math.min(Math.max(startHeight + delta, 96), window.innerHeight * 0.55);
      const px = `${next}px`;
      panel.style.height = px;
      panel.style.setProperty("--ub-bottom-panel-height", px);
    });
    window.addEventListener("mouseup", () => {
      if (!dragging) {
        return;
      }
      dragging = false;
      localStorage.setItem(STORAGE_KEY, panel.style.height || "10rem");
      window.dispatchEvent(new CustomEvent("unibrowser-bottom-panel-resize"));
    });
  }

  const regions: HTMLElement[] = [
    document.getElementById("chrome")!,
    document.getElementById("content-host")!,
    panel,
    document.getElementById("ai-assistant-panel")!,
    document.getElementById("status-bar")!,
  ].filter(Boolean);

  let regionIndex = 0;
  document.addEventListener("keydown", (e) => {
    if (e.key !== "F6") {
      return;
    }
    e.preventDefault();
    regionIndex = (regionIndex + 1) % regions.length;
    const el = regions[regionIndex];
    if (el.classList.contains("hidden")) {
      regionIndex = (regionIndex + 1) % regions.length;
      regions[regionIndex]?.focus();
      return;
    }
    el.focus({ preventScroll: true });
    if (el === panel) {
      panel.querySelector<HTMLElement>(".bottom-panel-tab.is-active")?.focus();
    }
  });
}
