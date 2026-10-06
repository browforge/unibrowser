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
  const cmStyle = cm ? getComputedStyle(cm) : null;
  const cmHidden =
    cmCollapsed || (cmStyle?.display === "none") || (cmStyle?.visibility === "hidden");
  const commandManagerPx =
    cm && !cmHidden ? Math.round(cm.getBoundingClientRect().height) : 0;
  return {
    titleBarPx,
    commandManagerPx,
    totalChromePx: titleBarPx + commandManagerPx,
    cmCollapsed,
    cmDensity: document.body.dataset.cmDensity,
  };
}

export function validateChromeHeightBudget(heights: ChromeHeights): string[] {
  const errors: string[] = [];
  if (heights.cmCollapsed) {
    if (heights.commandManagerPx !== 0) {
      errors.push(`cm-collapsed but CM height is ${heights.commandManagerPx}px`);
    }
    if (heights.totalChromePx < 38 || heights.totalChromePx > 42) {
      errors.push(`collapsed chrome ${heights.totalChromePx}px (expected ~40px)`);
    }
  } else if (heights.cmDensity === "compact") {
    if (heights.totalChromePx < 78 || heights.totalChromePx > 82) {
      errors.push(`compact chrome ${heights.totalChromePx}px outside 78-82px`);
    }
  }
  return errors;
}

export function exposeChromeMetrics() {
  (
    window as unknown as { __unibrowserMeasureChromeHeights?: () => ChromeHeights }
  ).__unibrowserMeasureChromeHeights = measureChromeHeights;
}
