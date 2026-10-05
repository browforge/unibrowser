/** Block autoplay until the user explicitly starts media. */
export function mountClickToPlay(host: HTMLElement, media: HTMLMediaElement, label = "Click to play") {
  media.autoplay = false;
  media.preload = "metadata";
  media.pause();

  const wrap = document.createElement("div");
  wrap.className = "media-play-gate-wrap";
  const gate = document.createElement("button");
  gate.type = "button";
  gate.className = "media-play-gate";
  gate.textContent = label;
  gate.onclick = () => {
    gate.remove();
    void media.play().catch(() => {});
  };
  host.appendChild(wrap);
  wrap.appendChild(media);
  wrap.appendChild(gate);
}
