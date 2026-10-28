import { BLOCK_PICKER_MENU_LABELS, compactToolbarIconFallbackError } from "./textCompactToolbar";
import { documentWordCount } from "./ui/documentCardStatus";

export interface MonacoCaptureDiagnostics {
  viewLineCount: number;
  viewTextLength: number;
  hostWidth: number;
  hostHeight: number;
  editorHeight: number;
  sampleText: string;
}

export interface ThemeCaptureDiagnostics {
  dataTheme: string | null;
  ubBase: string;
  colorScheme: string;
}

export interface EditorCanvasDiagnostics {
  canvasCount: number;
  sampleVariance: number;
  sampleWidth: number;
  sampleHeight: number;
}

export interface MonacoThemeCaptureDiagnostics {
  resolvedTheme: string | null;
  themeColorsJson: string | null;
  themeDtoJson: string | null;
  editorBackground: string;
  editorForeground: string;
  viewLineColor: string;
  viewLineBackground: string;
  viewLineFontFamily: string;
  viewLineFontSize: string;
  sourceCodeProLoaded: boolean;
  fallbackMonoLoaded: boolean;
}

export interface CommandManagerCaptureDiagnostics {
  viewportWidth: number;
  viewportHeight: number;
  controlCount: number;
  visibleCount: number;
  contrastOkCount: number;
  contextLabel: string;
  autoContext: string | null;
  pipeline: Record<string, unknown> | null;
  controls: Array<{
    controlId: string;
    label: string;
    classes: string[];
    capabilityState: string;
    enabled: boolean;
    disabledReason: string | null;
    width: number;
    height: number;
    labelBounds: { x: number; y: number; width: number; height: number };
    labelWidth: number;
    iconWidth: number;
    opacity: string;
    filter: string;
    foreground: string;
    labelForeground: string;
    background: string;
    contrast: number | null;
    chromeTextVariable: string;
    ancestors: Array<{
      tag: string;
      id: string;
      classes: string[];
      opacity: string;
      filter: string;
      color: string;
      background: string;
    }>;
  }>;
}

export interface StatusBarCaptureDiagnostics {
  segmentCount: number;
  segments: Array<{
    id: string;
    label: string;
    bounds: { x: number; y: number; width: number; height: number };
    foreground: string;
    background: string;
    contrast: number | null;
    opacity: string;
    filter: string;
  }>;
}

export interface DocCaptureDiagnostics {
  theme: ThemeCaptureDiagnostics;
  monaco: MonacoCaptureDiagnostics | null;
  canvas: EditorCanvasDiagnostics | null;
  monacoTheme: MonacoThemeCaptureDiagnostics | null;
  commandManager: CommandManagerCaptureDiagnostics | null;
  statusBar: StatusBarCaptureDiagnostics;
  problems: {
    tabText: string;
    rowCount: number;
    visibleRowCount: number;
    emptyVisible: boolean;
    panelHeight: number;
    tableHeight: number;
  };
  documentViewer: {
    hidden: boolean;
    childClasses: string[];
    imageComplete: boolean | null;
    imageNaturalWidth: number | null;
    imageRenderedWidth: number | null;
    imageStageTransform: string | null;
  } | null;
  editorPointStack: Array<{
    tag: string;
    id: string;
    classes: string;
    display: string;
    visibility: string;
    position: string;
    background: string;
    width: number;
    height: number;
  }>;
}

const LIGHT_UB_BASE = "#f1f0ec";

function normaliseHex(c: string): string {
  return c.trim().toLowerCase().replace(/\s+/g, "");
}

export function collectMonacoDiagnostics(): MonacoCaptureDiagnostics | null {
  const host = document.getElementById("text-editor-host");
  if (!host || host.classList.contains("hidden")) {
    return null;
  }
  const lines = host.querySelectorAll(".monaco-editor .view-line");
  const viewLines = host.querySelector(".monaco-editor .view-lines");
  const monacoRoot = host.querySelector(".monaco-editor") as HTMLElement | null;
  const text = (viewLines?.textContent ?? "").trim();
  const sample = text.slice(0, 120);
  return {
    viewLineCount: lines.length,
    viewTextLength: text.length,
    hostWidth: host.clientWidth,
    hostHeight: host.clientHeight,
    editorHeight: monacoRoot?.clientHeight ?? 0,
    sampleText: sample,
  };
}

export function collectThemeDiagnostics(): ThemeCaptureDiagnostics {
  const root = document.documentElement;
  const ubBase = getComputedStyle(root).getPropertyValue("--ub-base").trim();
  return {
    dataTheme: root.dataset.theme ?? null,
    ubBase,
    colorScheme: getComputedStyle(root).colorScheme,
  };
}

export function assertLightThemeForCapture(): string | null {
  const t = collectThemeDiagnostics();
  if (t.dataTheme !== "light") {
    return `expected data-theme=light, got ${t.dataTheme ?? "null"}`;
  }
  const base = normaliseHex(t.ubBase);
  const highContrast = document.documentElement.dataset.highContrast === "true";
  const expectedBase =
    base === LIGHT_UB_BASE ||
    base === "rgb(241,240,236)" ||
    (highContrast && (base === "#fff" || base === "#ffffff" || base === "rgb(255,255,255)"));
  if (!expectedBase) {
    return `expected --ub-base ${LIGHT_UB_BASE}, got ${t.ubBase}`;
  }
  return null;
}

export function sampleMonacoCanvasVariance(): EditorCanvasDiagnostics | null {
  const host = document.getElementById("text-editor-host");
  if (!host) {
    return null;
  }
  const canvases = Array.from(host.querySelectorAll(".monaco-editor canvas"));
  if (!canvases.length) {
    return { canvasCount: 0, sampleVariance: 0, sampleWidth: 0, sampleHeight: 0 };
  }
  let bestVar = 0;
  let bestW = 0;
  let bestH = 0;
  for (const canvas of canvases) {
    if (!(canvas instanceof HTMLCanvasElement)) {
      continue;
    }
    const w = Math.min(canvas.width, 240);
    const h = Math.min(canvas.height, 240);
    if (w < 8 || h < 8) {
      continue;
    }
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    if (!ctx) {
      continue;
    }
    try {
      const data = ctx.getImageData(0, 0, w, h).data;
      let sum = 0;
      let sumSq = 0;
      const n = data.length / 4;
      for (let i = 0; i < data.length; i += 4) {
        const lum = (data[i] + data[i + 1] + data[i + 2]) / 3;
        sum += lum;
        sumSq += lum * lum;
      }
      const mean = sum / n;
      const variance = sumSq / n - mean * mean;
      if (variance > bestVar) {
        bestVar = variance;
        bestW = w;
        bestH = h;
      }
    } catch {
      /* tainted or zero-size */
    }
  }
  return {
    canvasCount: canvases.length,
    sampleVariance: bestVar,
    sampleWidth: bestW,
    sampleHeight: bestH,
  };
}

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
  const m = rgb.match(
    /rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([0-9.]+))?/,
  );
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

function shouldCheckFormControlBorder(candidate: HTMLElement): boolean {
  if (
    candidate instanceof HTMLInputElement &&
    (candidate.type === "hidden" ||
      candidate.type === "checkbox" ||
      candidate.type === "radio") &&
    candidate.getAttribute("role") !== "switch"
  ) {
    return false;
  }
  return true;
}

