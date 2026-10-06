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

export interface DocCaptureDiagnostics {
  theme: ThemeCaptureDiagnostics;
  monaco: MonacoCaptureDiagnostics | null;
  canvas: EditorCanvasDiagnostics | null;
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

export function collectDocCaptureDiagnostics(): DocCaptureDiagnostics {
  return {
    theme: collectThemeDiagnostics(),
    monaco: collectMonacoDiagnostics(),
    canvas: sampleMonacoCanvasVariance(),
  };
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
  return null;
}

export function formatDiagnosticsForLog(d: DocCaptureDiagnostics): string {
  return JSON.stringify(d, null, 0);
}
