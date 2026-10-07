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
  controlCount: number;
  visibleCount: number;
  contrastOkCount: number;
  contextLabel: string;
  autoContext: string | null;
  pipeline: Record<string, unknown> | null;
  controls: Array<{
    controlId: string;
    label: string;
    width: number;
    height: number;
    labelWidth: number;
    iconWidth: number;
    opacity: string;
    foreground: string;
    background: string;
    contrast: number | null;
  }>;
}

export interface DocCaptureDiagnostics {
  theme: ThemeCaptureDiagnostics;
  monaco: MonacoCaptureDiagnostics | null;
  canvas: EditorCanvasDiagnostics | null;
  monacoTheme: MonacoThemeCaptureDiagnostics | null;
  commandManager: CommandManagerCaptureDiagnostics | null;
  problems: {
    tabText: string;
    rowCount: number;
    visibleRowCount: number;
    emptyVisible: boolean;
    panelHeight: number;
    tableHeight: number;
  };
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
  if (base !== LIGHT_UB_BASE && base !== "rgb(241,240,236)") {
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
    const visiblyStyled =
      style.visibility !== "hidden" &&
      style.display !== "none" &&
      Number(style.opacity) >= 0.8;
    if (hasGeometry && visiblyStyled) {
      visibleCount += 1;
    }
    if (hasGeometry && visiblyStyled && contrast !== null && contrast >= 3) {
      contrastOkCount += 1;
    }
    controlDiagnostics.push({
      controlId: el.dataset.controlId ?? el.dataset.commandId ?? "",
      label: label?.textContent?.trim() ?? "",
      width: rect.width,
      height: rect.height,
      labelWidth: labelRect?.width ?? 0,
      iconWidth: iconRect?.width ?? 0,
      opacity: style.opacity,
      foreground: style.color,
      background,
      contrast,
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
    controlCount: controls.length,
    visibleCount,
    contrastOkCount,
    contextLabel: contextEl?.textContent?.trim() ?? "",
    autoContext: document.body.dataset.cmAutoContext ?? null,
    pipeline,
    controls: controlDiagnostics,
  };
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
  if (
    visibleRows.some((row) => {
      const rect = row.getBoundingClientRect();
      return rect.top < tableRect.top || rect.bottom > tableRect.bottom + 1;
    })
  ) {
    return "problem row escapes the problems panel";
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
  return {
    theme: collectThemeDiagnostics(),
    monaco: collectMonacoDiagnostics(),
    canvas: sampleMonacoCanvasVariance(),
    monacoTheme: collectMonacoThemeDiagnostics(),
    commandManager: collectCommandManagerDiagnostics(),
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
  const requiredVisible = startContext ? Math.min(minVisible, 4) : minVisible;
  const requiredContrast = startContext ? Math.min(minContrast, 4) : minContrast;
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
