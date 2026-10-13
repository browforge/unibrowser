/** WCAG contrast helpers for shell density / workshop surfaces. */

export function parseRgb(rgb: string): { r: number; g: number; b: number; a: number } | null {
  const m = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([0-9.]+))?/);
  if (!m) {
    return null;
  }
  return {
    r: Number(m[1]),
    g: Number(m[2]),
    b: Number(m[3]),
    a: m[4] === undefined ? 1 : Number(m[4]),
  };
}

function channelLuminance(value: number): number {
  const normalised = value / 255;
  return normalised <= 0.04045
    ? normalised / 12.92
    : ((normalised + 0.055) / 1.055) ** 2.4;
}

export function parseRgbLuminance(rgb: string): number | null {
  const parsed = parseRgb(rgb);
  if (!parsed) {
    return null;
  }
  return (
    0.2126 * channelLuminance(parsed.r) +
    0.7152 * channelLuminance(parsed.g) +
    0.0722 * channelLuminance(parsed.b)
  );
}

export function contrastRatio(l1: number, l2: number): number {
  const hi = Math.max(l1, l2);
  const lo = Math.min(l1, l2);
  return (hi + 0.05) / (lo + 0.05);
}

export function contrastRatioForColors(foreground: string, background: string): number | null {
  const fg = parseRgbLuminance(foreground);
  const bg = parseRgbLuminance(background);
  if (fg === null || bg === null) {
    return null;
  }
  return contrastRatio(fg, bg);
}

export function resolveOpaqueBackground(el: HTMLElement): string {
  let node: HTMLElement | null = el;
  while (node) {
    const candidate = getComputedStyle(node).backgroundColor;
    const parsed = parseRgb(candidate);
    if (parsed && parsed.a >= 0.99) {
      return candidate;
    }
    node = node.parentElement;
  }
  return getComputedStyle(document.body).backgroundColor;
}

export interface ContrastSample {
  id: string;
  foreground: string;
  background: string;
  ratio: number | null;
}

export const SHELL_DENSITY_CONTRAST_TARGETS: Array<{
  id: string;
  selector: string;
  minRatio: number;
  when?: "workshop" | "mod-portal" | "settings" | "files" | "always";
  required?: boolean;
}> = [
  { id: "workshop-panel-heading", selector: ".workshop-region-heading", minRatio: 4.5, when: "workshop" },
  { id: "workshop-track-label", selector: ".workshop-track-label", minRatio: 4.5, when: "workshop" },
  { id: "workshop-source-badge", selector: ".workshop-source-badge", minRatio: 4.5, when: "workshop" },
  { id: "workshop-source-sub", selector: ".workshop-source-copy span", minRatio: 4.5, when: "workshop" },
  {
    id: "workshop-field-label",
    selector: ".workshop-field > span",
    minRatio: 4.5,
    when: "workshop",
    required: false,
  },
  {
    id: "workshop-selection-meta",
    selector: ".workshop-selection-meta",
    minRatio: 4.5,
    when: "workshop",
    required: false,
  },
  { id: "title-breadcrumb", selector: ".title-breadcrumb:not(.hidden)", minRatio: 4.5, when: "always" },
  { id: "context-status-item", selector: ".status-item", minRatio: 4.5, when: "always" },
  { id: "files-meta", selector: ".files-meta", minRatio: 4.5, when: "files" },
  { id: "settings-form-label", selector: ".settings-content label", minRatio: 4.5, when: "settings" },
  {
    id: "mod-suggestion-title",
    selector: ".mod-result-card--featured .mod-suggestion-card-title",
    minRatio: 4.5,
    when: "mod-portal",
  },
  {
    id: "mod-suggestion-reason",
    selector: ".mod-result-card--featured .mod-result-reason",
    minRatio: 4.5,
    when: "mod-portal",
  },
  {
    id: "mod-suggestion-meta",
    selector: ".mod-result-card--featured .mod-result-meta",
    minRatio: 4.5,
    when: "mod-portal",
  },
];

export function sampleContrastForSelector(
  selector: string,
  root: ParentNode = document,
): ContrastSample | null {
  const el = root.querySelector(selector) as HTMLElement | null;
  if (!el) {
    return null;
  }
  const style = getComputedStyle(el);
  const fg = style.color;
  const bg = resolveOpaqueBackground(el);
  const ratio = contrastRatioForColors(fg, bg);
  return { id: selector, foreground: fg, background: bg, ratio };
}

