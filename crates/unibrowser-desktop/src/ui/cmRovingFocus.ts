/** Roving tabindex and keyboard navigation for Command Manager control groups (CMD-04). */

export type CmRovingBindings = {
  groupsHost: HTMLElement;
  overflowButton: HTMLElement | null;
  onOpenOverflow: () => void;
  onAnnounce: (message: string) => void;
};

function focusableControls(root: HTMLElement): HTMLButtonElement[] {
  return Array.from(
    root.querySelectorAll<HTMLButtonElement>(".cm-group-tools button.ui-command:not(:disabled)"),
  );
}

export function bindCmRovingFocus(bind: CmRovingBindings) {
  const { groupsHost, overflowButton, onOpenOverflow, onAnnounce } = bind;

  const resetTabIndex = () => {
    const controls = focusableControls(groupsHost);
    controls.forEach((btn, i) => {
      btn.tabIndex = i === 0 ? 0 : -1;
    });
  };

  const observer = new MutationObserver(() => resetTabIndex());
  observer.observe(groupsHost, { childList: true, subtree: true });
  resetTabIndex();

  groupsHost.addEventListener("keydown", (e) => {
    const controls = focusableControls(groupsHost);
    if (!controls.length) {
      return;
    }
    const current = document.activeElement as HTMLElement | null;
    let index = controls.findIndex((btn) => btn === current);
    if (index < 0) {
      index = controls.findIndex((btn) => btn.tabIndex === 0);
    }
    if (index < 0) {
      index = 0;
    }

    if (e.key === "ArrowRight" || e.key === "ArrowDown") {
      e.preventDefault();
      const next = (index + 1) % controls.length;
      controls.forEach((btn, i) => {
        btn.tabIndex = i === next ? 0 : -1;
      });
      controls[next].focus();
      return;
    }
    if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
      e.preventDefault();
      const next = (index - 1 + controls.length) % controls.length;
      controls.forEach((btn, i) => {
        btn.tabIndex = i === next ? 0 : -1;
      });
      controls[next].focus();
      return;
    }
    if (e.key === "Home") {
      e.preventDefault();
      controls.forEach((btn, i) => {
        btn.tabIndex = i === 0 ? 0 : -1;
      });
      controls[0].focus();
      return;
    }
    if (e.key === "End") {
      e.preventDefault();
      const last = controls.length - 1;
      controls.forEach((btn, i) => {
        btn.tabIndex = i === last ? 0 : -1;
      });
      controls[last].focus();
      return;
    }
    if (e.key === "Enter" || e.key === " ") {
      const target = e.target as HTMLButtonElement | null;
      if (target && controls.includes(target)) {
        e.preventDefault();
        void announceControl(target, onAnnounce);
        target.click();
      }
      return;
    }
    if (e.key === "ArrowDown" && e.altKey) {
      e.preventDefault();
      if (overflowButton && !overflowButton.classList.contains("hidden")) {
        onOpenOverflow();
        overflowButton.focus();
      }
      return;
    }
    if (e.key === "Escape") {
      const active = document.activeElement as HTMLElement | null;
      if (active && groupsHost.contains(active)) {
        e.preventDefault();
        active.blur();
      }
    }
  });
}

function announceControl(btn: HTMLButtonElement, onAnnounce: (message: string) => void) {
  const label = btn.querySelector(".cm-cmd-label")?.textContent?.trim() ?? "Command";
  if (btn.classList.contains("emulated")) {
    const note = btn.getAttribute("aria-description") ?? btn.title;
    onAnnounce(note ? `${label}, emulated, ${note}` : `${label}, emulated`);
    return;
  }
  if (btn.disabled) {
    const reason = btn.getAttribute("aria-description") ?? btn.title;
    onAnnounce(reason ? `${label}, unavailable, ${reason}` : `${label}, unavailable`);
    return;
  }
  if (btn.classList.contains("mixed")) {
    onAnnounce(`${label}, mixed selection`);
  }
}
