let announcer: HTMLElement | null = null;

export function ensureCmAnnouncer(): HTMLElement {
  if (announcer && document.contains(announcer)) {
    return announcer;
  }
  announcer = document.getElementById("cm-a11y-announcer");
  if (announcer) {
    return announcer;
  }
  announcer = document.createElement("div");
  announcer.id = "cm-a11y-announcer";
  announcer.className = "visually-hidden";
  announcer.setAttribute("role", "status");
  announcer.setAttribute("aria-live", "polite");
  announcer.setAttribute("aria-atomic", "true");
  document.body.appendChild(announcer);
  return announcer;
}

export function announceCm(message: string) {
  const el = ensureCmAnnouncer();
  el.textContent = "";
  window.setTimeout(() => {
    el.textContent = message;
  }, 10);
}
