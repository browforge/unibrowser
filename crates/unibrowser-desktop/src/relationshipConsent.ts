import { createLucideIcon } from "./ui/lucideIcon";
import { t } from "./i18n";
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

function formatPluginLabel(id: string): string {
  return id
    .split(/[-_.]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function appendPluginRow(
  container: HTMLElement,
  id: string,
  description: string,
  kind: "required" | "recommend" | "optional" | "incompatible",
): void {
  const row = document.createElement("div");
  row.className = `relationship-consent-row relationship-consent-row--${kind}`;
  const control = document.createElement("div");
  control.className = "relationship-consent-row-control";
  if (kind === "required") {
    control.appendChild(createLucideIcon("lock", 16, "ub-icon svg-icon"));
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = true;
    cb.disabled = true;
    cb.dataset.pluginId = id;
    cb.setAttribute("aria-label", formatPluginLabel(id));
    control.appendChild(cb);
  } else if (kind === "incompatible") {
    control.appendChild(createLucideIcon("triangle-alert", 16, "ub-icon svg-icon"));
  } else {
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = kind === "recommend";
    cb.dataset.pluginId = id;
    cb.className = kind === "recommend" ? "consent-recommend" : "consent-optional";
    control.appendChild(cb);
  }
  const body = document.createElement("div");
  body.className = "relationship-consent-row-body";
  const name = document.createElement("div");
  name.className = "relationship-consent-row-name";
  name.textContent = formatPluginLabel(id);
  const desc = document.createElement("div");
  desc.className = "relationship-consent-row-desc";
  desc.textContent = description;
  body.append(name, desc);
  if (kind === "incompatible") {
    row.append(control, body);
  } else {
    const label = document.createElement("label");
    label.className = "relationship-consent-row-label";
    label.append(control, body);
    row.appendChild(label);
  }
  container.appendChild(row);
}

function appendSection(
  parent: HTMLElement,
  heading: string,
  build: (rows: HTMLElement) => void,
): void {
  const section = document.createElement("section");
  section.className = "relationship-consent-section";
  const h = document.createElement("h3");
  h.className = "relationship-consent-section-heading";
  h.textContent = heading;
  const rows = document.createElement("div");
  rows.className = "relationship-consent-rows";
  build(rows);
  if (!rows.childElementCount) {
    return;
  }
  section.append(h, rows);
  parent.appendChild(section);
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
    lead.className = "viewer-note relationship-consent-lead";
    lead.textContent =
      "This template may enable plugins. Required items stay on; you can uncheck recommendations and optional integrations.";
    panel.appendChild(lead);

    const sections = document.createElement("div");
    sections.className = "relationship-consent-sections";

    appendSection(sections, "Required", (rows) => {
      requires.forEach((id) => {
        appendPluginRow(
          rows,
          id,
          relationshipReason(relationships.requires[id], "Required for this template."),
          "required",
        );
      });
    });
    appendSection(sections, "Recommended", (rows) => {
      recommends.forEach((id) => {
        appendPluginRow(
          rows,
          id,
          relationshipReason(relationships.recommends[id], "Recommended companion plugin."),
          "recommend",
        );
      });
    });
    appendSection(sections, "Optional", (rows) => {
      optional.forEach((id) => {
        appendPluginRow(
          rows,
          id,
          relationshipReason(relationships.optional[id], "Optional integration."),
          "optional",
        );
      });
    });
    appendSection(sections, "Incompatible", (rows) => {
      conflicts.forEach((id) => {
        appendPluginRow(
          rows,
          id,
          relationshipReason(relationships.incompatible[id], "Cannot run with this template."),
          "incompatible",
        );
      });
    });

    if (!sections.childElementCount) {
      const empty = document.createElement("p");
      empty.className = "viewer-note";
      empty.textContent = "No plugin relationships declared for this template.";
      panel.appendChild(empty);
    } else {
      panel.appendChild(sections);
    }

    if (missing.length) {
      const callout = document.createElement("div");
      callout.className = "relationship-consent-callout";
      callout.setAttribute("role", "note");
      const icon = createLucideIcon("info", 18, "ub-icon svg-icon");
      const copy = document.createElement("div");
      copy.className = "relationship-consent-callout-copy";
      const calloutLead = document.createElement("p");
      calloutLead.textContent = t("relationshipConsent.missingLead", {
        plugins: missing.map(formatPluginLabel).join(", "),
      });
      copy.appendChild(calloutLead);
      const install = document.createElement("button");
      install.type = "button";
      install.className = "primary";
      install.textContent = t("relationshipConsent.install");
      install.onclick = () => {
        window.dispatchEvent(new CustomEvent("unibrowser-open-mod-portal-templates"));
      };
      callout.append(icon, copy, install);
      panel.appendChild(callout);
    }

    const actions = document.createElement("div");
    actions.className = "template-gallery-actions relationship-consent-actions";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.className = "quiet-button";
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
