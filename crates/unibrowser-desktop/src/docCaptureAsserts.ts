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

export function backgroundExceedsRaisedSurface(
  candidateLuminance: number,
  raisedSurfaceLuminance: number,
  tolerance = 0.06,
): boolean {
  return candidateLuminance > raisedSurfaceLuminance + tolerance;
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
        'input:not([type="hidden"]), textarea, select, [class*="chip"], [class*="card"]',
      )
      .forEach((candidate) => candidates.add(candidate));
  }
  for (const candidate of candidates) {
    const rect = candidate.getBoundingClientRect();
    const style = getComputedStyle(candidate);
    if (
      rect.width <= 0 ||
      rect.height <= 0 ||
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

/** Returns abort reason or null if CM shows enough visible, contrasting controls. */
export function validateCommandManagerForCapture(minVisible = 6, minContrast = 6): string | null {
  if (document.body.classList.contains("cm-collapsed")) {
    return null;
  }
  const cm = collectCommandManagerDiagnostics();
  const startContext =
    cm.autoContext === "start" || cm.contextLabel?.toLowerCase() === "start";
  const settingsContext = cm.contextLabel?.toLowerCase() === "settings";
  const modsContext = cm.contextLabel?.toLowerCase() === "mods";
  const mediaContext = cm.contextLabel?.toLowerCase() === "audio";
  const sparseContextLimit = mediaContext ? 1 : settingsContext ? 2 : modsContext ? 5 : 4;
  const sparseContext = startContext || settingsContext || modsContext || mediaContext;
  const requiredVisible = sparseContext ? Math.min(minVisible, sparseContextLimit) : minVisible;
  const requiredContrast = sparseContext ? Math.min(minContrast, sparseContextLimit) : minContrast;
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

/** Returns abort reason or null if editor area looks painted. */
export function validateEditorForCapture(minLines = 6, minVariance = 12): string | null {
  const m = collectMonacoDiagnostics();
  if (!m) {
    return "text editor host missing or hidden";
  }
  if (m.viewLineCount < minLines) {
    return `monaco view-line count ${m.viewLineCount} < ${minLines}`;
  }
  if (m.viewTextLength < 80) {
    return `monaco text length ${m.viewTextLength} < 80`;
  }
  if (m.hostHeight > 120 && m.editorHeight < m.hostHeight * 0.4) {
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

/** Blur focus and hide CM tooltips so captures do not show hover chrome. */
export function clearCaptureHoverAndTooltips(): void {
  document.getElementById("ub-command-tooltip-host")?.classList.add("hidden");
  const active = document.activeElement;
  if (active instanceof HTMLElement) {
    active.blur();
  }
  document.body.dispatchEvent(
    new MouseEvent("mousemove", { clientX: 0, clientY: 0, bubbles: true }),
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
