/** Hide transient chrome before doc-capture screenshots (rail flyouts, pinned CM). */
export function dismissDocCaptureChrome(): void {
  document.body.classList.add("cm-collapsed");
  const rail = document.getElementById("activity-rail");
  rail?.querySelectorAll(".rail-destination.active").forEach((el) => {
    el.classList.remove("active");
  });
  rail?.querySelectorAll(".rail-destination").forEach((el) => {
    if (el instanceof HTMLElement) {
      el.blur();
    }
  });
  rail?.querySelector<HTMLElement>(".rail-side-panel")?.classList.add("hidden");
  document.getElementById("command-manager-overflow")?.classList.remove("visible");
  document.querySelectorAll<HTMLElement>(".cm-overflow-menu").forEach((menu) => {
    menu.classList.add("hidden");
  });
}
