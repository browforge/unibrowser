import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n";
import { confirmTemplateRelationships, type TemplateRelationships } from "./relationshipConsent";

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
};

export type TemplateGalleryCaptureOptions = {
  step?: "pick" | "configure";
  searchQuery?: string;
  selectedTemplateId?: string;
};

/** Doc capture: open gallery frozen on pick or configure (does not create a project). */
export async function openTemplateGalleryForCapture(
  options: TemplateGalleryCaptureOptions = {},
): Promise<void> {
  const templates = await invoke<ProjectTemplateSummary[]>("list_project_templates");
  let step: "pick" | "configure" = options.step ?? "pick";
  let selected: ProjectTemplateSummary | null =
    options.selectedTemplateId
      ? templates.find((t) => t.id === options.selectedTemplateId) ?? null
      : null;
  if (step === "configure" && !selected && templates.length) {
    selected = templates[0];
  }
  let query = options.searchQuery ?? "";

  const overlay = document.createElement("div");
  overlay.className = "template-gallery-overlay";
  const panel = document.createElement("div");
  panel.className = "template-gallery-panel";
  panel.setAttribute("role", "dialog");

  const header = document.createElement("header");
  header.className = "template-gallery-header";
  const search = document.createElement("input");
  search.type = "search";
  search.className = "ub-form-control template-gallery-search";
  search.placeholder = t("templateGallery.search");
  search.value = query;
  header.appendChild(search);

  const filters = document.createElement("div");
  filters.className = "template-gallery-filters";
  const domain = document.createElement("select");
  domain.innerHTML =
    "<option value='all'>Domain: All</option><option value='document'>Document</option>";
  const workspace = document.createElement("select");
  workspace.innerHTML = "<option value='all'>Workspace: All</option>";
  const kind = document.createElement("select");
  kind.innerHTML = "<option value='all'>Kind: All</option><option value='starter'>Starter</option>";
  filters.append(domain, workspace, kind);

  const chips = document.createElement("div");
  chips.className = "template-gallery-chips";
  ["all", "builtin", "user", "installed"].forEach((chip) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "template-gallery-chip" + (chip === "builtin" ? " active" : "");
    btn.dataset.chip = chip;
    btn.textContent = t(`templateGallery.chip.${chip}` as "templateGallery.chip.all");
    chips.appendChild(btn);
  });

  const recentRow = document.createElement("div");
  recentRow.className = "template-gallery-recent";
  const body = document.createElement("div");
  body.className = "template-gallery-body";
  const footer = document.createElement("footer");
  footer.className = "template-gallery-footer";
  const actions = document.createElement("div");
  actions.className = "template-gallery-actions";

  const renderPick = () => {
    body.replaceChildren();
    const grid = document.createElement("div");
    grid.className = "template-gallery-grid";
    templates.forEach((tpl) => {
      const card = document.createElement("button");
      card.type = "button";
      card.className = "template-gallery-tile";
      if (selected?.id === tpl.id) {
        card.classList.add("selected");
      }
      const h = document.createElement("strong");
      h.textContent = tpl.title;
      const p = document.createElement("span");
      p.textContent = tpl.description;
      card.append(h, p);
      grid.appendChild(card);
    });
    body.append(grid);
    actions.replaceChildren();
    const next = document.createElement("button");
    next.type = "button";
    next.className = "primary";
    next.textContent = t("templateGallery.next");
    actions.appendChild(next);
  };

  const renderConfigure = () => {
    body.replaceChildren();
    const form = document.createElement("div");
    form.className = "template-gallery-configure";
    const nameLabel = document.createElement("label");
    nameLabel.textContent = t("templateGallery.projectName");
    const nameInput = document.createElement("input");
    nameInput.type = "text";
    nameInput.className = "ub-form-control";
    nameInput.value = selected?.title ?? "Office document";
    nameLabel.appendChild(nameInput);
    form.append(nameLabel);
    body.appendChild(form);
    actions.replaceChildren();
    const create = document.createElement("button");
    create.type = "button";
    create.className = "primary";
    create.textContent = t("templateGallery.create");
    actions.appendChild(create);
  };

  const render = () => {
    panel.replaceChildren(header, filters, chips, recentRow, body, footer, actions);
    if (step === "pick") {
      renderPick();
    } else {
      renderConfigure();
    }
  };

  search.oninput = () => {
    query = search.value;
  };

  overlay.appendChild(panel);
  document.body.appendChild(overlay);
  render();
  search.focus();
}

