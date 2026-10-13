import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { confirmTemplateRelationships, type TemplateRelationships } from "./relationshipConsent";
import { showRelationshipBannerIfNeeded } from "./relationshipBanner";
import { enhanceSelectsIn } from "./ui/themedSelect";
import { createLucideIcon, type LucideName } from "./ui/lucideIcon";

export type { TemplateRelationships };

export interface ProjectTemplateSummary {
  id: string;
  version: string;
  title: string;
  description: string;
  categories: string[];
  tags: string[];
  source: string;
  default_workspace?: string | null;
  relationships: TemplateRelationships;
  needs_mod_ids: string[];
  pack_path?: string | null;
}

const RECENT_KEY = "unibrowser.recentTemplates";
const isWeb = import.meta.env.VITE_UB_PLATFORM === "web";

function loadRecent(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    if (!raw) {
      return [];
    }
    const parsed = JSON.parse(raw) as string[];
    return Array.isArray(parsed) ? parsed.slice(0, 5) : [];
  } catch {
    return [];
  }
}

function rememberRecent(templateId: string) {
  const next = [templateId, ...loadRecent().filter((id) => id !== templateId)].slice(0, 5);
  localStorage.setItem(RECENT_KEY, JSON.stringify(next));
}

export type TemplateGalleryCreate = {
  templateId: string;
  projectName: string;
  location: string;
  sameFolder: boolean;
};

export type TemplateGalleryCaptureOptions = {
  step?: "pick" | "configure";
  searchQuery?: string;
  selectedTemplateId?: string;
};

type GalleryMode = "live" | "capture";

type GalleryMountOptions = {
  mode: GalleryMode;
  step?: "pick" | "configure";
  searchQuery?: string;
  selectedTemplateId?: string;
  onCreate?: (payload: TemplateGalleryCreate) => void | Promise<void>;
};

function templateIconName(tpl: ProjectTemplateSummary): LucideName {
  const id = tpl.id.toLowerCase();
  const cat = tpl.categories.join(" ").toLowerCase();
  if (id.includes("blank") || tpl.title.toLowerCase() === "blank") {
    return "file";
  }
  if (cat.includes("notes") || id.includes("notes")) {
    return "message-square";
  }
  if (cat.includes("document") || id.includes("document") || id.includes("office")) {
    return "file-text";
  }
  if (cat.includes("presentation")) {
    return "layout-grid";
  }
  if (cat.includes("video")) {
    return "play";
  }
  if (cat.includes("design")) {
    return "pen-line";
  }
  return "layout-grid";
}

function templateIconEl(tpl: ProjectTemplateSummary): HTMLElement {
  const icon = document.createElement("span");
  icon.className = "template-gallery-row-icon";
  icon.setAttribute("aria-hidden", "true");
  icon.appendChild(createLucideIcon(templateIconName(tpl), 20, "ub-icon svg-icon"));
  return icon;
}

