/** Pure capture layout/contrast helpers (no Monaco / toolbar imports — safe for Node tests). */

function parseRgb(rgb: string): { r: number; g: number; b: number; a: number } | null {
  const hex = rgb.trim().match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i);
  if (hex) {
    const value =
      hex[1].length === 3
        ? hex[1]
            .split("")
            .map((channel) => `${channel}${channel}`)
            .join("")
        : hex[1];
    return {
      r: Number.parseInt(value.slice(0, 2), 16),
      g: Number.parseInt(value.slice(2, 4), 16),
      b: Number.parseInt(value.slice(4, 6), 16),
      a: 1,
    };
  }
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

function parseRgbLuminance(rgb: string): number | null {
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

function contrastRatio(l1: number, l2: number): number {
  const hi = Math.max(l1, l2);
  const lo = Math.min(l1, l2);
  return (hi + 0.05) / (lo + 0.05);
}

export function colourContrastRatio(foreground: string, background: string): number | null {
  const foregroundLuminance = parseRgbLuminance(foreground);
  const backgroundLuminance = parseRgbLuminance(background);
  if (foregroundLuminance === null || backgroundLuminance === null) {
    return null;
  }
  return contrastRatio(foregroundLuminance, backgroundLuminance);
}

export function focusRingMeetsRequirements(
  width: number,
  ringColour: string,
  surroundingColour: string,
  minContrast = 3,
): boolean {
  const contrast = colourContrastRatio(ringColour, surroundingColour);
  return width >= 2 && contrast !== null && contrast >= minContrast;
}

export interface CaptureRect {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

export function rectWithinBounds(
  inner: CaptureRect,
  outer: CaptureRect,
  tolerance = 1,
): boolean {
  return (
    inner.left >= outer.left - tolerance &&
    inner.right <= outer.right + tolerance &&
    inner.top >= outer.top - tolerance &&
    inner.bottom <= outer.bottom + tolerance
  );
}

export function captureRectsOverlap(a: CaptureRect, b: CaptureRect): boolean {
  return a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
}

export function timelineEndpointClearsScrollbar(
  endpointRight: number,
  viewportRight: number,
  safeInset = 12,
): boolean {
  return endpointRight <= viewportRight - safeInset;
}

export function backgroundExceedsRaisedSurface(
  candidateLuminance: number,
  raisedSurfaceLuminance: number,
  tolerance = 0.06,
): boolean {
  return candidateLuminance > raisedSurfaceLuminance + tolerance;
}

export function selectOptionListMeetsDarkSurface(
  optionBackgroundLuminance: number,
  raisedSurfaceLuminance: number,
  tolerance = 0.06,
): boolean {
  return !backgroundExceedsRaisedSurface(
    optionBackgroundLuminance,
    raisedSurfaceLuminance,
    tolerance,
  );
}

export function inputBorderEdgesUniform(style: CSSStyleDeclaration): boolean {
  const top = style.borderTopColor;
  return (
    top === style.borderRightColor &&
    top === style.borderBottomColor &&
    top === style.borderLeftColor
  );
}

export function switchTrackSizeWithinTolerance(
  width: number,
  height: number,
  tolerance = 1,
): boolean {
  return Math.abs(width - 32) <= tolerance && Math.abs(height - 18) <= tolerance;
}

export function knobVerticalOffsetCentered(
  trackHeight: number,
  knobTop: number,
  knobHeight: number,
  tolerance = 1.5,
): boolean {
  const trackCenter = trackHeight / 2;
  const knobCenter = knobTop + knobHeight / 2;
  return Math.abs(knobCenter - trackCenter) <= tolerance;
}

export function provingFloatingToolbarScanError(
  labels: string[],
  provingControlIds: string[],
): string | null {
  for (const label of labels) {
    if (/proving/i.test(label)) {
      return `proving floating toolbar visible: "${label}"`;
    }
  }
  for (const id of provingControlIds) {
    if (id.includes("cmd-proving")) {
      return `proving mod control on floating toolbar: ${id}`;
    }
  }
  return null;
}

export function grayscalePreviewControlCaptureError(
  present: boolean,
  visible: boolean,
  capabilityState: string,
): string | null {
  if (!present) {
    return "missing Grayscale preview control (image-grayscale-preview.control.grayscale)";
  }
  if (!visible) {
    return "Grayscale preview control has no visible geometry";
  }
  if (capabilityState !== "supported" && capabilityState !== "emulated") {
    return `Grayscale preview capability state is "${capabilityState}", expected supported or emulated`;
  }
  return null;
}