function effectiveOpaqueBackground(el: HTMLElement): string | null {
  let node: HTMLElement | null = el;
  while (node) {
    const background = getComputedStyle(node).backgroundColor;
    const parsed = parseRgb(background);
    if (parsed && parsed.a >= 0.99) {
      return background;
    }
    node = node.parentElement;
  }
  return null;
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

function selectHasChevronBackground(style: CSSStyleDeclaration): boolean {
  const image = style.backgroundImage;
  return Boolean(image && image !== "none" && image.includes("url("));
}

export function validateUbSwitchGeometryForCapture(): string | null {
  const switches = document.querySelectorAll<HTMLInputElement>(
    ".ub-switch, .settings-toggle-input",
  );
  for (const switchEl of switches) {
    const rect = switchEl.getBoundingClientRect();
    const style = getComputedStyle(switchEl);
    if (
      rect.width <= 0 ||
      rect.height <= 0 ||
      style.display === "none" ||
      style.visibility === "hidden"
    ) {
      continue;
    }
    if (!switchTrackSizeWithinTolerance(rect.width, rect.height)) {
      const identity = switchEl.id || switchEl.className || "switch";
      return `switch ${identity} is ${rect.width.toFixed(1)}x${rect.height.toFixed(1)}; expected 32x18 (+/-1px)`;
    }
    const knobStyle = getComputedStyle(switchEl, "::after");
    const knobTop = Number.parseFloat(knobStyle.top);
    const knobHeight = Number.parseFloat(knobStyle.height);
    if (!Number.isFinite(knobTop) || !Number.isFinite(knobHeight)) {
      continue;
    }
    if (!knobVerticalOffsetCentered(rect.height, knobTop, knobHeight)) {
      const identity = switchEl.id || switchEl.className || "switch";
      return `switch ${identity} knob is not vertically centred (top ${knobTop}px, height ${knobHeight}px, track ${rect.height}px)`;
    }
  }
  return null;
}

const CAPTURE_SELECT_SCOPES = [
  "#chrome",
  ".settings-content",
  ".pdf-annotations-dock",
  ".files-panel",
  ".svg-inspector",
  ".workshop-inspector",
  ".ai-assistant-panel",
  ".settings-category-rail",
  ".template-gallery-panel",
];

export function validateSelectChevronForCapture(): string | null {
  for (const scopeSelector of CAPTURE_SELECT_SCOPES) {
    const scope = document.querySelector(scopeSelector);
    if (!scope) {
      continue;
    }
    const selects = scope.querySelectorAll<HTMLSelectElement>("select");
    for (const select of selects) {
      const rect = select.getBoundingClientRect();
      const style = getComputedStyle(select);
      if (
        rect.width <= 0 ||
        rect.height <= 0 ||
        style.display === "none" ||
        style.visibility === "hidden"
      ) {
        continue;
      }
      if (!selectHasChevronBackground(style)) {
        const identity =
          select.id ||
          select.getAttribute("aria-label") ||
          Array.from(select.classList).join(".") ||
          "select";
        return `select ${identity} in ${scopeSelector} is missing themed chevron background`;
      }
    }
  }
  return null;
}

function visibleElement(el: HTMLElement): boolean {
  const rect = el.getBoundingClientRect();
  const style = getComputedStyle(el);
  return (
    rect.width > 0 &&
    rect.height > 0 &&
    style.display !== "none" &&
    style.visibility !== "hidden"
  );
}

function visibleChromeChildHeight(id: string): number {
  const el = document.getElementById(id);
  if (!el || !visibleElement(el)) {
    return 0;
  }
  const style = getComputedStyle(el);
  if (style.display === "none" || style.visibility === "hidden") {
    return 0;
  }
  return el.getBoundingClientRect().height;
}

export function validateResponsiveShellForCapture(): string | null {
  if (document.body.dataset.ubNewProjectCapture === "1") {
    return null;
  }
  const chrome = document.getElementById("chrome");
  const manager = document.getElementById("command-manager");
  const groups = document.getElementById("command-manager-groups");
  if (!chrome || !manager || !groups) {
    return "responsive shell guard is missing chrome or Command Manager";
  }
  const chromeHeight = chrome.getBoundingClientRect().height;
  const expectedChrome =
    visibleChromeChildHeight("title-bar") +
    visibleChromeChildHeight("bookmark-bar") +
    visibleChromeChildHeight("recent-panel") +
    visibleChromeChildHeight("text-compact-toolbar-host") +
    visibleChromeChildHeight("text-mod-toolbar-host") +
    visibleChromeChildHeight("command-manager") +
    visibleChromeChildHeight("floating-toolbar-host");
  if (
    !document.body.classList.contains("cm-collapsed") &&
    Math.abs(chromeHeight - expectedChrome) > 4
  ) {
    return `top chrome is ${chromeHeight.toFixed(1)}px; expected ${expectedChrome.toFixed(1)}px from visible chrome children (+/-4px)`;
  }
  if (
    document.body.dataset.cmDensity === "compact" &&
    !document.body.classList.contains("text-compact-visible") &&
    !document.body.classList.contains("cm-collapsed")
  ) {
    const managerHeight = manager.getBoundingClientRect().height;
    if (Math.abs(managerHeight - 40) > 1) {
      return `compact Command Manager is ${managerHeight.toFixed(1)}px tall; expected 40px (+/-1px)`;
    }
  }
  if (document.body.classList.contains("text-compact-visible")) {
    const managerHeight = manager.getBoundingClientRect().height;
    if (managerHeight > 42) {
      return `text compact Command Manager strip is ${managerHeight.toFixed(1)}px tall; expected 40px (+/-2px)`;
    }
  }
  if (groups.scrollHeight > groups.clientHeight + 1) {
    return `Command Manager wraps or clips vertically (${groups.scrollHeight}px scroll vs ${groups.clientHeight}px client)`;
  }
  const groupsRect = groups.getBoundingClientRect();
  for (const group of groups.querySelectorAll<HTMLElement>(".cm-group")) {
    if (!visibleElement(group)) {
      continue;
    }
    if (!rectWithinBounds(group.getBoundingClientRect(), groupsRect, 1)) {
      return `Command Manager group "${group.dataset.groupId ?? group.textContent?.trim() ?? "unknown"}" is partially clipped`;
    }
  }
  for (const control of groups.querySelectorAll<HTMLElement>(".ui-command")) {
    if (!visibleElement(control)) {
      continue;
    }
    const label = control.querySelector<HTMLElement>(".cm-cmd-label");
    if (!label) {
      continue;
    }
    if (
      label.scrollWidth > label.clientWidth + 1 ||
      !rectWithinBounds(label.getBoundingClientRect(), control.getBoundingClientRect(), 1)
    ) {
      return `Command Manager label "${label.textContent?.trim() ?? ""}" is clipped`;
    }
  }
  const pipeline = document.documentElement.dataset.cmPipeline;
  if (pipeline) {
    try {
      const parsed = JSON.parse(pipeline) as { frontendOverflowModelCount?: number };
      if ((parsed.frontendOverflowModelCount ?? 0) > 0) {
        const overflow = document.getElementById("command-manager-overflow");
        if (!overflow || !visibleElement(overflow)) {
          return "Command Manager has overflowed groups but no visible chevron";
        }
      }
    } catch {
      return "Command Manager pipeline diagnostics are invalid JSON";
    }
  }
  return null;
}

export function validateFocusRingForCapture(): string | null {
  const active = document.activeElement;
  if (!(active instanceof HTMLElement) || active === document.body) {
    return null;
  }
  if (!active.matches("button, input, select, [role='tab'], .ui-command")) {
    return null;
  }
  const style = getComputedStyle(active);
  if (style.outlineStyle === "none") {
    return `focused ${active.id || active.className || active.tagName.toLowerCase()} has no visible outline`;
  }
  const width = Number.parseFloat(style.outlineWidth);
  const surrounding = effectiveOpaqueBackground(active.parentElement ?? active);
  if (!surrounding) {
    return "could not resolve focus-ring surrounding colour";
  }
  if (!focusRingMeetsRequirements(width, style.outlineColor, surrounding)) {
    const ratio = colourContrastRatio(style.outlineColor, surrounding);
    return `focus ring on ${active.id || active.className || active.tagName.toLowerCase()} is ${width}px at ${ratio?.toFixed(2) ?? "unknown"}:1; expected >=2px and >=3:1`;
  }
  return null;
}

export function validateDarkSurfaceBackgroundsForCapture(
  tolerance = 0.06,
): string | null {
  const root = document.documentElement;
  if (root.dataset.theme !== "dark") {
    return null;
  }
  const probe = document.createElement("span");
  probe.style.cssText =
    "position:fixed;left:-10000px;top:-10000px;background:var(--ub-surface-2);";
  document.body.appendChild(probe);
  const raisedBackground = getComputedStyle(probe).backgroundColor;
  probe.remove();
  const raisedLuminance = parseRgbLuminance(raisedBackground);
  if (raisedLuminance === null) {
    return `could not resolve dark raised-surface token (${raisedBackground})`;
  }
  const rootScheme = getComputedStyle(root).colorScheme;
  if (!rootScheme.split(/\s+/).includes("dark")) {
    return `dark theme root is missing color-scheme: dark (got ${rootScheme || "empty"})`;
  }
  const scopes = [
    "#chrome",
    ".files-places",
    ".files-side-panel",
    ".pdf-thumbs-panel",
    ".pdf-annotations-dock",
    ".image-dock",
    ".svg-inspector-rail",
    ".dxf-layer-panel",
    ".dxf-geom-panel",
    ".workshop-sources",
    ".workshop-inspector",
    ".ai-assistant-panel",
    ".settings-category-rail",
  ];
  const candidates = new Set<HTMLElement>();
  for (const scopeSelector of scopes) {
    const scope = document.querySelector(scopeSelector);
    if (!scope) {
      continue;
    }
    scope
      .querySelectorAll<HTMLElement>(
        'input:not([type="hidden"]), textarea, select, select option, [class*="chip"], [class*="card"]',
      )
      .forEach((candidate) => candidates.add(candidate));
  }
  document.querySelectorAll<HTMLElement>(
    "select option, .ub-select-list, .ub-select-option",
  ).forEach((node) => {
    candidates.add(node);
  });
  for (const candidate of candidates) {
    const rect = candidate.getBoundingClientRect();
    const style = getComputedStyle(candidate);
    const isOption =
      candidate instanceof HTMLOptionElement ||
      candidate.classList.contains("ub-select-option") ||
      candidate.classList.contains("ub-select-list");
    if (
      (!isOption && (rect.width <= 0 || rect.height <= 0)) ||
      style.display === "none" ||
      style.visibility === "hidden"
    ) {
      continue;
    }
    if (candidate instanceof HTMLInputElement && candidate.checked) {
      continue;
    }
    const background = effectiveOpaqueBackground(candidate);
    const luminance = background ? parseRgbLuminance(background) : null;
    if (
      luminance !== null &&
      backgroundExceedsRaisedSurface(luminance, raisedLuminance, tolerance)
    ) {
      const identity =
        candidate.id ||
        Array.from(candidate.classList).join(".") ||
        candidate.tagName.toLowerCase();
      return `dark surface ${identity} is brighter than --ub-surface-2 (background ${background}, raised ${raisedBackground})`;
    }
    const isFormField =
      candidate instanceof HTMLInputElement ||
      candidate instanceof HTMLTextAreaElement ||
      candidate instanceof HTMLSelectElement;
    if (!isFormField || !shouldCheckFormControlBorder(candidate)) {
      continue;
    }
    if (style.boxShadow && style.boxShadow !== "none") {
      const identity =
        candidate.id ||
        Array.from(candidate.classList).join(".") ||
        candidate.tagName.toLowerCase();
      return `dark form control ${identity} must not use box-shadow (${style.boxShadow})`;
    }
    if (!inputBorderEdgesUniform(style)) {
      const identity =
        candidate.id ||
        Array.from(candidate.classList).join(".") ||
        candidate.tagName.toLowerCase();
      return `dark form control ${identity} has uneven border colours (top ${style.borderTopColor}, right ${style.borderRightColor}, bottom ${style.borderBottomColor}, left ${style.borderLeftColor})`;
    }
  }
  return null;
}

export function validateWorkshopTimelineForCapture(): string | null {
  const viewport = document.querySelector<HTMLElement>(".workshop-timeline-viewport");
  const ruler = document.querySelector<HTMLElement>(".workshop-ruler");
  if (!viewport || !ruler) {
    return null;
  }
  const viewportRect = viewport.getBoundingClientRect();
  const rulerRect = ruler.getBoundingClientRect();
  if (!rectWithinBounds(rulerRect, viewportRect)) {
    return "workshop ruler escapes the timeline viewport";
  }
  const ticks = Array.from(
    ruler.querySelectorAll<HTMLElement>(".workshop-ruler-tick"),
  );
  for (const tick of ticks) {
    if (!rectWithinBounds(tick.getBoundingClientRect(), viewportRect)) {
      return `workshop ruler tick "${tick.textContent?.trim() ?? ""}" escapes the timeline viewport`;
    }
  }
  const lastTick = ticks[ticks.length - 1];
  if (
    lastTick &&
    !timelineEndpointClearsScrollbar(lastTick.getBoundingClientRect().right, viewportRect.right)
  ) {
    return `workshop ruler endpoint "${lastTick.textContent?.trim() ?? ""}" does not clear the scrollbar safe inset`;
  }
  const markerTags = Array.from(
    ruler.querySelectorAll<HTMLElement>(".workshop-marker-tag"),
  );
  for (const tag of markerTags) {
    const tagRect = tag.getBoundingClientRect();
    if (!rectWithinBounds(tagRect, viewportRect)) {
      return `workshop marker "${tag.textContent?.trim() ?? ""}" escapes the timeline viewport`;
    }
    const overlappingTick = ticks.find((tick) =>
      captureRectsOverlap(tagRect, tick.getBoundingClientRect()),
    );
    if (overlappingTick) {
      return `workshop marker "${tag.textContent?.trim() ?? ""}" overlaps ruler tick "${overlappingTick.textContent?.trim() ?? ""}"`;
    }
  }
  return null;
}

export function collectMonacoThemeDiagnostics(): MonacoThemeCaptureDiagnostics | null {
  const host = document.getElementById("text-editor-host");
  if (!host || host.classList.contains("hidden")) {
    return null;
  }
  const bgEl = host.querySelector(".monaco-editor-background") as HTMLElement | null;
  const root = host.querySelector(".monaco-editor") as HTMLElement | null;
  const line = host.querySelector(".view-line span") as HTMLElement | null;
  const bgStyle = bgEl ? getComputedStyle(bgEl) : root ? getComputedStyle(root) : null;
  const lineStyle = line ? getComputedStyle(line) : null;
  return {
    resolvedTheme: document.documentElement.dataset.monacoThemeResolved ?? null,
    themeColorsJson: document.documentElement.dataset.monacoThemeColors ?? null,
    themeDtoJson: document.documentElement.dataset.monacoThemeDto ?? null,
    editorBackground: bgStyle?.backgroundColor ?? "",
    editorForeground: bgStyle?.color ?? "",
    viewLineColor: lineStyle?.color ?? "",
    viewLineBackground: lineStyle?.backgroundColor ?? "",
    viewLineFontFamily: lineStyle?.fontFamily ?? "",
    viewLineFontSize: lineStyle?.fontSize ?? "",
    sourceCodeProLoaded: document.fonts.check('12px "Source Code Pro"'),
    fallbackMonoLoaded: document.fonts.check('12px "DejaVu Sans Mono"'),
  };
}

export function collectCommandManagerDiagnostics(): CommandManagerCaptureDiagnostics {
  const controls = Array.from(
    document.querySelectorAll("#command-manager-groups .ui-command"),
  ) as HTMLElement[];
  let visibleCount = 0;
  let contrastOkCount = 0;
  const controlDiagnostics: CommandManagerCaptureDiagnostics["controls"] = [];
  for (const el of controls) {
    const rect = el.getBoundingClientRect();
    const style = getComputedStyle(el);
    const label = el.querySelector(".cm-cmd-label") as HTMLElement | null;
    const icon = el.querySelector("svg") as SVGElement | null;
    const labelRect = label?.getBoundingClientRect();
    const iconRect = icon?.getBoundingClientRect();
    let backgroundNode: HTMLElement | null = el;
    let background = style.backgroundColor;
    while (backgroundNode) {
      const candidate = getComputedStyle(backgroundNode).backgroundColor;
      const parsed = parseRgb(candidate);
      if (parsed && parsed.a >= 0.99) {
        background = candidate;
        break;
      }
      backgroundNode = backgroundNode.parentElement;
    }
    const fg = parseRgbLuminance(style.color);
    const effectiveBg = parseRgbLuminance(background);
    const contrast =
      fg !== null && effectiveBg !== null ? contrastRatio(fg, effectiveBg) : null;
    const hasGeometry =
      rect.width >= 24 &&
      rect.height >= 20 &&
      (labelRect?.width ?? 0) >= 4 &&
      (iconRect?.width ?? 0) >= 12;
    const mutedUnsupported = el.classList.contains("unsupported");
    const visiblyStyled =
      style.visibility !== "hidden" &&
      style.display !== "none" &&
      (Number(style.opacity) >= 0.8 || mutedUnsupported);
    if (hasGeometry && visiblyStyled) {
      visibleCount += 1;
    }
    if (hasGeometry && visiblyStyled && contrast !== null && contrast >= 3) {
      contrastOkCount += 1;
    }
    controlDiagnostics.push({
      controlId: el.dataset.controlId ?? el.dataset.commandId ?? "",
      label: label?.textContent?.trim() ?? "",
      classes: Array.from(el.classList),
      capabilityState: el.dataset.capabilityState ?? "",
      enabled:
        el.dataset.capabilityState === "supported" ||
        el.dataset.capabilityState === "emulated",
      disabledReason: el.getAttribute("aria-description"),
      width: rect.width,
      height: rect.height,
      labelBounds: {
        x: labelRect?.x ?? rect.x,
        y: labelRect?.y ?? rect.y,
        width: labelRect?.width ?? rect.width,
        height: labelRect?.height ?? rect.height,
      },
      labelWidth: labelRect?.width ?? 0,
      iconWidth: iconRect?.width ?? 0,
      opacity: style.opacity,
      filter: style.filter,
      foreground: style.color,
      labelForeground: label ? getComputedStyle(label).color : style.color,
      background,
      contrast,
      chromeTextVariable: getComputedStyle(el)
        .getPropertyValue("--ub-concept-chrome-text")
        .trim(),
      ancestors: collectCommandAncestors(el),
    });
  }
  const contextEl = document.getElementById("command-manager-context");
  let pipeline: Record<string, unknown> | null = null;
  try {
    pipeline = JSON.parse(document.documentElement.dataset.cmPipeline ?? "null");
  } catch {
    pipeline = null;
  }
  return {
    viewportWidth: window.innerWidth,
    viewportHeight: window.innerHeight,
    controlCount: controls.length,
    visibleCount,
    contrastOkCount,
    contextLabel: contextEl?.textContent?.trim() ?? "",
    autoContext: document.body.dataset.cmAutoContext ?? null,
    pipeline,
    controls: controlDiagnostics,
  };
}

export function collectStatusBarDiagnostics(): StatusBarCaptureDiagnostics {
  const host = document.getElementById("status-bar");
  if (!host) {
    return { segmentCount: 0, segments: [] };
  }
  const segments = Array.from(
    host.querySelectorAll(".status-item, .status-utility:not(.hidden), .status-notifications-btn:not(.hidden)"),
  )
    .map((node) => {
      const el = node as HTMLElement;
      const rect = el.getBoundingClientRect();
      const style = getComputedStyle(el);
      const label = el.textContent?.trim() ?? "";
      if (
        !label ||
        rect.width <= 0 ||
        rect.height <= 0 ||
        style.display === "none" ||
        style.visibility === "hidden"
      ) {
        return null;
      }
      let backgroundNode: HTMLElement | null = el;
      let background = style.backgroundColor;
      while (backgroundNode) {
        const candidate = getComputedStyle(backgroundNode).backgroundColor;
        const parsed = parseRgb(candidate);
        if (parsed && parsed.a >= 0.99) {
          background = candidate;
          break;
        }
        backgroundNode = backgroundNode.parentElement;
      }
      const foregroundLuminance = parseRgbLuminance(style.color);
      const backgroundLuminance = parseRgbLuminance(background);
      return {
        id: el.id,
        label,
        bounds: {
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
        },
        foreground: style.color,
        background,
        contrast:
          foregroundLuminance !== null && backgroundLuminance !== null
            ? contrastRatio(foregroundLuminance, backgroundLuminance)
            : null,
        opacity: style.opacity,
        filter: style.filter,
      };
    })
    .filter((segment): segment is NonNullable<typeof segment> => segment !== null);
  return { segmentCount: segments.length, segments };
}

function collectCommandAncestors(el: HTMLElement) {
  const ancestors: CommandManagerCaptureDiagnostics["controls"][number]["ancestors"] = [];
  let node: HTMLElement | null = el.parentElement;
  while (node) {
    const style = getComputedStyle(node);
    ancestors.push({
      tag: node.tagName.toLowerCase(),
      id: node.id,
      classes: Array.from(node.classList),
      opacity: style.opacity,
      filter: style.filter,
      color: style.color,
      background: style.backgroundColor,
    });
    if (node.id === "chrome" || node.tagName === "BODY") {
      break;
    }
    node = node.parentElement;
  }
  return ancestors;
}

export function validateCommandContextLabel(expected: string): string | null {
  const label =
    document.getElementById("command-manager-context")?.textContent?.trim() ?? "";
  if (label !== expected) {
    return `expected CM context "${expected}", got "${label}"`;
  }
  return null;
}

export function validateProblemsForCapture(minRows = 3): string | null {
  const panel = document.getElementById("problems-panel");
  if (!panel) {
    return "problems panel missing";
  }
  const rows = Array.from(
    panel.querySelectorAll(".problems-table tbody tr"),
  ) as HTMLElement[];
  const visibleRows = rows.filter((row) => {
    const rect = row.getBoundingClientRect();
    const style = getComputedStyle(row);
    return rect.width > 0 && rect.height > 0 && style.display !== "none";
  });
  const empty = panel.querySelector(".problems-empty") as HTMLElement | null;
  const emptyVisible = Boolean(
    empty &&
      !empty.classList.contains("hidden") &&
      getComputedStyle(empty).display !== "none",
  );
  if (emptyVisible) {
    return "problems empty state is visible with populated evidence";
  }
  if (visibleRows.length < minRows) {
    return `visible problem rows ${visibleRows.length} < ${minRows}`;
  }
  const tableWrap = panel.querySelector(".problems-table-wrap") as HTMLElement | null;
  const tableRect = tableWrap?.getBoundingClientRect() ?? panel.getBoundingClientRect();
  const borderTolerance = Math.max(
    1,
    Number.parseFloat(getComputedStyle(tableWrap ?? panel).borderBottomWidth) || 0,
  );
  const escapedRow = visibleRows.find((row) => {
      const rect = row.getBoundingClientRect();
      return rect.top < tableRect.top || rect.bottom > tableRect.bottom + borderTolerance;
    });
  if (escapedRow) {
    const rect = escapedRow.getBoundingClientRect();
    return `problem row escapes the problems panel (row ${rect.top.toFixed(1)}..${rect.bottom.toFixed(1)}, panel ${tableRect.top.toFixed(1)}..${tableRect.bottom.toFixed(1)}, tolerance ${borderTolerance})`;
  }
  return null;
}

export function collectDocCaptureDiagnostics(): DocCaptureDiagnostics {
  const host = document.getElementById("text-editor-host");
  const rect = host?.getBoundingClientRect();
  const pointStack = rect
    ? document
        .elementsFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2)
        .slice(0, 12)
        .map((el) => {
          const style = getComputedStyle(el);
          const elRect = el.getBoundingClientRect();
          return {
            tag: el.tagName.toLowerCase(),
            id: el.id,
            classes: el.className,
            display: style.display,
            visibility: style.visibility,
            position: style.position,
            background: style.backgroundColor,
            width: elRect.width,
            height: elRect.height,
          };
        })
    : [];
  const problemsPanel = document.getElementById("problems-panel");
  const problemsTable = problemsPanel?.querySelector(".problems-table") as HTMLElement | null;
  const problemRows = Array.from(
    problemsPanel?.querySelectorAll(".problems-table tbody tr") ?? [],
  ) as HTMLElement[];
  const empty = problemsPanel?.querySelector(".problems-empty") as HTMLElement | null;
  const documentViewer = document.getElementById("document-viewer-host");
  const viewerImage = documentViewer?.querySelector<HTMLImageElement>(".image-view") ?? null;
  const imageStage =
    documentViewer?.querySelector<HTMLElement>(".image-stage") ?? null;
  return {
    theme: collectThemeDiagnostics(),
    monaco: collectMonacoDiagnostics(),
    canvas: sampleMonacoCanvasVariance(),
    monacoTheme: collectMonacoThemeDiagnostics(),
    commandManager: collectCommandManagerDiagnostics(),
    statusBar: collectStatusBarDiagnostics(),
    problems: {
      tabText:
        document.getElementById("bottom-panel-tab-problems")?.textContent?.trim() ?? "",
      rowCount: problemRows.length,
      visibleRowCount: problemRows.filter((row) => {
        const rowRect = row.getBoundingClientRect();
        const style = getComputedStyle(row);
        return rowRect.width > 0 && rowRect.height > 0 && style.display !== "none";
      }).length,
      emptyVisible: Boolean(
        empty &&
          !empty.classList.contains("hidden") &&
          getComputedStyle(empty).display !== "none",
      ),
      panelHeight: problemsPanel?.getBoundingClientRect().height ?? 0,
      tableHeight: problemsTable?.getBoundingClientRect().height ?? 0,
    },
    documentViewer: documentViewer
      ? {
          hidden: documentViewer.classList.contains("hidden"),
          childClasses: Array.from(
            documentViewer.querySelectorAll<HTMLElement>(":scope > *, .viewer-body > *"),
            (child) => child.className,
          ),
          imageComplete: viewerImage?.complete ?? null,
          imageNaturalWidth: viewerImage?.naturalWidth ?? null,
          imageRenderedWidth: viewerImage?.getBoundingClientRect().width ?? null,
          imageStageTransform: imageStage?.style.transform || null,
        }
      : null,
    editorPointStack: pointStack,
  };
}

