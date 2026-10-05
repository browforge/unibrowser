/** Plain-text empty states (no image assets). */

export function emptyStateBlock(heading: string, lines: string[]): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "empty-state";
  const art = document.createElement("pre");
  art.className = "empty-state-art";
  art.setAttribute("aria-hidden", "true");
  art.textContent = [
    "  +------------------+",
    "  |                  |",
    "  |    ( empty )     |",
    "  |                  |",
    "  +------------------+",
  ].join("\n");
  const h = document.createElement("p");
  h.className = "empty-state-heading";
  h.textContent = heading;
  wrap.appendChild(art);
  wrap.appendChild(h);
  for (const line of lines) {
    const p = document.createElement("p");
    p.className = "empty-state-line";
    p.textContent = line;
    wrap.appendChild(p);
  }
  return wrap;
}
