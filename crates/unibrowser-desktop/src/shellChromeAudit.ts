/** Batch-3 shell invariants for self-test and doc capture guards. */

import { measureChromeHeights } from "./ui/chromeMetrics";

export function runShellChromeAudit(): string[] {
  const errors: string[] = [];

  const legacyIds = [
    "mods-line",
    "btn-demo-download",
    "command-manager-export",
    "command-manager-import",
    "btn-mod-portal",
    "btn-split",
  ];
  for (const id of legacyIds) {
    if (document.getElementById(id)) {
      errors.push(`legacy chrome element #${id} still present`);
    }
  }
  if (document.querySelector(".ai-assistant-drag-handle")) {
    errors.push("floating AI drag handle still present");
  }
  if (!document.getElementById("app-menu-bar")) {
    errors.push("missing #app-menu-bar");
  }
  if (!document.getElementById("title-bar")) {
    errors.push("missing #title-bar");
  }
  if (!document.getElementById("command-manager-groups")) {
    errors.push("missing #command-manager-groups");
  }
  if (!document.querySelector(".workspace-rail")) {
    errors.push("missing .workspace-rail activity rail");
  }
  if (document.querySelector(".nav-row")) {
    errors.push("legacy .nav-row still present");
  }
  if (document.querySelector(".ub-shell-title-menu")) {
    errors.push("legacy .ub-shell-title-menu still present");
  }
  if (document.getElementById("command-manager-tabs")) {
    errors.push("legacy #command-manager-tabs still present");
  }
  if (!document.getElementById("work-area")) {
    errors.push("missing #work-area dock layout");
  }

  const chrome = document.getElementById("chrome");
  const ai = document.getElementById("ai-assistant-panel");
  const work = document.getElementById("work-area");
  const content = document.getElementById("content-host");
  if (ai && chrome && work && content && !ai.classList.contains("hidden")) {
    const pos = getComputedStyle(ai).position;
    if (pos === "fixed" || pos === "absolute") {
      errors.push(`AI panel uses ${pos} positioning (must be docked in work area)`);
    }
    const ar = ai.getBoundingClientRect();
    const cr = chrome.getBoundingClientRect();
    const wr = work.getBoundingClientRect();
    const contentR = content.getBoundingClientRect();
    if (ar.top < cr.bottom - 2) {
      errors.push("AI panel overlaps title/menu or command chrome");
    }
    if (ar.left < contentR.right - 4 && ar.right > contentR.left + 4) {
      if (ar.top < contentR.bottom - 8 && ar.bottom > contentR.top + 8) {
        errors.push("AI panel overlaps content region (non-push layout)");
      }
    }
    if (ar.right > wr.right + 2 || ar.left < wr.left - 2) {
      errors.push("AI panel outside work-area bounds");
    }
    const toolsDump = ai.querySelector(".ai-assistant-tools");
    if (toolsDump) {
      errors.push("AI panel still lists exhaustive tools in body");
    }
  }

  const shellGen = document.documentElement.getAttribute("data-ub-shell");
  if (shellGen !== "batch3-v1") {
    errors.push("missing data-ub-shell=batch3-v1 on document element");
  }

  const heights = measureChromeHeights();
  const density = document.body.dataset.cmDensity;
  if (heights.cmCollapsed) {
    if (heights.commandManagerPx !== 0) {
      errors.push(
        `cm-collapsed but command manager still measures ${heights.commandManagerPx}px tall`,
      );
    }
    if (heights.totalChromePx < 38 || heights.totalChromePx > 42) {
      errors.push(
        `collapsed chrome height ${heights.totalChromePx}px (expected ~40px title bar only)`,
      );
    }
  } else if (density === "compact") {
    if (heights.totalChromePx < 78 || heights.totalChromePx > 82) {
      errors.push(
        `compact chrome height ${heights.totalChromePx}px (title ${heights.titleBarPx}px, cm ${heights.commandManagerPx}px) outside 78-82px budget`,
      );
    }
  }

  return errors;
}

export function exposeShellChromeAudit() {
  (window as unknown as { __unibrowserRunShellChromeAudit?: () => string[] }).__unibrowserRunShellChromeAudit =
    runShellChromeAudit;
}