async function mountTemplateGallery(
  templates: ProjectTemplateSummary[],
  options: GalleryMountOptions,
): Promise<void> {
  let step: "pick" | "configure" = options.step ?? "pick";
  let selected: ProjectTemplateSummary | null =
    options.selectedTemplateId
      ? templates.find((tpl) => tpl.id === options.selectedTemplateId) ?? null
      : null;
  if (step === "configure" && !selected && templates.length) {
    selected = templates[0];
  }
  let sourceFilter = "all";
  let domainFilter = "all";
  let workspaceFilter = "all";
  let kindFilter = "all";
  let query = options.searchQuery ?? "";
  let sameFolder = true;

  const overlay = document.createElement("div");
  overlay.className = "template-gallery-overlay";
  const panel = document.createElement("div");
  panel.className = "template-gallery-panel";
  panel.setAttribute("role", "dialog");
  panel.setAttribute("aria-modal", "true");
  panel.setAttribute("aria-labelledby", "template-gallery-title");

  const heading = document.createElement("h1");
  heading.id = "template-gallery-title";
  heading.className = "template-gallery-title";
  heading.textContent = t("templateGallery.title");

  const layout = document.createElement("div");
  layout.className = "template-gallery-layout";

  const recentColumn = document.createElement("aside");
  recentColumn.className = "template-gallery-recent-column";
  const recentTitle = document.createElement("h2");
  recentTitle.className = "template-gallery-recent-heading";
  recentTitle.textContent = t("templateGallery.recentColumn");
  const recentList = document.createElement("div");
  recentList.className = "template-gallery-recent-list";
  recentColumn.append(recentTitle, recentList);

  const mainColumn = document.createElement("div");
  mainColumn.className = "template-gallery-main";

  const header = document.createElement("header");
  header.className = "template-gallery-header";
  const search = document.createElement("input");
  search.type = "search";
  search.className = "ub-form-control template-gallery-search";
  search.placeholder = t("templateGallery.search");
  search.setAttribute("aria-label", t("templateGallery.search"));
  search.value = query;

  const filters = document.createElement("div");
  filters.className = "template-gallery-filters";
  const domain = document.createElement("select");
  domain.className = "ub-form-control";
  domain.innerHTML =
    "<option value='all'>Domain: All</option><option value='document'>Document</option><option value='presentation'>Presentation</option><option value='video'>Video</option><option value='design'>Design</option><option value='notes'>Notes</option>";
  const kind = document.createElement("select");
  kind.className = "ub-form-control";
  kind.innerHTML =
    "<option value='all'>Kind: All</option><option value='starter'>Starter</option><option value='tutorial'>Tutorial</option><option value='hub'>Hub</option>";
  const source = document.createElement("select");
  source.className = "ub-form-control";
  source.innerHTML =
    "<option value='all'>Source: All</option><option value='builtin'>Built-in</option><option value='user'>My templates</option><option value='installed'>Installed</option>";
  filters.append(domain, kind, source);
  header.append(search, filters);

  const body = document.createElement("div");
  body.className = "template-gallery-body";

  const footer = document.createElement("footer");
  footer.className = "template-gallery-footer";
  const more = document.createElement("button");
  more.type = "button";
  more.className = "quiet-button";
  more.textContent = t("templateGallery.getMore");
  more.onclick = () => {
    window.dispatchEvent(new CustomEvent("unibrowser-open-mod-portal-templates"));
  };
  footer.appendChild(more);

  const actions = document.createElement("div");
  actions.className = "template-gallery-actions";

  mainColumn.append(header, body, footer);
  layout.append(recentColumn, mainColumn);
  panel.append(heading, layout, actions);

  const close = () => overlay.remove();

  const renderRecent = () => {
    recentList.replaceChildren();
    const recentIds = loadRecent();
    if (!recentIds.length) {
      const empty = document.createElement("p");
      empty.className = "viewer-note";
      empty.textContent = t("templateGallery.recentEmpty");
      recentList.appendChild(empty);
      return;
    }
    recentIds.forEach((id) => {
      const tpl = templates.find((x) => x.id === id);
      if (!tpl) {
        return;
      }
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "template-gallery-recent-item";
      btn.textContent = tpl.title;
      btn.onclick = () => {
        selected = tpl;
        step = "configure";
        render();
      };
      recentList.appendChild(btn);
    });
  };

  const filtered = () => {
    const q = query.trim().toLowerCase();
    return templates.filter((tpl) => {
      if (sourceFilter !== "all" && tpl.source !== sourceFilter) {
        return false;
      }
      if (domainFilter !== "all" && !tpl.categories.some((c) => c === domainFilter)) {
        return false;
      }
      if (
        workspaceFilter !== "all" &&
        (tpl.default_workspace ?? "").toLowerCase() !== workspaceFilter
      ) {
        return false;
      }
      if (kindFilter === "hub" && tpl.source !== "installed") {
        return false;
      }
      if (kindFilter === "starter" && tpl.source === "installed") {
        return false;
      }
      if (!q) {
        return true;
      }
      const hay =
        `${tpl.title} ${tpl.description} ${tpl.categories.join(" ")} ${tpl.tags.join(" ")}`.toLowerCase();
      return hay.includes(q);
    });
  };

  const renderPick = () => {
    heading.textContent = t("templateGallery.title");
    body.replaceChildren();
    renderRecent();
    const list = document.createElement("div");
    list.className = "template-gallery-list";
    const items = filtered();
    if (!items.length) {
      const empty = document.createElement("p");
      empty.className = "viewer-note";
      empty.textContent = t("templateGallery.empty");
      body.append(empty);
      return;
    }
    items.forEach((tpl) => {
      const row = document.createElement("button");
      row.type = "button";
      row.className = "template-gallery-row";
      if (selected?.id === tpl.id) {
        row.classList.add("selected");
      }
      const icon = templateIconEl(tpl);
      const copy = document.createElement("div");
      copy.className = "template-gallery-row-copy";
      const h = document.createElement("strong");
      h.textContent = tpl.title;
      const p = document.createElement("span");
      p.className = "template-gallery-row-desc";
      p.textContent = tpl.description;
      copy.append(h, p);
      const tags = document.createElement("div");
      tags.className = "template-gallery-tags";
      tpl.tags.forEach((tag) => {
        const chip = document.createElement("span");
        chip.className = "template-gallery-tag";
        chip.textContent = tag;
        tags.appendChild(chip);
      });
      if (tpl.needs_mod_ids.length) {
        const need = document.createElement("span");
        need.className = "template-gallery-needs-mod";
        need.textContent = `Needs ${tpl.needs_mod_ids[0]}`;
        tags.appendChild(need);
      }
      row.append(icon, copy, tags);
      row.onclick = () => {
        selected = tpl;
        list.querySelectorAll(".template-gallery-row").forEach((el) => el.classList.remove("selected"));
        row.classList.add("selected");
      };
      list.appendChild(row);
    });
    body.append(list);
  };

  const renderConfigure = () => {
    if (!selected) {
      step = "pick";
      render();
      return;
    }
    heading.textContent = t("templateGallery.configureTitle");
    body.replaceChildren();
    const sub = document.createElement("div");
    sub.className = "template-gallery-configure-subtitle";
    sub.append(templateIconEl(selected));
    const subTitle = document.createElement("span");
    subTitle.textContent = selected.title;
    sub.append(subTitle);
    const form = document.createElement("div");
    form.className = "template-gallery-configure template-gallery-configure-form";
    const nameField = document.createElement("label");
    nameField.className = "template-gallery-field";
    const nameLegend = document.createElement("span");
    nameLegend.textContent = t("templateGallery.projectName");
    const nameInput = document.createElement("input");
    nameInput.type = "text";
    nameInput.className = "ub-form-control";
    nameInput.value = selected.title;
    nameField.append(nameLegend, nameInput);
    const locField = document.createElement("label");
    locField.className = "template-gallery-field";
    const locLegend = document.createElement("span");
    locLegend.textContent = t("templateGallery.location");
    const locRow = document.createElement("div");
    locRow.className = "template-gallery-location-row";
    const locInput = document.createElement("input");
    locInput.type = "text";
    locInput.className = "ub-form-control";
    if (isWeb) {
      locInput.value = t("templateGallery.locationWebStored");
      locInput.readOnly = true;
    } else {
      locInput.placeholder = t("templateGallery.locationHint");
    }
    locRow.appendChild(locInput);
    if (!isWeb) {
      const browse = document.createElement("button");
      browse.type = "button";
      browse.textContent = t("templateGallery.browse");
      browse.onclick = async () => {
        const picked = await invoke<string | null>("pick_browse_folder");
        if (picked) {
          locInput.value = picked;
        }
      };
      locRow.appendChild(browse);
    }
    locField.append(locLegend, locRow);
    const sameLabel = document.createElement("label");
    sameLabel.className = "template-gallery-same-folder";
    const sameCb = document.createElement("input");
    sameCb.type = "checkbox";
    sameCb.checked = sameFolder;
    sameCb.onchange = () => {
      sameFolder = sameCb.checked;
    };
    sameLabel.append(sameCb, document.createTextNode(` ${t("templateGallery.sameFolder")}`));
    form.append(nameField, locField, sameLabel);
    body.append(sub, form);

    actions.replaceChildren();
    const back = document.createElement("button");
    back.type = "button";
    back.textContent = t("templateGallery.back");
    back.onclick = () => {
      step = "pick";
      render();
    };
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = t("templateGallery.cancel");
    cancel.onclick = () => close();
    const create = document.createElement("button");
    create.type = "button";
    create.className = "primary";
    create.textContent = t("templateGallery.create");
    if (options.mode === "capture") {
      create.disabled = true;
    } else {
      create.onclick = async () => {
        const consent = await confirmTemplateRelationships(selected!.title, selected!.relationships);
        if (consent.cancelled) {
          return;
        }
        rememberRecent(selected!.id);
        showRelationshipBannerIfNeeded(
          selected!.title,
          selected!.relationships,
          consent.enabledRequires,
        );
        await options.onCreate?.({
          templateId: selected!.id,
          projectName: nameInput.value.trim() || selected!.title,
          location: isWeb ? "" : locInput.value.trim(),
          sameFolder,
        });
        close();
      };
    }
    actions.append(back, cancel, create);
  };

  const renderActionsPick = () => {
    actions.replaceChildren();
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.textContent = t("templateGallery.cancel");
    cancel.onclick = () => close();
    const next = document.createElement("button");
    next.type = "button";
    next.className = "primary";
    next.textContent = t("templateGallery.next");
    if (options.mode === "capture") {
      next.disabled = true;
    } else {
      next.onclick = () => {
        if (!selected) {
          return;
        }
        step = "configure";
        render();
      };
    }
    actions.append(cancel, next);
  };

  const render = () => {
    const configuring = step === "configure";
    recentColumn.classList.toggle("hidden", configuring);
    header.classList.toggle("hidden", configuring);
    footer.classList.toggle("hidden", configuring);
    if (step === "pick") {
      renderPick();
      renderActionsPick();
    } else {
      renderConfigure();
    }
    enhanceSelectsIn(panel);
  };

  search.oninput = () => {
    query = search.value;
    if (step === "pick") {
      renderPick();
    }
  };
  domain.onchange = () => {
    domainFilter = domain.value;
    renderPick();
  };
  kind.onchange = () => {
    kindFilter = kind.value;
    renderPick();
  };
  source.onchange = () => {
    sourceFilter = source.value;
    renderPick();
  };

  overlay.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      close();
    }
    if (e.key === "/" && step === "pick") {
      e.preventDefault();
      search.focus();
    }
  });
  overlay.appendChild(panel);
  document.body.appendChild(overlay);
  render();
  if (options.selectedTemplateId) {
    const row = panel.querySelector(".template-gallery-row");
    row?.classList.add("selected");
  }
  if (step === "pick") {
    search.focus();
  }
}

/** Doc capture: open gallery frozen on pick or configure (does not create a project). */
export async function openTemplateGalleryForCapture(
  options: TemplateGalleryCaptureOptions = {},
): Promise<void> {
  const templates = await invoke<ProjectTemplateSummary[]>("list_project_templates");
  await mountTemplateGallery(templates, { mode: "capture", ...options });
}

export function openTemplateGallery(
  onCreate: (payload: TemplateGalleryCreate) => void | Promise<void>,
): void {
  void (async () => {
    const templates = await invoke<ProjectTemplateSummary[]>("list_project_templates");
    await mountTemplateGallery(templates, { mode: "live", onCreate });
  })();
}
