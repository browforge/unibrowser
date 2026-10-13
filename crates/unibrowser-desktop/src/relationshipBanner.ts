import type { TemplateRelationships } from "./relationshipConsent";
import { t } from "./i18n";

/** NEW-08: non-modal banner when template requires plugins still missing after consent. */
export function showRelationshipBannerIfNeeded(
  templateTitle: string,
  relationships: TemplateRelationships,
  enabledRequires: string[],
): void {
  document.querySelector(".relationship-require-banner")?.remove();
  const required = Object.keys(relationships.requires ?? {});
  const missing = required.filter((id) => !enabledRequires.includes(id));
  if (!missing.length) {
    return;
  }
  const banner = document.createElement("div");
  banner.className = "relationship-require-banner";
  banner.setAttribute("role", "status");
  const text = document.createElement("p");
  text.textContent = t("relationshipBanner.lead", {
    template: templateTitle,
    plugins: missing.join(", "),
  });
  const install = document.createElement("button");
  install.type = "button";
  install.className = "primary";
  install.textContent = t("relationshipBanner.install");
  install.onclick = () => {
    window.dispatchEvent(new CustomEvent("unibrowser-open-mod-portal-templates"));
  };
  const dismiss = document.createElement("button");
  dismiss.type = "button";
  dismiss.className = "quiet-button";
  dismiss.textContent = t("relationshipBanner.dismiss");
  dismiss.onclick = () => banner.remove();
  const actions = document.createElement("div");
  actions.className = "relationship-require-banner-actions";
  actions.append(install, dismiss);
  banner.append(text, actions);
  const host = document.getElementById("content-host") ?? document.body;
  host.prepend(banner);
}
