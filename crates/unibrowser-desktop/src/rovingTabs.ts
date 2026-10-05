/** Roving tabindex + arrow-key navigation for a row of tab buttons. */

export function bindRovingTablist(
  listEl: HTMLElement,
  getButtons: () => HTMLButtonElement[],
) {
  listEl.addEventListener("keydown", (e) => {
    const buttons = getButtons().filter((b) => !b.disabled);
    if (!buttons.length) {
      return;
    }
    const current = document.activeElement as HTMLElement | null;
    let index = buttons.findIndex((b) => b === current);
    if (index < 0) {
      index = buttons.findIndex((b) => b.getAttribute("aria-selected") === "true");
    }
    if (index < 0) {
      index = 0;
    }
    let next = index;
    if (e.key === "ArrowRight" || e.key === "ArrowDown") {
      next = (index + 1) % buttons.length;
      e.preventDefault();
    } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
      next = (index - 1 + buttons.length) % buttons.length;
      e.preventDefault();
    } else if (e.key === "Home") {
      next = 0;
      e.preventDefault();
    } else if (e.key === "End") {
      next = buttons.length - 1;
      e.preventDefault();
    } else if (e.key === "Enter" || e.key === " ") {
      const target = e.target as HTMLButtonElement | null;
      if (target && buttons.includes(target)) {
        e.preventDefault();
        target.click();
      }
      return;
    } else {
      return;
    }
    buttons.forEach((btn, i) => {
      btn.tabIndex = i === next ? 0 : -1;
    });
    buttons[next].focus();
  });
}

export function setTabA11y(
  btn: HTMLButtonElement,
  selected: boolean,
  id: string,
  panelId?: string,
) {
  btn.id = id;
  btn.setAttribute("role", "tab");
  btn.setAttribute("aria-selected", selected ? "true" : "false");
  btn.tabIndex = selected ? 0 : -1;
  if (panelId) {
    btn.setAttribute("aria-controls", panelId);
  }
}