export function rectsOverlap(a: DOMRect, b: DOMRect, minPx = 2): boolean {
  const w = Math.min(a.right, b.right) - Math.max(a.left, b.left);
  const h = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
  return w > minPx && h > minPx;
}

const MOD_CARD_TEXT_SELECTORS =
  "strong, .mod-suggestion-card-title, .mod-result-id, .mod-result-reason, .mod-result-meta, .mod-card-badge, .mod-card-thumb .ft-icon";

const MOD_FEATURED_TEXT_IN_CARD =
  ".mod-suggestion-card-title, .mod-result-reason, .mod-result-meta, .mod-result-details";

export function validateModFeaturedCardLayout(maxTextGapPx = 24): string | null {
  const card = document.querySelector(
    ".mod-result-card--featured",
  ) as HTMLElement | null;
  if (!card) {
    return null;
  }
  const thumb = card.querySelector(".mod-card-thumb") as HTMLElement | null;
  const body = card.querySelector(".mod-card-body") as HTMLElement | null;
  const install = card.querySelector(".mod-card-install") as HTMLElement | null;
  if (!thumb || !body || !install) {
    return "mod featured card layout: missing thumb, body, or install";
  }
  const thumbRect = thumb.getBoundingClientRect();
  const textEls = Array.from(card.querySelectorAll(MOD_FEATURED_TEXT_IN_CARD)).filter(
    (el): el is HTMLElement => el instanceof HTMLElement,
  );
  if (textEls.length === 0) {
    return "mod featured card layout: no text blocks";
  }
  let textLeft = Infinity;
  for (const el of textEls) {
    const rect = el.getBoundingClientRect();
    if (rect.width > 1 && rect.height > 1) {
      textLeft = Math.min(textLeft, rect.left);
    }
  }
  if (!Number.isFinite(textLeft)) {
    return "mod featured card layout: text blocks not visible";
  }
  const gap = textLeft - thumbRect.right;
  if (gap > maxTextGapPx) {
    return `mod featured card layout: text starts ${gap.toFixed(1)}px after tile (max ${maxTextGapPx}px)`;
  }
  if (gap < -2) {
    return `mod featured card layout: text overlaps tile (${gap.toFixed(1)}px)`;
  }
  const bodyRect = body.getBoundingClientRect();
  const installRect = install.getBoundingClientRect();
  const deadMid = installRect.left - bodyRect.right;
  const cardRect = card.getBoundingClientRect();
  if (cardRect.width > 480 && deadMid > cardRect.width * 0.35) {
    return `mod featured card layout: empty gap between text column and Install (${deadMid.toFixed(0)}px)`;
  }
  const shadow = getComputedStyle(install).boxShadow;
  if (shadow && shadow !== "none") {
    return `mod featured card layout: Install box-shadow must be none (got ${shadow})`;
  }
  return null;
}

export function validateModCardTextOverlap(): string | null {
  const cards = document.querySelectorAll(".mod-result-card");
  for (const card of cards) {
    const textEls = Array.from(card.querySelectorAll(MOD_CARD_TEXT_SELECTORS)).filter(
      (el): el is HTMLElement => {
        if (!(el instanceof HTMLElement)) {
          return false;
        }
        const style = getComputedStyle(el);
        if (style.display === "none" || style.visibility === "hidden") {
          return false;
        }
        const rect = el.getBoundingClientRect();
        return rect.width > 1 && rect.height > 1;
      },
    );
    for (let i = 0; i < textEls.length; i++) {
      const a = textEls[i].getBoundingClientRect();
      for (let j = i + 1; j < textEls.length; j++) {
        const b = textEls[j].getBoundingClientRect();
        if (textEls[i].contains(textEls[j]) || textEls[j].contains(textEls[i])) {
          continue;
        }
        if (rectsOverlap(a, b)) {
          return `mod card text overlap: "${textEls[i].textContent?.slice(0, 40)}" intersects "${textEls[j].textContent?.slice(0, 40)}"`;
        }
      }
    }
  }
  return null;
}

function sceneFlags() {
  const workshop = document.querySelector(".workshop");
  const modPortal = document.getElementById("mod-portal-panel");
  const settings = document.querySelector(".settings-page");
  const filesWrap = document.getElementById("files-browser-wrap");
  return {
    workshop: Boolean(workshop && !workshop.classList.contains("hidden")),
    modPortal: Boolean(
      modPortal && !modPortal.classList.contains("hidden") && modPortal.classList.contains("mod-portal"),
    ),
    settings: Boolean(settings),
    files: Boolean(filesWrap && !filesWrap.classList.contains("hidden")),
  };
}