function rgbTripletMatches(a: string, b: string): boolean {
  return a.trim().toLowerCase() === b.trim().toLowerCase();
}

function colorsMatchCss(a: string, b: string): boolean {
  if (rgbTripletMatches(a, b)) {
    return true;
  }
  const parsedA = parseRgb(a);
  const parsedB = parseRgb(b);
  if (!parsedA || !parsedB) {
    return false;
  }
  return parsedA.r === parsedB.r && parsedA.g === parsedB.g && parsedA.b === parsedB.b;
}

function footerUsesDocumentCardTokens(footerMain: HTMLElement | null): boolean {
  if (!footerMain || !document.body.classList.contains("text-document-card")) {
    return false;
  }
  const statusHost = footerMain.closest("#document-card-status");
  if (!(statusHost instanceof HTMLElement)) {
    return false;
  }
  const footerStyle = getComputedStyle(footerMain);
  const statusStyle = getComputedStyle(statusHost);
  const bodyStyle = getComputedStyle(document.body);
  const mutedToken = bodyStyle.getPropertyValue("--ub-doc-card-muted").trim();
  const inkToken = bodyStyle.getPropertyValue("--ub-doc-card-ink").trim();
  const color = footerStyle.color || statusStyle.color;
  const colorOk =
    color !== "rgb(0, 0, 0)" &&
    Boolean(
      mutedToken &&
        (colorsMatchCss(color, mutedToken) || colorsMatchCss(color, inkToken)),
    );
  const fontOk = statusStyle.fontSize === "12px";
  const textOk = /\bwords?\b/i.test(footerMain.textContent ?? "");
  return colorOk && fontOk && textOk;
}

function editorHostForMode(
  mode: "wysiwyg" | "source" | "split" | "reading",
): "monaco" | "rich" | "split" | "none" {
  const monacoHost = document.getElementById("text-editor-host");
  const richHost = document.getElementById("rich-text-editor-host");
  const splitHost = document.getElementById("document-split-preview-host");
  const monacoVisible = Boolean(
    monacoHost &&
      !monacoHost.classList.contains("hidden") &&
      !monacoHost.classList.contains("document-surface-source-hidden"),
  );
  const richVisible = Boolean(
    richHost &&
      !richHost.classList.contains("hidden") &&
      richHost.querySelector(".rich-text-doc"),
  );
  const splitVisible = Boolean(
    splitHost &&
      !splitHost.classList.contains("hidden") &&
      splitHost.querySelector(".rich-text-doc"),
  );
  if (mode === "split") {
    return splitVisible && monacoVisible ? "split" : "none";
  }
  if (mode === "source") {
    return monacoVisible && !richVisible ? "monaco" : "none";
  }
  if (mode === "wysiwyg" || mode === "reading") {
    return richVisible && !monacoVisible ? "rich" : "none";
  }
  return "none";
}

const RICH_EDITOR_STRAY_SELECTOR =
  ".monaco-editor, .view-lines, .view-line, .md-preview-body, .md-preview-shell, #document-split-preview-host";

function richEditorSubtreeViolationCount(
  bodyMount: HTMLElement,
  mode: string,
): number {
  if (mode !== "wysiwyg" && mode !== "reading") {
    return 0;
  }
  const richRoot = document.getElementById("rich-text-editor-host");
  if (!richRoot || richRoot.classList.contains("hidden")) {
    return 0;
  }
  const rect = bodyMount.getBoundingClientRect();
  if (rect.width < 40 || rect.height < 40) {
    return 0;
  }
  const xs = [
    rect.left + rect.width * 0.25,
    rect.left + rect.width / 2,
    rect.left + rect.width * 0.75,
  ];
  const ys = [
    rect.top + 16,
    rect.top + rect.height * 0.35,
    rect.top + rect.height * 0.65,
  ];
  let violations = 0;
  for (const y of ys) {
    for (const x of xs) {
      let reachedRich = false;
      for (const el of document.elementsFromPoint(x, y)) {
        if (!(el instanceof HTMLElement)) {
          continue;
        }
        if (!bodyMount.contains(el)) {
          break;
        }
        if (richRoot.contains(el)) {
          reachedRich = true;
          break;
        }
        if (!visibleElement(el)) {
          continue;
        }
        if (
          el.id === "text-editor-host" ||
          el.id === "document-card-body-mount" ||
          el.classList.contains("document-monaco-slot") ||
          el.classList.contains("rich-text-editor-shell")
        ) {
          continue;
        }
        if (el.closest(RICH_EDITOR_STRAY_SELECTOR)) {
          violations += 1;
          break;
        }
      }
      if (!reachedRich) {
        violations += 1;
      }
    }
  }
  return violations;
}

