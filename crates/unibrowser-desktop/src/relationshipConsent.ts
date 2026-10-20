import {
  conflictingEnabledIds,
  isRelationshipCreateBlocked,
  missingRequiredIds,
  relationshipReason,
} from "./relationshipLogic";

export interface TemplateRelationships {
  requires: Record<string, string>;
  recommends: Record<string, string>;
  optional: Record<string, string>;
  incompatible: Record<string, string>;
}

export interface RelationshipConsentOptions {
  installedModIds?: string[];
  enabledModIds?: string[];
}

export interface RelationshipConsentResult {
  enabledRequires: string[];
  enabledRecommends: string[];
  enabledOptional: string[];
  cancelled: boolean;
}

/** ADR 0037 relationship consent sheet before template Create. */
export function confirmTemplateRelationships(
  templateTitle: string,
  relationships: TemplateRelationships,
  options: RelationshipConsentOptions = {},
): Promise<RelationshipConsentResult> {
  const installed = options.installedModIds ?? [];
  const enabledMods = options.enabledModIds ?? installed;
  const requires = Object.keys(relationships.requires ?? {});
  const recommends = Object.keys(relationships.recommends ?? {});
  const optional = Object.keys(relationships.optional ?? {});
  const missing = missingRequiredIds(relationships, installed);
  const conflicts = conflictingEnabledIds(relationships, enabledMods);
  const blocked = isRelationshipCreateBlocked(relationships, installed, enabledMods);

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
      li.className = "relationship-consent-required";
      const label = document.createElement("label");
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = true;
      cb.disabled = true;
      cb.dataset.pluginId = id;
      const reason = relationshipReason(
        relationships.requires[id],
        "Required for this template.",
      );
      label.append(
        cb,
        document.createTextNode(` Required: ${id} — ${reason}`),
      );
      li.appendChild(label);
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
      const reason = relationshipReason(
        relationships.recommends[id],
        "Recommended companion plugin.",
      );
      label.append(cb, document.createTextNode(` Recommended: ${id} — ${reason}`));
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
      const reason = relationshipReason(
        relationships.optional[id],
        "Optional integration.",
      );
      label.append(cb, document.createTextNode(` Optional: ${id} — ${reason}`));
      li.appendChild(label);
      list.appendChild(li);
    });
    if (conflicts.length) {
      const li = document.createElement("li");
      li.className = "relationship-consent-incompatible";
      conflicts.forEach((id) => {
        const reason = relationshipReason(
          relationships.incompatible[id],
          "Cannot run with this template.",
        );
        li.appendChild(document.createTextNode(` Incompatible: ${id} — ${reason}`));
      });
      list.appendChild(li);
    }
    if (!requires.length && !recommends.length && !optional.length && !conflicts.length) {
      const li = document.createElement("li");
      li.textContent = "No plugin relationships declared for this template.";
      list.appendChild(li);
    }
    panel.appendChild(list);

    if (missing.length) {
      const note = document.createElement("p");
      note.className = "relationship-consent-missing viewer-note";
      note.textContent = `Install missing plugins before creating: ${missing.join(", ")}`;
      panel.appendChild(note);
    }

    const actions = document.createElement("div");
    actions.className = "template-gallery-actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = "Cancel";
    const create = document.createElement("button");
    create.type = "button";
    create.className = "primary";
    create.textContent = "Create";
    create.disabled = blocked;
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
      if (blocked) {
        return;
      }
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

const CAPTURE_RELATIONSHIPS: TemplateRelationships = {
  requires: {
    "base-video": "Video timeline and preview handlers.",
  },
  recommends: {
    "base-audio": "Mix audio clips on the timeline.",
  },
  optional: {
    "grammar-harper": "Spellcheck for title cards.",
  },
  incompatible: {
    "legacy-video-flat": "Uses a flat clip model; enable only one video path.",
  },
};

/** Doc capture overlay for relationship consent (NEW-08 evidence). */
export function showRelationshipConsentCaptureOverlay(): void {
  void confirmTemplateRelationships("Simple video", CAPTURE_RELATIONSHIPS, {
    installedModIds: [],
    enabledModIds: ["legacy-video-flat"],
  });
}
