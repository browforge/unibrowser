import { invoke } from "@tauri-apps/api/core";
import { formatCapabilityChip } from "./capabilityProviders";
import { t } from "./i18n";

type SurfaceProvider = {
  mod_name: string;
  mod_title: string;
  capability_id: string;
  default: boolean;
  priority: number;
  content_kinds: string[];
};

type SurfaceCapabilityGroup = {
  capability_id: string;
  label: string;
  providers: SurfaceProvider[];
  user_default: string | null;
};

export async function appendPluginDefaultsSection(root: HTMLElement) {
  const section = document.createElement("section");
  section.className = "settings-section";
  const heading = document.createElement("h3");
  heading.textContent = t("settings.pluginDefaultsHeading");
  section.appendChild(heading);
  const lead = document.createElement("p");
  lead.className = "viewer-note";
  lead.textContent = t("settings.pluginDefaultsLead");
  section.appendChild(lead);

  const groups = await invoke<SurfaceCapabilityGroup[]>("list_surface_capability_groups");
  if (!groups.length) {
    const empty = document.createElement("p");
    empty.textContent = t("settings.pluginDefaultsEmpty");
    section.appendChild(empty);
    root.appendChild(section);
    return section;
  }

  for (const group of groups) {
    const row = document.createElement("div");
    row.className = "settings-field-row";
    const label = document.createElement("label");
    label.textContent = group.label;
    const select = document.createElement("select");
    select.className = "themed-select";
    const system = document.createElement("option");
    system.value = "";
    system.textContent = t("settings.pluginDefaultsSystem");
    select.appendChild(system);
    for (const provider of group.providers) {
      const opt = document.createElement("option");
      opt.value = provider.mod_name;
      opt.textContent = `${provider.mod_title} (${provider.mod_name})`;
      select.appendChild(opt);
    }
    select.value = group.user_default ?? "";
    select.onchange = () => {
      void invoke("set_capability_default_provider", {
        capabilityId: group.capability_id,
        modName: select.value || null,
      });
    };
    const chips = document.createElement("p");
    chips.className = "viewer-note";
    chips.textContent = group.providers
      .map((p) => formatCapabilityChip(p.capability_id))
      .join(", ");
    row.append(label, select, chips);
    section.appendChild(row);
  }

  root.appendChild(section);
  return section;
}