export interface DocumentFeatureCaptureDomDiagnostics {
  visibleDocumentCardCount: number;
  commandManagerVisible: boolean;
  footerUsesCardTokens: boolean;
  activeMode: string;
  visibleEditorHost: string;
  modeMatchesVisibleHost: boolean;
  outsideOverlapCount: number;
  richEditorSubtreeViolations: number;
  documentCardFooterText: string;
  docxEmulationBadgeVisible: boolean;
  cardImageNaturalWidthOk: boolean;
  statusBarDocumentLanguage: string;
  projectHomeCardHeight: number | null;
  projectHomeCardTopRelative: number | null;
  projectHomeCardCenterDeltaX: number | null;
  horizontalOverflow: boolean;
  presentationToolbarThemed: boolean;
  relationshipConsentPanelHeight: number | null;
  relationshipConsentUsesListBullets: boolean;
  relationshipConsentSectionsScrollable: boolean;
  relationshipConsentPanelScrollable: boolean;
  relationshipConsentActionsInViewport: boolean;
  dialogPrimaryButtonMatchesAccent: boolean;
  firstSaveCancelButtonPresent: boolean;
  historyCaptureReady?: boolean;
  historyToolbarVisible?: boolean;
  historyRestoreVisible?: boolean;
  historyDocHasEditThree?: boolean;
  historyTopSummary?: string;
}

export function collectDocumentFeatureCaptureDom(): DocumentFeatureCaptureDomDiagnostics {
  const cards = Array.from(
    document.querySelectorAll<HTMLElement>(".document-card, .document-card-shell"),
  ).filter(visibleElement);
  const cm = document.getElementById("command-manager");
  const commandManagerVisible = Boolean(
    cm &&
      visibleElement(cm) &&
      !document.body.classList.contains("cm-collapsed") &&
      cm.getBoundingClientRect().height > 44 &&
      cm.getBoundingClientRect().width > 200,
  );
  const footerMain = document.querySelector<HTMLElement>(".document-card-status-main");
  const footerUsesCardTokens = footerUsesDocumentCardTokens(footerMain);
  const documentCardFooterText = footerMain?.textContent?.trim() ?? "";
  const approxBadge = document.querySelector<HTMLElement>(
    ".block-emulation-badge, .cm-emulation-mark",
  );
  const docxEmulationBadgeVisible = Boolean(approxBadge?.textContent?.includes("≈"));
  const richSurface =
    document.querySelector<HTMLElement>("#rich-text-editor-host:not(.hidden) .rich-text-doc") ??
    document.querySelector<HTMLElement>(
      "#document-split-preview-host:not(.hidden) .rich-text-doc",
    );
  const richImgs = richSurface
    ? Array.from(richSurface.querySelectorAll<HTMLImageElement>("img"))
    : [];
  const cardImageNaturalWidthOk =
    richImgs.length === 0 || richImgs.every((img) => img.naturalWidth > 0);
  const statusBarDocumentLanguage = Array.from(
    document.querySelectorAll<HTMLElement>("#status-bar .status-item"),
  )
    .map((el) => el.textContent?.trim() ?? "")
    .join(" ");
  const activeMode = document.body.dataset.documentEditorMode ?? "";
  const visibleEditorHost = editorHostForMode(
    (activeMode as "wysiwyg" | "source" | "split" | "reading") || "wysiwyg",
  );
  const modeMatchesVisibleHost =
    (activeMode === "source" && visibleEditorHost === "monaco") ||
    (activeMode === "split" && visibleEditorHost === "split") ||
    ((activeMode === "wysiwyg" || activeMode === "reading") && visibleEditorHost === "rich");
  const bodyMount = document.getElementById("document-card-body-mount");
  const bodyRect = bodyMount?.getBoundingClientRect();
  const richEditorSubtreeViolations = bodyMount
    ? richEditorSubtreeViolationCount(bodyMount, activeMode)
    : 0;
  let outsideOverlapCount = 0;
  if (bodyRect && bodyRect.width > 40 && bodyRect.height > 40) {
    const shell = document.getElementById("document-card-shell");
    const sampleYs = [
      bodyRect.top + 8,
      bodyRect.top + bodyRect.height * 0.35,
      bodyRect.top + bodyRect.height * 0.65,
    ];
    for (const y of sampleYs) {
      const x = bodyRect.left + bodyRect.width / 2;
      for (const el of document.elementsFromPoint(x, y)) {
        if (!(el instanceof HTMLElement)) {
          continue;
        }
        if (shell?.contains(el) || el === shell || el === bodyMount) {
          break;
        }
        if (el.id === "command-manager" || el.closest("#command-manager")) {
          outsideOverlapCount += 1;
          break;
        }
        if (
          el.id === "text-editor-host" &&
          el.classList.contains("document-surface-source-hidden")
        ) {
          outsideOverlapCount += 1;
          break;
        }
        if (el.id === "document-viewer-host" && !el.classList.contains("hidden")) {
          outsideOverlapCount += 1;
          break;
        }
      }
    }
  }
  const homeCard = document.querySelector<HTMLElement>(".project-home-card");
  const homeRect = homeCard?.getBoundingClientRect();
  const projectHomeCardHeight = homeRect ? homeRect.height : null;
  const workspaceEl = workspaceRootForCapture();
  const workRect = workspaceEl?.getBoundingClientRect();
  let projectHomeCardTopRelative: number | null = null;
  let projectHomeCardCenterDeltaX: number | null = null;
  if (homeRect && workRect && workRect.width > 40) {
    projectHomeCardTopRelative = homeRect.top - workRect.top;
    const cardCenterX = (homeRect.left + homeRect.right) / 2;
    const workCenterX = (workRect.left + workRect.right) / 2;
    projectHomeCardCenterDeltaX = Math.abs(cardCenterX - workCenterX);
  }
  const horizontalOverflow =
    document.documentElement.scrollWidth > document.documentElement.clientWidth + 1;
  const presentationToolbar = document.querySelector(".presentation-toolbar");
  const presentationToolbarThemed =
    !presentationToolbar ||
    Array.from(presentationToolbar.querySelectorAll<HTMLButtonElement>("button")).every(
      (btn) =>
        btn.classList.contains("presentation-toolbar-btn") &&
        Boolean(btn.querySelector(".ub-icon[data-lucide]")),
    );

  const consentPanel = document.querySelector<HTMLElement>(".relationship-consent-panel");
  const consentRect = consentPanel?.getBoundingClientRect();
  const relationshipConsentPanelHeight = consentRect ? consentRect.height : null;
  let relationshipConsentUsesListBullets = false;
  if (consentPanel) {
    for (const list of consentPanel.querySelectorAll("ul, ol")) {
      const style = getComputedStyle(list);
      if (style.listStyleType !== "none" && style.listStyleType !== "") {
        relationshipConsentUsesListBullets = true;
        break;
      }
      if (list.querySelector("li") && style.listStylePosition !== "inside") {
        const liStyle = getComputedStyle(list.querySelector("li")!);
        if (liStyle.display === "list-item") {
          relationshipConsentUsesListBullets = true;
          break;
        }
      }
    }
  }
  const firstSavePanel = document.querySelector<HTMLElement>(".first-save-panel");
  const firstSaveCancelButtonPresent = Boolean(
    firstSavePanel?.querySelector(".first-save-actions .quiet-button"),
  );
  const consentSections = consentPanel?.querySelector<HTMLElement>(
    ".relationship-consent-sections",
  );
  const relationshipConsentSectionsScrollable = Boolean(
    consentSections &&
      consentSections.scrollHeight > consentSections.clientHeight + 2,
  );
  const relationshipConsentPanelScrollable = Boolean(
    consentPanel && consentPanel.scrollHeight > consentPanel.clientHeight + 1,
  );
  const consentCancel = consentPanel?.querySelector<HTMLElement>(
    ".relationship-consent-actions .quiet-button",
  );
  const consentCreate = consentPanel?.querySelector<HTMLElement>(
    ".relationship-consent-actions button.primary",
  );
  const relationshipConsentActionsInViewport = Boolean(
    consentCancel &&
      consentCreate &&
      elementFullyInViewport(consentCancel) &&
      elementFullyInViewport(consentCreate),
  );
  const dialogPrimaryButtonMatchesAccent =
    dialogPrimaryButtonUsesAccent(consentPanel) ||
    dialogPrimaryButtonUsesAccent(firstSavePanel);

  const richDocText = document.querySelector(".rich-text-doc")?.textContent ?? "";
  const historyTopSummary =
    document.querySelector(".history-panel-table tbody tr:first-child .col-summary")
      ?.textContent?.trim() ?? "";
  const historyHeaderCount = document.querySelectorAll(".history-panel-table thead").length;
  const historyDemo = (document.body.dataset.ubDemoTokens ?? "").includes("doc-history-");

  return {
    visibleDocumentCardCount: cards.length,
    commandManagerVisible,
    footerUsesCardTokens,
    activeMode,
    visibleEditorHost,
    modeMatchesVisibleHost,
    outsideOverlapCount,
    richEditorSubtreeViolations,
    documentCardFooterText,
    docxEmulationBadgeVisible,
    cardImageNaturalWidthOk,
    statusBarDocumentLanguage,
    projectHomeCardHeight,
    projectHomeCardTopRelative,
    projectHomeCardCenterDeltaX,
    horizontalOverflow,
    presentationToolbarThemed,
    relationshipConsentPanelHeight,
    relationshipConsentUsesListBullets,
    relationshipConsentSectionsScrollable,
    relationshipConsentPanelScrollable,
    relationshipConsentActionsInViewport,
    dialogPrimaryButtonMatchesAccent,
    firstSaveCancelButtonPresent,
    ...(historyDemo
      ? {
          historyCaptureReady: document.body.dataset.ubHistoryCaptureReady === "1",
          historyToolbarVisible: Boolean(document.getElementById("history-action-trim")),
          historyRestoreVisible: Boolean(
            document.querySelector(".history-panel-table tr.is-selected .history-restore-btn"),
          ),
          historyHeaderCount,
          historyCurrentOnHead: Boolean(
            document.querySelector(
              ".history-panel-table tbody tr:first-child .history-current-pill",
            ),
          ),
          historyDocHasEditThree: richDocText.includes("Edit three"),
          historyTopSummary,
        }
      : {}),
  };
}

export function validateDocumentFeatureCaptureDom(): string | null {
  const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
  if (
    !demo.includes("doc-mode-") &&
    !demo.includes("doc-opaque-card") &&
    !demo.includes("doc-history-") &&
    !demo.includes("doc-merge-banner") &&
    !demo.includes("doc-visual-md-docx")
  ) {
    return null;
  }
  const dom = collectDocumentFeatureCaptureDom();
  if (dom.visibleDocumentCardCount !== 1) {
    return `expected exactly one visible document card, found ${dom.visibleDocumentCardCount}`;
  }
  if (dom.commandManagerVisible) {
    return "command manager row visible during doc feature capture";
  }
  if (!dom.footerUsesCardTokens) {
    return "document card footer is not styled by card tokens";
  }
  if (!dom.modeMatchesVisibleHost) {
    return `active mode ${dom.activeMode} does not match visible editor host (${dom.visibleEditorHost})`;
  }
  if (dom.outsideOverlapCount > 0) {
    return `outside chrome overlaps the document card body (${dom.outsideOverlapCount} hits)`;
  }
  if (dom.richEditorSubtreeViolations > 0) {
    return `card body hit-test must stay inside rich editor (${dom.richEditorSubtreeViolations} violations)`;
  }
  if (demo.includes("doc-visual-md-docx")) {
    const footerMain = document.querySelector<HTMLElement>(".document-card-status-main");
    const footerText = footerMain?.textContent ?? "";
    if (!footerText.includes("Word")) {
      return "doc-visual-md-docx capture footer must show Word kind";
    }
    const approx = document.querySelector<HTMLElement>(
      ".block-emulation-badge, .cm-emulation-mark",
    );
    if (!approx?.textContent?.includes("≈")) {
      return "doc-visual-md-docx capture must show emulated block-style (≈) badge";
    }
    if (!dom.cardImageNaturalWidthOk) {
      const richSurface =
        document.querySelector<HTMLElement>("#rich-text-editor-host:not(.hidden) .rich-text-doc") ??
        document.querySelector<HTMLElement>(
          "#document-split-preview-host:not(.hidden) .rich-text-doc",
        );
      const imgs = richSurface
        ? Array.from(richSurface.querySelectorAll<HTMLImageElement>("img")).map(
            (img) =>
              `nw=${img.naturalWidth} src=${(img.currentSrc || img.src || "").slice(0, 96)}`,
          )
        : [];
      return `doc-visual-md-docx capture requires every card image to load (naturalWidth > 0); ${imgs.join("; ")}`;
    }
    if (!dom.statusBarDocumentLanguage.includes("Word document")) {
      return "doc-visual-md-docx capture status bar must show Word document";
    }
  }
  return null;
}

