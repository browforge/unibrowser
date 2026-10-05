/** Space play/pause; arrows seek when a media element has focus. */

export function bindMediaViewerKeys(media: HTMLMediaElement) {
  media.tabIndex = 0;
  media.addEventListener("keydown", (ev) => {
    if (ev.key === " " || ev.code === "Space") {
      ev.preventDefault();
      if (media.paused) {
        void media.play().catch(() => {});
      } else {
        media.pause();
      }
      return;
    }
    if (ev.key === "ArrowLeft") {
      ev.preventDefault();
      media.currentTime = Math.max(0, media.currentTime - 5);
      return;
    }
    if (ev.key === "ArrowRight") {
      ev.preventDefault();
      const dur = media.duration;
      const next = media.currentTime + 5;
      media.currentTime =
        Number.isFinite(dur) && dur > 0 ? Math.min(dur, next) : next;
    }
  });
}
