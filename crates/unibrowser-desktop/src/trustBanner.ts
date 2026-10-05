export type TrustBannerKind = "remote-web" | "remote-html" | "user-script";

const COPY: Record<TrustBannerKind, string> = {
  "remote-web":
    "Remote web content. Scripts and media run in an isolated webview; treat as untrusted.",
  "remote-html":
    "HTML from this document may load remote resources. Layout preview only; do not run embedded scripts.",
  "user-script":
    "User script. Rhai runs locally with an allowlist; only enable document commands when you trust the file.",
};

export function appendTrustBanner(
  root: HTMLElement,
  kind: TrustBannerKind,
  detail?: string,
) {
  const existing = root.querySelector(".trust-banner");
  if (existing) {
    existing.remove();
  }
  const banner = document.createElement("p");
  banner.className = `trust-banner trust-banner--${kind}`;
  banner.setAttribute("role", "status");
  banner.textContent = detail ? `${COPY[kind]} ${detail}` : COPY[kind];
  root.prepend(banner);
}
