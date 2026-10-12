/** Measure compact chrome heights for audits and screenshot scripts. */

export interface ChromeHeights {
  titleBarPx: number;
  textCompactToolbarPx: number;
  commandManagerPx: number;
  totalChromePx: number;
  cmCollapsed: boolean;
  cmDensity: string | undefined;
  /** When set, overrides `document.body` for audits in unit tests. */
  textCompactVisible?: boolean;
}

export function measureChromeHeights(): ChromeHeights {
  const title = document.getElementById("title-bar");
  const compact = document.getElementById("text-compact-toolbar-host");
  const cm = document.getElementById("command-manager");
  const cmCollapsed = document.body.classList.contains("cm-collapsed");
  const titleBarPx = title ? Math.round(title.getBoundingClientRect().height) : 0;
  const compactHidden =
    cmCollapsed || compact?.classList.contains("hidden") || compact?.offsetParent === null;
  const textCompactToolbarPx =
    !compactHidden && compact ? Math.round(compact.getBoundingClientRect().height) : 0;
  const cmStyle = cm ? getComputedStyle(cm) : null;
  const cmHidden =
    cmCollapsed || (cmStyle?.display === "none") || (cmStyle?.visibility === "hidden");
  const commandManagerPx =
    cm && !cmHidden ? Math.round(cm.getBoundingClientRect().height) : 0;
  return {
    titleBarPx,
    textCompactToolbarPx,
    commandManagerPx,
    totalChromePx: titleBarPx + textCompactToolbarPx + commandManagerPx,
    cmCollapsed,
    cmDensity: document.body.dataset.cmDensity,
    textCompactVisible: document.body.classList.contains("text-compact-visible"),
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
  } else if (
    heights.textCompactVisible === true ||
    (typeof document !== "undefined" &&
      document.body.classList.contains("text-compact-visible"))
  ) {
    if (heights.textCompactToolbarPx > 34) {
      errors.push(`text compact toolbar ${heights.textCompactToolbarPx}px exceeds 32px row`);
    }
    if (heights.totalChromePx < 108 || heights.totalChromePx > 120) {
      errors.push(`text compact chrome ${heights.totalChromePx}px outside 108-120px (2 rows)`);
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
