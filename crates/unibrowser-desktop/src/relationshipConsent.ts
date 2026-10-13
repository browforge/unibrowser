export interface TemplateRelationships {
  requires: Record<string, string>;
  recommends: Record<string, string>;
  optional: Record<string, string>;
  incompatible: Record<string, string>;
}

export interface RelationshipConsentResult {
  enabledRequires: string[];
  enabledRecommends: string[];
  enabledOptional: string[];
  cancelled: boolean;
}

/** ADR 0037 consent sheet (stub UI — full sheet ships with REL-02). */
export function confirmTemplateRelationships(
  templateTitle: string,
  relationships: TemplateRelationships,
): Promise<RelationshipConsentResult> {
  const requires = Object.keys(relationships.requires ?? {});
  const recommends = Object.keys(relationships.recommends ?? {});
  const optional = Object.keys(relationships.optional ?? {});
  return new Promise((resolve) => {
    const overlay = document.createElement("div");
    overlay.className = "template-gallery-overlay relationship-consent-overlay";
    const panel = document.createElement("div");
    panel.className = "template-gallery-panel relationship-consent-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");

    const title = document.createElement("h2");
    title.textContent = `Create “${templateTitle}”`;
    panel.appendChild(title);

    const lead = document.createElement("p");
    lead.className = "viewer-note";
    lead.textContent =
      "This template may enable plugins. Required items stay on; you can uncheck recommendations and optional integrations.";
    panel.appendChild(lead);

    const list = document.createElement("ul");
    list.className = "relationship-consent-list";
    requires.forEach((id) => {
      const li = document.createElement("li");
      li.textContent = `Required: ${id} — ${relationships.requires[id] ?? ""}`;
      list.appendChild(li);
    });
    recommends.forEach((id) => {
      const li = document.createElement("li");
      const label = document.createElement("label");
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = true;
      cb.dataset.pluginId = id;
      cb.className = "consent-recommend";
      label.append(cb, document.createTextNode(` Recommended: ${id}`));
      li.appendChild(label);
      list.appendChild(li);
    });
    optional.forEach((id) => {
      const li = document.createElement("li");
      const label = document.createElement("label");
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = false;
      cb.dataset.pluginId = id;
      cb.className = "consent-optional";
      label.append(cb, document.createTextNode(` Optional: ${id}`));
      li.appendChild(label);
      list.appendChild(li);
    });
    if (!requires.length && !recommends.length && !optional.length) {
      const li = document.createElement("li");
      li.textContent = "No plugin relationships declared for this template.";
      list.appendChild(li);
    }
    panel.appendChild(list);

    const actions = document.createElement("div");
    actions.className = "template-gallery-actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = "Cancel";
    const create = document.createElement("button");
    create.type = "button";
    create.className = "primary";
    create.textContent = "Create";
    const close = (result: RelationshipConsentResult) => {
      overlay.remove();
      resolve(result);
    };
    cancel.onclick = () =>
      close({
        enabledRequires: requires,
        enabledRecommends: [],
        enabledOptional: [],
        cancelled: true,
      });
    create.onclick = () => {
      const enabledRecommends = Array.from(
        panel.querySelectorAll<HTMLInputElement>(".consent-recommend:checked"),
      ).map((el) => el.dataset.pluginId!);
      const enabledOptional = Array.from(
        panel.querySelectorAll<HTMLInputElement>(".consent-optional:checked"),
      ).map((el) => el.dataset.pluginId!);
      close({
        enabledRequires: requires,
        enabledRecommends,
        enabledOptional,
        cancelled: false,
      });
    };
    overlay.addEventListener("click", (e) => {
      if (e.target === overlay) {
        cancel.click();
      }
    });
    actions.append(cancel, create);
    panel.appendChild(actions);
    overlay.appendChild(panel);
    document.body.appendChild(overlay);
    create.focus();
  });
}
