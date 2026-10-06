/** Measure compact chrome heights for audits and screenshot scripts. */

export interface ChromeHeights {
  titleBarPx: number;
  commandManagerPx: number;
  totalChromePx: number;
  cmCollapsed: boolean;
  cmDensity: string | undefined;
}

export function measureChromeHeights(): ChromeHeights {
  const title = document.getElementById("title-bar");
  const cm = document.getElementById("command-manager");
  const cmCollapsed = document.body.classList.contains("cm-collapsed");
  const titleBarPx = title ? Math.round(title.getBoundingClientRect().height) : 0;
  const commandManagerPx =
    cm && !cmCollapsed ? Math.round(cm.getBoundingClientRect().height) : 0;
  return {
    titleBarPx,
    commandManagerPx,
    totalChromePx: titleBarPx + commandManagerPx,
    cmCollapsed,
    cmDensity: document.body.dataset.cmDensity,
  };
}

export function exposeChromeMetrics() {
  (
    window as unknown as { __unibrowserMeasureChromeHeights?: () => ChromeHeights }
  ).__unibrowserMeasureChromeHeights = measureChromeHeights;
}
