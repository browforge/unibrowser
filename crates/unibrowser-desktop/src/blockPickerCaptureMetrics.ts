export interface BlockPickerBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface BlockPickerClosedFingerprint {
  meanR: number;
  meanG: number;
  meanB: number;
  inkPixels: number;
  sampledPixels: number;
}

export interface BlockPickerCaptureMetrics {
  viewportWidth: number;
  viewportHeight: number;
  probe: BlockPickerBounds;
  trigger: BlockPickerBounds;
  menu: BlockPickerBounds | null;
  label: BlockPickerBounds | null;
  badge: BlockPickerBounds | null;
  closedFingerprint: BlockPickerClosedFingerprint | null;
}

function domRectToBounds(rect: DOMRect): BlockPickerBounds {
  return {
    x: rect.left,
    y: rect.top,
    width: rect.width,
    height: rect.height,
  };
}

function findOpenBlockPickerMenu(): HTMLElement | null {
  return (
    document.querySelector<HTMLElement>(
      "#ub-block-picker-popover-host .text-compact-block-menu:not(.hidden)",
    ) ??
    document.querySelector<HTMLElement>(
      "#text-compact-toolbar-host .text-compact-block-menu:not(.hidden)",
    )
  );
}

export function blockPickerProbeBelowTrigger(host: HTMLElement): BlockPickerBounds {
  const trigger = host.querySelector<HTMLElement>(".text-compact-block-trigger");
  const rect = trigger?.getBoundingClientRect();
  const left = rect?.left ?? 0;
  const width = Math.max(180, rect?.width ?? 180);
  return {
    x: left,
    y: (rect?.bottom ?? 0) + 4,
    width,
    height: 140,
  };
}

export function collectBlockPickerCaptureMetrics(
  host: HTMLElement,
  closedFingerprint: BlockPickerClosedFingerprint | null,
): BlockPickerCaptureMetrics {
  const trigger = host.querySelector<HTMLElement>(".text-compact-block-trigger");
  const triggerRect = trigger?.getBoundingClientRect();
  const menu = findOpenBlockPickerMenu();
  const label = host.querySelector<HTMLElement>(".text-compact-block-label");
  const badge = trigger?.querySelector<HTMLElement>(
    ".block-emulation-badge, .cm-emulation-mark",
  );
  return {
    viewportWidth: window.innerWidth,
    viewportHeight: window.innerHeight,
    probe: blockPickerProbeBelowTrigger(host),
    trigger: triggerRect ? domRectToBounds(triggerRect) : { x: 0, y: 0, width: 0, height: 0 },
    menu: menu ? domRectToBounds(menu.getBoundingClientRect()) : null,
    label: label ? domRectToBounds(label.getBoundingClientRect()) : null,
    badge: badge ? domRectToBounds(badge.getBoundingClientRect()) : null,
    closedFingerprint,
  };
}
