const BREADCRUMB_PARTS = ["Projects", "Field kit"];

export function pathToBreadcrumbSegments(filePath: string): string[] {
  const name = filePath.replace(/^file:\/\//, "").split(/[/\\]/).pop() ?? filePath;
  return [...BREADCRUMB_PARTS, name];
}

export function renderTitleBreadcrumb(filePath: string | null, useBreadcrumb: boolean) {
  const crumbEl = document.getElementById("title-breadcrumb");
  const omnibox = document.getElementById("omnibox") as HTMLInputElement | null;
  if (!crumbEl || !omnibox) {
    return;
  }
  const show = useBreadcrumb && Boolean(filePath) && !filePath!.startsWith("http");
  crumbEl.classList.toggle("hidden", !show);
  omnibox.classList.toggle("hidden", show);
  if (!show) {
    crumbEl.replaceChildren();
    return;
  }
  crumbEl.replaceChildren();
  const segments = pathToBreadcrumbSegments(filePath!);
  segments.forEach((part, idx) => {
    if (idx < segments.length - 1) {
      const muted = document.createElement("span");
      muted.className = "crumb-muted";
      muted.textContent = `${part} / `;
      crumbEl.appendChild(muted);
    } else {
      const strong = document.createElement("strong");
      strong.textContent = part;
      crumbEl.appendChild(strong);
    }
  });
}
