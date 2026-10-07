const HOST_ID = "ub-command-tooltip-host";

function host(): HTMLElement {
  let el = document.getElementById(HOST_ID);
  if (!el) {
    el = document.createElement("div");
    el.id = HOST_ID;
    el.className = "ub-command-tooltip hidden";
    el.setAttribute("role", "tooltip");
    document.body.appendChild(el);
  }
  return el;
}

export function formatCommandTooltip(
  title: string,
  reason: string | null,
  chord: string | null,
): string {
  const parts = [title];
  if (reason?.trim()) {
    parts.push(reason.trim());
  }
  if (chord?.trim()) {
    parts.push(chord.trim());
  }
  return parts.join("\n");
}

export function bindCommandTooltip(
  btn: HTMLButtonElement,
  opts: {
    title: string;
    reason: string | null;
    chord: string | null;
    enabled: boolean;
  },
) {
  const tipId = `ub-cmd-tip-${opts.title.replace(/\s+/g, "-").slice(0, 40)}-${Math.random().toString(36).slice(2, 7)}`;
  const text = formatCommandTooltip(opts.title, opts.reason, opts.chord);
  btn.setAttribute("aria-label", opts.title);
  if (!opts.enabled && opts.reason) {
    btn.setAttribute("aria-describedby", tipId);
  } else {
    btn.removeAttribute("aria-describedby");
  }
  btn.title = text;

  const show = () => {
    const h = host();
    h.innerHTML = "";
    const strong = document.createElement("strong");
    strong.textContent = opts.title;
    h.appendChild(strong);
    if (opts.reason) {
      const p = document.createElement("p");
      p.textContent = opts.reason;
      h.appendChild(p);
    }
    if (opts.chord) {
      const k = document.createElement("p");
      k.className = "ub-command-tooltip-chord";
      k.textContent = opts.chord;
      h.appendChild(k);
    }
    h.id = tipId;
    h.classList.remove("hidden");
    const rect = btn.getBoundingClientRect();
    h.style.left = `${Math.max(8, rect.left)}px`;
    h.style.top = `${rect.bottom + 6}px`;
  };

  const hide = () => {
    host().classList.add("hidden");
  };

  btn.addEventListener("mouseenter", show);
  btn.addEventListener("focus", show);
  btn.addEventListener("mouseleave", hide);
  btn.addEventListener("blur", hide);
}

/** Hide any visible command tooltip and release focus before doc-capture screenshots. */
export function dismissCommandTooltips(): void {
  host().classList.add("hidden");
}

export function paletteEntryHint(entry: {
  title: string;
  reason: string | null;
  chord: string | null;
  enabled: boolean;
}): string {
  if (!entry.enabled && entry.reason) {
    return formatCommandTooltip(entry.title, entry.reason, entry.chord);
  }
  if (entry.chord) {
    return `${entry.title}  ·  ${entry.chord}`;
  }
  return entry.title;
}
