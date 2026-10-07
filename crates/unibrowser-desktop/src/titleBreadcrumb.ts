export function pathToBreadcrumbSegments(filePath: string): string[] {
  const normalised = filePath.replace(/^file:\/\//, "");
  const parts = normalised.split(/[/\\]/).filter(Boolean);
  if (parts.length <= 3) {
    return parts;
  }
  return [...parts.slice(0, -2), parts[parts.length - 2], parts[parts.length - 1]];
}

export function renderTitleBreadcrumb(filePath: string | null, useBreadcrumb: boolean) {
  const crumbEl = document.getElementById("title-breadcrumb");
  const omnibox = document.getElementById("omnibox") as HTMLInputElement | null;
  if (!crumbEl || !omnibox) {
    return;
  }
  const show =
    useBreadcrumb &&
    Boolean(filePath) &&
    !filePath!.startsWith("http") &&
    !filePath!.startsWith("unibrowser://");
  crumbEl.classList.toggle("hidden", !show);
  omnibox.classList.toggle("hidden", show);
  omnibox.toggleAttribute("aria-hidden", show);
  omnibox.style.display = show ? "none" : "";
  if (!show) {
    crumbEl.replaceChildren();
    return;
  }
  crumbEl.replaceChildren();
  const segments = pathToBreadcrumbSegments(filePath!);
  segments.forEach((part, idx) => {
    const segment = document.createElement(idx < segments.length - 1 ? "span" : "strong");
    segment.className = idx < segments.length - 1 ? "crumb-muted" : "crumb-current";
    segment.textContent = part;
    crumbEl.appendChild(segment);
    if (idx >= segments.length - 1) {
      return;
    }
    const separator = document.createElement("span");
    separator.className = "crumb-separator";
    separator.setAttribute("aria-hidden", "true");
    separator.textContent = "/";
    crumbEl.appendChild(separator);
  });
}