function targetApplies(
  when: (typeof SHELL_DENSITY_CONTRAST_TARGETS)[0]["when"],
  flags: ReturnType<typeof sceneFlags>,
): boolean {
  if (!when || when === "always") {
    return true;
  }
  if (when === "workshop") {
    return flags.workshop;
  }
  if (when === "mod-portal") {
    return flags.modPortal;
  }
  if (when === "settings") {
    return flags.settings;
  }
  if (when === "files") {
    return flags.files;
  }
  return true;
}

export function validateShellDensityContrastForCapture(minRatio = 4.5): string | null {
  if (document.body.dataset.ubNewProjectCapture === "1") {
    return null;
  }
  const flags = sceneFlags();
  if (!flags.workshop && !flags.modPortal && !flags.settings && !flags.files) {
    return null;
  }

  for (const target of SHELL_DENSITY_CONTRAST_TARGETS) {
    if (!targetApplies(target.when, flags)) {
      continue;
    }
    const sample = sampleContrastForSelector(target.selector);
    if (!sample) {
      if (target.when === "always" || target.required === false) {
        continue;
      }
      return `shell contrast: missing ${target.id}`;
    }
    if (sample.ratio === null || sample.ratio < minRatio) {
      return `shell contrast: ${target.id} ratio ${sample.ratio?.toFixed(2) ?? "null"} < ${minRatio} (fg=${sample.foreground}, bg=${sample.background})`;
    }
  }

  if (flags.modPortal) {
    const overlapErr = validateModCardTextOverlap();
    if (overlapErr) {
      return overlapErr;
    }
    const layoutErr = validateModFeaturedCardLayout();
    if (layoutErr) {
      return layoutErr;
    }
  }
  return null;
}

/** Documented surface pairs (rgb) — keep in sync with light/dark/HC workshop rules. */
export const SHELL_DENSITY_TOKEN_PAIRS: Array<{
  theme: string;
  id: string;
  foreground: string;
  background: string;
  minRatio: number;
}> = [
  {
    theme: "light",
    id: "workshop-heading",
    foreground: "rgb(32, 34, 35)",
    background: "rgb(241, 240, 236)",
    minRatio: 4.5,
  },
  {
    theme: "light",
    id: "workshop-track-label",
    foreground: "rgb(61, 68, 72)",
    background: "rgb(241, 240, 236)",
    minRatio: 4.5,
  },
  {
    theme: "light",
    id: "workshop-source-badge",
    foreground: "rgb(61, 68, 72)",
    background: "rgb(243, 242, 238)",
    minRatio: 4.5,
  },
  {
    theme: "light",
    id: "workshop-field-label",
    foreground: "rgb(69, 75, 78)",
    background: "rgb(241, 240, 236)",
    minRatio: 4.5,
  },
  {
    theme: "light",
    id: "form-label",
    foreground: "rgb(69, 75, 78)",
    background: "rgb(241, 240, 236)",
    minRatio: 4.5,
  },
  {
    theme: "light",
    id: "mod-card-meta",
    foreground: "rgb(69, 75, 78)",
    background: "rgb(247, 246, 242)",
    minRatio: 4.5,
  },
  {
    theme: "light",
    id: "mod-card-title",
    foreground: "rgb(37, 42, 48)",
    background: "rgb(247, 246, 242)",
    minRatio: 4.5,
  },
  {
    theme: "light",
    id: "mod-suggestion-reason",
    foreground: "rgb(69, 75, 78)",
    background: "rgb(247, 246, 242)",
    minRatio: 4.5,
  },
  {
    theme: "dark",
    id: "workshop-heading",
    foreground: "rgb(233, 237, 239)",
    background: "rgb(34, 39, 43)",
    minRatio: 4.5,
  },
  {
    theme: "hc-light",
    id: "workshop-heading",
    foreground: "rgb(0, 0, 0)",
    background: "rgb(255, 255, 255)",
    minRatio: 4.5,
  },
  {
    theme: "hc-dark",
    id: "workshop-heading",
    foreground: "rgb(255, 255, 255)",
    background: "rgb(0, 0, 0)",
    minRatio: 4.5,
  },
];
