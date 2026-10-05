/** Chrome-hidden content focus (F11). Does not use the browser Fullscreen API. */

let active = false;

export function isContentFullscreen(): boolean {
  return active;
}

export function exitContentFullscreen(): void {
  if (!active) {
    return;
  }
  active = false;
  document.body.classList.remove("content-fullscreen");
  window.dispatchEvent(new CustomEvent("unibrowser-content-fullscreen", { detail: false }));
}

export function toggleContentFullscreen(): void {
  if (active) {
    exitContentFullscreen();
    return;
  }
  active = true;
  document.body.classList.add("content-fullscreen");
  window.dispatchEvent(new CustomEvent("unibrowser-content-fullscreen", { detail: true }));
}

export function initContentFullscreen(onLayoutChange: () => void) {
  window.addEventListener("unibrowser-content-fullscreen", () => {
    onLayoutChange();
  });
}