/** Doc-feature captures must use the paper card + compact toolbar, not full CM or rail flyouts. */
export function validateDocumentFeatureCaptureChrome(): string | null {
  const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
  if (
    !demo.includes("doc-mode-") &&
    !demo.includes("doc-opaque-card") &&
    !demo.includes("doc-history-") &&
    !demo.includes("doc-merge-banner") &&
    !demo.includes("doc-visual-docx") &&
    !demo.includes("doc-visual-md-docx")
  ) {
    return null;
  }
  if (demo.includes("doc-visual-md-docx")) {
    if (!document.body.classList.contains("text-document-card")) {
      return "doc-visual-md-docx capture requires text-document-card layout";
    }
  }
  if (!document.body.classList.contains("text-document-card")) {
    return "document feature capture requires text-document-card layout";
  }
  const shell = document.getElementById("document-card-shell");
  if (!shell || shell.classList.contains("hidden")) {
    return "document-card-shell hidden during doc feature capture";
  }
  const flyout = document.querySelector<HTMLElement>(".rail-side-panel:not(.hidden)");
  if (flyout && flyout.getBoundingClientRect().width > 8) {
    return "activity rail flyout visible during doc feature capture";
  }
  const cm = document.getElementById("command-manager");
  if (cm && !document.body.classList.contains("cm-collapsed")) {
    const rect = cm.getBoundingClientRect();
    if (rect.height > 44 && rect.width > 200) {
      return "full command manager row visible during doc feature capture";
    }
  }
  const pinTip = document.querySelector<HTMLElement>(".cm-pin-tip:not(.hidden)");
  if (pinTip) {
    return "command manager pin tip visible during doc feature capture";
  }
  return null;
}

/** Returns abort reason or null if CM shows enough visible, contrasting controls. */
export function validateCommandManagerForCapture(minVisible = 6, minContrast = 6): string | null {
  if (document.body.dataset.ubNewProjectCapture === "1") {
    return null;
  }
  if (document.body.classList.contains("cm-collapsed")) {
    return null;
  }
  const demoTokens = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
  if (demoTokens.includes("doc-opaque-card") || demoTokens.includes("doc-history-")) {
    return null;
  }
  const cm = collectCommandManagerDiagnostics();
  const startContext =
    cm.autoContext === "start" || cm.contextLabel?.toLowerCase() === "start";
  const settingsContext = cm.contextLabel?.toLowerCase() === "settings";
  const modsContext = cm.contextLabel?.toLowerCase() === "mods";
  const mediaContext = cm.contextLabel?.toLowerCase() === "audio";
  const sparseContextLimit = mediaContext ? 1 : settingsContext ? 2 : modsContext ? 5 : 4;
  const textCompactContext = document.body.classList.contains("text-compact-visible");
  const documentCardContext = document.body.classList.contains("text-document-card");
  const sparseContext = startContext || settingsContext || modsContext || mediaContext;
  let requiredVisible = sparseContext ? Math.min(minVisible, sparseContextLimit) : minVisible;
  let requiredContrast = sparseContext ? Math.min(minContrast, sparseContextLimit) : minContrast;
  if (documentCardContext) {
    return null;
  }
  if (textCompactContext) {
    requiredVisible = Math.min(requiredVisible, 5);
    requiredContrast = Math.min(requiredContrast, 5);
  }
  const overflowEl = document.getElementById("command-manager-overflow");
  const resolvedTotal =
    typeof cm.pipeline?.ipcControlCount === "number"
      ? (cm.pipeline.ipcControlCount as number)
      : cm.controlCount;
  const overflowModelCount =
    typeof cm.pipeline?.frontendOverflowModelCount === "number"
      ? (cm.pipeline.frontendOverflowModelCount as number)
      : 0;
  const overflowActive = Boolean(
    overflowEl &&
      !overflowEl.classList.contains("hidden") &&
      (overflowEl.classList.contains("visible") || overflowModelCount > 0),
  );
  if (cm.visibleCount < requiredVisible) {
    if (overflowActive && resolvedTotal >= requiredVisible) {
      /* never-clip: complete groups live in overflow at narrow widths */
    } else {
      return `command manager visible controls ${cm.visibleCount} < ${requiredVisible} (total ${resolvedTotal})`;
    }
  }
  if (cm.contrastOkCount < requiredContrast) {
    const overflowOk =
      overflowActive &&
      resolvedTotal >= requiredContrast &&
      cm.contrastOkCount >= cm.visibleCount &&
      cm.visibleCount > 0;
    if (!overflowOk) {
      return `command manager contrast-ok controls ${cm.contrastOkCount} < ${requiredContrast}`;
    }
  }
  return null;
}

/** Returns abort reason or null if markdown preview is painted for capture. */
export function validateMarkdownPreviewForCapture(): string | null {
  const shell = document.querySelector<HTMLElement>(".md-preview-shell");
  if (!shell || shell.classList.contains("hidden")) {
    return "markdown preview shell missing or hidden";
  }
  const preview = shell.querySelector<HTMLElement>(".md-preview-body");
  if (!preview) {
    return "markdown preview body missing";
  }
  const rect = preview.getBoundingClientRect();
  if (rect.height < 64 || rect.width < 160) {
    return `markdown preview geometry ${rect.width}x${rect.height} too small`;
  }
  if ((preview.textContent ?? "").trim().length < 20) {
    return "markdown preview body has insufficient rendered text";
  }
  if (!preview.querySelector("img")) {
    return "markdown preview missing inline image";
  }
  return null;
}

/** Returns abort reason or null if WYSIWYG rich document looks painted. */
export function validateSplitDocumentForCapture(): string | null {
  const host = document.getElementById("text-editor-host");
  if (!host || host.classList.contains("hidden")) {
    return "text editor host missing for split capture";
  }
  if (!host.classList.contains("document-editor-split-layout")) {
    return "document-editor-split-layout missing on text editor host";
  }
  const slot = host.querySelector<HTMLElement>(".document-monaco-slot");
  const preview = document.getElementById("document-split-preview-host");
  if (!slot) {
    return "document-monaco-slot missing in split mode";
  }
  if (!preview || preview.classList.contains("hidden")) {
    return "document-split-preview-host hidden or missing";
  }
  if (!preview.querySelector(".rich-text-doc")) {
    return "split preview missing rendered rich-text-doc";
  }
  const hostRect = host.getBoundingClientRect();
  const slotRect = slot.getBoundingClientRect();
  const previewRect = preview.getBoundingClientRect();
  if (hostRect.width < 200 || hostRect.height < 120) {
    return `editor host too small for split (${hostRect.width}x${hostRect.height})`;
  }
  if (slotRect.width > hostRect.width * 0.58) {
    return `monaco slot too wide (${slotRect.width.toFixed(0)}px, host ${hostRect.width.toFixed(0)}px)`;
  }
  if (previewRect.width < hostRect.width * 0.32) {
    return `split preview too narrow (${previewRect.width.toFixed(0)}px)`;
  }
  if (previewRect.left < slotRect.right - 12) {
    return "split preview not positioned to the right of monaco slot";
  }
  const modeFooter = document.getElementById("document-mode-inline-host");
  if (!modeFooter?.querySelector("[data-doc-mode]")) {
    return "document mode footer buttons missing";
  }
  return null;
}

export function validateRichTextDocumentForCapture(): string | null {
  const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
  if (demo.includes("doc-history-missing-plugin")) {
    const banner = document.querySelector(".history-plugin-banner:not(.hidden)");
    if (!banner) {
      return "missing-plugin capture needs history plugin banner";
    }
    return null;
  }
  const article = document.querySelector<HTMLElement>(
    "#rich-text-editor-host:not(.hidden) .rich-text-doc, #document-split-preview-host:not(.hidden) .rich-text-doc",
  );
  if (!article) {
    return "rich text document surface missing";
  }
  if (article.querySelector(".rich-text-opaque-parse")) {
    return null;
  }
  const rect = article.getBoundingClientRect();
  if (rect.width < 120 || rect.height < 80) {
    return `rich text document too small (${rect.width}x${rect.height})`;
  }
  if ((article.textContent ?? "").trim().length < 40) {
    return "rich text document has insufficient text";
  }
  const bold = article.querySelector("strong");
  const em = article.querySelector("em");
  const link = article.querySelector("a.rich-text-link, a[href]");
  if (!bold || !em || !link) {
    return "rich text missing bold, italic, or link rendering";
  }
  const blockquote = article.querySelector("blockquote");
  const img = article.querySelector<HTMLImageElement>("img");
  if (
    !blockquote &&
    (!img || img.classList.contains("rich-text-img--broken"))
  ) {
    return "rich text image missing or broken";
  }
  return null;
}

export function validateHistoryPluginBannerForCapture(): string | null {
  const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
  if (!demo.includes("doc-history-missing-plugin")) {
    return null;
  }
  const banner = document.querySelector(".history-plugin-banner:not(.hidden)");
  if (!banner) {
    return "missing-plugin capture requires visible history-plugin-banner";
  }
  const text = banner.querySelector(".history-plugin-banner-text")?.textContent ?? "";
  if (!text.includes("base-text") || !text.includes("99.0.0")) {
    return "history plugin banner must name base-text version 99.0.0";
  }
  if (!banner.querySelector(".history-plugin-banner-install")) {
    return "history plugin banner missing install action";
  }
  if (!banner.querySelector(".history-plugin-banner-continue")) {
    return "history plugin banner missing continue read-only action";
  }
  return null;
}

/** History captures must show the docked bottom panel in the viewport (not only in DOM). */
export function validateHistoryCaptureBottomPanelVisible(): string | null {
  const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
  if (!demo.includes("doc-history-") || demo.includes("doc-history-missing-plugin")) {
    return null;
  }
  const panel = document.getElementById("script-output-panel");
  if (!panel || panel.classList.contains("hidden")) {
    return "history capture requires visible script-output-panel";
  }
  const pane = document.getElementById("bottom-panel-history-pane");
  if (!pane || pane.classList.contains("hidden")) {
    return "history capture requires visible bottom-panel-history-pane";
  }
  const rect = panel.getBoundingClientRect();
  const minHeight = window.innerHeight * 0.2;
  if (rect.height < minHeight) {
    return `history bottom panel height ${rect.height.toFixed(0)} < ${minHeight.toFixed(0)}`;
  }
  if (rect.bottom > window.innerHeight + 2) {
    return "history bottom panel is below the viewport";
  }
  const table = document.querySelector(".history-panel-table");
  if (!table) {
    return "history capture missing history table";
  }
  const tableRect = table.getBoundingClientRect();
  if (tableRect.height < 40 || tableRect.bottom > window.innerHeight + 2) {
    return "history table is not painted inside the viewport";
  }
  return null;
}