export function openTemplateGallery(
  onCreate: (payload: TemplateGalleryCreate) => void | Promise<void>,
): void {
  void (async () => {
    const templates = await invoke<ProjectTemplateSummary[]>("list_project_templates");
    let step: "pick" | "configure" = "pick";
    let selected: ProjectTemplateSummary | null = null;
    let sourceChip = "all";
    let domainFilter = "all";
    let workspaceFilter = "all";
    let kindFilter = "all";
    let query = "";

    const overlay = document.createElement("div");
    overlay.className = "template-gallery-overlay";
    const panel = document.createElement("div");
    panel.className = "template-gallery-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-modal", "true");

    const header = document.createElement("header");
    header.className = "template-gallery-header";
    const search = document.createElement("input");
    search.type = "search";
    search.className = "ub-form-control template-gallery-search";
    search.placeholder = t("templateGallery.search");
    search.setAttribute("aria-label", t("templateGallery.search"));
    header.appendChild(search);

    const filters = document.createElement("div");
    filters.className = "template-gallery-filters";
    const domain = document.createElement("select");
    domain.innerHTML =
      "<option value='all'>Domain: All</option><option value='document'>Document</option><option value='presentation'>Presentation</option><option value='video'>Video</option><option value='design'>Design</option><option value='notes'>Notes</option>";
    const workspace = document.createElement("select");
    workspace.innerHTML =
      "<option value='all'>Workspace: All</option><option value='browse'>Browse</option><option value='write'>Write</option><option value='create'>Create</option><option value='live'>Live</option>";
    const kind = document.createElement("select");
    kind.innerHTML =
      "<option value='all'>Kind: All</option><option value='starter'>Starter</option><option value='tutorial'>Tutorial</option><option value='hub'>Hub</option>";
    filters.append(domain, workspace, kind);

    const chips = document.createElement("div");
    chips.className = "template-gallery-chips";
    ["all", "builtin", "user", "installed"].forEach((chip) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "template-gallery-chip";
      btn.dataset.chip = chip;
      btn.textContent = t(`templateGallery.chip.${chip}` as "templateGallery.chip.all");
      chips.appendChild(btn);
    });

    const recentRow = document.createElement("div");
    recentRow.className = "template-gallery-recent";

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

    const close = () => overlay.remove();

    const renderRecent = () => {
      recentRow.replaceChildren();
      const recentIds = loadRecent();
      if (!recentIds.length) {
        return;
      }
      const label = document.createElement("span");
      label.className = "template-gallery-recent-label";
      label.textContent = t("templateGallery.recent");
      recentRow.appendChild(label);
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
        recentRow.appendChild(btn);
      });
    };

    const filtered = () => {
      const q = query.trim().toLowerCase();
      return templates.filter((tpl) => {
        if (sourceChip !== "all" && tpl.source !== sourceChip) {
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
        const hay = `${tpl.title} ${tpl.description} ${tpl.categories.join(" ")} ${tpl.tags.join(" ")}`.toLowerCase();
        return hay.includes(q);
      });
    };

    const renderPick = () => {
      body.replaceChildren();
      renderRecent();
      const grid = document.createElement("div");
      grid.className = "template-gallery-grid";
      const list = filtered();
      if (!list.length) {
        const empty = document.createElement("p");
        empty.className = "viewer-note";
        empty.textContent = t("templateGallery.empty");
        body.append(empty);
        return;
      }
      list.forEach((tpl) => {
        const card = document.createElement("button");
        card.type = "button";
        card.className = "template-gallery-tile";
        if (selected?.id === tpl.id) {
          card.classList.add("selected");
        }
        const h = document.createElement("strong");
        h.textContent = tpl.title;
        const p = document.createElement("span");
        p.textContent = tpl.description;
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
        card.append(h, p, tags);
        card.onclick = () => {
          selected = tpl;
          grid.querySelectorAll(".template-gallery-tile").forEach((el) => el.classList.remove("selected"));
          card.classList.add("selected");
        };
        grid.appendChild(card);
      });
      body.append(grid);
    };

    const renderConfigure = () => {
      body.replaceChildren();
      if (!selected) {
        step = "pick";
        render();
        return;
      }
      const form = document.createElement("div");
      form.className = "template-gallery-configure";
      const nameLabel = document.createElement("label");
      nameLabel.textContent = t("templateGallery.projectName");
      const nameInput = document.createElement("input");
      nameInput.type = "text";
      nameInput.className = "ub-form-control";
      nameInput.value = selected.title;
      nameLabel.appendChild(nameInput);
      const locLabel = document.createElement("label");
      locLabel.textContent = t("templateGallery.location");
      const locInput = document.createElement("input");
      locInput.type = "text";
      locInput.className = "ub-form-control";
      locInput.placeholder = t("templateGallery.locationHint");
      locLabel.appendChild(locInput);
      form.append(nameLabel, locLabel);
      body.appendChild(form);
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
      create.onclick = async () => {
        const consent = await confirmTemplateRelationships(selected!.title, selected!.relationships);
        if (consent.cancelled) {
          return;
        }
        rememberRecent(selected!.id);
        await onCreate({
          templateId: selected!.id,
          projectName: nameInput.value.trim() || selected!.title,
          location: locInput.value.trim(),
        });
        close();
      };
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
      next.onclick = () => {
        if (!selected) {
          return;
        }
        step = "configure";
        render();
      };
      actions.append(cancel, next);
    };

    const render = () => {
      panel.replaceChildren(header, filters, chips, recentRow, body, footer, actions);
      if (step === "pick") {
        renderPick();
        renderActionsPick();
      } else {
        renderConfigure();
      }
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
    workspace.onchange = () => {
      workspaceFilter = workspace.value;
      renderPick();
    };
    kind.onchange = () => {
      kindFilter = kind.value;
      renderPick();
    };
    chips.querySelectorAll<HTMLButtonElement>(".template-gallery-chip").forEach((btn) => {
      btn.onclick = () => {
        sourceChip = btn.dataset.chip ?? "all";
        chips.querySelectorAll(".template-gallery-chip").forEach((el) => el.classList.remove("active"));
        btn.classList.add("active");
        renderPick();
      };
    });
    chips.querySelector(".template-gallery-chip")?.classList.add("active");

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
    search.focus();
  })();
}