/** Returns abort reason or null if editor area looks painted. */
export function validateEditorForCapture(
  minLines = 6,
  minVariance = 12,
  minTextLength = 80,
): string | null {
  const richDoc = document.querySelector(".rich-text-doc");
  const mergeBanner = document.querySelector(
    ".document-source-merge-banner:not(.hidden)",
  );
  if (mergeBanner && document.body.classList.contains("document-mode-split")) {
    return null;
  }
  if (document.querySelector(".rich-text-opaque-parse")) {
    return null;
  }
  const historyPane = document.getElementById("bottom-panel-history-pane");
  if (historyPane && !historyPane.classList.contains("hidden")) {
    return null;
  }
  if (
    richDoc &&
    !richDoc.classList.contains("hidden") &&
    (document.body.classList.contains("document-mode-wysiwyg") ||
      document.body.classList.contains("document-mode-reading"))
  ) {
    const text = richDoc.textContent ?? "";
    if (text.trim().length >= 80) {
      return null;
    }
  }
  const m = collectMonacoDiagnostics();
  if (!m) {
    return "text editor host missing or hidden";
  }
  if (m.viewLineCount < minLines) {
    return `monaco view-line count ${m.viewLineCount} < ${minLines}`;
  }
  if (m.viewTextLength < minTextLength) {
    return `monaco text length ${m.viewTextLength} < ${minTextLength}`;
  }
  if (
    m.hostHeight > 120 &&
    m.editorHeight < m.hostHeight * 0.4 &&
    !document.body.classList.contains("document-mode-split") &&
    !document.body.classList.contains("document-mode-wysiwyg")
  ) {
    return `monaco editor height ${m.editorHeight} vs host ${m.hostHeight}`;
  }
  const canvas = sampleMonacoCanvasVariance();
  if (
    canvas &&
    canvas.canvasCount > 0 &&
    canvas.sampleWidth > 0 &&
    canvas.sampleHeight > 0 &&
    canvas.sampleVariance < minVariance
  ) {
    return `monaco canvas pixel variance ${canvas.sampleVariance.toFixed(2)} < ${minVariance} (compositing or blank paint)`;
  }
  const theme = collectMonacoThemeDiagnostics();
  if (theme) {
    const shellTheme = document.documentElement.dataset.theme;
    if (shellTheme === "light" && theme.resolvedTheme !== "vs") {
      return `expected monaco theme vs for light shell, got ${theme.resolvedTheme ?? "null"}`;
    }
    if (shellTheme === "dark" && theme.resolvedTheme !== "vs-dark") {
      return `expected monaco theme vs-dark for dark shell, got ${theme.resolvedTheme ?? "null"}`;
    }
    const bg = parseRgbLuminance(theme.editorBackground);
    const fg = parseRgbLuminance(theme.viewLineColor || theme.editorForeground);
    if (bg !== null && fg !== null && contrastRatio(fg, bg) < 2.0) {
      return `monaco text/background contrast too low (fg=${theme.viewLineColor}, bg=${theme.editorBackground})`;
    }
    if (theme.editorBackground.includes("var(")) {
      return `monaco editor background still uses CSS var(): ${theme.editorBackground}`;
    }
  }
  return null;
}

export function formatDiagnosticsForLog(d: DocCaptureDiagnostics): string {
  return JSON.stringify(d, null, 0);
}

/** Returns abort reason when raster image viewer chrome violates one-home rules. */
/** Fail when muted CM chrome does not match declared capability states (CSS leak). */
export function validateCommandManagerMutedForCapture(): string | null {
  if (document.body.classList.contains("cm-collapsed")) {
    return null;
  }
  const controls = Array.from(
    document.querySelectorAll("#command-manager-groups .ui-command"),
  ) as HTMLElement[];
  const visible = controls.filter((el) => {
    const rect = el.getBoundingClientRect();
    const style = getComputedStyle(el);
    return (
      rect.width >= 24 &&
      rect.height >= 20 &&
      style.display !== "none" &&
      style.visibility !== "hidden"
    );
  });
  if (visible.length < 4) {
    return null;
  }
  let declaredSupported = 0;
  let declaredUnsupported = 0;
  let visuallyUnsupported = 0;
  for (const el of visible) {
    const state = el.dataset.capabilityState ?? "";
    if (state === "supported" || state === "emulated") {
      declaredSupported += 1;
    } else if (state === "unsupported" || state === "locked") {
      declaredUnsupported += 1;
    }
    if (el.classList.contains("unsupported")) {
      visuallyUnsupported += 1;
    }
  }
  if (declaredSupported < 2) {
    return null;
  }
  if (visuallyUnsupported > declaredUnsupported) {
    return `CM muted styling leak: ${visuallyUnsupported} visually unsupported vs ${declaredUnsupported} declared (supported ${declaredSupported})`;
  }
  if (visuallyUnsupported > visible.length / 2 && visuallyUnsupported > declaredUnsupported) {
    return `CM more than half of visible controls look unsupported (${visuallyUnsupported}/${visible.length}) while only ${declaredUnsupported} are declared unsupported`;
  }
  const brightLum =
    parseRgbLuminance("rgb(233, 237, 239)") ?? parseRgbLuminance("#e9edef");
  const mutedLightLum = parseRgbLuminance("rgb(174, 186, 192)") ?? parseRgbLuminance("#aebac0");
  const mutedDarkLum = parseRgbLuminance("rgb(146, 156, 170)") ?? parseRgbLuminance("#929caa");
  let supportedBright = 0;
  let supportedMuted = 0;
  for (const el of visible) {
    const state = el.dataset.capabilityState ?? "";
    if (state === "supported" || state === "emulated") {
      if (el.classList.contains("unsupported")) {
        return `supported control ${el.dataset.controlId ?? "?"} has unsupported CM styling`;
      }
      if (!el.classList.contains("cm-cap-supported")) {
        return `supported control ${el.dataset.controlId ?? "?"} missing cm-cap-supported chrome class`;
      }
      const labelEl = el.querySelector(".cm-cmd-label") as HTMLElement | null;
      const fg = parseRgbLuminance(getComputedStyle(labelEl ?? el).color);
      if (fg !== null && brightLum !== null && Math.abs(fg - brightLum) < 0.025) {
        supportedBright += 1;
      } else if (
        fg !== null &&
        ((mutedLightLum !== null && Math.abs(fg - mutedLightLum) < 0.025) ||
          (mutedDarkLum !== null && Math.abs(fg - mutedDarkLum) < 0.025))
      ) {
        supportedMuted += 1;
        return `supported control ${el.dataset.controlId ?? "?"} uses muted foreground (${getComputedStyle(el).color})`;
      }
    }
  }
  if (declaredSupported >= 2 && supportedBright < 1 && supportedMuted > 0) {
    return `no supported CM control uses bright chrome text (${supportedMuted} look muted)`;
  }
  let capSupported = 0;
  let capSupportedMuted = 0;
  for (const el of visible) {
    if (!el.classList.contains("cm-cap-supported")) {
      continue;
    }
    capSupported += 1;
    const labelEl = el.querySelector(".cm-cmd-label") as HTMLElement | null;
    const fg = parseRgbLuminance(getComputedStyle(labelEl ?? el).color);
    if (
      fg !== null &&
      brightLum !== null &&
      Math.abs(fg - brightLum) < 0.03
    ) {
      continue;
    }
    if (
      fg !== null &&
      ((mutedLightLum !== null && Math.abs(fg - mutedLightLum) < 0.03) ||
        (mutedDarkLum !== null && Math.abs(fg - mutedDarkLum) < 0.03))
    ) {
      capSupportedMuted += 1;
    }
  }
  if (capSupported >= 2 && capSupportedMuted > capSupported / 2) {
    return `more than half of cm-cap-supported controls look muted (${capSupportedMuted}/${capSupported})`;
  }
  return null;
}

/** Hide hover chrome and optionally preserve intentional keyboard-focus evidence. */
export function clearCaptureHoverAndTooltips(
  preserveFocus = false,
  preserveBlockPicker = false,
): void {
  document.getElementById("ub-command-tooltip-host")?.classList.add("hidden");
  document.querySelectorAll<HTMLElement>(".rail-destination, .rail-edge-hit").forEach((el) => {
    el.removeAttribute("title");
  });
  if (!preserveFocus) {
    const active = document.activeElement;
    if (active instanceof HTMLElement) {
      const keepPickerFocus =
        preserveBlockPicker &&
        Boolean(
          document.querySelector(
            "#text-compact-toolbar-host .text-compact-block-menu:not(.hidden)",
          ),
        );
      if (!keepPickerFocus) {
        active.blur();
      }
    }
  }
  document.body.dispatchEvent(
    new MouseEvent("mousemove", { clientX: 0, clientY: 0, bubbles: true }),
  );
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

export function validateProvingFloatingToolbarHiddenForCapture(): string | null {
  const labels: string[] = [];
  const provingControlIds: string[] = [];
  for (const bar of document.querySelectorAll<HTMLElement>(".floating-bar")) {
    labels.push(bar.querySelector(".floating-bar-label")?.textContent?.trim() ?? "");
    for (const control of bar.querySelectorAll<HTMLElement>("[data-control-id]")) {
      provingControlIds.push(control.dataset.controlId ?? "");
    }
  }
  return provingFloatingToolbarScanError(labels, provingControlIds);
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

export function validateGrayscalePreviewControlForCapture(): string | null {
  const selector =
    '[data-control-id="image-grayscale-preview.control.grayscale"]';
  const inCm = document.querySelector<HTMLElement>(
    `#command-manager-groups ${selector}`,
  );
  const inOverflow = document.querySelector<HTMLElement>(
    `.cm-overflow-menu ${selector}`,
  );
  const inOverflowHost = document.querySelector<HTMLElement>(
    `#command-manager-overflow-menu ${selector}, [role="menu"] ${selector}`,
  );
  const control = inCm ?? inOverflow ?? inOverflowHost;
  if (!control) {
    return grayscalePreviewControlCaptureError(false, false, "");
  }
  const rect = control.getBoundingClientRect();
  const style = getComputedStyle(control);
  const visible =
    rect.width > 0 &&
    rect.height > 0 &&
    style.display !== "none" &&
    style.visibility !== "hidden";
  return grayscalePreviewControlCaptureError(
    true,
    visible,
    control.dataset.capabilityState ?? "",
  );
}

export function validateImageViewerChrome(launchPaths: string[]): string | null {
  const joined = launchPaths.join("|").toLowerCase();
  if (!/\.(png|jpe?g|webp|gif|bmp)$/i.test(joined)) {
    return null;
  }
  const host = document.querySelector("#document-viewer-host");
  if (!host || host.classList.contains("hidden")) {
    return "image capture missing visible document-viewer-host";
  }
  if (host.querySelector(".image-toolbar")) {
    return "image viewer must not render a duplicate .image-toolbar row";
  }
  const strayNote = host.querySelector(":scope > .viewer-note");
  if (strayNote) {
    return "image viewer must not render a root-level .viewer-note hint";
  }
  const viewport = host.querySelector(".image-viewport") as HTMLElement | null;
  if (!viewport) {
    return "image viewer missing .image-viewport";
  }
  const height = viewport.getBoundingClientRect().height;
  if (height < 280) {
    return `image viewport height ${height.toFixed(0)}px is too short (expected fill)`;
  }
  return null;
}

export interface TextCompactToolbarCaptureOptions {
  requirePickerOpen?: boolean;
  requireH2Selected?: boolean;
  requireDocxEmulatedH1?: boolean;
  requireModToolbarDock?: boolean;
}

/** Quiet Utility: compact row shares CM chrome tokens; no light tiles on controls. */
function compactToolbarQuietUtilityMismatch(host: HTMLElement): string | null {
  const onCard = document.body.classList.contains("text-document-card");
  const reference = onCard
    ? document.querySelector<HTMLElement>(".document-card-toolbar-mount")
    : document.getElementById("command-manager");
  if (!reference) {
    return null;
  }
  const hostBg = getComputedStyle(host).backgroundColor;
  const refBg = getComputedStyle(reference).backgroundColor;
  if (onCard) {
    const hostLumCard = parseRgbLuminance(hostBg);
    const paperLum = parseRgbLuminance(refBg);
    if (
      hostLumCard !== null &&
      paperLum !== null &&
      hostBg !== "rgba(0, 0, 0, 0)" &&
      Math.abs(hostLumCard - paperLum) > 0.12
    ) {
      return `in-card toolbar background ${hostBg} must sit on paper card ${refBg}`;
    }
  } else if (hostBg !== refBg) {
    return `text compact toolbar background ${hostBg} must match command manager ${refBg}`;
  }
  const hostLum = parseRgbLuminance(hostBg);
  if (hostLum === null) {
    return `text compact toolbar background ${hostBg} is not parseable`;
  }
  for (const el of host.querySelectorAll<HTMLElement>(".ui-command, button.text-compact-btn")) {
    const style = getComputedStyle(el);
    const bgLum = parseRgbLuminance(style.backgroundColor);
    if (bgLum !== null && bgLum - hostLum > 0.08) {
      const id = el.dataset.controlId ?? el.dataset.commandId ?? "?";
      return `compact control ${id} has boxed/light background ${style.backgroundColor}`;
    }
    if (style.borderTopWidth && parseFloat(style.borderTopWidth) >= 1) {
      const borderLum = parseRgbLuminance(style.borderTopColor);
      if (
        borderLum !== null &&
        borderLum - hostLum > 0.15 &&
        style.borderTopColor !== "rgba(0, 0, 0, 0)"
      ) {
        const id = el.dataset.controlId ?? el.dataset.commandId ?? "?";
        return `compact control ${id} has visible boxed border ${style.borderTopColor}`;
      }
    }
  }
  return null;
}

/** TXT-06: compact text toolbar geometry and picker state for doc-capture. */
export function validateTextCompactToolbarForCapture(
  options: TextCompactToolbarCaptureOptions | boolean = false,
): string | null {
  const opts: TextCompactToolbarCaptureOptions =
    typeof options === "boolean" ? { requirePickerOpen: options } : options;
  const requirePickerOpen = opts.requirePickerOpen ?? false;
  const requireH2Selected = opts.requireH2Selected ?? false;
  const requireDocxEmulatedH1 = opts.requireDocxEmulatedH1 ?? false;
  const requireModToolbarDock = opts.requireModToolbarDock ?? false;
  if (!document.body.classList.contains("text-compact-visible")) {
    return null;
  }
  const cardMountedHost = document.querySelector<HTMLElement>(
    "#document-card-toolbar-mount #text-compact-toolbar-host",
  );
  const host = cardMountedHost ?? document.getElementById("text-compact-toolbar-host");
  if (!host) {
    return "text compact toolbar host missing while text-compact-visible";
  }
  if (host.classList.contains("hidden") && !cardMountedHost) {
    return "text compact toolbar host missing while text-compact-visible";
  }
  const rect = host.getBoundingClientRect();
  const maxRow = document.body.classList.contains("text-document-card") ? 38.5 : 33.5;
  if (rect.height > maxRow) {
    return `text compact toolbar height ${rect.height.toFixed(1)}px exceeds ${maxRow}px row budget`;
  }
  const tools = host.querySelector<HTMLElement>(".text-compact-tools");
  if (!tools) {
    return "text compact toolbar missing .text-compact-tools row";
  }
  const toolsStyle = getComputedStyle(tools);
  if (toolsStyle.flexWrap !== "nowrap") {
    return `text compact toolbar flex-wrap is ${toolsStyle.flexWrap}, expected nowrap`;
  }
  if (tools.scrollWidth > tools.clientWidth + 2) {
    return "text compact toolbar row appears wrapped or clipped without overflow";
  }
  for (const btn of host.querySelectorAll<HTMLElement>(".text-compact-btn")) {
    if (btn.classList.contains("text-compact-block-trigger")) {
      continue;
    }
    if (btn.querySelector(".cm-cmd-label")) {
      return `compact icon ${btn.dataset.controlId ?? "?"} still has a visible label`;
    }
  }
  const iconErr = compactToolbarIconFallbackError(host);
  if (iconErr) {
    return iconErr;
  }
  const cmGroups = document.getElementById("command-manager-groups");
  if (cmGroups) {
    const cmText = cmGroups.textContent ?? "";
    if (/\d+\s*pt/i.test(cmText)) {
      return "Command Manager must not show document font size (pt) in compact text context";
    }
  }
  const pickerLabel = host.querySelector<HTMLElement>(".text-compact-block-label");
  if (!pickerLabel?.textContent?.trim()) {
    return "block-style picker missing current style label";
  }
  const pickerText = pickerLabel.textContent.trim();
  if (pickerText === "Markdown" || pickerText === "Word" || pickerText === "Plain text") {
    return `block-style picker must show block type, not format name "${pickerText}"`;
  }
  if (opts.requireH2Selected && pickerText !== "Heading 2") {
    return `block-style picker trigger must show Heading 2 after txt-apply-h2, got "${pickerText}"`;
  }
  if (opts.requireDocxEmulatedH1 && pickerText !== "Heading 1") {
    return `block-style picker trigger must show Heading 1 for docx capture, got "${pickerText}"`;
  }
  const menu =
    document.querySelector<HTMLElement>(
      "#ub-block-picker-popover-host .text-compact-block-menu:not(.hidden)",
    ) ?? host.querySelector<HTMLElement>(".text-compact-block-menu:not(.hidden)");
  if (requirePickerOpen) {
    if (!menu || menu.classList.contains("hidden")) {
      return "block-style picker menu must be open for this capture (portal or inline)";
    }
    if (menu.getAttribute("role") !== "listbox") {
      return "block-style picker menu must have role=listbox";
    }
    const itemLabels = [...menu.querySelectorAll<HTMLElement>(".text-compact-block-item")].map(
      (row) => row.textContent?.trim() ?? "",
    );
    for (const expected of BLOCK_PICKER_MENU_LABELS) {
      if (!itemLabels.includes(expected)) {
        return `block-style picker menu missing "${expected}" (have: ${itemLabels.join(", ")})`;
      }
    }
    if (requireH2Selected) {
      const h2 = menu.querySelector<HTMLElement>(
        '[data-command-id="unibrowser.semantic.block.heading_2"]',
      );
      if (!h2?.hasAttribute("aria-selected")) {
        return "block-style picker must show Heading 2 as selected when H2 is applied";
      }
    }
    if (requireDocxEmulatedH1) {
      const h1 = menu.querySelector<HTMLElement>(
        '[data-command-id="unibrowser.semantic.block.heading_1"]',
      );
      if (!h1) {
        return "block-style picker must list Heading 1 for docx capture";
      }
      const trigger = host.querySelector<HTMLElement>(".text-compact-block-trigger");
      const mark = trigger?.querySelector<HTMLElement>(".block-emulation-badge, .cm-emulation-mark");
      if (!mark?.textContent?.includes("≈")) {
        return "docx block-style picker must show emulated Heading 1 (≈) on trigger";
      }
      const labelRect = pickerLabel.getBoundingClientRect();
      const markRect = mark.getBoundingClientRect();
      if (captureRectsOverlap(labelRect, markRect)) {
        return "docx emulation badge must not overlap block-style label bounding box";
      }
      if (markRect.left < labelRect.right - 0.5) {
        return "docx emulation badge must sit after the block-style label";
      }
      if (markRect.width < 14 || markRect.height < 14) {
        return "docx emulation badge must be legible (min 14×14px)";
      }
      const markStyle = getComputedStyle(mark);
      const fontSize = Number.parseFloat(markStyle.fontSize);
      if (!Number.isFinite(fontSize) || fontSize < 14) {
        return `docx emulation badge font must be at least 14px (got ${markStyle.fontSize})`;
      }
      const fg = markStyle.color;
      const bg = markStyle.backgroundColor;
      const badgeContrast = colourContrastRatio(fg, bg);
      if (badgeContrast === null || badgeContrast < 4.5) {
        return `docx emulation badge contrast ${badgeContrast?.toFixed(2) ?? "null"} < 4.5 (${fg} on ${bg})`;
      }
      const reason = mark.getAttribute("title")?.trim() || mark.getAttribute("aria-label")?.trim();
      if (!reason) {
        return "docx emulation badge must expose a tooltip reason (title or aria-label)";
      }
    }
  }
  const modHost = document.getElementById("text-mod-toolbar-host");
  if (modHost && !modHost.classList.contains("hidden")) {
    return "plugin text toolbar must be folded into the compact row (mod host must stay hidden)";
  }
  if (requireModToolbarDock) {
    const dock = host.querySelector<HTMLElement>(".text-compact-mod-dock");
    if (!dock) {
      return "sample mod toolbar dock must be visible for this capture";
    }
    const label = dock.querySelector<HTMLElement>(".text-compact-mod-label")?.textContent?.trim();
    if (!label) {
      return "sample mod toolbar must show a visible name label";
    }
    const modButtons = dock.querySelectorAll<HTMLElement>(
      "button.text-compact-btn[data-command-id]",
    );
    if (modButtons.length < 2) {
      return `sample mod toolbar needs at least 2 controls, found ${modButtons.length}`;
    }
  } else if (host.querySelector(".text-compact-mod-dock")) {
    return "mod docked toolbar must only appear when txt-mod-toolbar-visible demo is set";
  }
  const quietErr = compactToolbarQuietUtilityMismatch(host);
  if (quietErr) {
    return quietErr;
  }
  if (document.body.classList.contains("text-document-card")) {
    const demo = (document.body.dataset.ubDemoTokens ?? "").toLowerCase();
    const projectOfficeCapture =
      document.body.dataset.ubNewProjectCapture === "1" &&
      demo.includes("office-project");
    if (!projectOfficeCapture) {
      for (const id of [
        "unibrowser.control.text.link",
        "unibrowser.control.text.insert_image",
      ]) {
        if (!host.querySelector(`[data-control-id="${id}"]`)) {
          return `Render 1 in-card toolbar missing control ${id}`;
        }
      }
    }
  }
  return null;
}

const CONCEPT_RENDER_FOOTER_SNIPPETS = ["512 words", "Saved 10:18"];

/** Render 1 card footer must reflect live word count and save state (not concept art literals). */
export function validateDocumentCardFooterForCapture(): string | null {
  if (!document.body.classList.contains("text-document-card")) {
    return null;
  }
  const main = document.querySelector<HTMLElement>(".document-card-status-main");
  const text = main?.textContent?.trim() ?? "";
  if (!text) {
    return "document card footer missing status text";
  }
  for (const snippet of CONCEPT_RENDER_FOOTER_SNIPPETS) {
    if (text.includes(snippet)) {
      return `document card footer still uses concept placeholder "${snippet}"`;
    }
  }
  const match = text.match(/(\d+)\s+words?\b/i);
  if (!match) {
    return `document card footer missing word count in "${text}"`;
  }
  const shown = Number.parseInt(match[1], 10);
  const expected = documentWordCount();
  if (!Number.isFinite(shown) || shown !== expected) {
    return `document card footer word count ${shown} does not match document (${expected})`;
  }
  const dirty = document.body.dataset.documentCardDirty === "1";
  if (dirty && !text.includes("Not saved")) {
    return `document card footer must show Not saved when dirty; got "${text}"`;
  }
  if (!dirty && text.includes("Not saved")) {
    return `document card footer must not show Not saved when document is clean`;
  }
  const docPath = (
    document.body.dataset.activeDocumentPath ??
    document.querySelector<HTMLElement>("#document-card-tab .document-card-tab-label")?.textContent ??
    ""
  ).toLowerCase();
  if (docPath.includes("spell-test.txt") || docPath.endsWith(".txt")) {
    if (text.includes("Markdown")) {
      return `document card footer must not label .txt as Markdown; got "${text}"`;
    }
    if (!text.includes("Plain text")) {
      return `document card footer must show Plain text for .txt; got "${text}"`;
    }
  }
  return null;
}

function cssTokenColor(tokenName: string, on: HTMLElement): string {
  const raw = getComputedStyle(on).getPropertyValue(tokenName).trim();
  if (!raw) {
    return "";
  }
  const probe = document.createElement("span");
  probe.style.color = raw;
  document.body.appendChild(probe);
  const resolved = getComputedStyle(probe).color;
  probe.remove();
  return resolved;
}

function cssTokenBackground(tokenName: string, on: HTMLElement): string {
  const raw = getComputedStyle(on).getPropertyValue(tokenName).trim();
  if (!raw) {
    return "";
  }
  const probe = document.createElement("span");
  probe.style.backgroundColor = raw;
  document.body.appendChild(probe);
  const resolved = getComputedStyle(probe).backgroundColor;
  probe.remove();
  return resolved;
}

function rgbColorsMatch(a: string, b: string): boolean {
  const norm = (s: string) => s.replace(/\s+/g, "").toLowerCase();
  return norm(a) === norm(b);
}

function elementFullyInViewport(el: HTMLElement, pad = 1): boolean {
  const r = el.getBoundingClientRect();
  return (
    r.top >= pad &&
    r.left >= pad &&
    r.bottom <= window.innerHeight - pad &&
    r.right <= window.innerWidth - pad
  );
}

function dialogPrimaryButtonUsesAccent(panel: HTMLElement | null): boolean {
  const btn = panel?.querySelector<HTMLButtonElement>("button.primary");
  if (!btn || !panel) {
    return false;
  }
  const accent = cssTokenBackground("--ub-accent", panel);
  const bg = getComputedStyle(btn).backgroundColor;
  return Boolean(accent && bg && rgbColorsMatch(accent, bg));
}

function isPureWhiteRgb(color: string): boolean {
  const normalized = color.replace(/\s+/g, "").toLowerCase();
  return (
    normalized === "rgb(255,255,255)" ||
    normalized === "rgba(255,255,255,1)" ||
    normalized === "#ffffff" ||
    normalized === "#fff"
  );
}

function elementIsVisible(el: HTMLElement): boolean {
  const rect = el.getBoundingClientRect();
  const style = getComputedStyle(el);
  return (
    rect.width > 2 &&
    rect.height > 2 &&
    style.display !== "none" &&
    style.visibility !== "hidden" &&
    style.opacity !== "0"
  );
}

function workspaceRootForCapture(): HTMLElement | null {
  return (
    document.querySelector<HTMLElement>(".document-viewer-host--project-surface .viewer-body") ??
    document.querySelector<HTMLElement>("#split-primary:not(.hidden)") ??
    document.getElementById("split-primary")
  );
}

export function validatePresentationToolbarButtons(): string | null {
  const toolbar = document.querySelector(".presentation-toolbar");
  if (!toolbar) {
    return null;
  }
  const buttons = toolbar.querySelectorAll<HTMLButtonElement>("button");
  if (buttons.length === 0) {
    return "presentation toolbar missing action buttons";
  }
  for (const btn of buttons) {
    if (!btn.classList.contains("presentation-toolbar-btn")) {
      return "presentation toolbar uses unstyled raw button";
    }
    if (!btn.querySelector(".ub-icon[data-lucide]")) {
      return "presentation toolbar button missing Lucide icon";
    }
    const label = btn.querySelector(".presentation-toolbar-btn-label")?.textContent?.trim() ?? "";
    if (!label) {
      return "presentation toolbar button missing visible label";
    }
  }
  return null;
}

export function validateEmptyProjectHomeCardLayout(): string | null {
  const card = document.querySelector<HTMLElement>(".project-home-card");
  if (!card || !elementIsVisible(card)) {
    return null;
  }
  const rect = card.getBoundingClientRect();
  if (rect.height >= 300) {
    return `empty project home card too tall (${rect.height.toFixed(0)}px)`;
  }
  const workspace = workspaceRootForCapture();
  const workRect = workspace?.getBoundingClientRect();
  if (workRect && workRect.width > 40) {
    const cardCenterX = (rect.left + rect.right) / 2;
    const workCenterX = (workRect.left + workRect.right) / 2;
    const deltaX = Math.abs(cardCenterX - workCenterX);
    if (deltaX > 8) {
      return `empty project card horizontal center off by ${deltaX.toFixed(1)}px (max 8px)`;
    }
    const topRelative = rect.top - workRect.top;
    if (topRelative >= 80) {
      return `empty project card top ${topRelative.toFixed(0)}px must be < 80px`;
    }
  }
  const root = document.documentElement;
  if (root.scrollWidth > root.clientWidth + 1) {
    return `horizontal overflow on capture viewport (scrollWidth ${root.scrollWidth} > ${root.clientWidth})`;
  }
  return null;
}

/** Pixel-adjacent layout checks for new-project captures (geometry + computed colors). */
export function validateNewProjectCapturePixels(demoTokens: string): string | null {
  if (document.body.dataset.ubNewProjectCapture !== "1") {
    return null;
  }
  const demo = demoTokens.toLowerCase();
  const needsWorkspaceBandCheck =
    demo.includes("empty") || demo.includes("presentation-project");
  const needsOfficeToolbar = demo.includes("office-project");
  const needsConfigureChrome = demo.includes("configure");
  if (!needsWorkspaceBandCheck && !needsOfficeToolbar && !needsConfigureChrome) {
    return null;
  }

  if (needsOfficeToolbar) {
    const host =
      document.querySelector("#document-card-toolbar-mount") ??
      document.getElementById("text-compact-toolbar-host");
    const count = host
      ? host.querySelectorAll<HTMLElement>(
          ".text-compact-btn[data-control-id], .text-compact-block-trigger[data-control-id]",
        ).length
      : 0;
    if (count < 6) {
      return `office project capture requires ≥6 compact toolbar controls (found ${count})`;
    }
  }

  if (needsConfigureChrome) {
    const search = document.querySelector<HTMLInputElement>(
      ".template-gallery-header input[type='search'], .template-gallery-search",
    );
    if (search && elementIsVisible(search)) {
      return "template configure capture must not show gallery search input";
    }
  }

  if (!needsWorkspaceBandCheck) {
    return null;
  }

  const workspace = workspaceRootForCapture();
  if (!workspace) {
    return "new-project pixel check: workspace root missing";
  }
  const workRect = workspace.getBoundingClientRect();
  const workWidth = workRect.width;
  if (workWidth < 200) {
    return `new-project pixel check: workspace width ${workWidth.toFixed(0)}px too narrow`;
  }

  if (demo.includes("presentation-project")) {
    const frame = document.querySelector<HTMLElement>(".presentation-slide-frame");
    if (!frame) {
      return "presentation capture requires slide frame for aspect ratio check";
    }
    const rect = frame.getBoundingClientRect();
    if (rect.width < 80 || rect.height < 40) {
      return `presentation slide frame too small (${rect.width.toFixed(0)}×${rect.height.toFixed(0)})`;
    }
    const ratio = rect.width / rect.height;
    const target = 16 / 9;
    if (Math.abs(ratio - target) / target > 0.02) {
      return `presentation slide aspect ${ratio.toFixed(3)} not within 16:9 ±2%`;
    }
  }

  const minBandWidth = workWidth * 0.5;
  const cardSelectors =
    ".document-card-shell, .project-home-card, .presentation-slide-frame, .presentation-notes, .presentation-slides-rail, .template-gallery-panel";
  const candidates = workspace.querySelectorAll<HTMLElement>("*");
  for (const el of candidates) {
    if (!elementIsVisible(el)) {
      continue;
    }
    if (el.closest(cardSelectors)) {
      continue;
    }
    const rect = el.getBoundingClientRect();
    if (rect.width < minBandWidth || rect.height < 6 || rect.height > 220) {
      continue;
    }
    const bg = getComputedStyle(el).backgroundColor;
    if (!isPureWhiteRgb(bg)) {
      continue;
    }
    const top = rect.top - workRect.top;
    if (top >= 0 && top < 220) {
      return `workspace has pure-white band ${rect.width.toFixed(0)}×${rect.height.toFixed(0)}px near top (outside cards)`;
    }
  }

  return null;
}

/** ADR 0036 new-project captures: office card chrome, project workspace fill, no video status bleed. */
export function validateNewProjectCaptureDom(demoTokens: string): string | null {
  if (document.body.dataset.ubNewProjectCapture !== "1") {
    return null;
  }
  const demo = demoTokens.toLowerCase();
  if (demo.includes("office-project")) {
    if (!document.body.classList.contains("text-document-card")) {
      return "office project capture requires text-document-card stylesheet";
    }
    const shell = document.getElementById("document-card-shell");
    if (!shell || shell.classList.contains("hidden")) {
      return "office project capture requires visible document-card-shell";
    }
    const rich = document.querySelector(
      "#rich-text-editor-host:not(.hidden) .rich-text-doc, #document-split-preview-host:not(.hidden) .rich-text-doc",
    );
    if (!rich) {
      return "office project capture requires visual rich-text document surface";
    }
    const toolbarHost =
      document.querySelector("#document-card-toolbar-mount #text-compact-toolbar-host") ??
      document.getElementById("text-compact-toolbar-host");
    const controlCount = toolbarHost
      ? toolbarHost.querySelectorAll<HTMLElement>(
          ".text-compact-btn[data-control-id], .text-compact-block-trigger[data-control-id]",
        ).length
      : 0;
    if (controlCount < 6) {
      return `office project capture requires compact document toolbar (found ${controlCount} controls)`;
    }
  }
  if (demo.includes("empty") || demo.includes("presentation-project")) {
    const workArea =
      document.querySelector<HTMLElement>(".document-viewer-host--project-surface .viewer-body") ??
      document.getElementById("split-primary");
    if (workArea) {
      const expected = cssTokenColor("--ub-canvas", workArea);
      const actual = getComputedStyle(workArea).backgroundColor;
      if (
        expected &&
        actual &&
        actual !== "rgba(0, 0, 0, 0)" &&
        expected !== actual
      ) {
        return `workspace background ${actual} does not match --ub-canvas ${expected}`;
      }
    }
  }
  if (demo.includes("empty") || demo.includes("presentation-project")) {
    const statusText =
      document.getElementById("status-bar")?.textContent?.replace(/\s+/g, " ") ?? "";
    if (/1080p|48 kHz|00:00:00\.000/.test(statusText)) {
      return "non-video project capture must not show workshop/video status items";
    }
  }
  if (demo.includes("presentation-project")) {
    if (!document.querySelector(".presentation-slide-frame")) {
      return "presentation project capture requires slide editor frame";
    }
    if (!document.querySelector(".presentation-slide-thumb")) {
      return "presentation project capture requires slide thumbnail rail";
    }
    const toolbarErr = validatePresentationToolbarButtons();
    if (toolbarErr) {
      return toolbarErr;
    }
  }
  if (demo.includes("new-project-capture-empty") || demo.includes("capture-empty")) {
    const emptyLayoutErr = validateEmptyProjectHomeCardLayout();
    if (emptyLayoutErr) {
      return emptyLayoutErr;
    }
  }
  if (demo.includes("relationship-consent")) {
    const dom = collectDocumentFeatureCaptureDom();
    const panelH = dom.relationshipConsentPanelHeight;
    if (panelH === null) {
      return "relationship consent capture requires consent panel";
    }
    if (panelH / window.innerHeight >= 0.7) {
      return `relationship consent panel height ${panelH.toFixed(0)}px must be < 70% of viewport`;
    }
    if (dom.relationshipConsentUsesListBullets) {
      return "relationship consent capture must not use list-style bullets";
    }
    if (dom.relationshipConsentPanelScrollable) {
      return "relationship consent panel must not show a scrollbar at capture viewport";
    }
    if (dom.relationshipConsentSectionsScrollable) {
      return "relationship consent sections must not scroll when content fits in panel";
    }
    if (!dom.relationshipConsentActionsInViewport) {
      return "relationship consent Cancel and Create must be fully visible in the viewport";
    }
    if (!dom.dialogPrimaryButtonMatchesAccent) {
      return "relationship consent Create button must use accent primary fill";
    }
  }
  if (demo.includes("first-save")) {
    const dom = collectDocumentFeatureCaptureDom();
    if (!dom.firstSaveCancelButtonPresent) {
      return "first-save capture requires Cancel button in dialog actions";
    }
    if (!document.querySelector(".first-save-path-preview")?.textContent?.trim()) {
      return "first-save capture requires path preview";
    }
    if (!dom.dialogPrimaryButtonMatchesAccent) {
      return "first-save Save button must use accent primary fill";
    }
  }
  if (demo.includes("office-project")) {
    const footerErr = validateDocumentCardFooterForCapture();
    if (footerErr) {
      return footerErr;
    }
    const words = documentWordCount();
    if (words <= 0) {
      return `office project capture requires seeded document word count > 0 (got ${words})`;
    }
  }
  if (demo.includes("configure")) {
    if (!document.querySelector(".template-gallery-configure-form")) {
      return "template configure capture requires configure form layout";
    }
    const header = document.querySelector<HTMLElement>(".template-gallery-header");
    if (header && elementIsVisible(header)) {
      return "template configure capture must hide gallery search and filters";
    }
    const footer = document.querySelector<HTMLElement>(".template-gallery-footer");
    if (footer && elementIsVisible(footer)) {
      return "template configure capture must hide gallery footer";
    }
  }
  const pixelErr = validateNewProjectCapturePixels(demoTokens);
  if (pixelErr) {
    return pixelErr;
  }
  return null;
}

/** Markdown preview embedded images should not exceed the preview column width. */
export function validateMarkdownPreviewImageFitsPage(): string | null {
  const shell = document.querySelector<HTMLElement>(".md-preview-shell:not(.hidden)");
  if (!shell) {
    return null;
  }
  const body = shell.querySelector<HTMLElement>(".md-preview-body");
  if (!body) {
    return "markdown preview body missing for image fit check";
  }
  const bodyWidth = body.getBoundingClientRect().width;
  if (bodyWidth < 80) {
    return `markdown preview body width ${bodyWidth.toFixed(0)}px too narrow`;
  }
  for (const img of body.querySelectorAll<HTMLImageElement>("img")) {
    const rect = img.getBoundingClientRect();
    if (rect.width < 8 || rect.height < 8) {
      return "markdown preview image has no visible layout box";
    }
    if (rect.width > bodyWidth + 4) {
      return `markdown preview image width ${rect.width.toFixed(0)}px exceeds body ${bodyWidth.toFixed(0)}px`;
    }
  }
  return null;
}
